import { test } from 'node:test';
import assert from 'node:assert/strict';
import { IDBFactory } from 'fake-indexeddb';
import { Principal } from '@tokostudios/ic-platform/principal';
import { Endpoint, SubmitRequestType, ReadRequestType, Expiry, type Identity } from '@tokostudios/ic-platform/agent';
import { Ed25519KeyIdentity } from '@tokostudios/ic-platform/identity';
import { PortableAdmissionClient, PortableLogoutClient, IndexedDbLogoutStore, ClientError, AdmissionError,
  credentialBytes, type AdmissionReceipt, type AdmissionCredential, type PortableAdmissionTransport } from '../src/index.js';
import { MAX_TARGETS } from '../src/logout-store.js';

const MS = 1_000_000n;
const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(1));
const account = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(81));
const issuer = Principal.fromUint8Array(new Uint8Array([1]));
const a = Principal.fromUint8Array(new Uint8Array([2]));
const b = Principal.fromUint8Array(new Uint8Array([3]));
function receipt(): AdmissionReceipt {
  return { caller: identity.getPrincipal().toUint8Array(), issuer: issuer.toUint8Array(),
    subject: account.getPrincipal().toUint8Array(), expires_at_ns: 100_000_000_000n };
}
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(done => { resolve = done; });
  return { promise, resolve };
}
function signingRequest(signer: Identity, target: Principal): Parameters<Identity['transformRequest']>[0] {
  return { endpoint: Endpoint.Call, request: { method: 'POST' },
    body: { request_type: SubmitRequestType.Call, canister_id: target, method_name: 'issue',
      arg: new Uint8Array(), sender: signer.getPrincipal(),
      ingress_expiry: Expiry.fromJSON('{"__expiry__":"100000000000"}') } };
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
    issue: async caller => { assert.equal(caller.getPrincipal().toText(), account.getPrincipal().toText()); state.issues++; assert.equal((await journal.status('login')).issuanceUnknown, true); return material; },
    renew: async () => { throw new Error('unexpected renewal in this fixture'); }, reconcile: async caller => { assert.equal(caller.getPrincipal().toText(), account.getPrincipal().toText()); state.reconciles++; return material; },
    admit: async (target, caller, credential) => {
      assert.equal(caller.getPrincipal().toText(), identity.getPrincipal().toText());
      assert.ok((await journal.status('login')).targets.some(t => t.canister === target.toText()));
      assert.deepEqual(credential.claims, new Uint8Array([1, 2, 3]));
      state.admissions.push(target.toText()); return receipt();
    },
  };
  const client = () => new PortableAdmissionClient(journal, 'login', { browser: identity, account }, issuer, transport);
  return { factory, store, state, journal, material, transport, client };
}

test('journal actions retain exact intent and signer selection across reopen and renewal', async () => {
  const f = await fixture();
  let store = f.store;
  try {
    await assert.rejects(f.journal.claimCredential('login', false),
      (error: ClientError) => error.code === 'account_required');
    assert.deepEqual(await f.journal.claimCredential('login'), { kind: 'issue', sequence: 0n, signer: 'account' });
    store.close();
    store = await IndexedDbLogoutStore.open(f.factory, 'admission');
    const journal = new PortableLogoutClient(store,
      { id: 'test-clock', nowNs: () => f.state.now, wait: async () => {} },
      { revoke: async () => ({ tag: 'Revoked', value: 100_000n * MS }), dispose: async () => {} });
    assert.deepEqual(await journal.claimCredential('login'), { kind: 'reconcile', sequence: 0n, signer: 'account' });
    const bytes = credentialBytes(f.material.credential, 131_072);
    await journal.completeCredential('login', 0n, bytes, f.material.admissionUntilNs);
    assert.deepEqual(await journal.claimCredential('login', false),
      { kind: 'saved', sequence: 0n, bytes, until: f.material.admissionUntilNs });
    f.state.now = f.material.admissionUntilNs;
    assert.deepEqual(await journal.claimCredential('login', false), { kind: 'renew', sequence: 1n, signer: 'browser' });
    assert.deepEqual(await journal.claimCredential('login', false), { kind: 'reconcile', sequence: 1n, signer: 'browser' });
    assert.deepEqual((await journal.status('login')).issuance, { sequence: 1n });
    await journal.completeCredential('login', 1n, bytes, f.state.now + 60_000n * MS);
    assert.deepEqual(await journal.claimCredential('login', false),
      { kind: 'saved', sequence: 1n, bytes, until: f.state.now + 60_000n * MS });
    assert.equal(f.state.issues, 0);
    assert.equal(f.state.reconciles, 0);
    assert.deepEqual(f.state.admissions, []);
  } finally { store.close(); }
});

