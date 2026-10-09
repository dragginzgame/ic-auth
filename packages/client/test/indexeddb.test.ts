import { test } from 'node:test';
import assert from 'node:assert/strict';
import { IDBFactory } from 'fake-indexeddb';
import { ClientError, IndexedDbTokenStore, TokenSessionClient, type IndexedDbTokenStoreOptions, type StoredToken } from '../src/index.js';
import { harness } from './support.js';

const options = (factory = new IDBFactory()): IndexedDbTokenStoreOptions => ({
  factory, databaseName: `test-${crypto.randomUUID()}`, maximumEntries: 16,
  maximumEntryBytes: 131_072, maximumKeyBytes: 131_072, timeoutMs: 500,
});
const intent = (): StoredToken => ({ kind: 'prepare', request: new Uint8Array([1, 2]), candidate: 0, deadlineNs: 10n });
const error = (code: string) => (value: unknown) => value instanceof ClientError && value.code === code;
const request = <T>(operation: IDBRequest<T>): Promise<T> => new Promise((resolve, reject) => {
  operation.onsuccess = () => resolve(operation.result); operation.onerror = () => reject(operation.error);
});
const complete = (tx: IDBTransaction) => new Promise<void>((resolve, reject) => {
  tx.oncomplete = () => resolve(); tx.onabort = () => reject(tx.error);
});

test('independent connections serialize CAS; stored and returned bytes are detached', async () => {
  const configured = options();
  const a = await IndexedDbTokenStore.open(configured); const b = await IndexedDbTokenStore.open(configured);
  try {
    const material = intent();
    const pending = a.write('scope', 0n, material);
    if (material.kind === 'prepare') material.request[0] = 99;
    const results = await Promise.all([pending, b.write('scope', 0n, intent())]);
    assert.deepEqual(results, [true, false]);
    const saved = await b.read('scope');
    assert.equal(saved.entry?.kind === 'prepare' && saved.entry.request[0], 1);
    if (saved.entry?.kind === 'prepare') saved.entry.request[0] = 99;
    assert.equal((await a.read('scope')).entry?.kind === 'prepare' && saved.revision > 0n, true);
    const another = await b.read('scope');
    assert.equal(another.entry?.kind === 'prepare' && another.entry.request[0], 1);
    assert.equal(await b.write('scope', saved.revision, undefined), true);
    assert.equal(await a.write('scope', 0n, intent()), false);
    assert.equal(await a.write('scope', saved.revision, intent()), false);
  } finally { a.close(); b.close(); }
});

test('capacity is atomic across keys/connections and tombstones count against it', async () => {
  const configured = { ...options(), maximumEntries: 1 };
  const a = await IndexedDbTokenStore.open(configured); const b = await IndexedDbTokenStore.open(configured);
  try {
    const results = await Promise.allSettled([a.write('a', 0n, intent()), b.write('b', 0n, intent())]);
    assert.equal(results.filter(result => result.status === 'fulfilled' && result.value).length, 1);
    assert.equal(results.filter(result => result.status === 'rejected' && error('storage_capacity')(result.reason)).length, 1);
    const key = (await a.read('a')).revision === 0n ? 'b' : 'a';
    const current = await a.read(key);
    assert.equal(await a.write(key, current.revision, undefined), true);
    await assert.rejects(a.write('another', 0n, intent()), error('storage_capacity'));
    assert.equal(await b.write(key, (await a.read(key)).revision, intent()), true);
  } finally { a.close(); b.close(); }
});

test('a reopened store reconciles a lost prepare reply using the original intent', async () => {
  const configured = options(); const h = harness();
  const store = await IndexedDbTokenStore.open(configured);
  const base = h.options.issuer;
  const issuer: typeof base = (session, scope) => {
    const operations = base(session, scope);
    return { ...operations, prepare: async (value, signal) => { await operations.prepare(value, signal); throw new Error('lost reply'); } };
  };
  try { await assert.rejects(new TokenSessionClient({ ...h.options, issuer, store }).token(h.configured), error('prepare_uncertain')); }
  finally { store.close(); }
  const reopened = await IndexedDbTokenStore.open(configured);
  try {
    await new TokenSessionClient({ ...h.options, issuer, store: reopened }).token(h.configured);
    assert.equal(h.state.prepares, 1); assert.equal(h.state.ids, 1); assert.equal(h.state.reconciles, 1);
  } finally { reopened.close(); }
});

