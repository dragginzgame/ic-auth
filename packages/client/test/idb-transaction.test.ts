import { test } from 'node:test';
import assert from 'node:assert/strict';
import { strictTransaction } from '../src/idb-transaction.js';
import { ClientError } from '../src/contracts.js';

// Inject transaction lifecycle events separately from request success.
function transaction(durability: IDBTransactionDurability = 'strict') {
  const tx = {
    durability, onabort: undefined as (() => void) | undefined,
    oncomplete: undefined as (() => void) | undefined, error: new Error('storage'),
    abort() { queueMicrotask(() => tx.onabort?.()); },
  };
  return { tx, open: () => tx as unknown as IDBTransaction };
}
const storageFailure = (cause: unknown) => new ClientError('storage_unavailable', { cause });
const error = (code: string) => (cause: unknown) => cause instanceof ClientError && cause.code === code;

test('staged result waits for commit and late abort retains storage error mapping', async () => {
  const f = transaction();
  let finished = false;
  const pending = strictTransaction(f.open, 100, storageFailure, (_tx, result) => result(42));
  void pending.then(() => { finished = true; }, () => { finished = true; });
  await Promise.resolve();
  assert.equal(finished, false);
  f.tx.onabort?.();
  await assert.rejects(pending, error('storage_unavailable'));
  const success = transaction();
  const committed = strictTransaction(success.open, 100, storageFailure, (_tx, result) => result(42));
  success.tx.oncomplete?.();
  assert.equal(await committed, 42);
});

test('strict durability, timeout, missing result and callback failure cannot report success', async () => {
  const relaxed = transaction('relaxed');
  await assert.rejects(strictTransaction(relaxed.open, 100, storageFailure, () => assert.fail('must not execute')), error('storage_unavailable'));
  const timed = transaction();
  await assert.rejects(strictTransaction(timed.open, 1, storageFailure, (_tx, result) => result(42)), error('timeout'));
  const missing = transaction();
  const noResult = strictTransaction(missing.open, 100, storageFailure, () => {});
  missing.tx.oncomplete?.();
  await assert.rejects(noResult, error('storage_corrupt'));
  const failed = transaction();
  const refused = strictTransaction(failed.open, 100, storageFailure, (_tx, result, fail) => { result(42); fail(undefined); });
  // Even a spurious completion and a falsy error must not acknowledge the write.
  failed.tx.oncomplete?.();
  await assert.rejects(refused, cause => cause === undefined);
});