for (const phase of ['issuance', 'admission'] as const) {
  test(`outstanding admission work is bounded before storage and cannot postpone logout during ${phase}`, async t => {
    const f = await fixture();
    const entered = deferred<void>();
    const release = deferred<void>();
    const client = f.client();
    let pending: Promise<PromiseSettledResult<Identity>[]> | undefined;
    try {
      if (phase === 'issuance') {
        f.transport.issue = async () => {
          f.state.issues++; entered.resolve(); await release.promise; return f.material;
        };
      } else {
        f.transport.admit = async () => { entered.resolve(); await release.promise; return receipt(); };
      }
      pending = Promise.allSettled(Array.from({ length: MAX_TARGETS }, () => client.establish(a)));
      await entered.promise;
      const transact = t.mock.method(f.store, 'transact');
      const overflow = client.establish(b);
      assert.equal(transact.mock.callCount(), 0, 'overflow opened a journal transaction');
      t.mock.restoreAll();
      await assert.rejects(overflow, (error: unknown) =>
        error instanceof ClientError && error.code === 'storage_capacity');
      const deadline = f.state.now + (phase === 'issuance' ? 10_000n : 70_000n) * MS;
      await f.journal.begin('login', deadline);
      await f.journal.begin('login', deadline + MS);
      const state = await f.journal.cleanup('login', identity);
      assert.equal(state.phase, 'finished');
      assert.equal(state.deadlineNs, deadline);
      assert.equal(state.issuanceUnknown, true);
      assert.ok(state.targets.every(target => target.confirmedUntilNs !== undefined));
      for (let i = 0; i < 16; i++) await assert.rejects(client.establish(b), ClientError);
      release.resolve();
      assert.ok((await pending).every(result => result.status === 'rejected'
        && result.reason instanceof ClientError && result.reason.code === 'stale_generation'));
      await assert.rejects(client.establish(b), (error: ClientError) => error.code === 'stale_generation');
      assert.equal(f.state.issues, 1);
    } finally { release.resolve(); await pending; f.store.close(); }
  });
}

test('timed-out admission saturation releases capacity without clearing issuance uncertainty', async t => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const f = await fixture();
  const entered = deferred<void>();
  const client = f.client();
  let signal: AbortSignal | undefined;
  try {
    f.transport.issue = async (_identity, current) => {
      f.state.issues++; signal = current; entered.resolve(); return new Promise(() => {});
    };
    const pending = Promise.allSettled(Array.from({ length: MAX_TARGETS }, () => client.establish(a)));
    await entered.promise;
    await f.journal.status('login'); // Drain earlier IDB work before advancing only the network timer.
    t.mock.timers.tick(5_000);
    assert.ok((await pending).every(result => result.status === 'rejected'
      && result.reason instanceof ClientError && result.reason.code === 'timeout'));
    assert.equal(signal?.aborted, true);
    assert.equal((await f.journal.status('login')).issuanceUnknown, true);
    await client.establish(b);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.reconciles, 1);
    assert.deepEqual(f.state.admissions, [b.toText()]);
  } finally { f.store.close(); }
});

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
    let renewals = 0;
    f.transport.renew = async () => { renewals++; throw new ClientError('transport'); };
    await assert.rejects(client.establish(b), (e: ClientError) => e.code === 'transport');
    assert.equal(renewals, 1);
    assert.equal(f.state.issues, 1); // renewal never repeats account issuance
  } finally { f.store.close(); }
});

test('mutable SDK Principal cannot retarget admission beyond its durable logout destination', async () => {
  const f = await fixture();
  let signatures = 0;
  const originalAdmit = f.transport.admit;
  const browser = { getPrincipal: () => identity.getPrincipal(),
    transformRequest: async (request: Parameters<typeof identity.transformRequest>[0]) => {
      signatures++;
      return identity.transformRequest(request);
    } };
  try {
    f.transport.admit = async (target, caller) => {
      assert.equal(target.toText(), a.toText());
      // The SDK exposes mutable backing bytes. Transport must not be able to
      // change the signer's permitted target after that target was journaled.
      target.toUint8Array().set(b.toUint8Array());
      await caller.transformRequest(signingRequest(caller, target));
      return receipt();
    };
    const client = new PortableAdmissionClient(f.journal, 'login', { browser, account }, issuer, f.transport);
    await assert.rejects(client.establish(Principal.fromText(a.toText())),
      (error: unknown) => error instanceof ClientError && error.code === 'configuration');
    assert.equal(signatures, 0);
    const targets = (await f.journal.status('login')).targets.map(target => target.canister);
    assert.ok(targets.includes(a.toText()));
    assert.ok(!targets.includes(b.toText()));
    f.transport.admit = originalAdmit;
    await client.establish(a);
    assert.deepEqual(f.state.admissions, [a.toText()]);
    assert.equal(f.state.issues, 1);
  } finally { f.store.close(); }
});

test('mutable caller Principal cannot change an admission destination while issuance is pending', async () => {
  const f = await fixture();
  const entered = deferred<void>();
  const release = deferred<void>();
  const target = Principal.fromText(a.toText());
  let pending: Promise<unknown> | undefined;
  try {
    const issue = f.transport.issue;
    f.transport.issue = async (caller, signal) => {
      entered.resolve(); await release.promise;
      return issue(caller, signal);
    };
    pending = f.client().establish(target);
    await entered.promise;
    target.toUint8Array().set(b.toUint8Array());
    release.resolve();
    await pending;
    assert.equal(target.toText(), b.toText()); // Caller still owns its mutable object.
    assert.deepEqual(f.state.admissions, [a.toText()]);
    const targets = (await f.journal.status('login')).targets.map(item => item.canister);
    assert.ok(targets.includes(a.toText()));
    assert.ok(!targets.includes(b.toText()));
    assert.equal(f.state.issues, 1);
  } finally { release.resolve(); await pending; f.store.close(); }
});

