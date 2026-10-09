import { test } from 'node:test';
import assert from 'node:assert/strict';
import { Principal } from '@icp-sdk/core/principal';
import { ClientError, TokenSessionClient, type IssuerOperations, type PrepareResult, type RetrieveResult, type Session, type ClientErrorCode } from '../src/index.js';
import { deferred, harness, principal, scope, second } from './support.js';

const error = (code: ClientErrorCode) => (value: unknown) => value instanceof ClientError && value.code === code;

test('concurrent callers share issuance; cached/caller bytes remain detached', async () => {
  const h = harness();
  const [a, b] = await Promise.all([h.client.token(h.configured), h.client.token(h.configured)]);
  assert.equal(h.state.prepares, 1); assert.equal(h.state.retrieves, 1);
  a.claims.nonce[0] = 9;
  assert.equal(b.claims.nonce[0], 0);
  assert.equal((await h.client.token(h.configured)).claims.nonce[0], 0);
});

test('concurrent login is single flight', async () => {
  const h = harness(); h.sessions.value = undefined;
  const pending = deferred<Session>(); let logins = 0;
  h.sessions.connect = () => { logins++; return pending.promise; };
  const a = h.client.token(h.configured); const b = h.client.token(h.configured);
  h.sessions.value = { generation: 'connected', identity: principal(2) };
  pending.resolve(h.sessions.value);
  await Promise.all([a, b]); assert.equal(logins, 1); assert.equal(h.state.prepares, 1);
});

test('unknown prepare reply survives a new client and reconciles exact request', async () => {
  const h = harness(); const base = h.options.issuer;
  h.options.issuer = (session, fixed) => {
    const operations = base(session, fixed);
    return { ...operations, prepare: async (request, signal) => { await operations.prepare(request, signal); throw new Error('lost reply'); } };
  };
  const first = new TokenSessionClient(h.options);
  await assert.rejects(first.token(h.configured), error('prepare_uncertain'));
  await new TokenSessionClient(h.options).token(h.configured);
  assert.equal(h.state.prepares, 1); assert.equal(h.state.reconciles, 1); assert.equal(h.state.ids, 1);
  assert.equal(h.requests[0]?.ttl_ns, 10n * second); assert.deepEqual(h.sessions.invalidations, []);
});

test('uncertain prepare keeps its original exclusive deadline and intent across reloads', async () => {
  const h = harness(); const base = h.options.issuer;
  let key: string | undefined;
  const options = { ...h.options,
    store: {
      read: (value: string) => h.store.read(value),
      write: (value: string, revision: bigint, entry: Parameters<typeof h.store.write>[2]) => {
        key = value;
        return h.store.write(value, revision, entry);
      },
    },
    issuer: (session: Session, fixed: Parameters<typeof base>[1]): IssuerOperations => {
      const operations = base(session, fixed);
      return { ...operations,
        prepare: async (request, signal) => {
          await operations.prepare(request, signal);
          throw new Error('reply lost after issuance');
        },
        reconcile: async () => { h.state.reconciles++; return { kind: 'unresolved' }; },
      };
    },
  };
  await assert.rejects(new TokenSessionClient(options).token(h.configured), error('prepare_uncertain'));
  assert.ok(key);
  const original = await h.store.read(key);
  assert.ok(original.entry && original.entry.kind === 'prepare');

  // A fresh instance and a longer configured operation lifetime cannot restart
  // the deadline of a possibly completed issuance retained in shared storage.
  const reloaded = { ...options, operationTtlNs: 100n * second };
  h.state.now = original.entry.deadlineNs - 1n;
  await assert.rejects(new TokenSessionClient(reloaded).token(h.configured), error('prepare_uncertain'));
  assert.equal(h.state.reconciles, 1);
  h.state.now = original.entry.deadlineNs;
  for (let reload = 0; reload < 2; reload++) {
    await assert.rejects(new TokenSessionClient(reloaded).token(h.configured), error('retrieval_expired'));
  }
  assert.equal(h.state.prepares, 1); assert.equal(h.state.reconciles, 1);
  assert.equal(h.state.retrieves, 0); assert.equal(h.state.ids, 1);
  assert.deepEqual(await h.store.read(key), original);
  assert.deepEqual(h.sessions.invalidations, []);
});

