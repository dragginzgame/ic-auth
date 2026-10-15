import { Endpoint, SubmitRequestType, ReadRequestType, Expiry } from '@icp-sdk/core/agent';
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { IDBFactory } from 'fake-indexeddb';
import { Principal } from '@icp-sdk/core/principal';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { IndexedDbLogoutStore, PortableLogoutClient, ClientError, type LogoutResult, type LogoutTransport } from '../src/index.js';

const MS = 1_000_000n;
const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(1));
const other = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(2));
const issuer = Principal.fromUint8Array(new Uint8Array([1]));
const receiver = Principal.fromUint8Array(new Uint8Array([2]));
const revoked: LogoutResult = { Revoked: { deny_until_ns: 86_400_000_000_000n } };

async function fixture(revoke: LogoutTransport['revoke']) {
  const factory = new IDBFactory();
  const store = await IndexedDbLogoutStore.open(factory, 'logout');
  const state = { now: 1_000n * MS, disposed: 0 };
  const clock = { id: 'qualified-test-clock', nowNs: () => state.now,
    wait: async (ms: number) => { state.now += BigInt(Math.ceil(ms)) * MS; } };
  const transport = { revoke, dispose: async () => { state.disposed++; } };
  const client = new PortableLogoutClient(store, clock, transport, () => 0);
  await client.start('login-a', identity, issuer);
  return { factory, store, state, clock, transport, client };
}

test('logout commits invalidation before fan-out and retries Capacity after late admission', async () => {
  const calls: string[] = [];
  let attempts = 0;
  const f = await fixture(async (target, caller) => {
    assert.equal((await f.client.status('login-a')).phase, 'cleanup');
    assert.equal(caller.getPrincipal().toText(), identity.getPrincipal().toText());
    calls.push(target.toText());
    if (target.toText() === receiver.toText() && attempts++ === 0) return { Unavailable: { Capacity: null } };
    return revoked;
  });
  try {
    await f.client.destination('login-a', receiver, f.state.now + 2_000n * MS);
    await f.client.issuing('login-a');
    await f.client.begin('login-a', f.state.now + 10_000n * MS);
    await assert.rejects(f.client.destination('login-a', receiver, f.state.now), ClientError);
    await assert.rejects(f.client.issuing('login-a'), ClientError);
    const status = await f.client.cleanup('login-a', identity);
    assert.equal(status.phase, 'finished');
    assert.equal(status.issuanceUnknown, true);
    assert.equal(status.targets.filter(t => t.confirmedUntilNs !== undefined).length, 2);
    assert.deepEqual(calls, [issuer.toText(), receiver.toText(), receiver.toText()]);
    assert.equal(f.state.disposed, 1);
  } finally { f.store.close(); }
});

test('proof expiry does not imply logout: reserve final attempt after admission window, retain issuer uncertainty', async () => {
  const times: bigint[] = [];
  const f = await fixture(async target => {
    if (target.toText() === receiver.toText()) times.push(f.state.now);
    return { Unavailable: { Capacity: null } };
  });
  try {
    const expiry = f.state.now + 20_000n * MS;
    const deadline = f.state.now + 30_000n * MS;
    await f.client.destination('login-a', receiver, expiry);
    await f.client.issuing('login-a');
    await f.client.begin('login-a', deadline);
    await f.client.begin('login-a', deadline + 100_000n * MS);
    const result = await f.client.cleanup('login-a', identity);
    assert.equal(result.deadlineNs, deadline);
    assert.equal(result.phase, 'finished');
    assert.equal(result.issuanceUnknown, true);
    assert.equal(times.length, 8);
    assert.ok(times.at(-1)! > expiry);
    assert.ok(times.at(-1)! < deadline);
    assert.ok(result.targets.every(t => t.confirmedUntilNs === undefined && t.attempts <= 8));
  } finally { f.store.close(); }
});