test('issuer binding is captured before caller mutation and stays fixed across issuance', async () => {
  const f = await fixture();
  const mutableIssuer = Principal.fromText(issuer.toText());
  let signatures = 0;
  const signer: Identity = { getPrincipal: () => account.getPrincipal(), transformRequest: async request => {
    signatures++;
    return account.transformRequest(request);
  } };
  try {
    const issue = f.transport.issue;
    f.transport.issue = async (caller, signal) => {
      // Even after constructor input mutation, only the original issuer may sign.
      await assert.rejects(caller.transformRequest(signingRequest(caller, mutableIssuer)),
        (error: ClientError) => error.code === 'configuration');
      await caller.transformRequest(signingRequest(caller, issuer));
      return issue(caller, signal);
    };
    const client = new PortableAdmissionClient(f.journal, 'login', { browser: identity, account: signer }, mutableIssuer, f.transport);
    mutableIssuer.toUint8Array().set(b.toUint8Array());
    await client.establish(a);
    await client.establish(b);
    assert.equal(signatures, 1);
    assert.equal(f.state.issues, 1);
    assert.equal((await f.journal.status('login')).issuer, issuer.toText());
  } finally { f.store.close(); }
});

test('account signing is issuer-bound; receiver admission and returned identity use the browser key', async () => {
  const f = await fixture();
  let accountSignatures = 0;
  const accountSigner: Identity = {
    getPrincipal: () => account.getPrincipal(),
    transformRequest: async request => { accountSignatures++; return account.transformRequest(request); },
  };
  let retained: Identity | undefined;
  try {
    f.transport.issue = async caller => {
      retained = caller;
      assert.equal(caller.getPrincipal().toText(), account.getPrincipal().toText());
      await assert.rejects(caller.transformRequest(signingRequest(caller, a)), (e: ClientError) => e.code === 'configuration');
      assert.equal(accountSignatures, 0);
      await caller.transformRequest(signingRequest(caller, issuer));
      return f.material;
    };
    const client = new PortableAdmissionClient(f.journal, 'login', { browser: identity, account: accountSigner }, issuer, f.transport);
    const browser = await client.establish(a);
    assert.equal(browser.getPrincipal().toText(), identity.getPrincipal().toText());
    assert.notEqual(browser.getPrincipal().toText(), account.getPrincipal().toText());
    await browser.transformRequest(signingRequest(browser, a));
    assert.equal(accountSignatures, 1);
    assert.ok(retained);
    await assert.rejects(retained.transformRequest(signingRequest(retained, issuer)), (e: ClientError) => e.code === 'timeout');
    assert.equal(accountSignatures, 1);
  } finally { f.store.close(); }
});

test('durable logout blocks a retained account signer before it can sign issuance', async () => {
  const f = await fixture();
  const entered = deferred<void>();
  const release = deferred<void>();
  let accountSignatures = 0;
  const accountSigner: Identity = { getPrincipal: () => account.getPrincipal(),
    transformRequest: async request => { accountSignatures++; return account.transformRequest(request); } };
  let pending: Promise<unknown> | undefined;
  try {
    f.transport.issue = async caller => {
      entered.resolve(); await release.promise;
      await caller.transformRequest(signingRequest(caller, issuer));
      return f.material;
    };
    const client = new PortableAdmissionClient(f.journal, 'login', { browser: identity, account: accountSigner }, issuer, f.transport);
    pending = assert.rejects(client.establish(a), (e: ClientError) => e.code === 'stale_generation');
    await entered.promise;
    await f.journal.begin('login', f.state.now + 10_000n * MS);
    release.resolve();
    await pending;
    assert.equal(accountSignatures, 0);
    assert.equal(f.state.admissions.length, 0);
    assert.equal((await f.journal.status('login')).issuanceUnknown, true);
  } finally { release.resolve(); await pending; f.store.close(); }
});

test('account principal changes during signing refuse before receiver admission', async () => {
  const f = await fixture();
  let current = account;
  const accountSigner: Identity = { getPrincipal: () => current.getPrincipal(),
    transformRequest: async request => {
      const signed = await current.transformRequest(request);
      current = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(82));
      return signed;
    } };
  try {
    f.transport.issue = async caller => {
      await caller.transformRequest(signingRequest(caller, issuer));
      return f.material;
    };
    const client = new PortableAdmissionClient(f.journal, 'login', { browser: identity, account: accountSigner }, issuer, f.transport);
    await assert.rejects(client.establish(a), (e: ClientError) => e.code === 'stale_generation');
    assert.equal(f.state.admissions.length, 0);
    assert.equal((await f.journal.status('login')).issuanceUnknown, true);
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
      return receipt();
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
      return receipt();
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
  const client = new PortableAdmissionClient(f.journal, 'login', { browser: identity, account }, issuer, f.transport, 50);
  let retained: Parameters<PortableAdmissionTransport['admit']>[1] | undefined;
  let calls = 0;
  try {
    f.transport.admit = async (_target, caller) => {
      calls++; retained = caller; entered.resolve();
      return new Promise<AdmissionReceipt>(() => {});
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
    f.transport.admit = async () => { calls++; return receipt(); };
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
    f.transport.admit = async () => { f.state.admissions.push(b.toText()); return receipt(); };
    const recovered = () => new PortableAdmissionClient(journal, 'login', { browser: identity, account }, issuer, f.transport);
    await recovered().establish(b);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.reconciles, 1);
    f.transport.reconcile = async () => { throw new Error('unresolved'); };
    await recovered().establish(a); // Exact evidence survives coordinator recreation.
    assert.equal(f.state.reconciles, 1);
    assert.equal((await journal.status('login')).issuanceUnknown, true);
    assert.equal(f.state.issues, 1);
  } finally { reopened?.close(); f.store.close(); }
});

