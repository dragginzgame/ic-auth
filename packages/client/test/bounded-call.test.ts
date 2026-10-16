import { test } from 'node:test';
import assert from 'node:assert/strict';
import { boundedCall } from '../src/bounded-call.js';
import { ClientError } from '../src/contracts.js';

test('bounded calls abort transport access after success and failure', async () => {
  let captured: AbortSignal | undefined;
  assert.equal(await boundedCall(100, async signal => { captured = signal; return 42; }), 42);
  assert.equal(captured!.aborted, true);
  const refusal = new Error('transport refusal');
  for (const synchronous of [true, false]) {
    await assert.rejects(boundedCall(100, signal => {
      captured = signal;
      if (synchronous) throw refusal;
      return Promise.reject(refusal);
    }), cause => cause === refusal);
    assert.equal(captured!.aborted, true);
  }
});

test('a transport ignoring cancellation cannot settle the call after timeout', async () => {
  let captured: AbortSignal | undefined;
  let late!: (value: number) => void;
  const pending = boundedCall(1, signal => {
    captured = signal;
    return new Promise<number>(resolve => { late = resolve; });
  });
  await assert.rejects(pending, (cause: ClientError) => cause.code === 'timeout');
  assert.equal(captured!.aborted, true);
  late(42);
  await assert.rejects(pending, (cause: ClientError) => cause.code === 'timeout');
});
