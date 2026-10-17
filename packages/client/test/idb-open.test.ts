import { test } from 'node:test';
import assert from 'node:assert/strict';
import { IndexedDbLogoutStore, ClientError } from '../src/index.js';

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
  return { request, factory: { open: () => request } as unknown as IDBFactory,
    counts: () => ({ closed, aborted }) };
}

test('timeout and blocked journal opens abort late upgrades and close late connections', async t => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  for (const blocked of [false, true]) {
    const f = opening();
    const pending = IndexedDbLogoutStore.open(f.factory, 'journal');
    const refused = assert.rejects(pending, (e: ClientError) => e.code === 'storage_unavailable'
      && e.cause instanceof ClientError && e.cause.code === (blocked ? 'storage_unavailable' : 'timeout'));
    if (blocked) f.request.onblocked?.(); else t.mock.timers.tick(5_000);
    await refused;
    f.request.onupgradeneeded?.({ oldVersion: 0 } as IDBVersionChangeEvent);
    f.request.onsuccess?.();
    assert.deepEqual(f.counts(), { closed: 1, aborted: 1 });
  }
});

test('journal schema refusal aborts and closes a late connection without resetting it', async () => {
  const f = opening();
  const pending = IndexedDbLogoutStore.open(f.factory, 'journal');
  f.request.onupgradeneeded?.({ oldVersion: 1 } as IDBVersionChangeEvent);
  await assert.rejects(pending, (e: ClientError) => e.code === 'storage_unavailable'
    && e.cause instanceof ClientError && e.cause.code === 'storage_corrupt');
  f.request.onsuccess?.();
  assert.deepEqual(f.counts(), { closed: 1, aborted: 1 });
});