for (const phase of ['prepare', 'reconcile'] as const) {
  test(`${phase} replies at or after the deadline preserve intent across reloads`, async () => {
    for (const kind of ['prepared', 'ttlUnavailable', 'sessionInvalid'] as const) {
      for (const lateBy of [0n, 1n]) {
        const h = harness(); const base = h.options.issuer;
        const started = deferred<PrepareResult>(); const reply = deferred<PrepareResult>();
        let key: string | undefined;
        let completed: PrepareResult | undefined;
        // Keep the issuer's retrieval material valid across the earlier client
        // deadline, isolating the cutoff from claim/retrieval expiry checks.
        const options = { ...h.options, operationTtlNs: second / 2n,
          store: {
            read: (value: string) => h.store.read(value),
            write: (value: string, revision: bigint, entry: Parameters<typeof h.store.write>[2]) => {
              key = value;
              return h.store.write(value, revision, entry);
            },
          },
          issuer: (session: Session, fixed: Parameters<typeof base>[1]): IssuerOperations => {
            const operations = base(session, fixed);
            return { ...operations,
              prepare: async (request, signal) => {
                completed = await operations.prepare(request, signal);
                if (phase === 'reconcile') throw new Error('original reply lost');
                started.resolve(completed);
                return reply.promise;
              },
              reconcile: async () => {
                h.state.reconciles++;
                assert.ok(completed);
                started.resolve(completed);
                return reply.promise;
              },
            };
          },
        };
        const client = new TokenSessionClient(options);
        if (phase === 'reconcile') {
          await assert.rejects(client.token(h.configured), error('prepare_uncertain'));
        }
        const flight = client.token(h.configured);
        const prepared = await started.promise;
        assert.ok(key);
        const original = await h.store.read(key);
        assert.ok(original.entry && original.entry.kind === 'prepare');
        h.state.now = original.entry.deadlineNs + lateBy;
        reply.resolve(kind === 'prepared' ? prepared : { kind });
        await assert.rejects(flight, error('retrieval_expired'));
        assert.deepEqual(await h.store.read(key), original);

        const reloaded = new TokenSessionClient({ ...options, operationTtlNs: 100n * second });
        await assert.rejects(reloaded.token(h.configured), error('retrieval_expired'));
        assert.equal(h.state.prepares, 1); assert.equal(h.state.ids, 1);
        assert.equal(h.state.reconciles, phase === 'reconcile' ? 1 : 0);
        assert.equal(h.state.retrieves, 0);
        assert.deepEqual(await h.store.read(key), original);
        assert.deepEqual(h.sessions.invalidations, []);
      }
    }
  });
}

test('prepare and reconciliation replies just before the deadline can complete retrieval', async () => {
  for (const phase of ['prepare', 'reconcile'] as const) {
    const h = harness({ operationTtlNs: second / 2n }); const base = h.options.issuer;
    const deadline = h.state.now + h.options.operationTtlNs;
    let completed: PrepareResult | undefined;
    const client = new TokenSessionClient({ ...h.options,
      issuer: (session, fixed) => {
        const operations = base(session, fixed);
        return { ...operations,
          prepare: async (request, signal) => {
            completed = await operations.prepare(request, signal);
            if (phase === 'reconcile') throw new Error('original reply lost');
            h.state.now = deadline - 1n;
            return completed;
          },
          reconcile: async () => {
            h.state.reconciles++;
            assert.ok(completed);
            h.state.now = deadline - 1n;
            return completed;
          },
        };
      },
    });
    if (phase === 'reconcile') await assert.rejects(client.token(h.configured), error('prepare_uncertain'));
    const token = await client.token(h.configured);
    assert.deepEqual(token.claims, h.issued.get('login-1')!.claims);
    assert.equal(h.state.prepares, 1); assert.equal(h.state.ids, 1);
    assert.equal(h.state.retrieves, 1); assert.equal(h.state.reconciles, phase === 'reconcile' ? 1 : 0);
  }
});