test('crash/reopen preserves cleanup generation, deadline and prior attempt consumption', async () => {
  const f = await fixture(async () => revoked);
  await f.client.begin('login-a', f.state.now + 10_000n * MS);
  await f.store.transact(record => {
    assert.ok(record);
    record.targets[0]!.attempts = 1;
    record.busyUntilNs = f.state.now + 5_000n * MS;
    record.targets[0]!.nextNs = record.busyUntilNs;
    return { record, value: undefined };
  });
  f.store.close();
  const store = await IndexedDbLogoutStore.open(f.factory, 'logout');
  try {
    const resumed = new PortableLogoutClient(store, f.clock, f.transport, () => 0);
    await assert.rejects(resumed.start('login-b', other, issuer), ClientError);
    await assert.rejects(resumed.destination('login-a', receiver, f.state.now), ClientError);
    await assert.rejects(resumed.cleanup('login-a', other), ClientError);
    const result = await resumed.cleanup('login-a', identity);
    assert.equal(result.targets[0]!.attempts, 2);
    assert.ok(result.targets[0]!.confirmedUntilNs);
    assert.ok(f.state.now >= 6_000n * MS);
  } finally { store.close(); }
});

test('missing handle and lost replies remain unknown, never implicit success', async () => {
  let calls = 0;
  const f = await fixture(async () => { calls++; throw new Error('reply lost'); });
  try {
    await f.client.issuing('login-a');
    await f.client.begin('login-a', f.state.now + 10_000n * MS);
    const result = await f.client.cleanup('login-a', undefined);
    assert.equal(calls, 0);
    assert.equal(result.targets[0]!.confirmedUntilNs, undefined);
    assert.equal(result.issuanceUnknown, true);
    assert.equal(result.phase, 'cleanup');
    assert.equal(f.state.disposed, 0);
    f.state.now = result.deadlineNs;
    const expired = await f.client.cleanup('login-a', undefined);
    assert.equal(expired.phase, 'finished');
    assert.equal(expired.targets[0]!.confirmedUntilNs, undefined);
    assert.equal(f.state.disposed, 1);
  } finally { f.store.close(); }
  const lost = await fixture(async () => { throw new Error('unknown'); });
  try {
    await lost.client.begin('login-a', lost.state.now + 10_000n * MS);
    const result = await lost.client.cleanup('login-a', identity);
    assert.equal(result.targets[0]!.confirmedUntilNs, undefined);
    assert.ok(result.targets[0]!.attempts <= 8);
  } finally { lost.store.close(); }
});

test('journal transaction failure cannot publish logout or partially add a target', async () => {
  const f = await fixture(async () => { assert.fail('must not dispatch'); });
  try {
    await assert.rejects(f.store.transact(record => {
      assert.ok(record); record.phase = 'cleanup'; throw new Error('abort');
    }));
    assert.equal((await f.client.status('login-a')).phase, 'active');
    await assert.rejects(f.client.begin('login-a', f.state.now + 1n), ClientError);
    await assert.rejects(f.client.cleanup('login-a', identity), ClientError);
    await assert.rejects(f.client.status('other-generation'), ClientError);
    f.state.now--;
    await assert.rejects(f.client.status('login-a'), ClientError);
  } finally { f.store.close(); }
});

test('competing tabs cannot claim another call while the retained lease is live', async () => {
  let complete!: (value: LogoutResult) => void;
  let dispatched!: () => void;
  const started = new Promise<void>(resolve => { dispatched = resolve; });
  let calls = 0;
  const f = await fixture(async () => {
    calls++; dispatched();
    return new Promise<LogoutResult>(resolve => { complete = resolve; });
  });
  const store = await IndexedDbLogoutStore.open(f.factory, 'logout');
  try {
    await f.client.begin('login-a', f.state.now + 10_000n * MS);
    const first = f.client.cleanup('login-a', identity);
    await started;
    let resume!: () => void;
    let waiting!: () => void;
    const waited = new Promise<void>(resolve => { waiting = resolve; });
    const clock = { ...f.clock, wait: async () => { waiting(); await new Promise<void>(resolve => { resume = resolve; }); } };
    const second = new PortableLogoutClient(store, clock, f.transport, () => 0).cleanup('login-a', identity);
    await waited;
    assert.equal(calls, 1);
    complete(revoked);
    await first;
    resume();
    assert.equal((await second).phase, 'finished');
    assert.equal(calls, 1);
  } finally { store.close(); f.store.close(); }
});

