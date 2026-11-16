import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { decodeRequest, encodeRequest } from '../src/generated/credential.js';

// Current fixture bytes independently assembled by the Wire fixture check.
const frames = new Map(readFileSync(new URL('../../../crates/ic-auth-wire/tests/fixtures/admission-frames.tsv', import.meta.url), 'utf8')
  .trim().split('\n').map(line => { const [name, hex] = line.split('\t'); return [name!, Buffer.from(hex!, 'hex')]; }));

test('adopted generated credential codec retains independent Ed25519 and certified frames', () => {
  for (const name of ['credential-ed25519-request', 'credential-certified-request', 'credential-empty']) {
    const frame = frames.get(name)!;
    const decoded = decodeRequest(frame);
    assert.deepEqual(Buffer.from(encodeRequest(decoded)), frame);
  }
  const ed = decodeRequest(frames.get('credential-ed25519-request')!);
  assert.deepEqual(Array.from(ed.claims), [0, 128, 255]);
  assert.equal(ed.proof.tag, 'Ed25519');
  if (ed.proof.tag === 'Ed25519') assert.deepEqual(Array.from(ed.proof.value), Array.from({ length: 64 }, (_, i) => i));
  const certified = decodeRequest(frames.get('credential-certified-request')!);
  assert.deepEqual(Array.from(certified.claims), [128, 0, 255, 1]);
  assert.equal(certified.proof.tag, 'Certified');
  if (certified.proof.tag === 'Certified') assert.equal(certified.proof.value.delegation.cert.issued_at_ns, 127n);
});

test('credential decoding refuses wrong direction/method and malformed framing', () => {
  const valid = frames.get('credential-ed25519-request')!;
  for (const bytes of [frames.get('credential-ed25519-response')!, frames.get('establish-none')!,
    Buffer.alloc(0), valid.subarray(0, valid.length - 1), Buffer.concat([valid, Buffer.of(0)])]) {
    assert.throws(() => decodeRequest(bytes));
  }
});
