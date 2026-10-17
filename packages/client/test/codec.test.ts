import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { ClientError, credentialBytes, credentialFromBytes, type PortableCredential } from '../src/index.js';
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
