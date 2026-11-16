import assert from 'node:assert/strict';

export async function schedulerCases({ check, a, b }) {
  for (const page of [a, b]) await page.evaluate(() => {
    globalThis.scheduleSetup = async (name, duration = 20_000, changes = {}) => {
      clockId = 'clock'; now = 100n;
      await seed(name, { ...record(), issuanceUnknown: true, ...changes });
      let offset = 0n;
      const clock = { id: 'clock', nowNs: () => BigInt(Date.now()) * 1_000_000n + offset };
      const journal = await authTest.PublicJournal.open(indexedDB, name, clock);
      await journal.beginLogout('login', clock.nowNs() + BigInt(duration) * 1_000_000n);
      return { options: { journal, generation: 'login', retryAfterMs: 500,
        browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)) },
        advance: ns => { offset += ns; }, clock };
    };
  });

  await check('scheduler retries typed nonconfirmation and visits every inventoried destination', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const other = authTest.Principal.fromUint8Array(Uint8Array.of(2)).toText();
      const s = await scheduleSetup('schedule-success', 20_000, { targets: [
        { canister: issuer.toText(), attempts: 0, nextNs: 100n }, { canister: other, attempts: 0, nextNs: 100n }] });
      const calls = []; let saved;
      const result = await authTest.cleanupUntilFinished({ ...s.options, revoke: async (target, identity, bytes) => {
        saved = identity; calls.push(target.toText()); await authTest.signCall(identity, target, bytes);
        return frames[calls.length === 1 ? 'logout-Unconfirmed-zero' : 'logout-Revoked-max'];
      } });
      const closed = await failure(authTest.signCall(saved)); s.options.journal.close();
      return [calls.length, new Set(calls).size, result.phase, result.destinations.map(t => t.attempts).sort(),
        result.destinations.every(t => t.confirmedUntilNs !== undefined), result.issuanceUnknown, closed];
    }), [3, 2, 'finished', [1, 2], true, true, 'closed']);
  });

  await check('lost response stops the run; reopening resumes retained attempt count and lease', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await scheduleSetup('schedule-lost'); let calls = 0;
      const lost = new Error('lost response');
      const stopped = await authTest.cleanupUntilFinished({ ...s.options, revoke: async () => { calls++; throw lost; } })
        .then(() => false, error => error === lost);
      const before = await s.options.journal.inspect(); s.options.journal.close(); s.advance(5_000_000_000n);
      const journal = await authTest.PublicJournal.open(indexedDB, 'schedule-lost', s.clock);
      const result = await authTest.cleanupUntilFinished({ ...s.options, journal,
        revoke: async () => { calls++; return frames['logout-Revoked-max']; } });
      journal.close(); return [stopped, before.destinations[0].attempts, result.destinations[0].attempts, calls, result.phase];
    }), [true, 1, 2, 2, 'finished']);
  });

  await check('cancellation interrupts a scheduled wait without another attempt and permits resumption', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await scheduleSetup('schedule-cancel');
      const once = await authTest.cleanupOnce({ ...s.options, revoke: async () => frames['logout-Unconfirmed-zero'] });
      let calls = 0; const controller = new AbortController();
      const pending = failure(authTest.cleanupUntilFinished({ ...s.options, signal: controller.signal,
        revoke: async () => { calls++; return frames['logout-Revoked-max']; } }));
      setTimeout(() => controller.abort(), 30);
      const code = await pending; const status = await s.options.journal.inspect(); s.advance(500_000_000n);
      const done = await authTest.cleanupUntilFinished({ ...s.options,
        revoke: async () => { calls++; return frames['logout-Revoked-max']; } });
      s.options.journal.close(); return [once.kind, code, status.destinations[0].attempts, calls, done.phase];
    }), ['settled', 'cancelled', 1, 1, 'finished']);
  });

  await check('two schedulers share journal leases and the waiting tab observes confirmed completion', async () => {
    await a.evaluate(async () => {
      globalThis.scheduled = await scheduleSetup('schedule-tabs', 2_000);
      let entered; const started = new Promise(resolve => { entered = resolve; });
      const gate = new Promise(resolve => { globalThis.releaseSchedule = resolve; });
      globalThis.scheduleCalls = 0;
      globalThis.scheduleResult = authTest.cleanupUntilFinished({ ...scheduled.options, revoke: async () => {
        scheduleCalls++; entered(); await gate; return frames['logout-Revoked-max'];
      } });
      await started;
    });
    await b.evaluate(async () => {
      globalThis.otherJournal = await authTest.PublicJournal.open(indexedDB, 'schedule-tabs',
        { id: 'clock', nowNs: () => BigInt(Date.now()) * 1_000_000n });
      globalThis.otherCalls = 0;
      globalThis.otherResult = authTest.cleanupUntilFinished({ journal: otherJournal, generation: 'login',
        browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)),
        revoke: async () => { otherCalls++; return frames['logout-Revoked-max']; } });
    });
    assert.deepEqual(await a.evaluate(async () => {
      releaseSchedule(); const result = await scheduleResult; scheduled.options.journal.close(); return [result.phase, scheduleCalls];
    }), ['finished', 1]);
    assert.deepEqual(await b.evaluate(async () => {
      const result = await otherResult; otherJournal.close(); return [result.phase, otherCalls, result.destinations[0].attempts];
    }), ['finished', 0, 1]);
  });

  await check('exhausted attempts wait for the fixed deadline and retain unconfirmed uncertainty', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await scheduleSetup('schedule-exhausted', 100, { phase: 'cleanup',
        deadlineNs: BigInt(Date.now() + 100) * 1_000_000n, targets: [
        { canister: issuer.toText(), attempts: 8, nextNs: 100n }] });
      const original = await s.options.journal.inspect(); let calls = 0;
      const result = await authTest.cleanupUntilFinished({ ...s.options,
        revoke: async () => { calls++; throw new Error('exhausted'); } });
      s.options.journal.close();
      const long = await scheduleSetup('schedule-long-wait', 20_000, { phase: 'cleanup',
        deadlineNs: BigInt(Date.now() + 20_000) * 1_000_000n, targets: [
          { canister: issuer.toText(), attempts: 8, nextNs: 100n }] });
      const wait = await authTest.cleanupOnce({ ...long.options, revoke: async () => { throw new Error('exhausted'); } });
      long.options.journal.close();
      return [result.phase, result.deadlineNs === original.deadlineNs,
        result.destinations[0].attempts, result.destinations[0].confirmedUntilNs, result.issuanceUnknown, calls, wait.delayMs];
    }), ['finished', true, 8, undefined, true, 0, 5_000]);
  });

  await check('scheduler requires committed local logout and propagates storage failure without dispatch', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const journal = await authTest.PublicJournal.open(indexedDB, 'schedule-active',
        { id: 'clock', nowNs: () => BigInt(Date.now()) * 1_000_000n });
      await journal.start({ generation: 'login', caller, issuer }); let calls = 0;
      const options = { journal, generation: 'login', browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)),
        revoke: async () => { calls++; return frames['logout-Revoked-max']; } };
      const active = await failure(authTest.cleanupUntilFinished(options));
      const phase = (await journal.inspect()).phase;
      journal.close();
      const storage = await failure(authTest.cleanupUntilFinished(options));
      return [active, phase, storage, calls];
    }), ['not_current', 'active', 'storage_unavailable', 0]);
  });

  await check('scheduler captures its identity and refuses identity changes while waiting', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const s = await scheduleSetup('schedule-identity');
      await authTest.cleanupOnce({ ...s.options, revoke: async () => frames['logout-Unconfirmed-zero'] });
      let calls = 0;
      const pending = failure(authTest.cleanupUntilFinished({ ...s.options,
        revoke: async () => { calls++; return frames['logout-Revoked-max']; } }));
      setTimeout(() => { s.options.browser.getPrincipal = () => issuer; }, 20);
      const code = await pending; s.options.journal.close(); return [code, calls];
    }), ['identity_changed', 0]);
  });
}