test('a late transaction abort never reports success or leaves partial capacity/revision writes', async () => {
  const factory = new IDBFactory(); const configured = options(factory);
  let abortNext = false;
  const originalOpen = factory.open.bind(factory);
  factory.open = (...args: Parameters<IDBFactory['open']>) => {
    const operation = originalOpen(...args);
    operation.addEventListener('success', () => {
      const database = operation.result; const transact = database.transaction.bind(database);
      database.transaction = (...arguments_: Parameters<IDBDatabase['transaction']>) => {
        const tx = transact(...arguments_); const get = tx.objectStore.bind(tx);
        tx.objectStore = name => {
          const store = get(name); const put = store.put.bind(store);
          store.put = (...putArgs: Parameters<IDBObjectStore['put']>) => {
            const update = put(...putArgs);
            if (name === 'tokens' && abortNext) {
              abortNext = false; update.addEventListener('success', () => tx.abort(), { once: true });
            }
            return update;
          };
          return store;
        };
        return tx;
      };
    });
    return operation;
  };
  const store = await IndexedDbTokenStore.open({ ...configured, maximumEntries: 1 });
  try {
    abortNext = true;
    await assert.rejects(store.write('a', 0n, intent()), error('storage_unavailable'));
    assert.deepEqual(await store.read('a'), { revision: 0n, entry: undefined });
    assert.equal(await store.write('b', 0n, intent()), true);
    assert.equal(await store.write('b', 0n, undefined), false);
  } finally { store.close(); }
});

test('unknown schema and incompatible stored profile fail without clearing existing data', async () => {
  const factory = new IDBFactory(); const configured = options(factory);
  const opening = factory.open(configured.databaseName, 1);
  opening.onupgradeneeded = () => opening.result.createObjectStore('business').put('keep', 'record');
  const foreign = await request(opening);
  try {
    await assert.rejects(IndexedDbTokenStore.open(configured), error('storage_corrupt'));
    assert.equal(await request(foreign.transaction('business').objectStore('business').get('record')), 'keep');
  } finally { foreign.close(); }
  const dedicated = { ...options(factory), maximumEntries: 1 };
  const store = await IndexedDbTokenStore.open(dedicated);
  await store.write('scope', 0n, intent()); store.close();
  await assert.rejects(IndexedDbTokenStore.open({ ...dedicated, maximumEntries: 2 }), error('configuration'));
  const restored = await IndexedDbTokenStore.open(dedicated);
  try { assert.equal((await restored.read('scope')).entry?.kind, 'prepare'); } finally { restored.close(); }
});

test('corrupt retained entries refuse both reading and overwriting', async () => {
  const configured = options(); const store = await IndexedDbTokenStore.open(configured);
  const raw = await request(configured.factory.open(configured.databaseName, 1));
  try {
    const tx = raw.transaction('tokens', 'readwrite'); const finished = complete(tx);
    tx.objectStore('tokens').put({ revision: 10n, entry: intent() }, 'corrupt'); await finished;
    await assert.rejects(store.read('corrupt'), error('storage_corrupt'));
    await assert.rejects(store.write('corrupt', 10n, undefined), error('storage_corrupt'));
    const retained: unknown = await request(raw.transaction('tokens').objectStore('tokens').get('corrupt'));
    assert.deepEqual(retained, { revision: 10n, entry: intent() });
  } finally { store.close(); raw.close(); }
});

test('oversized keys/material and closed connections fail without allocating entries', async () => {
  const configured = { ...options(), maximumEntryBytes: 2, maximumKeyBytes: 4 };
  const store = await IndexedDbTokenStore.open(configured);
  await assert.rejects(store.write('larger', 0n, intent()), error('invalid_material'));
  await assert.rejects(store.write('a', 0n, { kind: 'ready', token: new Uint8Array(3) }), error('invalid_material'));
  await assert.rejects(store.write('a', -1n, intent()), error('invalid_material'));
  assert.deepEqual(await store.read('a'), { revision: 0n, entry: undefined });
  store.close(); await assert.rejects(store.read('a'), error('storage_unavailable'));
});

test('version-change closes this connection and an unsupported format is never upgraded/reset', async () => {
  const configured = options(); const store = await IndexedDbTokenStore.open(configured);
  await store.write('scope', 0n, intent());
  const upgraded = await request(configured.factory.open(configured.databaseName, 2));
  try {
    await assert.rejects(store.read('scope'), error('storage_unavailable'));
    await assert.rejects(IndexedDbTokenStore.open(configured), error('storage_unavailable'));
    assert.equal((await request(upgraded.transaction('tokens').objectStore('tokens').get('scope'))).entry.kind, 'prepare');
  } finally { upgraded.close(); }
});

