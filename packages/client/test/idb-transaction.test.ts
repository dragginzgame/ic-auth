import { test } from 'node:test';
import assert from 'node:assert/strict';
import { IndexedDbLogoutStore, ClientError, type LogoutJournal } from '../src/index.js';

// Drive real public journal operations while separating request success from commit.
async function transaction() {
  let automatic = true;
  let durability: IDBTransactionDurability = 'strict';
  let tx!: ReturnType<typeof next>;
  function next() {
    const current = {
      durability, onabort: undefined as (() => void) | undefined,
      oncomplete: undefined as (() => void) | undefined, error: new Error('storage'),
      abort() { queueMicrotask(() => current.onabort?.()); },
      objectStore: () => ({ keyPath: null, autoIncrement: false, indexNames: [], put() {},
        get(key: string) {
          const request = { result: key === 'format' ? 'ic-auth-logout-journal-v1' : undefined,
            onsuccess: undefined as (() => void) | undefined };
          queueMicrotask(() => {
            request.onsuccess?.();
            if (automatic && key === 'login') queueMicrotask(() => current.oncomplete?.());
          });
          return request;
        },
      }),
    };
    return current;
  }
  const db = { close() {}, onversionchange: undefined,
    objectStoreNames: { length: 1, contains: (name: string) => name === 'journal' },
    transaction() { tx = next(); return tx; } };
  const factory = { open() {
    const request = { result: db, onsuccess: undefined as (() => void) | undefined };
    queueMicrotask(() => request.onsuccess?.());
    return request;
  } } as unknown as IDBFactory;
  const store = await IndexedDbLogoutStore.open(factory, 'journal');
  automatic = false;
  return { store, current: () => tx, relaxed: () => { durability = 'relaxed'; } };
}
const error = (code: string) => (cause: unknown) => cause instanceof ClientError && cause.code === code;
const record: LogoutJournal = { generation: 'login', caller: 'caller', issuer: 'issuer', clockId: 'test',
  observedNs: 0n, busyUntilNs: 0n, phase: 'active', deadlineNs: 0n, issuanceUnknown: true,
  targets: [{ canister: 'issuer', admissionUntilNs: 1n, attempts: 0, nextNs: 0n }] };

test('staged journal result waits for commit and late abort retains storage error mapping', async () => {
  const f = await transaction();
  let finished = false;
  const pending = f.store.transact(() => ({ record, value: 42 }));
  void pending.then(() => { finished = true; }, () => { finished = true; });
  await Promise.resolve(); await Promise.resolve();
  assert.equal(finished, false);
  f.current().onabort?.();
  await assert.rejects(pending, error('storage_unavailable'));
  const committed = f.store.transact(() => ({ record, value: 42 }));
  await Promise.resolve(); await Promise.resolve();
  f.current().oncomplete?.();
  assert.equal(await committed, 42);
  f.store.close();
});

test('journal strict durability, timeout, incomplete requests and callback failure cannot report success', async t => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const relaxed = await transaction(); relaxed.relaxed();
  await assert.rejects(relaxed.store.transact(() => assert.fail('must not execute')), error('storage_unavailable'));
  const timed = await transaction();
  let timedOperations = 0;
  const timeout = assert.rejects(timed.store.transact(() => { timedOperations++; return undefined; }), error('timeout'));
  t.mock.timers.tick(5_000); await timeout;
  await Promise.resolve(); await Promise.resolve();
  assert.equal(timedOperations, 0, 'late request events must not execute an aborted mutation');
  const missing = await transaction();
  const noResult = missing.store.transact(() => undefined);
  missing.current().oncomplete?.();
  await assert.rejects(noResult, error('storage_corrupt'));
  const failed = await transaction();
  const refused = failed.store.transact(() => { throw undefined; });
  await Promise.resolve(); await Promise.resolve();
  failed.current().oncomplete?.(); // Even spurious completion with a falsy failure refuses.
  await assert.rejects(refused, cause => cause === undefined);
  for (const f of [relaxed, timed, missing, failed]) f.store.close();
});
