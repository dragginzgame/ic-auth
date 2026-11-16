import assert from 'node:assert/strict';

export async function issuanceCases({ check, a, b, prepare }) {
  await check('concurrent issuance has one dispatch owner; reopening remains lookup-only', async () => {
    await a.evaluate(async () => { await seed('issuance', record()); });
    await Promise.all([a, b].map(p => p.evaluate(async () => { globalThis.issuing = await open('issuance'); })));
    assert.equal(await a.evaluate(() => failure(issuing.claimCredential('login', { accountAvailable: false }))), 'account_required');
    const claims = await Promise.all([a, b].map(p => p.evaluate(() => issuing.claimCredential('login', { accountAvailable: true }))));
    assert.deepEqual(claims.map(c => c.kind).sort(), ['issue', 'reconcile']);
    assert(claims.every(c => c.sequence === 0n && c.signer === 'account'));
    await a.evaluate(() => issuing.close());
    await prepare(a);
    assert.deepEqual(await a.evaluate(async () => {
      globalThis.issuing = await open('issuance');
      return [await issuing.claimCredential('login', { accountAvailable: true }),
        await failure(issuing.claimCredential('login', { accountAvailable: false })), (await issuing.inspect()).issuanceUnknown];
    }), [{ kind: 'reconcile', sequence: 0n, signer: 'account' }, 'account_required', true]);
    await Promise.all([a, b].map(p => p.evaluate(() => issuing.close())));
  });

  await check('completion captures exact bytes before await and refuses conflicting retries', async () => {
    const result = await a.evaluate(async () => {
      let j = await open('issuance');
      const bytes = new Uint8Array(frames.ed25519);
      const completion = j.completeCredential('login', 0n, bytes, 1_000n);
      bytes.fill(0); await completion;
      const results = [await failure(j.completeCredential('login', 0n, frames.ed25519, 1_000n)),
        await failure(j.completeCredential('login', 0n, frames.certified, 1_000n)),
        await failure(j.completeCredential('login', 0n, frames.ed25519, 1_001n)),
        await failure(j.completeCredential('login', 1n, frames.ed25519, 1_000n))];
      j.close(); j = await open('issuance');
      const saved = await j.claimCredential('login', { accountAvailable: false });
      const exact = saved.bytes.every((byte, i) => byte === frames.ed25519[i]) && saved.bytes.length === frames.ed25519.length;
      saved.bytes.fill(0);
      const again = await j.claimCredential('login', { accountAvailable: false });
      j.close();
      return { results, exact, detached: again.bytes[0] === 0x49, kind: again.kind, until: again.untilNs };
    });
    assert.deepEqual(result, { results: ['accepted', 'issuance_conflict', 'issuance_conflict', 'issuance_conflict'],
      exact: true, detached: true, kind: 'saved', until: 1_000n });
  });

  await check('renewal uses browser identity, preserves pending work and advances only one sequence', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const j = await open('issuance');
      const renew = await j.claimCredential('login', { accountAvailable: false, refreshThrough: 0n });
      const pending = await j.claimCredential('login', { accountAvailable: false, refreshThrough: 9n });
      const removed = await raw('issuance', b => { const r = b.count('credential'); return () => r.result; });
      await j.completeCredential('login', 1n, frames.certified, 500n);
      const saved = await j.claimCredential('login', { accountAvailable: false, refreshThrough: 0n });
      now = 500n;
      const expired = await j.claimCredential('login', { accountAvailable: false });
      const stale = await failure(j.completeCredential('login', 1n, frames.certified, 500n));
      j.close(); now = 100n;
      return [renew, pending, removed, saved.kind, saved.sequence, expired, stale];
    }), [{ kind: 'renew', sequence: 1n, signer: 'browser' },
      { kind: 'reconcile', sequence: 1n, signer: 'browser' }, 0, 'saved', 1n,
      { kind: 'renew', sequence: 2n, signer: 'browser' }, 'issuance_conflict']);
  });

  await check('unknown sequence and exhausted sequence refuse without discarding evidence', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const results = [];
      for (const [name, issuance, bytes] of [
        ['unknown-sequence', undefined, undefined],
        ['exhausted', { sequence: 0xffff_ffff_ffff_ffffn, untilNs: 100n }, frames.ed25519],
      ]) {
        const r = { ...record(), issuanceUnknown: true, ...(issuance ? { issuance } : {}) };
        await seed(name, r, bytes);
        const j = await open(name);
        results.push(await failure(j.claimCredential('login', { accountAvailable: true })));
        results.push((await j.inspect()).issuanceUnknown); j.close();
      }
      const retained = await raw('exhausted', b => { const r = b.get('credential'); return () => Array.from(r.result); });
      results.push(retained.every((byte, i) => byte === frames.ed25519[i]) && retained.length === frames.ed25519.length);
      return results;
    }), ['issuance_unrecoverable', true, 'sequence_exhausted', true, true]);
  });

  await check('aborted reservations and completions never publish a result or partially paired evidence', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      await seed('issuance-abort', record());
      const j = await open('issuance-abort');
      async function abortWrite(key, operation) {
        const put = IDBObjectStore.prototype.put;
        let intercepted = false;
        IDBObjectStore.prototype.put = function(value, name) {
          const request = put.call(this, value, name);
          if (name === key) {
            intercepted = true;
            request.addEventListener('success', () => this.transaction.abort());
          }
          return request;
        };
        try { return [await failure(operation()), intercepted]; }
        finally { IDBObjectStore.prototype.put = put; }
      }
      const reservation = await abortWrite('login', () => j.claimCredential('login', { accountAvailable: true }));
      const untouched = (await j.inspect()).issuanceUnknown;
      const fresh = await j.claimCredential('login', { accountAvailable: true });
      const completion = await abortWrite('credential', () => j.completeCredential('login', 0n, frames.ed25519, 1_000n));
      const pending = await j.claimCredential('login', { accountAvailable: true });
      const count = await raw('issuance-abort', b => { const r = b.count('credential'); return () => r.result; });
      j.close(); return [reservation, untouched, fresh.kind, completion, pending.kind, count];
    }), [['storage_unavailable', true], false, 'issue', ['storage_unavailable', true], 'reconcile', 0]);
  });

  await check('credential corruption refuses recovery while metadata logout stays available', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const corruptions = [undefined, Uint8Array.of(0), new Uint8Array(1_048_577),
        new Uint8Array([...frames.ed25519, 0]), frames.ed25519.subarray(0, frames.ed25519.length - 1)];
      const results = [];
      for (const [i, bytes] of corruptions.entries()) {
        await seed(`bad-evidence-${i}`, { ...record(), issuanceUnknown: true, issuance: { sequence: 0n, untilNs: 1_000n } }, bytes);
        const j = await open(`bad-evidence-${i}`);
        results.push(await failure(j.claimCredential('login', { accountAvailable: true })));
        results.push(await failure(j.completeCredential('login', 0n, frames.ed25519, 1_000n)));
        results.push((await j.beginLogout('login', 1_000n)).phase); j.close();
      }
      for (const [i, r] of [undefined, record(), { ...record(), issuanceUnknown: true, issuance: { sequence: 0n } }].entries()) {
        await seed(`orphan-evidence-${i}`, r, frames.ed25519);
        const j = await open(`orphan-evidence-${i}`);
        results.push(await failure(j.claimCredential('login', { accountAvailable: true }))); j.close();
      }
      await seed('invalid-completion', { ...record(), issuanceUnknown: true, issuance: { sequence: 0n } });
      const j = await open('invalid-completion');
      for (const bytes of corruptions.slice(1)) results.push(await failure(j.completeCredential('login', 0n, bytes, 1_000n)));
      results.push(await failure(j.completeCredential('login', 0n, frames.ed25519, 0n)));
      results.push(await failure(j.claimCredential('login', { accountAvailable: true, refreshThrough: -1n })));
      results.push((await j.claimCredential('login', { accountAvailable: true })).kind); j.close();
      return results;
    }), [...Array(5).fill(['storage_corrupt', 'storage_corrupt', 'cleanup']).flat(),
      ...Array(3).fill('storage_corrupt'), ...Array(4).fill('invalid_material'), 'configuration', 'configuration', 'reconcile']);
  });

  await check('two-tab completion conflicts keep the winner and logout fences every late reply', async () => {
    await a.evaluate(async () => { await seed('conflicting-completion', { ...record(), issuanceUnknown: true, issuance: { sequence: 0n } }); });
    await Promise.all([a, b].map(p => p.evaluate(async () => { globalThis.issuing = await open('conflicting-completion'); })));
    const outcomes = await Promise.all([a, b].map((p, i) => p.evaluate(i =>
      failure(issuing.completeCredential('login', 0n, i ? frames.certified : frames.ed25519, 1_000n)), i)));
    assert.deepEqual([...outcomes].sort(), ['accepted', 'issuance_conflict']);
    assert.equal(await a.evaluate(async winner => {
      const saved = await issuing.claimCredential('login', { accountAvailable: false });
      const expected = winner ? frames.certified : frames.ed25519;
      return saved.bytes.length === expected.length && saved.bytes.every((v, i) => v === expected[i]);
    }, outcomes.indexOf('accepted')), true);
    await b.evaluate(() => issuing.beginLogout('login', 1_000n));
    assert.deepEqual(await a.evaluate(async () => [
      await failure(issuing.completeCredential('login', 0n, frames.ed25519, 1_000n)),
      await failure(issuing.claimCredential('login', { accountAvailable: true })),
      await raw('conflicting-completion', b => { const r = b.count('credential'); return () => r.result; }),
    ]), ['not_current', 'not_current', 0]);
    await Promise.all([a, b].map(p => p.evaluate(() => issuing.close())));
  });
}
