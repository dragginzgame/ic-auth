import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { decodeRequest, encodeRequest, decodeResponse, encodeResponse } from '../src/generated/establish.js';

const frames = new Map(readFileSync(new URL('../../../crates/ic-auth-wire/tests/fixtures/admission-frames.tsv', import.meta.url), 'utf8')
  .trim().split('\n').map(line => { const [name, hex] = line.split('\t'); return [name!, Buffer.from(hex!, 'hex')]; }));

test('adopted admission codec preserves independent frames, raw scopes and optional TTL', () => {
  for (const name of ['establish-none', 'establish-some']) {
    const bytes = frames.get(name)!;
    assert.deepEqual(Buffer.from(encodeRequest(decodeRequest(bytes))), bytes);
  }
  const none = decodeRequest(frames.get('establish-none')!);
  assert.deepEqual(none.scopes, ['write', 'read', 'write']);
  assert.equal(none.ttl_secs, null);
  const some = decodeRequest(frames.get('establish-some')!);
  assert.deepEqual(some.scopes, ['', 'Read', 'read::write']);
  assert.deepEqual(some.ttl_secs, { some: 0xffff_ffff_ffff_ffffn });
  for (const [name, bytes] of frames) {
    if (!name.startsWith('establish-') || name === 'establish-none' || name === 'establish-some') continue;
    const decoded = decodeResponse(bytes);
    assert.deepEqual(Buffer.from(encodeResponse(decoded)), bytes);
    if (name === 'establish-success') {
      assert('ok' in decoded);
      assert.deepEqual(Array.from(decoded.ok.caller), [30]);
      assert.deepEqual(Array.from(decoded.ok.issuer), [31]);
      assert.equal(decoded.ok.expires_at_ns, 0xffff_ffff_ffff_ffffn);
    } else {
      assert('err' in decoded);
      assert.equal(decoded.err.tag, name.slice('establish-'.length));
    }
  }
});

test('admission codec refuses framing and request bounds without normalizing invalid inputs', () => {
  const request = decodeRequest(frames.get('establish-none')!);
  for (const scopes of [Array(17).fill('read'), ['a'.repeat(65)]]) {
    assert.throws(() => encodeRequest({ ...request, scopes }));
  }
  for (const some of [-1n, 0x1_0000_0000_0000_0000n]) {
    assert.throws(() => encodeRequest({ ...request, ttl_secs: { some } }));
  }
  const success = frames.get('establish-success')!;
  for (const bytes of [frames.get('establish-none')!, frames.get('credential-ed25519-response')!,
    success.subarray(0, success.length - 1), Buffer.concat([success, Buffer.of(0)])]) {
    assert.throws(() => decodeResponse(bytes));
  }
});
