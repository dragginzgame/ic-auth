import assert from 'node:assert/strict';

export async function logoutCases({ check, a, b }) {
  await a.evaluate(() => {
    globalThis.logoutSetup = async (name, deadline = 100_000_000_000n) => {
      now = 100n; clockId = 'clock';
      await seed(name, { ...record(), issuanceUnknown: true, issuance: { sequence: 7n } });
      const journal = await authTest.PublicJournal.open(indexedDB, name, { id: clockId, nowNs: () => now });
      await journal.beginLogout('login', deadline);
      return { journal, generation: 'login', browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)) };
    };
  });

  await check('logout signs one exact Wire request with the browser after durable reservation', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const { cleanupOnce, signCall } = authTest;
      const options = await logoutSetup('logout-success');
      let calls = 0, signer, reply, committed = false, exact = false, signature;
      const guard = options.journal.assertCleanup.bind(options.journal);
      options.journal.assertCleanup = (...args) => { if (reply) reply.fill(0); return guard(...args); };
      options.revoke = async (target, identity, request) => {
        calls++; signer = identity;
        const persisted = await raw('logout-success', bucket => {
          const r = bucket.get('login'); return () => r.result;
        });
        committed = persisted.phase === 'cleanup' && persisted.targets[0].attempts === 1 && persisted.busyUntilNs > now;
        exact = request.length === frames['logout-request'].length && request.every((v, i) => v === frames['logout-request'][i]);
        signature = (await signCall(identity, target, request, 'contract')).body.sender_sig.length;
        if (identity.getPrincipal().toText() !== caller.toText()) throw new Error('wrong signer');
        const wrong = authTest.Principal.fromUint8Array(Uint8Array.of(2));
        if (await failure(signCall(identity, wrong)) !== 'wrong_destination') throw new Error('redirected logout');
        reply = new Uint8Array(frames['logout-Revoked-max']); return reply;
      };
      const pending = cleanupOnce(options);
      options.revoke = async () => { throw new Error('adapter mutated during storage'); };
      const result = await pending;
      const closed = await failure(signCall(signer));
      const done = await cleanupOnce({ ...options, revoke: async () => { calls++; throw new Error('unexpected repeat'); } });
      options.journal.close();
      return [calls, committed, exact, signature, closed, result.kind, result.response.tag,
        result.status.destinations[0].confirmedUntilNs, result.status.issuanceUnknown, done.kind];
    }), [1, true, true, 64, 'closed', 'settled', 'Revoked', 0xffff_ffff_ffff_ffffn, true, 'finished']);
  });

  await check('every typed nonconfirmation preserves uncertainty and schedules without retrying', async () => {
    const rows = await a.evaluate(async () => {
      const rows = [];
      for (const name of ['Unconfirmed-zero', 'Unconfirmed-max', 'InvalidCaller', 'Capacity', 'Storage', 'Clock', 'ResetPending']) {
        const options = await logoutSetup(`logout-${name}`); let calls = 0;
        const revoke = async () => { calls++; return frames[`logout-${name}`]; };
        const result = await authTest.cleanupOnce({ ...options, revoke, retryAfterMs: 500 });
        const wait = await authTest.cleanupOnce({ ...options, revoke });
        options.journal.close();
        rows.push([calls, result.status.phase, result.status.destinations[0].confirmedUntilNs,
          result.status.issuanceUnknown, wait.kind, wait.untilNs,
          result.response.tag === 'Unavailable' ? result.response.value.tag : result.response.tag]);
      }
      return rows;
    });
    assert.deepEqual(rows, ['Unconfirmed', 'Unconfirmed', 'InvalidCaller', 'Capacity', 'Storage', 'Clock', 'ResetPending']
      .map(tag => [1, 'cleanup', undefined, true, 'wait', 500_000_100n, tag]));
  });

  await check('malformed, oversized and zero-Revoked replies retain the consumed attempt without confirmation', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const results = [];
      const good = frames['logout-Revoked-max'];
      const replies = [frames['logout-Revoked-zero'], frames['logout-request'], frames['establish-success'],
        good.subarray(0, good.length - 1), new Uint8Array([...good, 0]), new Uint8Array(1_025), Array.from(good)];
      for (const [i, reply] of replies.entries()) {
        const options = await logoutSetup(`logout-malformed-${i}`); let calls = 0;
        const revoke = async () => { calls++; return reply; };
        const code = await failure(authTest.cleanupOnce({ ...options, revoke }));
        const waiting = await authTest.cleanupOnce({ ...options, revoke });
        options.journal.close();
        results.push([code, calls, waiting.kind, waiting.status.destinations[0].attempts,
          waiting.status.destinations[0].confirmedUntilNs]);
      }
      return results;
    }), Array(7).fill(['invalid_material', 1, 'wait', 1, undefined]));
  });

  await check('lost replies, cancellation and timeout close signers and never accept late success', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const results = [];
      for (const mode of ['lost', 'cancel', 'timeout']) {
        const options = await logoutSetup(`logout-${mode}`, mode === 'timeout' ? 100_000_100n : 100_000_000_000n);
        const controller = new AbortController(), lost = new Error('unknown remote effect');
        let calls = 0, signer, resume;
        const operation = authTest.cleanupOnce({ ...options, signal: controller.signal, revoke: async (_target, identity) => {
          calls++; signer = identity;
          if (mode === 'lost') throw lost;
          const reply = new Promise(resolve => { resume = () => resolve(frames['logout-Revoked-max']); });
          if (mode === 'cancel') controller.abort();
          return reply;
        } });
        const code = await operation.then(() => 'accepted', error => error === lost ? 'same-error' : error.code);
        resume?.();
        await Promise.resolve();
        const closed = await failure(authTest.signCall(signer));
        const state = await options.journal.inspect(); options.journal.close();
        results.push([code, calls, closed, state.destinations[0].attempts, state.destinations[0].confirmedUntilNs]);
      }
      return results;
    }), [['same-error', 1, 'closed', 1, undefined], ['cancelled', 1, 'cancelled', 1, undefined], ['timeout', 1, 'timeout', 1, undefined]]);
  });

  await check('pre-cancellation and invalid configuration consume nothing; aborted reservation dispatches nothing', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const options = await logoutSetup('logout-preflight'); let calls = 0;
      const request = extra => authTest.cleanupOnce({ ...options, revoke: async () => { calls++; return frames['logout-Revoked-max']; }, ...extra });
      const codes = [await failure(request({ signal: AbortSignal.abort() })), await failure(request({ retryAfterMs: 1_500 })),
        await failure(request({ browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10)) }))];
      const before = await options.journal.inspect();
      const put = IDBObjectStore.prototype.put;
      IDBObjectStore.prototype.put = function(value, key) {
        const r = put.call(this, value, key);
        if (key === 'login') r.addEventListener('success', () => this.transaction.abort());
        return r;
      };
      try { codes.push(await failure(request({}))); }
      finally { IDBObjectStore.prototype.put = put; }
      const after = await options.journal.inspect(); options.journal.close();
      return [codes, calls, before.destinations[0].attempts, after.destinations[0].attempts];
    }), [['cancelled', 'configuration', 'not_current', 'storage_unavailable'], 0, 0, 0]);
  });

  await check('a valid remote reply is not reported confirmed when its local settlement aborts', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const options = await logoutSetup('logout-settlement'); let calls = 0, signer;
      const put = IDBObjectStore.prototype.put;
      IDBObjectStore.prototype.put = function(value, key) {
        const r = put.call(this, value, key);
        if (key === 'login' && value.targets[0].confirmedUntilNs !== undefined)
          r.addEventListener('success', () => this.transaction.abort());
        return r;
      };
      let code;
      try { code = await failure(authTest.cleanupOnce({ ...options, revoke: async (_t, identity) => {
        calls++; signer = identity; return frames['logout-Revoked-max'];
      } })); }
      finally { IDBObjectStore.prototype.put = put; }
      const state = await options.journal.inspect();
      const closed = await failure(authTest.signCall(signer)); options.journal.close();
      return [code, calls, closed, state.destinations[0].attempts, state.destinations[0].confirmedUntilNs];
    }), ['storage_unavailable', 1, 'closed', 1, undefined]);
  });

  await check('another tab cannot duplicate a live cleanup call and a reply after the deadline remains unconfirmed', async () => {
    await a.evaluate(async () => {
      globalThis.logoutOptions = await logoutSetup('logout-two-tabs');
      let entered;
      const ready = new Promise(resolve => { entered = resolve; });
      globalThis.logoutPending = failure(authTest.cleanupOnce({ ...logoutOptions, revoke: async () => {
        entered(); return new Promise(resolve => { globalThis.resumeLogout = () => resolve(frames['logout-Revoked-max']); });
      } }));
      await ready;
    });
    assert.deepEqual(await b.evaluate(async () => {
      now = 100n; clockId = 'clock';
      const journal = await open('logout-two-tabs');
      let calls = 0;
      const browser = authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
      const result = await authTest.cleanupOnce({ journal, generation: 'login', browser,
        revoke: async () => { calls++; return frames['logout-Revoked-max']; } });
      journal.close(); return [result.kind, calls];
    }), ['wait', 0]);
    assert.deepEqual(await a.evaluate(async () => {
      now = 100_000_000_000n;
      resumeLogout(); const code = await logoutPending;
      const result = await authTest.cleanupOnce({ ...logoutOptions, revoke: async () => { throw new Error('late dispatch'); } });
      logoutOptions.journal.close(); now = 100n;
      return [code, result.kind, result.status.destinations[0].confirmedUntilNs, result.status.issuanceUnknown];
    }), ['not_current', 'finished', undefined, true]);
  });
}
