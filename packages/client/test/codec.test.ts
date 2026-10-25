import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { AdmissionError, ClientError, credentialBytes, credentialFromBytes, type PortableCredential } from '../src/index.js';
import { logoutResultFromBytes } from '../src/codec.js';
import * as logout from '../src/generated/logout.js';
import * as admission from '../src/generated/establish.js';
const evidence = new URL('../../../../target/browser-client/', import.meta.url);
const value: PortableCredential = { claims: new Uint8Array([0, 255]), proof: { tag: 'Ed25519', value: new Uint8Array(64).fill(7) } };

test('Rust to TypeScript to Rust preserves opaque signed bytes and u64 replies', () => {
  const encoded = new Uint8Array(Buffer.from(readFileSync(new URL('credential-vector.hex', evidence), 'utf8').trim(), 'hex'));
  const credential = credentialFromBytes(encoded, 131_072);
  assert.deepEqual(credential.claims, new Uint8Array([0, 1, 127, 128, 255]));
  assert.deepEqual(credential.proof, { tag: 'Ed25519', value: Uint8Array.from({ length: 64 }, (_, i) => i) });
  writeFileSync(fileURLToPath(new URL('credential-from-ts.bin', evidence)), credentialBytes(credential, 131_072));
  const reply = new Uint8Array(Buffer.from(readFileSync(new URL('logout-vector.hex', evidence), 'utf8').trim(), 'hex'));
  const result = logoutResultFromBytes(reply);
  assert.deepEqual(result, { tag: 'Revoked', value: 9_007_199_254_740_993n });
  writeFileSync(fileURLToPath(new URL('logout-from-ts.bin', evidence)), logout.encodeResponse(result));
  const refused = new Uint8Array(Buffer.from(readFileSync(new URL('admission-vector.hex', evidence), 'utf8').trim(), 'hex'));
  const refusal = admission.decodeResponse(refused);
  assert.deepEqual(refusal, { err: { tag: 'NotYetValid', value: null } });
  assert.ok('err' in refusal);
  const error = new AdmissionError(refusal.err.tag);
  assert.equal(error.code, 'admission_refused');
  assert.equal(error.refusal, 'NotYetValid');
  assert.ok(error instanceof ClientError);
  writeFileSync(fileURLToPath(new URL('admission-from-ts.bin', evidence)), admission.encodeResponse(refusal));
});
test('wire rejects truncation, trailing bytes, wrong method/direction and unsafe limits', () => {
  const bytes = credentialBytes(value, 131_072);
  for (let length = 0; length < bytes.length; length++) assert.throws(() => credentialFromBytes(bytes.subarray(0, length), 131_072), ClientError);
  assert.throws(() => credentialFromBytes(new Uint8Array([...bytes, 0]), 131_072), ClientError);
  assert.throws(() => credentialFromBytes(logout.encodeRequest(null), 131_072), ClientError);
  assert.throws(() => credentialFromBytes(logout.encodeResponse({ tag: 'InvalidCaller', value: null }), 131_072), ClientError);
  assert.throws(() => admission.decodeRequest(bytes));
  for (const maximum of [NaN, Infinity, 0, -1, 1.5, Number.MAX_SAFE_INTEGER + 1, undefined, null, '131072']) {
    assert.throws(() => Reflect.apply(credentialBytes, undefined, [value, maximum]), ClientError);
    assert.throws(() => Reflect.apply(credentialFromBytes, undefined, [bytes, maximum]), ClientError);
  }
  assert.deepEqual({ ...credentialFromBytes(bytes, bytes.length) }, value);
  assert.throws(() => credentialFromBytes(bytes, bytes.length - 1), ClientError);
  assert.throws(() => credentialBytes(value, bytes.length - 1), ClientError);
});
test('subarray and Buffer views own exactly the admitted bytes', () => {
  const bytes = credentialBytes(value, 131_072);
  const outer = new Uint8Array(bytes.length + 9); outer.set(bytes, 5);
  const decoded = credentialFromBytes(outer.subarray(5, -4), 131_072);
  outer.fill(0);
  assert.deepEqual({ ...decoded }, value);
  const buffer = Buffer.concat([Buffer.from([99]), Buffer.from(bytes), Buffer.from([99])]);
  const fromBuffer = credentialFromBytes(buffer.subarray(1, -1), 131_072);
  buffer.fill(0);
  assert.deepEqual({ ...fromBuffer }, value);
});
test('field bounds, extra fields and malformed tags reject before returning a value', () => {
  assert.throws(() => credentialBytes({ ...value, claims: new Uint8Array(65_537) }, 1_048_576), ClientError);
  assert.throws(() => credentialBytes({ ...value, proof: { tag: 'Ed25519', value: new Uint8Array(65) } }, 131_072), ClientError);
  assert.throws(() => credentialBytes(Object.assign({}, value, { issuer: 'untrusted' }), 131_072), ClientError);
  const bytes = credentialBytes(value, 131_072); bytes[11] = 255;
  assert.throws(() => credentialFromBytes(bytes, 131_072), ClientError);
});