for (const phase of ['initial-read', 'contended-reservation', 'ttl-clear'] as const) {
  test(`${phase} completing at or after expiry cannot reserve another issuance intent`, async () => {
    for (const lateBy of [0n, 1n]) {
      const h = harness(); const base = h.options.issuer;
      const deadline = h.state.now + h.options.operationTtlNs;
      let delayed = false; let key: string | undefined; let writes = 0;
      let retained: Awaited<ReturnType<typeof h.store.read>> | undefined;
      const client = new TokenSessionClient({ ...h.options,
        store: {
          read: async value => {
            key = value;
            const snapshot = await h.store.read(value);
            if (phase === 'initial-read' && !delayed) {
              delayed = true; retained = snapshot; h.state.now = deadline + lateBy;
            }
            return snapshot;
          },
          write: async (value, revision, entry) => {
            writes++;
            if (phase === 'contended-reservation' && !delayed) {
              // Another writer advances the empty slot's CAS revision while
              // this caller's reservation loses the race.
              assert.equal(await h.store.write(value, revision, undefined), true);
              delayed = true; retained = await h.store.read(value);
              h.state.now = deadline + lateBy;
              return false;
            }
            const committed = await h.store.write(value, revision, entry);
            if (phase === 'ttl-clear' && entry === undefined && committed) {
              delayed = true; retained = await h.store.read(value);
              h.state.now = deadline + lateBy;
            }
            return committed;
          },
        },
        issuer: (session, fixed) => ({ ...base(session, fixed), prepare: async () => {
          h.state.prepares++;
          return { kind: 'ttlUnavailable' };
        } }),
      });
      await assert.rejects(client.token(h.configured), error('retrieval_expired'));
      assert.ok(delayed && key && retained);
      assert.deepEqual(await h.store.read(key), retained);
      assert.equal(retained.entry, undefined);
      assert.equal(h.state.ids, phase === 'initial-read' ? 0 : 1);
      assert.equal(writes, phase === 'initial-read' ? 0 : phase === 'ttl-clear' ? 2 : 1);
      assert.equal(h.state.prepares, phase === 'ttl-clear' ? 1 : 0);
      assert.equal(h.state.reconciles, 0); assert.equal(h.state.retrieves, 0);
      assert.deepEqual(h.sessions.invalidations, []);
    }
  });
}

test('storage completing just before expiry can reserve and finish issuance', async () => {
  for (const phase of ['initial-read', 'ttl-clear'] as const) {
    const h = harness(); const base = h.options.issuer;
    const deadline = h.state.now + h.options.operationTtlNs;
    let delayed = false; let rejected = false;
    const client = new TokenSessionClient({ ...h.options,
      store: {
        read: async key => {
          const snapshot = await h.store.read(key);
          if (phase === 'initial-read' && !delayed) { delayed = true; h.state.now = deadline - 1n; }
          return snapshot;
        },
        write: async (key, revision, entry) => {
          const committed = await h.store.write(key, revision, entry);
          if (phase === 'ttl-clear' && entry === undefined && committed) {
            delayed = true; h.state.now = deadline - 1n;
          }
          return committed;
        },
      },
      issuer: (session, fixed) => {
        const operations = base(session, fixed);
        return { ...operations, prepare: async (request, signal) => {
          if (phase === 'ttl-clear' && !rejected) {
            rejected = true; h.state.prepares++; return { kind: 'ttlUnavailable' };
          }
          return operations.prepare(request, signal);
        } };
      },
    });
    const token = await client.token(h.configured);
    assert.ok(delayed);
    assert.deepEqual(token.claims, h.issued.get('login-1')!.claims);
    assert.equal(h.state.ids, phase === 'ttl-clear' ? 2 : 1);
    assert.equal(h.state.prepares, phase === 'ttl-clear' ? 2 : 1);
    assert.equal(h.state.retrieves, 1);
  }
});

