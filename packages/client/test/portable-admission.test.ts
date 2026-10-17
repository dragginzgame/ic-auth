import { test } from 'node:test';
import assert from 'node:assert/strict';
import { IDBFactory } from 'fake-indexeddb';
import { Principal } from '@icp-sdk/core/principal';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { PortableAdmissionClient, PortableLogoutClient, IndexedDbLogoutStore, ClientError,
  type AdmissionCredential, type PortableAdmissionTransport } from '../src/index.js';

const MS = 1_000_000n;
const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(1));
const issuer = Principal.fromUint8Array(new Uint8Array([1]));
const a = Principal.fromUint8Array(new Uint8Array([2]));
const b = Principal.fromUint8Array(new Uint8Array([3]));
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(done => { resolve = done; });
  return { promise, resolve };
}
async function fixture() {
  const factory = new IDBFactory();
  const store = await IndexedDbLogoutStore.open(factory, 'admission');
  const state = { now: 1_000n * MS, issues: 0, reconciles: 0, admissions: [] as string[] };
  const journal = new PortableLogoutClient(store, { id: 'test-clock', nowNs: () => state.now, wait: async () => {} },
    { revoke: async () => ({ tag: 'Revoked', value: 100_000n * MS }), dispose: async () => {} });
  await journal.start('login', identity, issuer);
  const material: AdmissionCredential = {
    credential: { claims: new Uint8Array([1, 2, 3]), proof: { tag: 'Ed25519', value: new Uint8Array(64) } },
    admissionUntilNs: state.now + 60_000n * MS,
  };
  const transport: PortableAdmissionTransport = {
    issue: async () => { state.issues++; assert.equal((await journal.status('login')).issuanceUnknown, true); return material; },
    reconcile: async () => { state.reconciles++; return material; },
    admit: async (target, caller, credential) => {
      assert.equal(caller.getPrincipal().toText(), identity.getPrincipal().toText());
      assert.ok((await journal.status('login')).targets.some(t => t.canister === target.toText()));
      assert.deepEqual(credential.claims, new Uint8Array([1, 2, 3]));
      state.admissions.push(target.toText());
    },
  };
  const client = () => new PortableAdmissionClient(journal, 'login', identity, issuer, transport);
  return { factory, store, state, journal, material, transport, client };
}

test('one issuance serves multiple receivers; each destination is durable before dispatch', async () => {
  const f = await fixture();
  try {
    const client = f.client();
    await Promise.all([client.establish(a), client.establish(b)]);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.reconciles, 0);
    assert.deepEqual(f.state.admissions.sort(), [a.toText(), b.toText()].sort());
    (f.material.credential.claims as Uint8Array).fill(9);
    await client.establish(a); // cached signed bytes were detached from the adapter
    f.state.now = f.material.admissionUntilNs;
    await assert.rejects(client.establish(b), (e: ClientError) => e.code === 'retrieval_expired');
    assert.equal(f.state.issues, 1); // no automatic renewal or application-call retry
  } finally { f.store.close(); }
});

test('concurrent first contact shares one receiver update while another receiver stays independent', async () => {
  const f = await fixture();
  const entered = deferred<void>();
  const release = deferred<void>();
  const calls: string[] = [];
  const client = f.client();
  const pending: ReturnType<typeof client.establish>[] = [];
  try {
    f.transport.admit = async target => {
      calls.push(target.toText());
      if (target.toText() === a.toText()) { entered.resolve(); await release.promise; }
    };
    pending.push(client.establish(a));
    await entered.promise;
    for (let i = 0; i < 8; i++) pending.push(client.establish(a));
    const other = await client.establish(b);
    assert.equal(other.getPrincipal().toText(), identity.getPrincipal().toText());
    release.resolve();
    const identities = await Promise.all(pending);
    assert.ok(identities.every(value => value === identities[0]));
    assert.equal(calls.filter(target => target === a.toText()).length, 1);
    assert.equal(calls.filter(target => target === b.toText()).length, 1);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.reconciles, 0);
  } finally {
    release.resolve();
    await Promise.allSettled(pending);
    f.store.close();
  }
});

