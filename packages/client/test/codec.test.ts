import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { ClientError, MemoryTokenStore, tokenBytes, tokenFromBytes, credentialBytes, credentialFromBytes, type PortableCredential } from '../src/index.js';
import { encode, requestBytes, requestFromBytes } from '../src/codec.js';
import { IDL } from '@icp-sdk/core/candid';
import { DelegatedRoleGrant, LogoutResult, LogoutRequest } from '../src/generated/protocol.did.js';
import { harness } from './support.js';

const evidence = new URL('../../../../target/browser-client/', import.meta.url);

test('Rust to TypeScript to Rust preserves canonical request, opt/blob and >2^53 nat64', () => {
  const bytes = new Uint8Array(Buffer.from(readFileSync(new URL('request-vector.hex', evidence), 'utf8').trim(), 'hex'));
  const request = requestFromBytes(bytes, 65_536);
  assert.equal(request.ttl_ns, 9_007_199_254_740_993n);
  assert.deepEqual(request.ext, [new Uint8Array([0, 255])]);
  assert.equal(request.metadata[0]?.ttl_ns, 1_000_000_000n);
  assert.equal(request.aud.Fleet.canonical_network_id, 'a'.repeat(64));
  assert.deepEqual(request.grants, [{ target: 'users', scopes: ['read'] }]);
  writeFileSync(fileURLToPath(new URL('request-from-ts.bin', evidence)), requestBytes(request, 65_536));
});

test('token Candid roundtrip retains principals, exact bigint times and proof blobs', async () => {
  const h = harness(); h.state.now = 9_007_199_254_740_993n;
  const token = await h.client.token(h.configured);
  const decoded = tokenFromBytes(tokenBytes(token, 65_536), 65_536);
  assert.equal(decoded.claims.issued_at_ns, h.state.now);
  assert.equal(decoded.claims.subject.toText(), h.sessions.value?.identity.getPrincipal().toText());
  assert.deepEqual(decoded.issuer_proof, token.issuer_proof);
  const encoded = tokenBytes(token, 65_536);
  assert.throws(() => tokenFromBytes(encoded.subarray(0, encoded.length - 1), 65_536), ClientError);
  assert.throws(() => tokenFromBytes(new Uint8Array([1]), 65_536), ClientError);
  assert.throws(() => tokenBytes(token, 1), ClientError);
  assert.throws(() => tokenFromBytes(new Uint8Array(100), 99), ClientError);
});

test('store CAS copies material, rejects stale writers and retains deletion revision', async () => {
  const store = new MemoryTokenStore(1);
  const entry = { kind: 'prepare' as const, request: new Uint8Array([1]), candidate: 0, deadlineNs: 10n };
  assert.equal(await store.write('a', 0n, entry), true);
  entry.request[0] = 9;
  const first = await store.read('a');
  assert.equal(first.entry?.kind === 'prepare' && first.entry.request[0], 1);
  assert.equal(await store.write('a', 0n, undefined), false);
  assert.equal(await store.write('a', first.revision, undefined), true);
  assert.equal(await store.write('a', 0n, entry), false);
  await assert.rejects(store.write('b', 0n, entry), (error: unknown) => error instanceof ClientError && error.code === 'storage_capacity');
});


test('bounded grant and extension encoders retain exact Candid bytes and typed failures', () => {
  for (const [type, value] of [
    [IDL.Vec(DelegatedRoleGrant), [{ target: 'users', scopes: ['read', 'write'] }]],
    [IDL.Opt(IDL.Vec(IDL.Nat8)), []],
    [IDL.Opt(IDL.Vec(IDL.Nat8)), [new Uint8Array()]],
    [IDL.Opt(IDL.Vec(IDL.Nat8)), [new Uint8Array([0, 255])]],
  ] as const) {
    const bytes = IDL.encode([type], [value]);
    assert.deepEqual(encode(type, value, bytes.length), bytes);
    assert.throws(() => encode(type, value, bytes.length - 1),
      (cause: unknown) => cause instanceof ClientError && cause.code === 'invalid_material');
  }
  assert.throws(() => encode(IDL.Vec(DelegatedRoleGrant), [{ target: 'users', scopes: [3] }], 65_536),
    (cause: unknown) => cause instanceof ClientError && cause.code === 'invalid_material');
});