test('browser-only renewal survives lost replies and journal reopen without account issuance', async () => {
  const f = await fixture();
  let reopened: IndexedDbLogoutStore | undefined;
  let renewals = 0;
  try {
    await f.client().establish(a);
    f.state.now = f.material.admissionUntilNs;
    const renewed = { ...f.material, admissionUntilNs: f.state.now + 60_000n * MS };
    f.transport.renew = async (signer, sequence) => {
      assert.equal(signer.getPrincipal().toText(), identity.getPrincipal().toText());
      assert.equal(sequence, 1n);
      assert.deepEqual((await f.journal.status('login')).issuance, { sequence: 1n });
      renewals++;
      throw new ClientError('transport');
    };
    const browserOnly = (journal = f.journal) => new PortableAdmissionClient(journal, 'login', { browser: identity }, issuer, f.transport);
    await assert.rejects(browserOnly().establish(a), ClientError);
    f.transport.reconcile = async (_signer, sequence) => {
      assert.equal(sequence, 1n);
      throw new ClientError('transport');
    };
    await assert.rejects(browserOnly().establish(a), (e: ClientError) => e.code === 'transport');
    // The unresolved sequence cannot advance just because a local timeout passes.
    f.store.close();
    reopened = await IndexedDbLogoutStore.open(f.factory, 'admission');
    const journal = new PortableLogoutClient(reopened, { id: 'test-clock', nowNs: () => f.state.now, wait: async () => {} },
      { revoke: async () => ({ tag: 'Revoked', value: 1n }), dispose: async () => {} });
    f.transport.admit = async target => {
      assert.ok((await journal.status('login')).targets.some(t => t.canister === target.toText()));
      f.state.admissions.push(target.toText()); return receipt();
    };
    f.transport.reconcile = async (signer, sequence) => {
      assert.equal(sequence, 1n);
      assert.equal(signer.getPrincipal().toText(), identity.getPrincipal().toText());
      f.state.reconciles++;
      return renewed;
    };
    await browserOnly(journal).establish(b);
    await browserOnly(journal).establish(a);
    assert.equal(renewals, 1);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.reconciles, 1);
    assert.equal((await journal.status('login')).issuance?.untilNs, renewed.admissionUntilNs);
  } finally { reopened?.close(); f.store.close(); }
});

test('missing initial account identity leaves issuance intent untouched', async () => {
  const f = await fixture();
  try {
    const client = new PortableAdmissionClient(f.journal, 'login', { browser: identity }, issuer, f.transport);
    await assert.rejects(client.establish(a), (e: ClientError) => e.code === 'account_required');
    assert.equal((await f.journal.status('login')).issuanceUnknown, false);
    assert.equal((await f.journal.status('login')).issuance, undefined);
    assert.equal(f.state.issues, 0);
  } finally { f.store.close(); }
});

test('readiness keeps a live receiver lease after proof expiry without loading evidence', async t => {
  const f = await fixture();
  try {
    const client = f.client();
    const guarded = await client.establish(a);
    f.state.now = f.material.admissionUntilNs;
    const evidence = t.mock.method(f.journal, 'claimCredential');
    const same = await client.ensureSession(a, value => value.expires_at_ns);
    assert.equal(same.getPrincipal().toText(), guarded.getPrincipal().toText());
    assert.equal(evidence.mock.callCount(), 0);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.admissions.length, 1);
    await f.journal.begin('login', f.state.now + 10_000n * MS);
    await assert.rejects(client.ensureSession(a, value => value.expires_at_ns),
      (e: ClientError) => e.code === 'stale_generation');
    await assert.rejects(same.transformRequest(signingRequest(same, a)),
      (e: ClientError) => e.code === 'stale_generation');
  } finally { f.store.close(); }
});

test('readiness after suspend or reopen silently renews with the browser key and shares admission', async () => {
  const f = await fixture();
  try {
    const client = f.client();
    await client.establish(a);
    f.state.now = receipt().expires_at_ns;
    let renewals = 0;
    f.transport.renew = async (signer, sequence) => {
      assert.equal(signer.getPrincipal().toText(), identity.getPrincipal().toText());
      assert.equal(sequence, 1n); renewals++;
      return { ...f.material, admissionUntilNs: f.state.now + 60_000n * MS };
    };
    f.transport.admit = async () => {
      f.state.admissions.push(a.toText());
      return { ...receipt(), expires_at_ns: f.state.now + 30_000n * MS };
    };
    const identities = await Promise.all(Array.from({ length: 8 }, () => client.ensureSession(a, value => value.expires_at_ns)));
    assert.ok(identities.every(value => value.getPrincipal().toText() === identity.getPrincipal().toText()));
    assert.equal(renewals, 1);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.admissions.length, 2);
    const reopened = new PortableAdmissionClient(f.journal, 'login', { browser: identity }, issuer, f.transport);
    // A new coordinator admits before use; it cannot treat old receipts as authority.
    await reopened.ensureSession(a, value => value.expires_at_ns);
    assert.equal(f.state.issues, 1);
    assert.equal(renewals, 1);
    assert.equal(f.state.admissions.length, 3);
  } finally { f.store.close(); }
});