test('a suspended callback cannot restore login or change status after cleanup deadline', async () => {
  const f = await fixture(async () => {
    f.state.now += 20_000n * MS;
    return revoked;
  });
  try {
    await f.client.begin('login-a', f.state.now + 10_000n * MS);
    const result = await f.client.cleanup('login-a', identity);
    assert.equal(result.phase, 'finished');
    assert.equal(result.targets[0]!.confirmedUntilNs, undefined);
    await assert.rejects(f.client.issuing('login-a'), ClientError);
    assert.equal(f.state.disposed, 1);
  } finally { f.store.close(); }
});

test('durable target capacity is shared across connections and does not evict evidence', async () => {
  const f = await fixture(async () => revoked);
  const store = await IndexedDbLogoutStore.open(f.factory, 'logout');
  try {
    const second = new PortableLogoutClient(store, f.clock, f.transport);
    for (let i = 2; i <= 64; i++) {
      await f.client.destination('login-a', Principal.fromUint8Array(new Uint8Array([i, 1])), f.state.now + MS);
    }
    await assert.rejects(second.destination('login-a', Principal.fromUint8Array(new Uint8Array([65, 1])), f.state.now + MS),
      (error: unknown) => error instanceof ClientError && error.code === 'storage_capacity');
    assert.equal((await second.status('login-a')).targets.length, 64);
  } finally { store.close(); f.store.close(); }
});

test('journal rejects incompatible stored shape without overwriting retained state', async () => {
  const f = await fixture(async () => revoked);
  try {
    await assert.rejects(f.store.transact(record => {
      assert.ok(record);
      return { record: { ...record, unknownFutureField: true }, value: undefined };
    }), (error: unknown) => error instanceof ClientError && error.code === 'storage_corrupt');
    assert.equal((await f.client.status('login-a')).phase, 'active');
    const wrongClock = new PortableLogoutClient(f.store, { ...f.clock, id: 'different-domain' }, f.transport);
    await assert.rejects(wrongClock.begin('login-a', f.state.now + 10_000n * MS), ClientError);
    assert.equal((await f.client.status('login-a')).phase, 'active');
  } finally { f.store.close(); }
});

test('storage without strict durability refuses opening and preserves the login journal', async () => {
  const f = await fixture(async () => revoked);
  f.store.close();
  const open = f.factory.open.bind(f.factory);
  f.factory.open = (...args: Parameters<IDBFactory['open']>) => {
    const operation = open(...args);
    operation.addEventListener('success', () => {
      const database = operation.result;
      const transact = database.transaction.bind(database);
      database.transaction = (stores, mode) => transact(stores, mode, { durability: 'relaxed' });
    });
    return operation;
  };
  await assert.rejects(IndexedDbLogoutStore.open(f.factory, 'logout'),
    (error: unknown) => error instanceof ClientError && error.code === 'storage_unavailable');
  f.factory.open = open;
  const store = await IndexedDbLogoutStore.open(f.factory, 'logout');
  try {
    const client = new PortableLogoutClient(store, f.clock, f.transport);
    assert.equal((await client.status('login-a')).phase, 'active');
  } finally { store.close(); }
});