test('portable Rust-to-TS-to-Rust envelope retains opaque claims and signature bytes', () => {
  const bytes = new Uint8Array(Buffer.from(readFileSync(new URL('credential-vector.hex', evidence), 'utf8').trim(), 'hex'));
  const credential = credentialFromBytes(bytes, 131_072);
  assert.deepEqual(credential.claims, new Uint8Array([0, 1, 127, 128, 255]));
  assert.ok('Ed25519' in credential.proof);
  assert.deepEqual(credential.proof.Ed25519.signature, Uint8Array.from({ length: 64 }, (_, i) => i));
  writeFileSync(fileURLToPath(new URL('credential-from-ts.bin', evidence)), credentialBytes(credential, 131_072));
});

test('portable codec preserves both proof variants and enforces transport byte limits', async () => {
  const h = harness();
  h.state.now = 9_007_199_254_740_993n;
  const token = await h.client.token(h.configured);
  const credentials: PortableCredential[] = [
    { claims: new Uint8Array([0, 255]), proof: { Ed25519: { signature: new Uint8Array(64).fill(7) } } },
    { claims: new Uint8Array([128, 0]), proof: { Certified: {
      delegation: token.proof, issuer_signature: token.issuer_proof.IcCanisterSignatureV1,
    } } },
  ];
  for (const credential of credentials) {
    const bytes = credentialBytes(credential, 131_072);
    assert.deepEqual(credentialFromBytes(bytes, bytes.length), credential);
    assert.throws(() => credentialBytes(credential, bytes.length - 1), ClientError);
    assert.throws(() => credentialFromBytes(bytes, bytes.length - 1), ClientError);
    assert.throws(() => credentialFromBytes(bytes.subarray(0, bytes.length - 1), bytes.length), ClientError);
    assert.throws(() => tokenFromBytes(bytes, bytes.length), ClientError);
  }
});

test('portable codecs reject nonfinite, fractional and unsafe byte limits', () => {
  const credential: PortableCredential = {
    claims: new Uint8Array([0, 255]), proof: { Ed25519: { signature: new Uint8Array(64) } },
  };
  const bytes = credentialBytes(credential, 131_072);
  // Reflect.apply also exercises untyped JavaScript callers omitting/mistyping the limit.
  for (const maximum of [NaN, Infinity, -Infinity, 0, -1, 1.5, bytes.length + 0.5, Number.MAX_SAFE_INTEGER + 1, undefined, null, '131072']) {
    assert.throws(() => Reflect.apply(credentialBytes, undefined, [credential, maximum]),
      (error: unknown) => error instanceof ClientError && error.code === 'invalid_material', `encode limit ${maximum}`);
    assert.throws(() => Reflect.apply(credentialFromBytes, undefined, [bytes, maximum]),
      (error: unknown) => error instanceof ClientError && error.code === 'invalid_material', `decode limit ${maximum}`);
  }
  assert.deepEqual(credentialBytes(credential, bytes.length), bytes);
  assert.deepEqual(credentialFromBytes(bytes, bytes.length), credential);
});

test('invalid codec limits refuse before reading payload fields or input bytes', () => {
  const credential: PortableCredential = {
    claims: new Uint8Array([1]), proof: { Ed25519: { signature: new Uint8Array(64) } },
  };
  const bytes = credentialBytes(credential, 131_072);
  let reads = 0;
  const value = new Proxy(credential, { get: (target, key, receiver) => {
    reads++; return Reflect.get(target, key, receiver);
  } });
  const input = new Proxy(bytes, { get: (target, key) => {
    reads++; return Reflect.get(target, key, target);
  } });
  for (const maximum of [NaN, Infinity, -Infinity, 0, -1, 1.5, Number.MAX_SAFE_INTEGER + 1, undefined, null, '131072']) {
    assert.throws(() => Reflect.apply(credentialBytes, undefined, [value, maximum]), ClientError);
    assert.throws(() => Reflect.apply(credentialFromBytes, undefined, [input, maximum]), ClientError);
  }
  assert.equal(reads, 0);
});

test('Rust logout results preserve every refusal and exact nat64 deadlines in both directions', () => {
  const bytes = new Uint8Array(Buffer.from(readFileSync(new URL('logout-vector.hex', evidence), 'utf8').trim(), 'hex'));
  const type = IDL.Vec(LogoutResult);
  const [results] = IDL.decode([type], bytes);
  assert.deepEqual(results, [
    { Revoked: { deny_until_ns: 9_007_199_254_740_993n } },
    { InvalidCaller: null },
    ...['Capacity', 'Storage', 'Clock', 'ResetPending'].map(reason => ({ Unavailable: { [reason]: null } })),
  ]);
  writeFileSync(fileURLToPath(new URL('logout-from-ts.bin', evidence)), IDL.encode([type], [results]));
  assert.deepEqual(IDL.decode([LogoutRequest], IDL.encode([LogoutRequest], [{}])), [{}]);
});