test('shared admission failure permits explicit retry with the same proof and never caches session authority', async () => {
  const f = await fixture();
  const entered = deferred<void>();
  const release = deferred<void>();
  const client = f.client();
  const calls: string[] = [];
  let failing = true;
  let settled: Promise<PromiseSettledResult<unknown>[]> | undefined;
  try {
    f.transport.admit = async target => {
      calls.push(target.toText());
      if (target.toText() === a.toText() && failing) {
        entered.resolve(); await release.promise;
        throw new Error('lost admission reply');
      }
    };
    const first = client.establish(a);
    await entered.promise;
    settled = Promise.allSettled([first, client.establish(a)]);
    await client.establish(b);
    release.resolve();
    const results = await settled;
    assert.ok(results.every(result => result.status === 'rejected' && result.reason.message === 'lost admission reply'));
    assert.equal(calls.filter(target => target === a.toText()).length, 1);
    failing = false;
    await client.establish(a);
    await client.establish(a);
    assert.equal(calls.filter(target => target === a.toText()).length, 3);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.reconciles, 0);
    assert.equal((await f.journal.status('login')).issuanceUnknown, true);
  } finally {
    release.resolve();
    await settled;
    f.store.close();
  }
});

test('admission timeout releases the shared attempt and rejects its retained signing capability', async () => {
  const f = await fixture();
  const entered = deferred<void>();
  const client = new PortableAdmissionClient(f.journal, 'login', identity, issuer, f.transport, 50);
  let retained: Parameters<PortableAdmissionTransport['admit']>[1] | undefined;
  let calls = 0;
  try {
    f.transport.admit = async (_target, caller) => {
      calls++; retained = caller; entered.resolve();
      return new Promise<void>(() => {});
    };
    const first = client.establish(a);
    const refused = assert.rejects(first, (e: ClientError) => e.code === 'timeout');
    await entered.promise;
    const second = assert.rejects(client.establish(a), (e: ClientError) => e.code === 'timeout');
    await Promise.all([refused, second]);
    assert.equal(calls, 1);
    assert.ok(retained);
    await assert.rejects(retained.transformRequest({} as Parameters<typeof retained.transformRequest>[0]),
      (e: ClientError) => e.code === 'timeout');
    f.transport.admit = async () => { calls++; };
    await client.establish(a);
    assert.equal(calls, 2);
    assert.equal(f.state.issues, 1);
    assert.equal((await f.journal.status('login')).issuanceUnknown, true);
  } finally { f.store.close(); }
});

test('lost issuance stays uncertain across client recreation; recovery never issues again', async () => {
  const f = await fixture();
  let reopened: IndexedDbLogoutStore | undefined;
  try {
    f.transport.issue = async () => { f.state.issues++; throw new Error('lost reply'); };
    await assert.rejects(f.client().establish(a), /lost reply/);
    assert.equal(f.state.admissions.length, 0);
    f.store.close();
    reopened = await IndexedDbLogoutStore.open(f.factory, 'admission');
    const journal = new PortableLogoutClient(reopened, { id: 'test-clock', nowNs: () => f.state.now, wait: async () => {} },
      { revoke: async () => ({ tag: 'Revoked', value: 100_000n * MS }), dispose: async () => {} });
    f.transport.admit = async () => { f.state.admissions.push(b.toText()); };
    const recovered = () => new PortableAdmissionClient(journal, 'login', identity, issuer, f.transport);
    await recovered().establish(b);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.reconciles, 1);
    f.transport.reconcile = async () => { throw new Error('unresolved'); };
    await assert.rejects(recovered().establish(a), /unresolved/);
    assert.equal((await journal.status('login')).issuanceUnknown, true);
    assert.equal(f.state.issues, 1);
  } finally { reopened?.close(); f.store.close(); }
});

