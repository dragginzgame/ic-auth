import { test } from 'node:test';
import assert from 'node:assert/strict';
import { Cbor } from '@icp-sdk/core/agent';
import { IDL } from '@icp-sdk/core/candid';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { Principal } from '@icp-sdk/core/principal';
import { httpLogoutTransport, ClientError } from '../src/index.js';
import { LogoutRequest, LogoutResult } from '../src/generated/protocol.did.js';
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
    const signed = Cbor.decode<{ content: { request_type: string; method_name?: string; arg?: Uint8Array; sender: Uint8Array }; sender_pubkey: Uint8Array; sender_sig: Uint8Array }>(init!.body as Uint8Array);
    assert.equal(Principal.fromUint8Array(signed.content.sender).toText(), identity.getPrincipal().toText());
    assert.deepEqual([...signed.sender_pubkey], [...identity.getPublicKey().toDer()]);
    assert.equal(signed.sender_sig.length, 64);
    if (path.endsWith('/call')) {
      assert.equal(signed.content.request_type, 'call');
      assert.equal(signed.content.method_name, methodName);
      assert.deepEqual(IDL.decode([LogoutRequest], signed.content.arg!), [{}]);
      return new Response(null, { status: 202 });
    }
    assert.equal(signed.content.request_type, 'read_state');
    // A server can supply arbitrary bytes, but cannot acknowledge revocation
    // without a certificate authenticating this exact request's reply.
    return new Response(Cbor.encode({ certificate: new Uint8Array([0]) }));
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
  for (const value of [{ Revoked: { deny_until_ns: 123n } }, { InvalidCaller: null },
    ...['Capacity', 'Storage', 'Clock', 'ResetPending'].map(kind => ({ Unavailable: { [kind]: null } }))]) {
    assert.deepEqual(logoutResultFromBytes(IDL.encode([LogoutResult], [value])), value);
  }
  const reply = IDL.encode([LogoutResult], [{ Revoked: { deny_until_ns: 123n } }]);
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
