import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { decodeRequest, encodeRequest, decodeResponse, encodeResponse, maximumResponseBytes } from '../src/generated/logout.js';

const frames = new Map(readFileSync(new URL('../../../crates/ic-auth-wire/tests/fixtures/caller-frames.tsv', import.meta.url), 'utf8')
  .trim().split('\n').map(line => { const [name, hex] = line.split('\t'); return [name!, Buffer.from(hex!, 'hex')]; }));

test('logout codec preserves independent request, all result tags and wide timestamps', () => {
  assert.equal(maximumResponseBytes, 1_024);
  assert.equal(decodeRequest(frames.get('logout-request')!), null);
  assert.deepEqual(Buffer.from(encodeRequest(null)), frames.get('logout-request'));
  for (const [name, bytes] of frames) {
    if (!name.startsWith('logout-') || name === 'logout-request') continue;
    const reply = decodeResponse(bytes);
    assert.deepEqual(Buffer.from(encodeResponse(reply)), bytes);
    if (name.includes('-max')) {
      assert(reply.tag === 'Revoked' || reply.tag === 'Unconfirmed');
      assert.equal(reply.value, 0xffff_ffff_ffff_ffffn);
    }
    if (name.includes('-zero')) {
      assert(reply.tag === 'Revoked' || reply.tag === 'Unconfirmed');
      assert.equal(reply.value, 0n); // Structural decoding alone is not acknowledgement policy.
    }
  }
});

test('logout codec refuses wrong methods, direction, trailing bytes and truncation', () => {
  const good = frames.get('logout-Revoked-max')!;
  for (const bad of [frames.get('logout-request')!, frames.get('read-request')!,
    good.subarray(0, good.length - 1), Buffer.concat([good, Buffer.of(0)])]) {
    assert.throws(() => decodeResponse(bad));
  }
});