test('independent clients atomically claim issuance and reconcile competing attempts', async () => {
  const f = await fixture();
  try {
    await Promise.all([f.client().establish(a), f.client().establish(b)]);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.reconciles, 1);
  } finally { f.store.close(); }
});

test('logout during issuance drops the late credential before admission', async () => {
  const f = await fixture();
  try {
    const entered = deferred<void>();
    const result = deferred<AdmissionCredential>();
    f.transport.issue = async () => { entered.resolve(); return result.promise; };
    const pending = f.client().establish(a);
    await entered.promise;
    await f.journal.begin('login', f.state.now + 10_000n * MS);
    result.resolve(f.material);
    await assert.rejects(pending, (e: ClientError) => e.code === 'stale_generation');
    assert.equal(f.state.admissions.length, 0);
    assert.equal((await f.journal.status('login')).issuanceUnknown, true);
  } finally { f.store.close(); }
});

test('logout during admission retains destination and rejects late success', async () => {
  const f = await fixture();
  try {
    const entered = deferred<void>();
    const result = deferred<void>();
    f.transport.admit = async () => { entered.resolve(); await result.promise; };
    const client = f.client();
    const pending = Promise.allSettled([client.establish(a), client.establish(a)]);
    await entered.promise;
    await f.journal.begin('login', f.state.now + 70_000n * MS);
    result.resolve();
    assert.ok((await pending).every(result => result.status === 'rejected'
      && result.reason instanceof ClientError && result.reason.code === 'stale_generation'));
    assert.ok((await f.journal.status('login')).targets.some(t => t.canister === a.toText()));
  } finally { f.store.close(); }
});

test('timeout preserves uncertainty and aborts the per-call identity capability', async () => {
  const f = await fixture();
  try {
    let retained: Parameters<PortableAdmissionTransport['issue']>[0] | undefined;
    const late = deferred<AdmissionCredential>();
    f.transport.issue = async caller => { retained = caller; return late.promise; };
    const client = new PortableAdmissionClient(f.journal, 'login', identity, issuer, f.transport, 5);
    await assert.rejects(client.establish(a), (e: ClientError) => e.code === 'timeout');
    assert.ok(retained);
    // Cancellation must refuse before inspecting or signing any request.
    await assert.rejects(retained.transformRequest({} as Parameters<typeof retained.transformRequest>[0]),
      (e: ClientError) => e.code === 'timeout');
    assert.equal((await f.journal.status('login')).issuanceUnknown, true);
    assert.equal(f.state.admissions.length, 0);
    late.resolve(f.material);
    await new Promise(resolve => setTimeout(resolve, 0));
    assert.equal(f.state.admissions.length, 0);
  } finally { f.store.close(); }
});

test('invalid identity binding and failed journal access refuse before issuance', async () => {
  const f = await fixture();
  try {
    const wrong = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(4));
    await assert.rejects(new PortableAdmissionClient(f.journal, 'login', wrong, issuer, f.transport).establish(a),
      (e: ClientError) => e.code === 'stale_generation');
    assert.equal((await f.journal.status('login')).issuanceUnknown, false);
    f.store.close();
    await assert.rejects(f.client().establish(a));
    assert.equal(f.state.issues, 0);
  } finally { f.store.close(); }
});

test('malformed or expired recovered material never reaches a receiver or triggers issuance', async () => {
  const f = await fixture();
  try {
    assert.equal(await f.journal.claimIssuance('login'), true);
    f.material.credential = { ...f.material.credential, claims: new Uint8Array(131_073) };
    await assert.rejects(f.client().establish(a), (e: ClientError) => e.code === 'invalid_material');
    f.material.credential = { ...f.material.credential, claims: new Uint8Array([1]) };
    f.material.admissionUntilNs = f.state.now;
    await assert.rejects(f.client().establish(a), (e: ClientError) => e.code === 'retrieval_expired');
    assert.equal(f.state.issues, 0);
    assert.equal(f.state.admissions.length, 0);
    assert.equal((await f.journal.status('login')).issuanceUnknown, true);
  } finally { f.store.close(); }
});