test('readiness uses the explicit clock mapping, detaches receipts and refuses invalid hints', async () => {
  const f = await fixture();
  try {
    const client = f.client();
    await client.establish(a);
    // The controlled test's receiver epoch is ahead of its journal epoch.
    const offset = 10_000n * MS;
    const map = (value: AdmissionReceipt) => {
      value.caller.fill(0);
      return value.expires_at_ns - offset;
    };
    assert.ok(await client.ensureSession(a, map));
    assert.deepEqual(await client.sessionReceipt(a), receipt());
    await assert.rejects(client.ensureSession(a, () => -1n), (e: ClientError) => e.code === 'configuration');
    assert.equal(f.state.admissions.length, 1);
    f.state.now = receipt().expires_at_ns - offset;
    f.transport.renew = async () => ({ ...f.material, admissionUntilNs: f.state.now + 60_000n * MS });
    f.transport.admit = async () => {
      f.state.admissions.push(a.toText());
      return { ...receipt(), expires_at_ns: f.state.now + offset };
    };
    await assert.rejects(client.ensureSession(a, map), (e: ClientError) => e.code === 'lease_expired');
    assert.equal(f.state.admissions.length, 2, 'an expired new hint must not start an admission loop');
  } finally { f.store.close(); }
});

test('readiness propagates renewal refusal without account fallback or application effects', async () => {
  const f = await fixture();
  try {
    const client = f.client();
    await client.establish(a);
    f.state.now = receipt().expires_at_ns;
    let renewals = 0;
    f.transport.renew = async () => { renewals++; throw new ClientError('account_required'); };
    await assert.rejects(client.ensureSession(a, value => value.expires_at_ns),
      (e: ClientError) => e.code === 'account_required');
    assert.equal(f.state.issues, 1);
    assert.equal(renewals, 1);
    assert.equal(f.state.admissions.length, 1);
  } finally { f.store.close(); }
});

test('readiness observes a newer lease installed while its journal check was queued', async t => {
  const f = await fixture();
  const entered = deferred<void>();
  const release = deferred<void>();
  let pending: Promise<Identity> | undefined;
  try {
    const client = f.client();
    await client.establish(a);
    f.state.now = receipt().expires_at_ns;
    f.transport.renew = async () => ({ ...f.material, admissionUntilNs: f.state.now + 60_000n * MS });
    f.transport.admit = async () => {
      f.state.admissions.push(a.toText());
      return { ...receipt(), expires_at_ns: f.state.now + 30_000n * MS };
    };
    const status = f.journal.admissionStatus.bind(f.journal);
    let pause = true;
    t.mock.method(f.journal, 'admissionStatus', async (generation: string) => {
      const value = await status(generation);
      if (pause) { pause = false; entered.resolve(); await release.promise; }
      return value;
    });
    pending = client.ensureSession(a, value => value.expires_at_ns);
    await entered.promise;
    await client.establish(a);
    release.resolve();
    await pending;
    assert.equal(f.state.admissions.length, 2, 'queued readiness must reuse the newly installed lease');
  } finally { release.resolve(); await pending?.catch(() => {}); f.store.close(); }
});

test('readiness renews short receiver leases before proof expiry and shares the newer sequence', async () => {
  const f = await fixture();
  try {
    f.transport.admit = async target => {
      f.state.admissions.push(target.toText());
      return { ...receipt(), expires_at_ns: f.state.now + 5_000n * MS };
    };
    const client = f.client();
    await client.establish(a);
    await client.establish(b);
    f.state.now += 5_000n * MS;
    assert.ok(f.state.now < f.material.admissionUntilNs);
    let renewals = 0;
    f.transport.renew = async (signer, sequence) => {
      assert.equal(sequence, 1n);
      assert.equal(signer.getPrincipal().toText(), identity.getPrincipal().toText());
      renewals++;
      return { credential: { ...f.material.credential, claims: new Uint8Array([9]) },
        admissionUntilNs: f.state.now + 60_000n * MS };
    };
    await Promise.all([client.ensureSession(a, value => value.expires_at_ns), client.ensureSession(b, value => value.expires_at_ns)]);
    assert.equal(renewals, 1);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.admissions.length, 4);
    assert.equal((await f.journal.status('login')).issuance?.sequence, 1n);
  } finally { f.store.close(); }
});

