import { test } from 'node:test';
import assert from 'node:assert/strict';
import { openDatabase } from '../src/idb-open.js';
import { ClientError } from '../src/contracts.js';

function opening() {
  let closed = 0, aborted = 0;
  const request = {
    onblocked: undefined as (() => void) | undefined,
    onerror: undefined as (() => void) | undefined,
    onsuccess: undefined as (() => void) | undefined,
    onupgradeneeded: undefined as ((event: IDBVersionChangeEvent) => void) | undefined,
    result: { close() { closed++; } },
    transaction: { abort() { aborted++; } }, error: new Error('open failed'),
  };
  return { request, open: () => request as unknown as IDBOpenDBRequest,
    counts: () => ({ closed, aborted }) };
}
const unchanged = (cause: unknown) => cause;

test('timeout and blocked opens abort late upgrades and close late connections', async () => {
  for (const blocked of [false, true]) {
    const f = opening();
    const pending = openDatabase(f.open, 1, unchanged, () => assert.fail('abandoned schema mutation'));
    if (blocked) f.request.onblocked?.();
    await assert.rejects(pending, (e: ClientError) => e.code === (blocked ? 'storage_unavailable' : 'timeout'));
    f.request.onupgradeneeded?.({ oldVersion: 0 } as IDBVersionChangeEvent);
    f.request.onsuccess?.();
    assert.deepEqual(f.counts(), { closed: 1, aborted: 1 });
  }
});

test('schema refusal aborts without resolving, and preserves the projected error', async () => {
  const f = opening();
  const refusal = new Error('schema refused');
  const pending = openDatabase(f.open, 100, unchanged, () => { throw refusal; });
  f.request.onupgradeneeded?.({ oldVersion: 0 } as IDBVersionChangeEvent);
  await assert.rejects(pending, e => e === refusal);
  f.request.onsuccess?.();
  assert.deepEqual(f.counts(), { closed: 1, aborted: 1 });
  const success = opening();
  const ready = openDatabase(success.open, 100, unchanged, () => {});
  success.request.onsuccess?.();
  assert.equal(await ready, success.request.result);
  assert.deepEqual(success.counts(), { closed: 0, aborted: 0 });
});