test('a transaction blocked by another connection times out without changing stored intent', async () => {
  const configured = options(); const base = await IndexedDbTokenStore.open(configured);
  await base.write('scope', 0n, intent());
  const bounded = await IndexedDbTokenStore.open({ ...configured, timeoutMs: 10 });
  const raw = await request(configured.factory.open(configured.databaseName, 1));
  let hold = true;
  const lock = raw.transaction(['tokens', 'metadata'], 'readwrite'); const finished = complete(lock);
  const tokens = lock.objectStore('tokens');
  const keepAlive = () => { if (hold) tokens.get('scope').onsuccess = keepAlive; };
  await new Promise<void>(resolve => { tokens.get('scope').onsuccess = () => { keepAlive(); resolve(); }; });
  try {
    await assert.rejects(bounded.read('scope'), error('timeout'));
    hold = false; await finished;
    assert.equal((await bounded.read('scope')).entry?.kind, 'prepare');
  } finally { hold = false; base.close(); bounded.close(); raw.close(); }
});

test('a matching profile does not admit a different object-store schema', async () => {
  const configured = options(); const store = await IndexedDbTokenStore.open(configured); store.close();
  const owned = await request(configured.factory.open(configured.databaseName, 1));
  const profile: unknown = await request(owned.transaction('metadata').objectStore('metadata').get('profile')); owned.close();
  const foreignName = configured.databaseName + '-foreign';
  const foreign = configured.factory.open(foreignName, 1);
  foreign.onupgradeneeded = () => {
    foreign.result.createObjectStore('tokens', { autoIncrement: true }).put('keep', 'business');
    foreign.result.createObjectStore('metadata').put(profile, 'profile');
  };
  const db = await request(foreign);
  try {
    await assert.rejects(IndexedDbTokenStore.open({ ...configured, databaseName: foreignName }), error('storage_corrupt'));
    assert.equal(await request(db.transaction('tokens').objectStore('tokens').get('business')), 'keep');
  } finally { db.close(); }
});

test('unavailable strict durability refuses admission without a fallback or changing intent', async () => {
  const configured = options(); const original = await IndexedDbTokenStore.open(configured);
  await original.write('scope', 0n, intent()); original.close();
  const open = configured.factory.open.bind(configured.factory);
  configured.factory.open = (...args: Parameters<IDBFactory['open']>) => {
    const operation = open(...args);
    operation.addEventListener('success', () => {
      const database = operation.result; const transact = database.transaction.bind(database);
      database.transaction = (stores, mode, selected) => {
        assert.equal(selected?.durability, 'strict');
        return transact(stores, mode, { durability: 'relaxed' });
      };
    });
    return operation;
  };
  await assert.rejects(IndexedDbTokenStore.open(configured), error('storage_unavailable'));
  configured.factory.open = open;
  const reopened = await IndexedDbTokenStore.open(configured);
  try { assert.deepEqual(await reopened.read('scope'), { revision: 1n, entry: intent() }); }
  finally { reopened.close(); }
});

test('revision exhaustion preserves existing intent and never wraps or admits another key', async () => {
  const configured = options(); const store = await IndexedDbTokenStore.open(configured);
  await store.write('scope', 0n, intent());
  const raw = await request(configured.factory.open(configured.databaseName, 1));
  try {
    const previous: unknown = await request(raw.transaction('metadata').objectStore('metadata').get('profile'));
    assert.equal(typeof previous, 'object'); assert.ok(previous !== null);
    const profile = { ...previous, revision: (1n << 64n) - 1n };
    const tx = raw.transaction('metadata', 'readwrite'); const finished = complete(tx);
    tx.objectStore('metadata').put(profile, 'profile'); await finished;
    await assert.rejects(store.write('scope', 1n, undefined), error('storage_capacity'));
    await assert.rejects(store.write('another', 0n, intent()), error('storage_capacity'));
    assert.equal(await store.write('scope', 0n, intent()), false);
    assert.deepEqual(await store.read('scope'), { revision: 1n, entry: intent() });
    assert.deepEqual(await store.read('another'), { revision: 0n, entry: undefined });
    assert.deepEqual(await request(raw.transaction('metadata').objectStore('metadata').get('profile')), profile);
    assert.equal(await request(raw.transaction('tokens').objectStore('tokens').count()), 1);
  } finally { store.close(); raw.close(); }
});
