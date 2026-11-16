import assert from 'node:assert/strict';

export async function cleanupCases({ check, a, b, prepare }) {
  await check('two tabs share a committed cleanup lease and reload consumes the lost attempt', async () => {
    for (const page of [a, b]) await page.evaluate(async () => {
      now = 100n; clockId = 'clock'; globalThis.cleanup = await open('cleanup-tabs');
    });
    await a.evaluate(async () => {
      await cleanup.start({ generation: 'login', caller, issuer });
      await cleanup.beginLogout('login', 100_000_000_000n);
    });
    const claims = await Promise.all([a, b].map(page => page.evaluate(async () => {
      globalThis.claim = await cleanup.claimCleanup('login', caller); return claim;
    })));
    assert.deepEqual(claims.map(c => c.kind).sort(), ['attempt', 'wait']);
    const winner = claims[0].kind === 'attempt' ? a : b;
    const loser = winner === a ? b : a;
    const ticket = claims.find(c => c.kind === 'attempt').ticket;
    assert.equal(ticket.attempt, 1);
    assert.equal(await loser.evaluate(ticket => failure(cleanup.assertCleanup('login', caller, ticket)), ticket), 'not_current');
    await prepare(winner);
    assert.deepEqual(await winner.evaluate(async () => {
      const j = await open('cleanup-tabs');
      const waiting = await j.claimCleanup('login', caller);
      now = 5_000_000_100n;
      const next = await j.claimCleanup('login', caller);
      const snapshot = await raw('cleanup-tabs', bucket => {
        const r = bucket.get('login'); return () => r.result;
      });
      j.close();
      return [waiting.kind, waiting.untilNs, next.ticket.attempt, snapshot.targets[0].attempts, snapshot.busyUntilNs];
    }), ['wait', 5_000_000_100n, 2, 2, 10_000_000_100n]);
    await loser.evaluate(() => cleanup.close());
  });

  await check('only per-destination confirmation finishes cleanup and historical issuance stays unknown', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      now = 100n;
      const r = { ...record(), issuanceUnknown: true };
      r.targets.push({ canister: authTest.Principal.fromUint8Array(Uint8Array.of(2)).toText(), attempts: 0, nextNs: 100n });
      await seed('cleanup-confirm', r);
      const j = await open('cleanup-confirm');
      await j.beginLogout('login', 100_000_000_000n);
      const one = await j.claimCleanup('login', caller);
      const first = await j.completeCleanup('login', caller, one.ticket, { kind: 'revoked', untilNs: 1n });
      const two = await j.claimCleanup('login', caller);
      const last = await j.completeCleanup('login', caller, two.ticket, { kind: 'revoked', untilNs: 0xffff_ffff_ffff_ffffn });
      const done = await j.claimCleanup('login', caller);
      const repeated = await j.beginLogout('login', 200_000_000_000n);
      last.destinations[0].attempts = 7;
      const detached = (await j.inspect()).destinations[0].attempts;
      j.close();
      return [first.phase, first.destinations.map(t => t.confirmedUntilNs), done.kind,
        done.status.issuanceUnknown, done.status.sequence, repeated.deadlineNs, detached];
    }), ['cleanup', [1n, undefined], 'finished', true, undefined, 100_000_000_000n, 1]);
  });

  await check('unconfirmed replies exhaust exactly eight attempts and deadline expiry never confirms', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      now = 100n;
      await seed('cleanup-limit', { ...record(), issuanceUnknown: true, issuance: { sequence: 7n } });
      const j = await open('cleanup-limit');
      await j.beginLogout('login', 100_000_000_000n);
      const attempts = [];
      for (let i = 0; i < 8; i++) {
        const c = await j.claimCleanup('login', caller); attempts.push(c.ticket.attempt);
        await j.completeCleanup('login', caller, c.ticket, { kind: 'unconfirmed', retryAfterMs: 500 });
        const waiting = await j.claimCleanup('login', caller);
        if (waiting.kind !== 'wait') throw new Error('unconfirmed reply made work immediately eligible');
        if (i < 7 && waiting.untilNs !== now + 500_000_000n) throw new Error('wrong backoff');
        now = waiting.untilNs;
      }
      const done = await j.claimCleanup('login', caller);
      j.close();
      return [attempts, done.kind, done.status.destinations[0], done.status.issuanceUnknown, done.status.sequence];
    }), [[1, 2, 3, 4, 5, 6, 7, 8], 'finished', { canister: 'uuc56-gyb', attempts: 8 }, true, 7n]);
  });

  await check('consumed cleanup tickets cannot revive suspended SDK signatures at a reused lease timestamp', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      now = 100n;
      const j = await open('cleanup-signing');
      await j.start({ generation: 'login', caller, issuer });
      await j.addDestination('login', authTest.Principal.fromUint8Array(Uint8Array.of(2)), now);
      await j.beginLogout('login', 2_000_000_000n);
      const first = await j.claimCleanup('login', caller);
      const signing = await authTest.pauseSigning(j, first.ticket);
      await j.completeCleanup('login', caller, first.ticket, { kind: 'unconfirmed', retryAfterMs: 500 });
      const second = await j.claimCleanup('login', caller);
      const copied = await failure(j.assertCleanup('login', caller, { ...second.ticket }));
      const wrong = await failure(j.assertCleanup('login', issuer, second.ticket));
      signing.resume(); const outcome = await signing.result;
      j.close();
      return [first.ticket.untilNs === second.ticket.untilNs, copied, wrong, outcome];
    }), [true, 'not_current', 'not_current', { code: 'not_current', escaped: false }]);
  });

  await check('expired leases, late acknowledgements and clock changes cannot authorize cleanup', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      now = 100n;
      const j = await open('cleanup-expired');
      await j.start({ generation: 'login', caller, issuer });
      const active = await failure(j.claimCleanup('login', caller));
      await j.beginLogout('login', 20_000_000_000n);
      const wrongGeneration = await failure(j.claimCleanup('other', caller));
      const wrongCaller = await failure(j.claimCleanup('login', issuer));
      const c = await j.claimCleanup('login', caller);
      now = c.ticket.untilNs;
      const expired = await failure(j.assertCleanup('login', caller, c.ticket));
      const late = await failure(j.completeCleanup('login', caller, c.ticket, { kind: 'revoked', untilNs: 99n }));
      const next = await j.claimCleanup('login', caller);
      now--;
      const rollback = await failure(j.assertCleanup('login', caller, next.ticket));
      now = 20_000_000_000n;
      const end = await j.claimCleanup('login', caller); j.close();
      return [active, wrongGeneration, wrongCaller, expired, late, rollback, end.kind, end.status.destinations[0].confirmedUntilNs];
    }), [...Array(5).fill('not_current'), 'clock_changed', 'finished', undefined]);
  });

  await check('aborted reservation dispatches no ticket and aborted settlement retains uncertainty', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      now = 100n;
      const j = await open('cleanup-abort');
      await j.start({ generation: 'login', caller, issuer });
      await j.beginLogout('login', 100_000_000_000n);
      const put = IDBObjectStore.prototype.put;
      const abortWrites = () => {
        IDBObjectStore.prototype.put = function(value, key) {
          const r = put.call(this, value, key);
          if (key === 'login') r.addEventListener('success', () => this.transaction.abort());
          return r;
        };
      };
      let reservation, settlement;
      abortWrites();
      try { reservation = await failure(j.claimCleanup('login', caller)); }
      finally { IDBObjectStore.prototype.put = put; }
      const afterReservation = await j.inspect();
      const c = await j.claimCleanup('login', caller);
      abortWrites();
      try { settlement = await failure(j.completeCleanup('login', caller, c.ticket, { kind: 'revoked', untilNs: 1n })); }
      finally { IDBObjectStore.prototype.put = put; }
      const dead = await failure(j.assertCleanup('login', caller, c.ticket));
      const afterSettlement = await j.inspect(); j.close();
      const reopened = await open('cleanup-abort');
      const waiting = await reopened.claimCleanup('login', caller); reopened.close();
      return [reservation, afterReservation.destinations[0].attempts, settlement, dead,
        afterSettlement.destinations[0].attempts, afterSettlement.destinations[0].confirmedUntilNs, waiting.kind];
    }), ['storage_unavailable', 0, 'storage_unavailable', 'not_current', 1, undefined, 'wait']);
  });

  await check('u64 deadline edges and malformed acknowledgements retain bounded unconfirmed state', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const max = 0xffff_ffff_ffff_ffffn;
      const results = [];
      for (const [i, result] of [
        { kind: 'revoked', untilNs: 0n }, { kind: 'revoked', untilNs: max + 1n },
        { kind: 'unconfirmed', retryAfterMs: 499 }, { kind: 'unconfirmed', retryAfterMs: 1_500 },
      ].entries()) {
        now = max - 1_000n;
        const j = await open(`cleanup-bounds-${i}`);
        await j.start({ generation: 'login', caller, issuer });
        await j.beginLogout('login', max);
        const c = await j.claimCleanup('login', caller);
        results.push(c.ticket.untilNs === max);
        results.push(await failure(j.completeCleanup('login', caller, c.ticket, result)));
        results.push(await failure(j.assertCleanup('login', caller, c.ticket)));
        now = max;
        const done = await j.claimCleanup('login', caller);
        results.push(done.kind === 'finished' && done.status.destinations[0].confirmedUntilNs === undefined);
        j.close();
      }
      now = 100n;
      return results;
    }), Array(4).fill([true, 'configuration', 'not_current', true]).flat());
  });
}