test('reopened readiness refreshes once after authenticated expiry and retains older proof inventory', async () => {
  const f = await fixture();
  try {
    await f.client().establish(a);
    let attempts = 0;
    let renewals = 0;
    const until = f.state.now + 10_000n * MS;
    f.transport.renew = async (_signer, sequence) => {
      assert.equal(sequence, 1n); renewals++;
      return { credential: { ...f.material.credential, claims: new Uint8Array([9]) }, admissionUntilNs: until };
    };
    f.transport.admit = async (_target, _identity, credential) => {
      attempts++;
      if (attempts === 1) { assert.equal(credential.claims[0], 1); throw new AdmissionError('Expired'); }
      assert.equal(credential.claims[0], 9);
      return { ...receipt(), expires_at_ns: f.state.now + 5_000n * MS };
    };
    const reopened = new PortableAdmissionClient(f.journal, 'login', { browser: identity }, issuer, f.transport);
    await reopened.ensureSession(a, value => value.expires_at_ns);
    assert.equal(attempts, 2);
    assert.equal(renewals, 1);
    assert.equal(f.state.issues, 1);
    assert.equal((await f.journal.status('login')).targets.find(t => t.canister === a.toText())!.admissionUntilNs,
      f.material.admissionUntilNs, 'shorter new proofs must not erase older outstanding windows');
    assert.equal((await f.journal.status('login')).issuance?.untilNs, until);
  } finally { f.store.close(); }
});

test('readiness bounds expiry recovery and never retries other receiver refusals', async () => {
  for (const refusal of ['Expired', 'NotYetValid', 'Revoked', 'AuthorityUnavailable', 'Throttled'] as const) {
    const f = await fixture();
    try {
      await f.client().establish(a);
      let attempts = 0;
      let renewals = 0;
      f.transport.renew = async () => {
        renewals++;
        return { ...f.material, admissionUntilNs: f.state.now + 10_000n * MS };
      };
      f.transport.admit = async () => { attempts++; throw new AdmissionError(refusal); };
      const reopened = new PortableAdmissionClient(f.journal, 'login', { browser: identity }, issuer, f.transport);
      await assert.rejects(reopened.ensureSession(a, value => value.expires_at_ns),
        (e: AdmissionError) => e.refusal === refusal);
      assert.equal(attempts, refusal === 'Expired' ? 2 : 1);
      assert.equal(renewals, refusal === 'Expired' ? 1 : 0);
      assert.equal(f.state.issues, 1);
    } finally { f.store.close(); }
  }
});

test('readiness waits for an older proof flight then shares one fresh renewal', async t => {
  const f = await fixture();
  const entered = deferred<void>();
  const release = deferred<void>();
  const pending: Promise<Identity>[] = [];
  try {
    f.transport.admit = async target => {
      f.state.admissions.push(target.toText());
      return { ...receipt(), expires_at_ns: f.state.now + 5_000n * MS };
    };
    const client = f.client();
    await client.establish(a);
    await client.establish(b);
    f.state.now += 5_000n * MS;
    const claim = f.journal.claimCredential.bind(f.journal);
    let pause = true;
    t.mock.method(f.journal, 'claimCredential', async (generation: string, accountAvailable?: boolean, through?: bigint) => {
      const value = await claim(generation, accountAvailable, through);
      if (pause) { pause = false; entered.resolve(); await release.promise; }
      return value;
    });
    let renewals = 0;
    f.transport.renew = async (_signer, sequence) => {
      assert.equal(sequence, 1n); renewals++;
      return { ...f.material, admissionUntilNs: f.state.now + 60_000n * MS };
    };
    pending.push(client.establish(issuer));
    await entered.promise;
    pending.push(client.ensureSession(a, value => value.expires_at_ns), client.ensureSession(b, value => value.expires_at_ns));
    await new Promise(resolve => setImmediate(resolve));
    release.resolve();
    await Promise.all(pending);
    assert.equal(renewals, 1);
    assert.equal(f.state.issues, 1);
  } finally { release.resolve(); await Promise.allSettled(pending); f.store.close(); }
});

test('readiness identity renews before signing after suspend and stays receiver-bound', async () => {
  const f = await fixture();
  let signatures = 0;
  const browser: Identity = { getPrincipal: () => identity.getPrincipal(), transformRequest: async request => {
    signatures++;
    assert.equal(f.state.admissions.length, signatures, 'admission must finish before application signing');
    return identity.transformRequest(request);
  } };
  try {
    f.transport.admit = async target => {
      f.state.admissions.push(target.toText());
      return { ...receipt(), expires_at_ns: f.state.now + 5_000n * MS };
    };
    const client = new PortableAdmissionClient(f.journal, 'login', { browser, account }, issuer, f.transport);
    const signer = await client.ensureSession(a, value => value.expires_at_ns);
    await signer.transformRequest(signingRequest(signer, a));
    f.state.now += 5_000n * MS;
    let renewals = 0;
    f.transport.renew = async () => {
      renewals++;
      return { ...f.material, admissionUntilNs: f.state.now + 60_000n * MS };
    };
    await signer.transformRequest(signingRequest(signer, a));
    assert.equal(signatures, 2);
    assert.equal(renewals, 1);
    assert.equal(f.state.issues, 1);
    await assert.rejects(signer.transformRequest(signingRequest(signer, b)),
      (e: ClientError) => e.code === 'configuration');
    assert.equal(signatures, 2);
    await f.journal.begin('login', f.state.now + 70_000n * MS);
    await assert.rejects(signer.transformRequest(signingRequest(signer, a)),
      (e: ClientError) => e.code === 'stale_generation');
    assert.equal(signatures, 2);
  } finally { f.store.close(); }
});