test('guarded SDK identity signs unchanged requests only for active, inventoried destinations', async () => {
  const f = await fixture(async () => revoked);
  try {
    const guarded = f.client.guardedIdentity('login-a', identity);
    assert.equal(guarded.getPrincipal().toText(), identity.getPrincipal().toText());
    const request = signingRequest(issuer);
    assert.deepEqual(await guarded.transformRequest(request), await identity.transformRequest(request));
    await assert.rejects(guarded.transformRequest(signingRequest(receiver)), ClientError);
    await f.client.destination('login-a', receiver, f.state.now + MS);
    assert.deepEqual(await guarded.transformRequest(signingRequest(receiver)), await identity.transformRequest(signingRequest(receiver)));
    await f.client.begin('login-a', f.state.now + 10_000n * MS);
    await assert.rejects(guarded.transformRequest(request), (error: unknown) => error instanceof ClientError && error.code === 'stale_generation');
  } finally { f.store.close(); }
});

test('logout in another tab rejects a signature that completes asynchronously afterward', async () => {
  const f = await fixture(async () => revoked);
  const store = await IndexedDbLogoutStore.open(f.factory, 'logout');
  try {
    let release!: () => void;
    let started!: () => void;
    const begun = new Promise<void>(resolve => { started = resolve; });
    const slow = {
      getPrincipal: () => identity.getPrincipal(),
      transformRequest: async (request: Parameters<typeof identity.transformRequest>[0]) => {
        started(); await new Promise<void>(resolve => { release = resolve; });
        return identity.transformRequest(request);
      },
    };
    const signature = f.client.guardedIdentity('login-a', slow).transformRequest(signingRequest(issuer));
    const refused = assert.rejects(signature, (error: unknown) => error instanceof ClientError && error.code === 'stale_generation');
    await begun;
    await new PortableLogoutClient(store, f.clock, f.transport).begin('login-a', f.state.now + 10_000n * MS);
    release();
    await refused;
  } finally { store.close(); f.store.close(); }
});

test('guard rejects a wrong generation, replaced identity and unavailable storage before signing', async () => {
  let signatures = 0;
  const f = await fixture(async () => revoked);
  let principal = identity.getPrincipal();
  const tracked = { getPrincipal: () => principal, transformRequest: async () => { signatures++; return {}; } };
  try {
    await assert.rejects(f.client.guardedIdentity('wrong', tracked).transformRequest(signingRequest(issuer)), ClientError);
    const guarded = f.client.guardedIdentity('login-a', tracked);
    principal = other.getPrincipal();
    await assert.rejects(guarded.transformRequest(signingRequest(issuer)), ClientError);
    principal = identity.getPrincipal();
    f.store.close();
    await assert.rejects(guarded.transformRequest(signingRequest(issuer)), ClientError);
    assert.equal(signatures, 0);
  } finally { f.store.close(); }
});

function signingRequest(target: Principal): Parameters<typeof identity.transformRequest>[0] {
  return {
    endpoint: Endpoint.Call, request: { method: 'POST' },
    body: { request_type: SubmitRequestType.Call, canister_id: target, method_name: 'read',
      arg: new Uint8Array(), sender: identity.getPrincipal(), ingress_expiry: Expiry.fromJSON('{"__expiry__":"100000000000"}') },
  };
}

test('cleanup signing cannot escape its selected destination or survive completion in another tab', async () => {
  let release!: () => void;
  let signing!: () => void;
  const started = new Promise<void>(resolve => { signing = resolve; });
  const slow = {
    getPrincipal: () => identity.getPrincipal(),
    transformRequest: async (request: Parameters<typeof identity.transformRequest>[0]) => {
      signing(); await new Promise<void>(resolve => { release = resolve; });
      return identity.transformRequest(request);
    },
  };
  let handedBack = false;
  const f = await fixture(async (_target, guarded) => {
    await assert.rejects(guarded.transformRequest(signingRequest(receiver)), ClientError);
    await guarded.transformRequest(signingRequest(issuer));
    handedBack = true;
    return revoked;
  });
  const store = await IndexedDbLogoutStore.open(f.factory, 'logout');
  try {
    await f.client.begin('login-a', f.state.now + 10_000n * MS);
    const running = f.client.cleanup('login-a', slow);
    await started;
    f.state.now += 10_000n * MS;
    await new PortableLogoutClient(store, f.clock, f.transport).cleanup('login-a', undefined);
    release();
    assert.equal((await running).targets[0]!.confirmedUntilNs, undefined);
    assert.equal(handedBack, false);
  } finally { store.close(); f.store.close(); }
});