test('client byte policy can tighten but cannot relax generated field bounds', () => {
  const maximumClaims = { ...value, claims: new Uint8Array(65_536) };
  const bytes = credentialBytes(maximumClaims, 1_048_576);
  assert.deepEqual({ ...credentialFromBytes(bytes, 1_048_576) }, maximumClaims);
  assert.throws(() => credentialBytes(maximumClaims, bytes.length - 1), ClientError);
  assert.throws(() => credentialFromBytes(bytes, bytes.length - 1), ClientError);
  // A permissive outer cap must never bypass the Rust-owned field bound.
  assert.throws(() => credentialBytes({ ...value, claims: new Uint8Array(65_537) }, 1_048_576), ClientError);
});
test('every current logout response fits the additional browser cap', () => {
  const replies: logout.Response[] = [
    { tag: 'Revoked', value: (1n << 64n) - 1n },
    { tag: 'InvalidCaller', value: null },
    ...(['Capacity', 'Storage', 'Clock', 'ResetPending'] as const).map(tag => ({
      tag: 'Unavailable' as const, value: { tag, value: null },
    })),
  ];
  for (const reply of replies) {
    assert.deepEqual(logoutResultFromBytes(logout.encodeResponse(reply)), reply);
  }
});
test('a bounded certified envelope can exceed the default browser policy', () => {
  // Structural transport evidence only: these are not cryptographically valid proofs.
  const hash = Array<number>(32).fill(0);
  const pid = new Uint8Array([1]);
  const audience = { tag: 'Fleet' as const, value: { canonical_network_id: hash, fleet_id: hash } };
  const algorithm = { tag: 'EcdsaSecp256k1' as const, value: null };
  const key_id = { name: 'key' };
  const proofAlgorithm = { tag: 'IcCanisterSignatureV1' as const, value: null };
  const binding = { tag: 'IcCanisterSignatureV1' as const, value: hash };
  const large: PortableCredential = { claims: value.claims, proof: { tag: 'Certified', value: {
    delegation: {
      cert: {
        root_pid: pid, issuer_pid: pid, issuer_proof_alg: proofAlgorithm,
        issuer_proof_binding_hash: hash, issuer_proof_binding: binding,
        issued_at_ns: 1n, not_before_ns: 1n, expires_at_ns: 2n, max_token_ttl_ns: 1n,
        aud: audience, grants: [],
      },
      root_proof: { tag: 'IcChainKeyBatchSignatureV1', value: {
        header: {
          schema_version: 1, root_canister_id: pid, batch_id: hash,
          proof_epoch: 1n, registry_epoch: 1n, registry_hash: hash, tree_root: hash,
          not_before_ns: 1n, expires_at_ns: 2n, algorithm, key_id,
          derivation_path_hash: hash, key_version: 1n,
        },
        delegation_cert: {
          root_canister_id: pid, issuer_canister_id: pid, proof_epoch: 1n,
          issuer_proof_algorithm: proofAlgorithm, issuer_proof_binding_hash: hash,
          issuer_proof_binding: binding, max_token_ttl_ns: 1n, audience, grants: [],
          not_before_ns: 1n, expires_at_ns: 2n, registry_epoch: 1n, registry_hash: hash,
        },
        issuer_witness: { steps: [] },
        signature: { algorithm, key_id, derivation_path: [], public_key: new Uint8Array(33), signature: new Uint8Array(64) },
      } },
    },
    issuer_signature: { signature_cbor: new Uint8Array(131_072), public_key_der: new Uint8Array(0) },
  } } };
  const bytes = credentialBytes(large, 524_288);
  assert.ok(bytes.length > 131_072);
  assert.equal(credentialFromBytes(bytes, 524_288).proof.tag, 'Certified');
  assert.throws(() => credentialBytes(large, 131_072), ClientError);
  assert.throws(() => credentialFromBytes(bytes, 131_072), ClientError);
});
