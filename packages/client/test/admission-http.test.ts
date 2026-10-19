import { test } from 'node:test';
import assert from 'node:assert/strict';
import { Cbor } from '@icp-sdk/core/agent';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { Principal } from '@icp-sdk/core/principal';
import { ClientError, httpAdmissionTransport, type PortableCredential } from '../src/index.js';
import { decodeRequest } from '../src/generated/establish.js';

const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(7));
const target = Principal.fromUint8Array(new Uint8Array([1]));
const credential: PortableCredential = {
  claims: new Uint8Array([1, 2]), proof: { tag: 'Ed25519', value: new Uint8Array(64) },
};
const options = { host: 'https://gateway.invalid', methodName: 'establish_portable_session', scopes: ['read'] };

test('HTTP admission signs the exact credential and fixed scopes; HTTP acceptance is not admission', async () => {
  const scopes = ['read'];
  const paths: string[] = [];
  const controller = new AbortController();
  const admit = httpAdmissionTransport({ ...options, scopes, ttlSecs: 30n, fetch: async (input, init) => {
    const path = new URL(input.toString()).pathname;
    paths.push(path);
    assert.equal(init?.signal, controller.signal);
    const signed = Cbor.decode<{ content: { method_name?: string; arg?: Uint8Array; sender: Uint8Array }; sender_pubkey: Uint8Array }>(init!.body as Uint8Array);
    assert.equal(Principal.fromUint8Array(signed.content.sender).toText(), identity.getPrincipal().toText());
    assert.deepEqual([...signed.sender_pubkey], [...identity.getPublicKey().toDer()]);
    if (path.endsWith('/call')) {
      assert.equal(signed.content.method_name, options.methodName);
      const request = decodeRequest(signed.content.arg!);
      assert.deepEqual([...request.credential.claims], [...credential.claims]);
      assert.equal(request.credential.proof.tag, 'Ed25519');
      assert.deepEqual([...request.credential.proof.value as Uint8Array], new Array(64).fill(0));
      assert.deepEqual(request.scopes, ['read']);
      assert.deepEqual(request.ttl_secs, { some: 30n });
      return new Response(null, { status: 202 });
    }
    return new Response(Cbor.encode({ certificate: new Uint8Array([0]) }));
  } });
  scopes[0] = 'write';
  await assert.rejects(admit(target, identity, credential, controller.signal));
  assert.deepEqual(paths, [`/api/v2/canister/${target}/call`, `/api/v3/canister/${target}/read_state`]);
});

test('HTTP admission refuses oversized material before network effects and respects pre-dispatch cancellation', async () => {
  let calls = 0;
  const admit = httpAdmissionTransport({ ...options, fetch: async () => { calls++; return new Response(null, { status: 503 }); } });
  await assert.rejects(admit(target, identity, { ...credential, claims: new Uint8Array(1_048_577) }, new AbortController().signal),
    (error: unknown) => error instanceof ClientError && error.code === 'invalid_material');
  const controller = new AbortController();
  controller.abort();
  await assert.rejects(admit(target, identity, credential, controller.signal));
  assert.equal(calls, 0);
});

test('HTTP admission does not retry a failed update and uses the current signing capability', async () => {
  let calls = 0;
  const admit = httpAdmissionTransport({ ...options, fetch: async () => { calls++; return new Response(null, { status: 503 }); } });
  const signal = new AbortController().signal;
  await assert.rejects(admit(target, identity, credential, signal));
  assert.equal(calls, 1);
  await assert.rejects(admit(target, {
    getPrincipal: () => identity.getPrincipal(),
    transformRequest: async () => { throw new ClientError('stale_generation'); },
  }, credential, signal));
  assert.equal(calls, 1);
});