test('lost retrieval reply reuses prepared material without allocating another prepare', async () => {
  const h = harness(); const base = h.options.issuer; let lost = true;
  const client = new TokenSessionClient({ ...h.options, issuer: (session, fixed) => {
    const operations = base(session, fixed);
    return { ...operations, retrieve: async (hash, signal) => {
      if (lost) { lost = false; throw new Error('not an authentication failure'); }
      return operations.retrieve(hash, signal);
    } };
  } });
  await assert.rejects(client.token(h.configured), error('transport'));
  await client.token(h.configured); assert.equal(h.state.prepares, 1); assert.equal(h.state.ids, 1);
  assert.deepEqual(h.sessions.invalidations, []);
});

test('only definitive no-effect TTL rejection selects a smaller TTL', async () => {
  const h = harness(); const base = h.options.issuer; let attempts = 0;
  const seen: bigint[] = [];
  const client = new TokenSessionClient({ ...h.options, issuer: (session, fixed) => {
    const operations = base(session, fixed);
    return { ...operations, prepare: async (request, signal) => {
      seen.push(request.ttl_ns);
      return ++attempts === 1 ? { kind: 'ttlUnavailable' } : operations.prepare(request, signal);
    } };
  } });
  await client.token(h.configured);
  assert.deepEqual(seen, [10n * second, 5n * second]); assert.equal(h.state.ids, 2);
  const rejected = harness({ issuer: () => ({ prepare: async () => ({ kind: 'ttlUnavailable' }),
    reconcile: async () => ({ kind: 'unresolved' }), retrieve: async () => ({ kind: 'pending' }) }) });
  await assert.rejects(rejected.client.token(rejected.configured), error('ttl_exhausted'));
  assert.equal(rejected.state.ids, 2);
});

test('timeout retains the uncertain intent and never silently retries prepare', async () => {
  let prepares = 0; let reconciles = 0;
  const h = harness({ callTimeoutMs: 10, issuer: () => ({
    prepare: () => { prepares++; return new Promise(() => {}); },
    reconcile: async () => { reconciles++; return { kind: 'unresolved' }; }, retrieve: async () => ({ kind: 'pending' }),
  }) });
  await assert.rejects(h.client.token(h.configured), error('prepare_uncertain'));
  await assert.rejects(h.client.token(h.configured), error('prepare_uncertain'));
  assert.equal(prepares, 1); assert.equal(reconciles, 1); assert.equal(h.state.ids, 1);
});

test('polling is bounded and resumption retains the prepared request', async () => {
  const h = harness(); const base = h.options.issuer; let polls = 0; let pending = true;
  const client = new TokenSessionClient({ ...h.options, issuer: (session, fixed) => ({
    ...base(session, fixed), retrieve: async () => {
      polls++; return pending ? { kind: 'pending' } : { kind: 'ready', value: h.issued.get(session.generation)! };
    },
  }) });
  await assert.rejects(client.token(h.configured), error('polling_exhausted'));
  assert.equal(polls, 3); pending = false;
  await client.token(h.configured); assert.equal(h.state.prepares, 1);
});

test('retrieval expiry is exclusive and cannot trigger a replacement issuance', async () => {
  const h = harness(); const base = h.options.issuer;
  const client = new TokenSessionClient({ ...h.options, issuer: (session, fixed) => ({
    ...base(session, fixed), retrieve: async () => { h.state.now += second; return { kind: 'pending' }; },
  }) });
  await assert.rejects(client.token(h.configured), error('retrieval_expired'));
  await assert.rejects(client.token(h.configured), error('retrieval_expired'));
  assert.equal(h.state.prepares, 1); assert.equal(h.state.ids, 1);
});