test('readiness identity polls an existing request without refreshing an expired lease', async () => {
  const f = await fixture();
  try {
    const signer = await f.client().ensureSession(a, value => value.expires_at_ns);
    f.state.now = receipt().expires_at_ns;
    const request: Parameters<Identity['transformRequest']>[0] = {
      endpoint: Endpoint.ReadState, request: { method: 'POST' },
      body: { request_type: ReadRequestType.ReadState, paths: [], sender: signer.getPrincipal(),
        ingress_expiry: Expiry.fromJSON('{"__expiry__":"100000000000"}') },
    };
    assert.deepEqual(await signer.transformRequest(request), await identity.transformRequest(request));
    assert.equal(f.state.admissions.length, 1);
    assert.equal(f.state.issues, 1);
    await f.journal.begin('login', f.state.now + 10_000n * MS);
    await assert.rejects(signer.transformRequest(request), (e: ClientError) => e.code === 'stale_generation');
  } finally { f.store.close(); }
});

test('readiness settles expired exact recovery before silently renewing a pending sequence', async () => {
  const f = await fixture();
  try {
    await f.client().establish(a);
    f.state.now = f.material.admissionUntilNs;
    f.transport.renew = async () => { throw new ClientError('transport'); };
    await assert.rejects(f.client().establish(a), ClientError); // Pending sequence one.
    let lookups = 0;
    let renewals = 0;
    f.transport.reconcile = async (_signer, sequence) => {
      assert.equal(sequence, 1n); lookups++;
      return { ...f.material, admissionUntilNs: f.state.now };
    };
    f.transport.renew = async (signer, sequence) => {
      assert.equal(signer.getPrincipal().toText(), identity.getPrincipal().toText());
      assert.equal(sequence, 2n);
      assert.equal(lookups, 1, 'the uncertain operation must resolve before renewal');
      renewals++;
      return { ...f.material, admissionUntilNs: f.state.now + 60_000n * MS };
    };
    f.transport.admit = async target => {
      f.state.admissions.push(target.toText());
      return { ...receipt(), expires_at_ns: f.state.now + 5_000n * MS };
    };
    const resumed = new PortableAdmissionClient(f.journal, 'login', { browser: identity }, issuer, f.transport);
    await resumed.ensureSession(a, value => value.expires_at_ns);
    assert.equal(renewals, 1);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.admissions.length, 2);
    assert.equal((await f.journal.status('login')).issuance?.sequence, 2n);
  } finally { f.store.close(); }
});

test('session receipts are detached scheduling metadata and logout fences their retrieval', async () => {
  const f = await fixture();
  try {
    const client = f.client();
    assert.equal(await client.sessionReceipt(a), undefined);
    await client.establish(a);
    const saved = (await client.sessionReceipt(a))!;
    assert.deepEqual(saved, receipt());
    saved.caller.fill(0);
    assert.deepEqual(await client.sessionReceipt(a), receipt());
    f.transport.admit = async () => ({ ...receipt(), caller: account.getPrincipal().toUint8Array() });
    await assert.rejects(client.establish(b), (e: ClientError) => e.code === 'invalid_material');
    assert.equal(await client.sessionReceipt(b), undefined);
    await f.journal.begin('login', f.state.now + 70_000n * MS);
    await assert.rejects(client.sessionReceipt(a), (e: ClientError) => e.code === 'stale_generation');
  } finally { f.store.close(); }
});

test('expired exact recovery completes its sequence before a separate fresh renewal', async () => {
  const f = await fixture();
  try {
    await f.client().establish(a);
    f.state.now = f.material.admissionUntilNs;
    f.transport.renew = async () => { throw new ClientError('transport'); };
    await assert.rejects(f.client().establish(a), ClientError);
    const expired = { ...f.material, admissionUntilNs: f.state.now };
    f.transport.reconcile = async (_signer, sequence) => { assert.equal(sequence, 1n); return expired; };
    await assert.rejects(f.client().establish(a), (e: ClientError) => e.code === 'retrieval_expired');
    assert.equal(f.state.admissions.length, 1);
    let renewals = 0;
    f.transport.renew = async (_signer, sequence) => {
      assert.equal(sequence, 2n); renewals++;
      return { ...f.material, admissionUntilNs: f.state.now + 60_000n * MS };
    };
    await f.client().establish(a);
    assert.equal(renewals, 1);
    assert.equal(f.state.issues, 1);
  } finally { f.store.close(); }
});

