import assert from 'node:assert/strict';

export async function admissionCases({ check, a, b }) {
  // Each case owns its DB; this group can run without the earlier lifecycle groups.
  await a.evaluate(() => {
    const { Ed25519KeyIdentity, Principal } = authTest;
    globalThis.admissionSetup = async name => {
      const journal = await open(name);
      await journal.start({ generation: 'login', caller, issuer });
      return { journal, generation: 'login', issuer, target: Principal.fromUint8Array(Uint8Array.of(2)), scopes: ['read'],
        browser: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)),
        account: Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10)),
        transport: {
          issue: async () => ({ bytes: frames.ed25519, untilNs: 1_000n }),
          renew: async () => { throw new Error('unexpected renewal'); },
          reconcile: async () => { throw new Error('unexpected lookup'); },
        } };
    };
  });

  await check('admission inventories the receiver and signs exact Wire input with the browser identity', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { establishSession, signCall, admissionReply } = authTest;
      const options = await admissionSetup('admission-success');
      options.scopes = ['write', 'read', 'write'];
      let dispatches = 0, signer, response, signedByBrowser = false, exactFrame = false, inventoried = false, committed = false;
      const guard = options.journal.assertAdmission.bind(options.journal);
      options.journal.assertAdmission = (...args) => { if (response) response.fill(0); return guard(...args); };
      const pending = establishSession({ ...options, admit: async (target, identity, request) => {
        dispatches++; signer = identity;
        const expected = frames['establish-none'];
        exactFrame = request.length === expected.length && request.every((v, i) => v === expected[i]);
        const state = await options.journal.inspect(); inventoried = state.destinations.some(t => t.canister === target.toText());
        committed = await raw('admission-success', b => { const r = b.count('credential'); return () => r.result === 1; });
        const signed = await signCall(identity, target, request, 'contract');
        signedByBrowser = identity.getPrincipal().toText() === caller.toText() && signed.body.sender_sig.length === 64;
        response = admissionReply(caller, issuer, 1n); return response;
      } });
      options.target.toUint8Array()[0] = 3; options.scopes[0] = 'changed';
      const receipt = await pending;
      const closed = await failure(signCall(signer)); options.journal.close();
      return { dispatches, exactFrame, inventoried, committed, signedByBrowser, closed,
        detached: receipt.caller[0] === caller.toUint8Array()[0], receiverClock: receipt.expires_at_ns };
    }), { dispatches: 1, exactFrame: true, inventoried: true, committed: true, signedByBrowser: true,
      closed: 'closed', detached: true, receiverClock: 1n });
  });

  await check('every retained admission refusal and uncertain transport failure propagates without retry', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { establishSession, AdmissionError } = authTest;
      const options = await admissionSetup('admission-refusal');
      const tags = ['NotYetValid', 'Expired', 'InvalidCredential', 'AuthorityUnavailable', 'BeforeReset',
        'ScopeRejected', 'ReplayConflict', 'Revoked', 'Capacity', 'Throttled', 'Unavailable'];
      let calls = 0;
      const refused = [];
      for (const tag of tags) {
        refused.push(await establishSession({ ...options, admit: async () => { calls++; return frames[`establish-${tag}`]; } })
          .then(() => false, e => e instanceof AdmissionError && e.code === 'admission_refused' && e.refusal === tag));
      }
      const lost = new Error('unknown admission outcome');
      const same = await establishSession({ ...options, admit: async () => { calls++; throw lost; } }).then(() => false, e => e === lost);
      const state = await options.journal.inspect(); options.journal.close();
      return [refused.every(Boolean), calls, same, state.destinations.length, state.sequence];
    }), [true, 12, true, 2, 0n]);
  });

  await check('malformed replies and caller/issuer/expiry substitution cannot produce receipts', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { establishSession, admissionReply } = authTest;
      const options = await admissionSetup('admission-bad-reply');
      const outputs = [Uint8Array.of(0), frames.ed25519, frames['establish-none'],
        new Uint8Array([...admissionReply(caller, issuer), 0]),
        admissionReply(issuer, issuer), admissionReply(caller, options.target), admissionReply(caller, issuer, 0n)];
      const codes = [];
      for (const reply of outputs) codes.push(await failure(establishSession({ ...options, admit: async () => reply })));
      options.journal.close(); return codes;
    }), Array(7).fill('invalid_material'));
  });

  await check('aborted inventory commit prevents receiver dispatch and preserves the completed credential', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { establishSession, admissionReply } = authTest;
      const options = await admissionSetup('admission-inventory');
      let calls = 0, intercepted = false;
      const put = IDBObjectStore.prototype.put;
      IDBObjectStore.prototype.put = function(value, key) {
        const req = put.call(this, value, key);
        if (key === 'login' && value.targets.length > 1) {
          intercepted = true; req.addEventListener('success', () => this.transaction.abort());
        }
        return req;
      };
      let code;
      try { code = await failure(establishSession({ ...options, admit: async () => { calls++; return admissionReply(caller, issuer); } })); }
      finally { IDBObjectStore.prototype.put = put; }
      const state = await options.journal.inspect();
      const saved = await options.journal.claimCredential('login', { accountAvailable: false }); options.journal.close();
      return [code, intercepted, calls, state.destinations.length, saved.kind];
    }), ['storage_unavailable', true, 0, 1, 'saved']);
  });

  await check('proof expiry at signing refuses even after receiver inventory has committed', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { establishSession, signCall, admissionReply } = authTest;
      const options = await admissionSetup('admission-expiry');
      let signatures = 0;
      const code = await failure(establishSession({ ...options, admit: async (target, identity, request) => {
        now = 1_000n;
        await signCall(identity, target, request, 'contract'); signatures++;
        return admissionReply(caller, issuer);
      } }));
      const state = await options.journal.inspect(); options.journal.close(); now = 100n;
      return [code, signatures, state.destinations.length];
    }), ['credential_expired', 0, 2]);
  });

  await check('another tab can log out during admission; the late receipt is refused and inventory retained', async () => {
    await a.evaluate(async () => {
      const { establishSession, signCall, admissionReply } = authTest;
      globalThis.admissionOptions = await admissionSetup('admission-logout');
      let entered; const ready = new Promise(resolve => { entered = resolve; });
      const gate = new Promise(resolve => { globalThis.resumeAdmission = resolve; });
      globalThis.pendingAdmission = failure(establishSession({ ...admissionOptions, admit: async (target, identity, request) => {
        await signCall(identity, target, request, 'contract'); entered(); await gate;
        return admissionReply(caller, issuer);
      } }));
      await ready;
    });
    await b.evaluate(async () => { const j = await open('admission-logout'); await j.beginLogout('login', 1_000n); j.close(); });
    assert.deepEqual(await a.evaluate(async () => {
      resumeAdmission(); const code = await pendingAdmission;
      const state = await admissionOptions.journal.inspect(); admissionOptions.journal.close();
      return [code, state.phase, state.destinations.length];
    }), ['not_current', 'cleanup', 2]);
  });

  await check('newer credential sequence prevents an older receiver attempt from signing', async () => {
    await a.evaluate(async () => {
      const { establishSession, signCall, admissionReply } = authTest;
      globalThis.admissionOptions = await admissionSetup('admission-sequence');
      let entered; const ready = new Promise(resolve => { entered = resolve; });
      const gate = new Promise(resolve => { globalThis.resumeAdmission = resolve; });
      globalThis.admissionEscaped = false;
      globalThis.pendingAdmission = failure(establishSession({ ...admissionOptions, admit: async (target, identity, request) => {
        entered(); await gate; await signCall(identity, target, request, 'contract'); admissionEscaped = true;
        return admissionReply(caller, issuer);
      } }));
      await ready;
    });
    await b.evaluate(async () => {
      const j = await open('admission-sequence');
      await j.claimCredential('login', { accountAvailable: false, refreshThrough: 0n }); j.close();
    });
    assert.deepEqual(await a.evaluate(async () => {
      resumeAdmission(); const code = await pendingAdmission; admissionOptions.journal.close();
      return [code, admissionEscaped];
    }), ['not_current', false]);
  });
}