test('renewal near expiry is single flight and expiration obtains fresh material', async () => {
  const h = harness(); const first = await h.client.token(h.configured);
  h.state.now = first.claims.expires_at_ns - second;
  const [a, b] = await Promise.all([h.client.token(h.configured), h.client.token(h.configured)]);
  assert.equal(h.state.prepares, 2); assert.equal(a.claims.expires_at_ns, b.claims.expires_at_ns);
  h.state.now = a.claims.expires_at_ns;
  await h.client.token(h.configured); assert.equal(h.state.prepares, 3);
});

test('late session rejection cannot invalidate a newer login', async () => {
  const h = harness(); const late = deferred<RetrieveResult>(); const started = deferred<void>();
  const base = h.options.issuer;
  const client = new TokenSessionClient({ ...h.options, issuer: (session, fixed) => ({
    ...base(session, fixed), retrieve: () => { started.resolve(); return late.promise; },
  }) });
  const old = client.token(h.configured); await started.promise;
  h.sessions.value = { generation: 'login-2', identity: principal(2) };
  late.resolve({ kind: 'sessionInvalid' });
  await assert.rejects(old, error('stale_generation'));
  assert.equal(h.sessions.value.generation, 'login-2'); assert.deepEqual(h.sessions.invalidations, []);
  assert.equal(await client.invalidate('login-1'), false);
  await h.client.token(h.configured); assert.equal(h.sessions.value.generation, 'login-2');
});

test('cross-tab logout invalidates an in-flight prepare', async () => {
  const h = harness(); const response = deferred<PrepareResult>(); const started = deferred<void>();
  const base = h.options.issuer;
  const client = new TokenSessionClient({ ...h.options, issuer: (session, fixed) => ({
    ...base(session, fixed), prepare: async (request, signal) => {
      await base(session, fixed).prepare(request, signal); started.resolve();
      return response.promise;
    },
  }) });
  const flight = client.token(h.configured); await started.promise;
  await client.logout(); response.resolve({ kind: 'sessionInvalid' });
  await assert.rejects(flight, error('stale_generation')); assert.equal(h.sessions.value, undefined);
});

test('typed session rejection invalidates only its current generation', async () => {
  const h = harness({ issuer: () => ({ prepare: async () => ({ kind: 'sessionInvalid' }),
    reconcile: async () => ({ kind: 'unresolved' }), retrieve: async () => ({ kind: 'pending' }) }) });
  await assert.rejects(h.client.token(h.configured), error('unauthenticated'));
  assert.deepEqual(h.sessions.invalidations, ['login-1']); assert.equal(h.sessions.value, undefined);
});

test('issuer/network/audience/policy/grant/extension/session contexts isolate caches', async () => {
  const h = harness(); await h.client.token(h.configured);
  const variants = [
    { ...scope(), issuer: Principal.fromUint8Array(new Uint8Array([2, 1])) },
    { ...scope(), network: 'mainnet' }, { ...scope(), policyId: 'policy-2' },
    { ...scope(), audience: { Fleet: { canonical_network_id: 'c'.repeat(64), fleet_id: 'b'.repeat(64) } } },
    { ...scope(), audience: { Fleet: { canonical_network_id: 'a'.repeat(64), fleet_id: 'c'.repeat(64) } } },
    { ...scope(), grants: [{ target: 'users', scopes: ['write'] }] },
    { ...scope(), ext: [new Uint8Array([7])] as [Uint8Array] },
  ];
  for (const configured of variants) await h.client.token(configured);
  h.sessions.value = { generation: 'login-2', identity: principal(1) };
  await h.client.token(h.configured);
  assert.equal(h.state.prepares, variants.length + 2);
});

