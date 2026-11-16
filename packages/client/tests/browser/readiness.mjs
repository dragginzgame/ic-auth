import assert from 'node:assert/strict';

export async function readinessCases({ check, a, b }) {
  await a.evaluate(() => {
    globalThis.readySetup = async (name, changes = {}) => {
      const { SessionCoordinator, Ed25519KeyIdentity, Principal, signCall, admissionReply } = authTest;
      now = 100n;
      const journal = await open(name);
      await journal.start({ generation: 'login', caller, issuer });
      const count = { issue: 0, renew: 0, lookup: 0, admit: 0 };
      const options = { journal, generation: 'login', issuer,
        browser: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)),
        account: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10)), scopes: ['read'],
        transport: {
          issue: async () => { count.issue++; return { bytes: frames.ed25519, untilNs: 10_000n }; },
          renew: async () => { count.renew++; return { bytes: frames.ed25519, untilNs: 10_000n }; },
          reconcile: async () => { count.lookup++; return { bytes: frames.ed25519, untilNs: 10_000n }; },
        },
        admit: async (target, identity, request) => {
          count.admit++; await signCall(identity, target, request, 'admit'); return admissionReply(caller, issuer, 50_000n);
        },
        refreshAt: () => now + 100n,
        ...changes,
      };
      return { journal, count, options, coordinator: new SessionCoordinator(options),
        target: Principal.fromUint8Array(Uint8Array.of(2)) };
    };
  });

  await check('duplicate receiver admissions coalesce and returned hints are detached and metadata-only', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await readySetup('ready-coalesce');
      const receipts = await Promise.all(Array.from({ length: 12 }, () => s.coordinator.ensureSession(s.target)));
      receipts[0].caller.fill(0);
      const get = IDBObjectStore.prototype.get; let proofReads = 0;
      IDBObjectStore.prototype.get = function(key) { if (key === 'credential') proofReads++; return get.call(this, key); };
      let live;
      try { live = await s.coordinator.ensureSession(s.target); } finally { IDBObjectStore.prototype.get = get; }
      const isolated = authTest.Principal.fromUint8Array(live.caller).toText() === caller.toText();
      s.journal.close(); return [s.count, proofReads, isolated, receipts[1].caller.some(b => b !== 0)];
    }), [{ issue: 1, renew: 0, lookup: 0, admit: 1 }, 0, true, true]);
  });

  await check('different receivers share issuance but admit independently and reuse a newer renewal', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await readySetup('ready-receivers'); const other = authTest.Principal.fromUint8Array(Uint8Array.of(3));
      await Promise.all([s.coordinator.ensureSession(s.target), s.coordinator.ensureSession(other)]);
      now = 200n; await s.coordinator.ensureSession(s.target); await s.coordinator.ensureSession(other);
      s.journal.close(); return s.count;
    }), { issue: 1, renew: 1, lookup: 0, admit: 4 });
  });

  await check('64 outstanding callers include duplicates; overflow opens no transaction and capacity returns', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      let release, entered;
      const gate = new Promise(resolve => { release = resolve; }), started = new Promise(resolve => { entered = resolve; });
      let issues = 0;
      const s = await readySetup('ready-capacity', { transport: {
        issue: async () => { issues++; entered(); await gate; return { bytes: frames.ed25519, untilNs: 10_000n }; },
        renew: async () => { throw new Error('renew'); }, reconcile: async () => { throw new Error('lookup'); },
      } });
      const work = Array.from({ length: 64 }, () => s.coordinator.ensureSession(s.target)); await started;
      let reads = 0; const original = s.journal.readinessTime.bind(s.journal);
      s.journal.readinessTime = (...args) => { reads++; return original(...args); };
      const overflow = await failure(s.coordinator.ensureSession(s.target)); const overflowReads = reads;
      release(); await Promise.all(work); await s.coordinator.ensureSession(s.target);
      s.journal.close(); return [overflow, overflowReads, issues, s.count.admit];
    }), ['operation_limit', 0, 1, 1]);
  });

  await check('authenticated Expired retries once with a fresh sequence; explicit establish never retries', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      let attempts = 0;
      const s = await readySetup('ready-expired', { admit: async () => ++attempts === 1
        ? frames['establish-Expired'] : authTest.admissionReply(caller, issuer, 500n) });
      await s.coordinator.ensureSession(s.target);
      const first = [attempts, s.count.renew]; s.journal.close();
      const e = await readySetup('explicit-expired', { admit: async () => frames['establish-Expired'] });
      const code = await e.coordinator.establish(e.target).then(() => 'accepted', e => e.refusal);
      e.journal.close(); return [first, code, e.count.renew];
    }), [[2, 1], 'Expired', 0]);
  });

  await check('repeated expiry stops at two attempts; other refusals and uncertain responses never retry', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const results = [];
      for (const tag of ['Expired', 'NotYetValid', 'AuthorityUnavailable', 'lost']) {
        let attempts = 0; const lost = new Error('uncertain');
        const s = await readySetup(`ready-error-${tag}`, { admit: async () => {
          attempts++; if (tag === 'lost') throw lost; return frames[`establish-${tag}`];
        } });
        const code = await s.coordinator.ensureSession(s.target).then(() => 'accepted', e => e === lost ? 'lost' : e.refusal);
        results.push([code, attempts, s.count.renew]); s.journal.close();
      }
      return results;
    }), [['Expired', 2, 1], ['NotYetValid', 1, 0], ['AuthorityUnavailable', 1, 0], ['lost', 1, 0]]);
  });

  await check('recovered expired completion commits before one renewal and reopening readmits saved evidence', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await readySetup('ready-recovery');
      await s.journal.claimCredential('login', { accountAvailable: true });
      const transport = { ...s.options.transport, reconcile: async () => {
        s.count.lookup++; return { bytes: frames.ed25519, untilNs: 100n };
      } };
      const coordinator = new authTest.SessionCoordinator({ ...s.options, transport });
      await coordinator.ensureSession(s.target);
      s.journal.close();
      const journal = await open('ready-recovery');
      const reopened = new authTest.SessionCoordinator({ ...s.options, journal, account: undefined });
      await reopened.ensureSession(s.target); journal.close(); return s.count;
    }), { issue: 0, renew: 1, lookup: 1, admit: 2 });
  });

  await check('already expired mapped deadlines refuse without an admission loop and live hints survive proof expiry', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await readySetup('ready-deadline', { refreshAt: () => 100n });
      const code = await failure(s.coordinator.ensureSession(s.target)); s.journal.close();
      const live = await readySetup('ready-proof-expiry', { refreshAt: () => 20_000n });
      await live.coordinator.ensureSession(live.target); now = 10_000n;
      await live.coordinator.ensureSession(live.target); live.journal.close(); now = 100n;
      return [code, s.count.admit, s.count.renew, live.count.admit];
    }), ['lease_expired', 1, 0, 1]);
  });

  await check('cross-tab logout does not wait for coalesced admission and fences all late results', async () => {
    await a.evaluate(async () => {
      let entered; const started = new Promise(resolve => { entered = resolve; });
      const gate = new Promise(resolve => { globalThis.resumeReady = resolve; });
      globalThis.rs = await readySetup('ready-logout', { admit: async () => {
        entered(); await gate; return authTest.admissionReply(caller, issuer, 500n);
      } });
      globalThis.waitingReady = Promise.all(Array.from({ length: 4 }, () => failure(rs.coordinator.ensureSession(rs.target))));
      await started;
    });
    await b.evaluate(async () => { const journal = await open('ready-logout'); await journal.beginLogout('login', 1_000n); journal.close(); });
    assert.deepEqual(await a.evaluate(async () => {
      resumeReady(); const codes = await waitingReady; codes.push(await failure(rs.coordinator.ensureSession(rs.target)));
      rs.journal.close(); return codes;
    }), Array(5).fill('not_current'));
  });

  await check('fixed coordinator bindings reject changed identities and capture mutable scopes and targets', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await readySetup('ready-bindings');
      s.options.scopes[0] = 'substituted';
      const target = authTest.Principal.fromUint8Array(Uint8Array.of(2));
      const pending = s.coordinator.ensureSession(target); target.toUint8Array().fill(3);
      await pending;
      const status = await s.journal.inspect();
      const original = s.options.browser.getPrincipal.bind(s.options.browser);
      s.options.browser.getPrincipal = () => authTest.Principal.fromUint8Array(Uint8Array.of(9));
      const code = await failure(s.coordinator.ensureSession(s.target));
      s.options.browser.getPrincipal = original; s.journal.close();
      return [code, status.destinations.some(t => t.canister === s.target.toText()), status.destinations.length];
    }), ['identity_changed', true, 2]);
  });
  await check('one stalled receiver does not block another and explicit establish remains forced', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      let release, entered;
      const gate = new Promise(resolve => { release = resolve; }), started = new Promise(resolve => { entered = resolve; });
      const s = await readySetup('ready-independent', { admit: async target => {
        if (target.toText() === s.target.toText()) { entered(); await gate; }
        s.count.admit++; return authTest.admissionReply(caller, issuer, 500n);
      } });
      const stalled = s.coordinator.ensureSession(s.target); await started;
      const other = authTest.Principal.fromUint8Array(Uint8Array.of(3));
      await s.coordinator.ensureSession(other); const before = s.count.admit;
      release(); await stalled; await s.coordinator.establish(other);
      s.journal.close(); return [before, s.count.admit, s.count.issue];
    }), [1, 3, 1]);
  });

  await check('timeout releases coalesced capacity but late replies cannot install a hint', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      let calls = 0, release;
      const late = new Promise(resolve => { release = resolve; });
      const s = await readySetup('ready-timeout', { admit: async () => {
        calls++; if (calls === 1) await late; return authTest.admissionReply(caller, issuer, 500n);
      } });
      const codes = await Promise.all(Array.from({ length: 64 }, () => failure(s.coordinator.ensureSession(s.target))));
      release(); await s.coordinator.ensureSession(s.target);
      s.journal.close(); return [codes.every(c => c === 'timeout'), calls, s.count.renew];
    }), [true, 2, 0]);
  });

  await check('cancelled coordinator refuses work and invalid or regressing clocks cannot reuse hints', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const controller = new AbortController();
      const s = await readySetup('ready-cancel', { signal: controller.signal });
      controller.abort(); const cancelled = await failure(s.coordinator.ensureSession(s.target)); s.journal.close();
      const invalid = await readySetup('ready-invalid-map', { refreshAt: () => -1n });
      const bad = await failure(invalid.coordinator.ensureSession(invalid.target)); invalid.journal.close();
      const clock = await readySetup('ready-clock'); await clock.coordinator.ensureSession(clock.target);
      now = 99n; const regression = await failure(clock.coordinator.ensureSession(clock.target));
      clock.journal.close(); now = 100n;
      return [cancelled, s.count.issue, bad, regression];
    }), ['cancelled', 0, 'configuration', 'clock_changed']);
  });

}
