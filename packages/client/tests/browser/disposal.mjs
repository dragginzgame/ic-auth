import assert from 'node:assert/strict';

export async function disposalCases({ check, a }) {
  await a.evaluate(() => {
    globalThis.disposalSetup = async (name, finish = true) => {
      now = 100n; clockId = 'clock';
      await seed(name, { ...record(), issuanceUnknown: true, issuance: { sequence: 7n } });
      const journal = await authTest.PublicJournal.open(indexedDB, name, { id: 'clock', nowNs: () => now });
      if (finish) {
        await journal.beginLogout('login', 100_000_000_000n);
        await authTest.cleanupOnce({ journal, generation: 'login',
          browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)),
          revoke: async () => frames['logout-Revoked-max'] });
      }
      return { journal, generation: 'login', caller };
    };
  });

  await check('key disposal refuses active/incomplete cleanup and mismatched bindings without invoking custody', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await disposalSetup('dispose-refusals', false); let calls = 0;
      const dispose = async () => { calls++; };
      const active = await failure(authTest.disposeLoginKey({ ...s, dispose }));
      await s.journal.beginLogout('login', 1_000n);
      const pending = await failure(authTest.disposeLoginKey({ ...s, dispose }));
      now = 1_000n;
      await authTest.cleanupOnce({ ...s, browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)),
        revoke: async () => { throw new Error('deadline passed'); } });
      const generation = await failure(authTest.disposeLoginKey({ ...s, generation: 'replacement', dispose }));
      const principal = await failure(authTest.disposeLoginKey({ ...s, caller: issuer, dispose }));
      s.journal.close(); return [active, pending, generation, principal, calls];
    }), ['not_current', 'not_current', 'not_current', 'not_current', 0]);
  });

  await check('disposal receives exact public binding and detached terminal evidence while journal remains retained', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await disposalSetup('dispose-success'); let bound;
      const result = await authTest.disposeLoginKey({ ...s, dispose: async (binding, status) => {
        bound = [binding.generation, binding.caller === caller.toText(), Object.isFrozen(binding), status.phase];
        status.destinations[0].confirmedUntilNs = 1n; status.issuanceUnknown = false;
      } });
      const retained = await s.journal.inspect(); s.journal.close();
      return [bound, result.destinations[0].confirmedUntilNs, result.issuanceUnknown, JSON.stringify(result, (_, v) => typeof v === 'bigint' ? String(v) : v) ===
        JSON.stringify(retained, (_, v) => typeof v === 'bigint' ? String(v) : v)];
    }), [['login', true, true, 'finished'], 0xffff_ffff_ffff_ffffn, true, true]);
  });

  await check('deadline completion exposes unconfirmed destinations to host disposal without erasing uncertainty', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await disposalSetup('dispose-deadline', false);
      await s.journal.beginLogout('login', 1_000n); now = 1_000n;
      await authTest.cleanupOnce({ ...s, browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)),
        revoke: async () => { throw new Error('expired'); } });
      let seen;
      await authTest.disposeLoginKey({ ...s, dispose: async (_, status) => {
        seen = [status.issuanceUnknown, status.destinations[0].confirmedUntilNs, status.deadlineNs];
      } });
      const retained = await s.journal.inspect(); s.journal.close();
      return [seen, retained.phase, retained.issuanceUnknown, retained.destinations[0].attempts];
    }), [[true, undefined, 1_000n], 'finished', true, 0]);
  });

  await check('lost disposal reply resumes without a signing key and conditional host deletion preserves replacements', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await disposalSetup('dispose-retry'); let key = { generation: 'login', caller: caller.toText() };
      let invocations = 0, deletions = 0; const lost = new Error('lost custody reply');
      const dispose = async binding => {
        invocations++;
        if (key && key.generation === binding.generation && key.caller === binding.caller) { key = undefined; deletions++; }
        if (invocations === 1) throw lost;
      };
      const failed = await authTest.disposeLoginKey({ ...s, dispose }).then(() => false, e => e === lost);
      s.journal.close();
      const journal = await authTest.PublicJournal.open(indexedDB, 'dispose-retry', { id: 'clock', nowNs: () => now });
      await authTest.disposeLoginKey({ ...s, journal, dispose });
      key = { generation: 'replacement', caller: caller.toText() };
      await authTest.disposeLoginKey({ ...s, journal, dispose });
      journal.close(); return [failed, invocations, deletions, key.generation];
    }), [true, 3, 1, 'replacement']);
  });

  await check('cancellation before or during terminal journal checking prevents disposal', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await disposalSetup('dispose-cancel'); let calls = 0;
      const dispose = async () => { calls++; };
      const pre = await failure(authTest.disposeLoginKey({ ...s, dispose, signal: AbortSignal.abort() }));
      const controller = new AbortController(); const original = s.journal.disposalStatus.bind(s.journal);
      s.journal.disposalStatus = async (...args) => { const status = await original(...args); controller.abort(); return status; };
      const during = await failure(authTest.disposeLoginKey({ ...s, dispose, signal: controller.signal }));
      s.journal.close(); return [pre, during, calls];
    }), ['cancelled', 'cancelled', 0]);
  });

  await check('cancellation after custody invocation does not falsely report that completed deletion was cancelled', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await disposalSetup('dispose-inflight'); const controller = new AbortController(); let invoked = false;
      const result = await authTest.disposeLoginKey({ ...s, signal: controller.signal, dispose: async () => {
        invoked = true; controller.abort(); await Promise.resolve();
      } });
      s.journal.close(); return [invoked, controller.signal.aborted, result.phase];
    }), [true, true, 'finished']);
  });

  await check('storage and clock failures refuse key disposal', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await disposalSetup('dispose-storage'); let calls = 0;
      const dispose = async () => { calls++; }; now = 99n;
      const clock = await failure(authTest.disposeLoginKey({ ...s, dispose }));
      s.journal.close(); now = 100n;
      const storage = await failure(authTest.disposeLoginKey({ ...s, dispose }));
      return [clock, storage, calls];
    }), ['clock_changed', 'storage_unavailable', 0]);
  });
}