test('wrong issuer/identity/audience and altered retrieved claims are rejected', async () => {
  for (const mutation of ['issuer', 'subject', 'audience', 'nonce']) {
    const h = harness(); const base = h.options.issuer;
    const client = new TokenSessionClient({ ...h.options, issuer: (session, fixed) => {
      const operations = base(session, fixed);
      return { ...operations, retrieve: async (hash, signal) => {
        const result = await operations.retrieve(hash, signal);
        if (result.kind !== 'ready') throw new Error('fixture');
        if (mutation === 'issuer') result.value.claims.issuer_pid = principal(2).getPrincipal();
        if (mutation === 'subject') result.value.claims.subject = principal(2).getPrincipal();
        if (mutation === 'audience') result.value.claims.aud = { Fleet: { canonical_network_id: 'c'.repeat(64), fleet_id: 'b'.repeat(64) } };
        if (mutation === 'nonce') result.value.claims.nonce[0] = 42;
        return result;
      } };
    } });
    await assert.rejects(client.token(h.configured), error('invalid_material'));
  }
});

test('storage refusal occurs before issuer dispatch', async () => {
  const h = harness();
  const client = new TokenSessionClient({ ...h.options, store: { read: key => h.store.read(key), write: async () => { throw new Error('disk full'); } } });
  await assert.rejects(client.token(h.configured), /disk full/); assert.equal(h.state.prepares, 0);
});

test('anonymous identity, noncanonical grants and invalid configuration fail before dispatch', async () => {
  const h = harness();
  await assert.rejects(h.client.token({ ...scope(), grants: [{ target: 'users', scopes: ['write', 'read'] }] }), error('configuration'));
  assert.throws(() => new TokenSessionClient({ ...h.options, ttlCandidatesNs: [second, second] }), error('configuration'));
  h.sessions.value = { generation: 'anonymous', identity: { getPrincipal: () => Principal.anonymous(), transformRequest: async request => request } };
  await assert.rejects(h.client.token(h.configured), error('unauthenticated')); assert.equal(h.state.prepares, 0);
});

test('two clients sharing CAS storage never dispatch a second prepare', async () => {
  const h = harness(); const base = h.options.issuer;
  const response = deferred<void>(); const started = deferred<void>();
  const options = { ...h.options, issuer: (session: Session, fixed: ReturnType<typeof scope>): IssuerOperations => ({
    ...base(session, fixed), prepare: async (request, signal) => {
      const result = await base(session, fixed).prepare(request, signal); started.resolve();
      await response.promise; return result;
    },
    reconcile: async () => ({ kind: 'unresolved' }),
  }) };
  const first = new TokenSessionClient(options).token(h.configured);
  await started.promise;
  await assert.rejects(new TokenSessionClient(options).token(h.configured), error('prepare_uncertain'));
  response.resolve(); await first;
  assert.equal(h.state.prepares, 1); assert.equal(h.state.ids, 1);
});

test('corrupt durable candidate cannot select a replacement TTL', async () => {
  const h = harness();
  const operations: IssuerOperations = { prepare: async () => { throw new Error('lost'); },
    reconcile: async () => ({ kind: 'unresolved' }), retrieve: async () => ({ kind: 'pending' }) };
  let corrupt = false;
  const client = new TokenSessionClient({ ...h.options, issuer: () => operations, store: {
    write: (key, revision, entry) => h.store.write(key, revision, entry),
    read: async key => {
      const saved = await h.store.read(key);
      if (corrupt && saved.entry?.kind === 'prepare') saved.entry.candidate = 99;
      return saved;
    },
  } });
  await assert.rejects(client.token(h.configured), error('prepare_uncertain'));
  corrupt = true;
  await assert.rejects(client.token(h.configured), error('invalid_material'));
  assert.equal(h.state.ids, 1);
});

test('clock rollback and deadline overflow are configuration errors', async () => {
  const h = harness(); await h.client.token(h.configured);
  h.state.now--;
  await assert.rejects(h.client.token(h.configured), error('configuration'));
  const overflowing = harness(); overflowing.state.now = (1n << 64n) - 1n;
  await assert.rejects(overflowing.client.token(overflowing.configured), error('configuration'));
});
