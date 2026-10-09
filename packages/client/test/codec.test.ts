import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { ClientError, MemoryTokenStore, tokenBytes, tokenFromBytes } from '../src/index.js';
import { requestBytes, requestFromBytes } from '../src/codec.js';
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
