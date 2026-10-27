import { test } from 'node:test';
import assert from 'node:assert/strict';
import { HttpAgent } from '@tokostudios/ic-platform/agent';
import { Ed25519KeyIdentity } from '@tokostudios/ic-platform/identity';
import { Principal } from '@tokostudios/ic-platform/principal';
import { httpUpdateTransport } from '../src/update-http.js';

test('HTTP trust configuration owns Buffer and subarray bytes across attempts', async t => {
  const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(7));
  const target = Principal.fromUint8Array(new Uint8Array([1]));
  const stop = new Error('stop at SDK construction');
  const observed: number[][] = [];
  // Inspect the actual adapter-to-SDK boundary; no certificate acceptance is mocked.
  t.mock.method(HttpAgent, 'create', async (options: Parameters<typeof HttpAgent.create>[0]) => {
    assert.ok(options?.rootKey);
    observed.push([...options.rootKey]);
    options.rootKey.fill(8); // Each SDK attempt must also receive a separate copy.
    throw stop;
  });
  for (const backing of [Buffer.from([99, 1, 2, 3, 99]), new Uint8Array([99, 1, 2, 3, 99])]) {
    const update = httpUpdateTransport({ host: 'https://gateway.invalid', methodName: 'auth_logout',
      rootKey: backing.subarray(1, 4) });
    backing.fill(9); // Caller mutation after construction must not change trust.
    for (let attempt = 0; attempt < 2; attempt++) {
      await assert.rejects(update(target, identity, new Uint8Array(), new AbortController().signal), error => error === stop);
    }
  }
  assert.deepEqual(observed, [[1, 2, 3], [1, 2, 3], [1, 2, 3], [1, 2, 3]]);
});

test('HTTP submission and polling keep the destination captured before SDK creation', async () => {
  const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(7));
  const target = Principal.fromUint8Array(new Uint8Array([1]));
  const original = target.toText();
  const paths: string[] = [];
  const update = httpUpdateTransport({ host: 'https://gateway.invalid', methodName: 'auth_logout',
    fetch: async input => {
      const path = new URL(input.toString()).pathname;
      paths.push(path);
      if (path.endsWith('/call')) return new Response(null, { status: 202 });
      return new Response(new Uint8Array([0])); // Cannot authenticate a reply.
    } });
  const pending = update(target, identity, new Uint8Array(), new AbortController().signal);
  target.toUint8Array().set([2]);
  await assert.rejects(pending);
  assert.deepEqual(paths, [`/api/v2/canister/${original}/call`, `/api/v3/canister/${original}/read_state`]);
});