test('competing tabs recover one renewal sequence and retain identical original evidence', async () => {
  const f = await fixture();
  const entered = deferred<void>();
  const release = deferred<void>();
  let first: Promise<Identity> | undefined;
  try {
    await f.client().establish(a);
    f.state.now = f.material.admissionUntilNs;
    const result = { ...f.material, admissionUntilNs: f.state.now + 60_000n * MS };
    let renewals = 0;
    let lookups = 0;
    f.transport.renew = async (signer, sequence) => {
      assert.equal(signer.getPrincipal().toText(), identity.getPrincipal().toText());
      assert.equal(sequence, 1n); renewals++; entered.resolve(); await release.promise; return result;
    };
    f.transport.reconcile = async (signer, sequence) => {
      assert.equal(signer.getPrincipal().toText(), identity.getPrincipal().toText());
      assert.equal(sequence, 1n); lookups++; return result;
    };
    first = f.client().establish(a);
    await entered.promise;
    await f.client().establish(b);
    release.resolve(); await first;
    assert.equal(renewals, 1);
    assert.equal(lookups, 1);
    const saved = (await f.journal.status('login')).issuance!;
    assert.equal(saved.sequence, 1n);
    assert.equal(saved.bytes, undefined, 'status reads metadata without loading proof bytes');
    const claim = await f.journal.claimCredential('login');
    assert.ok(claim.kind === 'saved');
    const frame = claim.bytes;
    assert.deepEqual(frame, credentialBytes(result.credential, 131_072));
    await assert.rejects(f.journal.completeCredential('login', 1n, frame, result.admissionUntilNs + 1n),
      (e: ClientError) => e.code === 'storage_conflict');
    const changed = credentialBytes({ ...result.credential, claims: new Uint8Array([99]) }, 131_072);
    await assert.rejects(f.journal.completeCredential('login', 1n, changed, result.admissionUntilNs),
      (e: ClientError) => e.code === 'storage_conflict');
  } finally { release.resolve(); await first?.catch(() => {}); f.store.close(); }
});

test('failed evidence persistence leaves lookup intent and dispatches no receiver update', async t => {
  const f = await fixture();
  try {
    const mock = t.mock.method(f.journal, 'completeCredential', async () => { throw new ClientError('storage_unavailable'); });
    await assert.rejects(f.client().establish(a), (e: ClientError) => e.code === 'storage_unavailable');
    assert.equal(f.state.admissions.length, 0);
    assert.deepEqual((await f.journal.status('login')).issuance, { sequence: 0n });
    mock.mock.restore();
    await f.client().establish(a);
    assert.equal(f.state.issues, 1);
    assert.equal(f.state.reconciles, 1);
  } finally { f.store.close(); }
});

test('exact evidence owns Buffer input before an asynchronous journal boundary', async () => {
  const f = await fixture();
  try {
    const claim = await f.journal.claimCredential('login');
    const frame = credentialBytes(f.material.credential, 131_072);
    const borrowed = Buffer.from(frame);
    const pending = f.journal.completeCredential('login', claim.sequence, borrowed, f.material.admissionUntilNs);
    borrowed.fill(99);
    await pending;
    const saved = await f.journal.claimCredential('login');
    assert.ok(saved.kind === 'saved');
    assert.deepEqual(saved.bytes, frame);
  } finally { f.store.close(); }
});

test('logout fences in-flight renewal and removes retained proof evidence', async () => {
  const f = await fixture();
  const entered = deferred<void>();
  const release = deferred<void>();
  let pending: Promise<unknown> | undefined;
  try {
    await f.client().establish(a);
    f.state.now = f.material.admissionUntilNs;
    f.transport.renew = async () => {
      entered.resolve(); await release.promise;
      return { ...f.material, admissionUntilNs: f.state.now + 60_000n * MS };
    };
    pending = f.client().establish(a);
    await entered.promise;
    await f.journal.begin('login', f.state.now + 10_000n * MS);
    release.resolve();
    await assert.rejects(pending, (e: ClientError) => e.code === 'stale_generation');
    assert.equal(f.state.admissions.length, 1);
    assert.deepEqual((await f.journal.status('login')).issuance, { sequence: 1n });
  } finally { release.resolve(); await pending?.catch(() => {}); f.store.close(); }
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
    f.transport.admit = async () => { entered.resolve(); await result.promise; return receipt(); };
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

test('timeout preserves uncertainty and aborts the per-call identity capability', async t => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const f = await fixture();
  try {
    let retained: Parameters<PortableAdmissionTransport['issue']>[0] | undefined;
    const entered = deferred<void>();
    const late = deferred<AdmissionCredential>();
    f.transport.issue = async caller => { retained = caller; entered.resolve(); return late.promise; };
    const client = new PortableAdmissionClient(f.journal, 'login', { browser: identity, account }, issuer, f.transport, 5);
    const timedOut = assert.rejects(client.establish(a), (e: ClientError) => e.code === 'timeout');
    await entered.promise; // Exercise cancellation after transport entry, independent of IDB scheduling.
    t.mock.timers.tick(5);
    await timedOut;
    assert.ok(retained);
    // Cancellation must refuse before inspecting or signing any request.
    await assert.rejects(retained.transformRequest({} as Parameters<typeof retained.transformRequest>[0]),
      (e: ClientError) => e.code === 'timeout');
    assert.equal((await f.journal.status('login')).issuanceUnknown, true);
    assert.equal(f.state.admissions.length, 0);
    late.resolve(f.material);
    await new Promise(resolve => setImmediate(resolve));
    assert.equal(f.state.admissions.length, 0);
  } finally { f.store.close(); }
});

test('invalid identity binding and failed journal access refuse before issuance', async () => {
  const f = await fixture();
  try {
    const wrong = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(4));
    await assert.rejects(new PortableAdmissionClient(f.journal, 'login', { browser: wrong, account }, issuer, f.transport).establish(a),
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
    assert.equal((await f.journal.claimCredential('login')).kind, 'issue');
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
