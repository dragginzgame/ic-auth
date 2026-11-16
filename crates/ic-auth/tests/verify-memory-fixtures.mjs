// Independent schema-2 packing and OpenSSL signatures for retained-state cases.
// Public test seed and protocol framing follow the pinned reference described in
// fixtures/README.md. No replacement Rust codec or cryptography is used.
import { createHash, createPrivateKey, createPublicKey, sign, verify } from 'node:crypto';
import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
// Installed hosts advance in real nanoseconds; reuse the same independent packer.
const host = process.argv[2] === '--host';
const time = n => host ? 2_000_000_000_000_000_000n + BigInt(n) * 1_000_000_000n : BigInt(n);
const hash = b => createHash('sha256').update(b).digest();
const uint = (n, size) => { const b = Buffer.alloc(size); if (size === 8) b.writeBigUInt64BE(BigInt(n)); else b.writeUIntBE(n, 0, size); return b; };
const sized = b => Buffer.concat([uint(b.length, 4), b]);
const text = s => sized(Buffer.from(s));
const privateKey = createPrivateKey({ key: Buffer.concat([
  Buffer.from('302e020100300506032b657004220420', 'hex'), Buffer.from(Array.from({ length: 32 }, (_, i) => i)),
]), format: 'der', type: 'pkcs8' });
const publicKey = createPublicKey(privateKey);
const key = publicKey.export({ format: 'der', type: 'spki' }).subarray(12);
const expected = new Map([['public-key', key], ['key-binding', hash(key)]]);
for (const [name, browser, generation, issued, nonce, subject, start = 10, deadline = 100, slot = 0, issuer = 1] of [
  ['initial', 0x44, 1, 20, 0x55, 2], ['renewal', 0x44, 1, 21, 0x56, 2],
  ['replacement', 0x45, 2, 22, 0x57, 2], ['third', 0x46, 3, 23, 0x58, 2],
  ['rebound', 0x44, 1, 20, 0x59, 3], ['late-old', 0x44, 1, 35, 0x60, 2],
  ['future', 0x47, 1, 120, 0x61, 3, 110, 200],
  ['other-device', 0x48, 1, 20, 0x62, 2, 10, 100, 1],
  ['other-issuer', 0x49, 1, 20, 0x63, 2, 10, 100, 0, 3],
]) {
  const browserKey = Buffer.alloc(32, browser);
  const claims = Buffer.concat([
    Buffer.from('IC-AUTH-PORTABLE-CREDENTIAL\0'), uint(2, 2), Buffer.of(1),
    Buffer.alloc(32, 0x11), Buffer.alloc(32, 0x22), sized(Buffer.of(issuer)), hash(key), browserKey,
    sized(Buffer.of(subject)), Buffer.of(slot), uint(generation, 8), uint(time(start), 8), uint(time(deadline), 8),
    uint(time(issued), 8), uint(time(issued + 10), 8), Buffer.alloc(16, nonce), uint(1, 4), text('user'), uint(1, 4), text('read'),
  ]);
  const digest = hash(claims), signature = sign(null, digest, privateKey);
  assert(verify(null, digest, publicKey, signature));
  const caller = Buffer.concat([createHash('sha224').update(Buffer.concat([
    Buffer.from('302a300506032b6570032100', 'hex'), browserKey,
  ])).digest(), Buffer.of(2)]);
  expected.set(`${name}-claims`, claims);
  expected.set(`${name}-signature`, signature);
  expected.set(`${name}-caller`, caller);
}
// Cross-check the initial case against the separately retained reference vector.
const reference = new Map(readFileSync(new URL('fixtures/ed25519.tsv', import.meta.url), 'utf8').trim().split('\n').map(l => l.split('\t')));
if (!host) {
  assert.equal(expected.get('initial-claims').toString('hex'), reference.get('claims'));
  assert.equal(expected.get('initial-signature').toString('hex'), reference.get('signature'));
}
if (host || process.argv[2] === '--print') {
  for (const [name, bytes] of expected) console.log(`${name}\t${bytes.toString('hex')}`);
} else {
  const saved = new Map(readFileSync(new URL('fixtures/memory.tsv', import.meta.url), 'utf8').trim().split('\n').map(l => l.split('\t')));
  assert.equal(saved.size, expected.size);
  for (const [name, bytes] of expected) assert.equal(saved.get(name), bytes.toString('hex'), name);
  console.log('Verified nine independent memory admission credentials with Node/OpenSSL');
}
