import { test } from 'node:test';
import assert from 'node:assert/strict';
import { inspectIngress } from '@tokostudios/ic-platform/agent';
import { readStateResponseFixture } from '@tokostudios/ic-platform/testing';
import { Ed25519KeyIdentity } from '@tokostudios/ic-platform/identity';
import { Principal } from '@tokostudios/ic-platform/principal';
import { httpLogoutTransport, ClientError } from '../src/index.js';
import { decodeRequest, encodeResponse, type Response } from '../src/generated/logout.js';
import { logoutResultFromBytes } from '../src/codec.js';

const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(7));
const target = Principal.fromUint8Array(new Uint8Array([1]));
const host = 'https://gateway.invalid';
const methodName = 'auth_logout';
const dispose = async () => {};

test('HTTP logout signs one empty update with the supplied identity; HTTP acceptance is not success', async () => {
  const paths: string[] = [];
  const controller = new AbortController();
  const transport = httpLogoutTransport({ host, methodName, dispose, fetch: async (input, init) => {
    const path = new URL(input.toString()).pathname;
    paths.push(path);
    assert.equal(init?.signal, controller.signal);
    const signed = inspectIngress(init!.body as Uint8Array);
    assert.equal(signed.sender.toText(), identity.getPrincipal().toText());
    assert(signed.senderPublicKey);
    assert.deepEqual([...signed.senderPublicKey], [...identity.getPublicKey().toDer()]);
    assert.equal(signed.senderSignature?.length, 64);
    if (path.endsWith('/call')) {
      assert(signed.requestType === 'call');
      assert.equal(signed.methodName, methodName);
      assert.equal(decodeRequest(signed.arg), null);
      return new Response(null, { status: 202 });
    }
    assert.equal(signed.requestType, 'read_state');
    // A server can supply arbitrary bytes, but cannot acknowledge revocation
    // without a certificate authenticating this exact request's reply.
    return new Response(new Uint8Array(readStateResponseFixture(new Uint8Array([0]))).buffer);
  } });
  await assert.rejects(transport.revoke(target, identity, controller.signal));
  assert.deepEqual(paths, [`/api/v2/canister/${target}/call`, `/api/v3/canister/${target}/read_state`]);
});

test('HTTP failure is not retried inside a durable logout attempt', async () => {
  let calls = 0;
  const transport = httpLogoutTransport({ host, methodName, dispose, fetch: async () => {
    calls++;
    return new Response(null, { status: 503 });
  } });
  await assert.rejects(transport.revoke(target, identity, new AbortController().signal));
  assert.equal(calls, 1);
});

test('abort before dispatch and during a lost reply prevents further polling', async () => {
  const controller = new AbortController();
  let calls = 0;
  const transport = httpLogoutTransport({ host, methodName, dispose, fetch: async () => {
    calls++;
    controller.abort();
    return new Response(null, { status: 202 });
  } });
  await assert.rejects(transport.revoke(target, identity, controller.signal));
  assert.equal(calls, 1);
  await assert.rejects(transport.revoke(target, identity, controller.signal));
  assert.equal(calls, 1);
});

test('each HTTP attempt uses its current guarded identity, never a cached signer', async () => {
  let calls = 0;
  let signs = 0;
  const transport = httpLogoutTransport({ host, methodName, dispose, fetch: async () => {
    calls++; return new Response(null, { status: 503 });
  } });
  await assert.rejects(transport.revoke(target, identity, new AbortController().signal));
  await assert.rejects(transport.revoke(target, {
    getPrincipal: () => identity.getPrincipal(),
    transformRequest: async () => { signs++; throw new ClientError('stale_generation'); },
  }, new AbortController().signal));
  assert.equal(signs, 1);
  assert.equal(calls, 1);
});

test('shared logout result decoding preserves typed failures and rejects malformed/unbounded replies', () => {
  const values: Response[] = [{ tag: 'Revoked', value: 123n }, { tag: 'InvalidCaller', value: null },
    ...(['Capacity', 'Storage', 'Clock', 'ResetPending'] as const).map(tag => ({ tag: 'Unavailable' as const, value: { tag, value: null } }))];
  for (const value of values) assert.deepEqual(logoutResultFromBytes(encodeResponse(value)), value);
  const reply = encodeResponse({ tag: 'Revoked', value: 123n });
  assert.throws(() => logoutResultFromBytes(reply.slice(0, -1)), ClientError);
  assert.throws(() => logoutResultFromBytes(new Uint8Array(1_025)), ClientError);
});

test('transport requires explicit endpoint configuration and leaves disposal with the host', async () => {
  for (const value of ['', 'auth/logout', 'x'.repeat(129)]) {
    assert.throws(() => httpLogoutTransport({ host, methodName: value, dispose }), ClientError);
  }
  assert.throws(() => httpLogoutTransport({ host: 'https://user:pass@gateway.invalid', methodName, dispose }), ClientError);
  const generations: string[] = [];
  const transport = httpLogoutTransport({ host, methodName, dispose: async generation => { generations.push(generation); } });
  await transport.dispose('login-a');
  assert.deepEqual(generations, ['login-a']);
});
