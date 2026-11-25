import assert from 'node:assert/strict';

export async function multiLogoutCases({ check, a, b }) {
  for (const page of [a, b]) await page.evaluate(() => {
    globalThis.openMulti = async name => {
      now = 100n;
      const journal = await authTest.PublicJournal.open(indexedDB, name, { id: 'clock', nowNs: () => now });
      return { journal, generation: 'login', retryAfterMs: 500,
        browser: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)) };
    };
    globalThis.startMulti = async name => {
      const options = await openMulti(name);
      await options.journal.start({ generation: 'login', caller, issuer });
      const targets = [issuer.toText()];
      for (const byte of [2, 3, 5]) {
        const target = authTest.Principal.fromUint8Array(Uint8Array.of(byte)); targets.push(target.toText());
        await authTest.establishSession({ ...options, issuer, target, scopes: ['read'],
          account: authTest.Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10)),
          transport: {
            issue: async identity => { await authTest.signCall(identity, issuer); return { bytes: frames.ed25519, untilNs: 100_000_000_000n }; },
            renew: async () => { throw Error('unexpected renewal'); },
            reconcile: async () => { throw Error('unexpected lookup'); },
          },
          admit: async (target, identity, bytes) => { await authTest.signCall(identity, target, bytes); return authTest.admissionReply(caller, issuer, 100_000_000_000n); },
        });
      }
      await options.journal.beginLogout('login', 100_000_000_000n);
      return { options, targets };
    };
    globalThis.multiRevoke = async (target, identity, bytes) => {
      await authTest.signCall(identity, target, bytes); return frames['logout-Revoked-max'];
    };
  });

  await check('mixed receiver replies retain progress and allow another tab to reach the next receiver', async () => {
    const before = await a.evaluate(async () => {
      globalThis.multi = await startMulti('multi-mixed'); const calls = [];
      const revoked = await authTest.cleanupOnce({ ...multi.options, revoke: async (...args) => { calls.push(args[0].toText()); return multiRevoke(...args); } });
      await authTest.cleanupOnce({ ...multi.options, revoke: async (target, identity, bytes) => {
        calls.push(target.toText()); await authTest.signCall(identity, target, bytes); return frames['logout-Unconfirmed-zero'];
      } });
      const lost = new Error('reply lost after signing');
      const stopped = await authTest.cleanupOnce({ ...multi.options, revoke: async (target, identity, bytes) => {
        calls.push(target.toText()); await authTest.signCall(identity, target, bytes); throw lost;
      } }).then(() => false, e => e === lost);
      return { status: await multi.options.journal.inspect(), targets: multi.targets, calls, stopped, phase: revoked.status.phase };
    });
    assert.equal(before.stopped, true); assert.equal(before.phase, 'cleanup');
    assert.deepEqual(before.calls, before.targets.slice(0, 3));
    const after = await b.evaluate(async () => {
      const options = await openMulti('multi-mixed'); let calls = [];
      const revoke = async (...args) => { calls.push(args[0].toText()); return multiRevoke(...args); };
      const waiting = await authTest.cleanupOnce({ ...options, revoke });
      const waitingCalls = calls.length;
      now = 5_000_000_100n;
      const finished = await authTest.cleanupUntilFinished({ ...options, revoke });
      const repeat = await options.journal.beginLogout('login', 200_000_000_000n);
      const again = await authTest.cleanupUntilFinished({ ...options, revoke });
      options.journal.close(); return { waiting, waitingCalls, finished, repeat, again, calls };
    });
    assert.equal(after.waiting.kind, 'settled'); assert.equal(after.waitingCalls, 1);
    assert.deepEqual(after.calls, [before.targets[3], before.targets[1], before.targets[2]]);
    assert.deepEqual(after.finished.destinations.map(d => d.attempts), [1, 2, 2, 1]);
    assert.equal(after.finished.phase, 'finished'); assert.equal(after.finished.issuanceUnknown, true);
    assert.equal(after.finished.deadlineNs, 100_000_000_000n);
    assert.deepEqual(after.repeat, after.finished); assert.deepEqual(after.again, after.finished);
    await a.evaluate(() => multi.options.journal.close());
  });

  await check('expired first-tab response cannot overwrite a second tab completing all receivers', async () => {
    await a.evaluate(async () => {
      globalThis.multi = await startMulti('multi-late');
      let entered; const started = new Promise(resolve => { entered = resolve; });
      const gate = new Promise(resolve => { globalThis.releaseMulti = resolve; });
      globalThis.lateMulti = failure(authTest.cleanupOnce({ ...multi.options, revoke: async (...args) => {
        await multiRevoke(...args); entered(); await gate; return frames['logout-Revoked-max'];
      } }));
      await started;
    });
    const done = await b.evaluate(async () => {
      const options = await openMulti('multi-late'); now = 5_000_000_100n;
      const calls = [];
      const finished = await authTest.cleanupUntilFinished({ ...options, revoke: async (...args) => {
        calls.push(args[0].toText()); return multiRevoke(...args);
      } });
      options.journal.close(); return { calls, finished };
    });
    assert.equal(done.calls.length, 4); assert.equal(new Set(done.calls).size, 4);
    assert.deepEqual(done.finished.destinations.map(d => d.attempts), [2, 1, 1, 1]);
    const late = await a.evaluate(async () => {
      now = 5_000_000_100n; releaseMulti();
      const code = await lateMulti; const status = await multi.options.journal.inspect();
      multi.options.journal.close(); return { code, status };
    });
    assert.equal(late.code, 'not_current'); assert.deepEqual(late.status, done.finished);
  });

  await check('mixed confirmations, exhausted retries and unavailable receiver finish only at the original deadline', async () => {
    const result = await a.evaluate(async () => {
      const { options, targets } = await startMulti('multi-deadline'); const calls = [];
      const revoke = async (target, identity, bytes) => {
        const name = target.toText(); calls.push(name); await authTest.signCall(identity, target, bytes);
        if (name === targets[1]) return frames['logout-Unconfirmed-zero'];
        if (name === targets[2]) throw Error('receiver unavailable');
        return frames['logout-Revoked-max'];
      };
      // Drive public one-step cleanup with an explicit host clock; no seeded counts.
      for (let step = 0; step < 30; step++) {
        const snapshot = await options.journal.inspect();
        if (snapshot.destinations[1].attempts === 8 && snapshot.destinations[2].attempts === 8
          && snapshot.destinations[3].confirmedUntilNs !== undefined) break;
        const outcome = await authTest.cleanupOnce({ ...options, revoke }).catch(error => {
          if (error.message !== 'receiver unavailable') throw error;
          return undefined;
        });
        if (outcome?.kind === 'wait') now = outcome.untilNs;
      }
      const before = await options.journal.inspect(); const count = calls.length;
      const wait = await authTest.cleanupOnce({ ...options, revoke });
      now = 100_000_000_000n;
      const finished = await authTest.cleanupUntilFinished({ ...options, revoke });
      options.journal.close();
      const reopened = await authTest.PublicJournal.open(indexedDB, 'multi-deadline', { id: 'clock', nowNs: () => now });
      const retained = await reopened.inspect(); reopened.close();
      return { before, wait, finished, retained, calls, count, targets };
    });
    assert.deepEqual(result.before.destinations.map(d => d.attempts), [1, 8, 8, 1]);
    assert.equal(result.before.phase, 'cleanup'); assert.equal(result.wait.kind, 'wait');
    assert.equal(result.wait.untilNs, 100_000_000_000n);
    assert.equal(result.calls.length, result.count); assert.equal(result.count, 18);
    assert.equal(result.finished.phase, 'finished'); assert.equal(result.finished.deadlineNs, 100_000_000_000n);
    assert.deepEqual(result.finished.destinations.map(d => d.confirmedUntilNs !== undefined), [true, false, false, true]);
    assert.equal(result.finished.issuanceUnknown, true);
    assert.deepEqual(result.retained, result.finished);
  });
}