test('signed read-state polling is phase-gated and a nonresponding cleanup call ends at its deadline', async () => {
  let aborted = false;
  const f = await fixture(async (_target, _identity, signal) => {
    signal.addEventListener('abort', () => { aborted = true; });
    return new Promise<LogoutResult>(() => {});
  });
  try {
    const guarded = f.client.guardedIdentity('login-a', identity);
    const request: Parameters<typeof identity.transformRequest>[0] = {
      endpoint: Endpoint.ReadState, request: { method: 'POST' },
      body: { request_type: ReadRequestType.ReadState, paths: [], sender: identity.getPrincipal(),
        ingress_expiry: Expiry.fromJSON('{"__expiry__":"100000000000"}') },
    };
    assert.deepEqual(await guarded.transformRequest(request), await identity.transformRequest(request));
    const deadline = f.state.now + 10_000n * MS;
    await f.client.begin('login-a', deadline);
    await assert.rejects(guarded.transformRequest(request), ClientError);
    f.state.now = deadline - MS;
    const result = await f.client.cleanup('login-a', identity);
    assert.equal(aborted, true);
    assert.equal(result.phase, 'finished');
    assert.equal(result.targets[0]!.confirmedUntilNs, undefined);
    assert.equal(result.targets[0]!.attempts, 1);
    assert.equal(f.state.disposed, 1);
  } finally { f.store.close(); }
});

test('a keyless tab cannot finish cleanup or dispose a key held by another worker', async () => {
  let release!: (response: LogoutResult) => void;
  let dispatched!: () => void;
  const started = new Promise<void>(resolve => { dispatched = resolve; });
  const f = await fixture(async () => {
    dispatched();
    return new Promise<LogoutResult>(resolve => { release = resolve; });
  });
  const store = await IndexedDbLogoutStore.open(f.factory, 'logout');
  let running: ReturnType<PortableLogoutClient['cleanup']> | undefined;
  try {
    await f.client.begin('login-a', f.state.now + 10_000n * MS);
    running = f.client.cleanup('login-a', identity);
    await started;
    const observer = new PortableLogoutClient(store, f.clock, f.transport);
    const state = await observer.cleanup('login-a', undefined);
    assert.equal(state.phase, 'cleanup');
    assert.equal(state.targets[0]!.attempts, 1);
    assert.equal(state.targets[0]!.confirmedUntilNs, undefined);
    assert.equal(f.state.disposed, 0);
    release(revoked);
    const result = await running;
    assert.equal(result.phase, 'finished');
    assert.ok(result.targets[0]!.confirmedUntilNs);
    assert.equal(f.state.disposed, 1);
  } finally {
    release?.(revoked);
    await running;
    store.close(); f.store.close();
  }
});

test('admission windows reserve enough room for immediate bounded logout', async () => {
  const f = await fixture(async () => revoked);
  try {
    const latest = f.state.now + 295_000n * MS;
    await assert.rejects(f.client.destination('login-a', receiver, latest),
      (error: unknown) => error instanceof ClientError && error.code === 'configuration');
    assert.equal((await f.client.status('login-a')).targets.length, 1);
    await f.client.destination('login-a', receiver, latest - 1n);
    await f.client.begin('login-a', f.state.now + 300_000n * MS);
    assert.equal((await f.client.status('login-a')).phase, 'cleanup');
  } finally { f.store.close(); }
});
