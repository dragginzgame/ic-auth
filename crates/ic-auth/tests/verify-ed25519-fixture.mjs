// Independently pack and sign the pinned reference's portable-signature vector.
// Node/OpenSSL supplies cryptography; no replacement Rust encoder/signer is used.
import { createHash, createPrivateKey, createPublicKey, sign, verify } from 'node:crypto';
import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const hash = bytes => createHash('sha256').update(bytes).digest();
const u16 = n => { const b = Buffer.alloc(2); b.writeUInt16BE(n); return b; };
const u32 = n => { const b = Buffer.alloc(4); b.writeUInt32BE(n); return b; };
const u64 = n => { const b = Buffer.alloc(8); b.writeBigUInt64BE(BigInt(n)); return b; };
const sized = bytes => Buffer.concat([u32(bytes.length), bytes]);
const text = value => sized(Buffer.from(value));
// Public test-only seed 00..1f, as used by the reference's independent vector.
const seed = Buffer.from(Array.from({ length: 32 }, (_, i) => i));
const privateKey = createPrivateKey({ key: Buffer.concat([Buffer.from('302e020100300506032b657004220420', 'hex'), seed]), format: 'der', type: 'pkcs8' });
const publicKey = createPublicKey(privateKey);
const otherPublicKey = createPublicKey(createPrivateKey({
  key: Buffer.concat([Buffer.from('302e020100300506032b657004220420', 'hex'), Buffer.alloc(32)]),
  format: 'der', type: 'pkcs8',
})).export({ format: 'der', type: 'spki' }).subarray(12);
const key = publicKey.export({ format: 'der', type: 'spki' }).subarray(12);
const browserKey = Buffer.alloc(32, 0x44);
const claims = Buffer.concat([
  Buffer.from('IC-AUTH-PORTABLE-CREDENTIAL\0'), u16(2), Buffer.of(1),
  Buffer.alloc(32, 0x11), Buffer.alloc(32, 0x22), sized(Buffer.of(1)), hash(key), browserKey,
  sized(Buffer.of(2)), Buffer.of(0), u64(1), u64(10), u64(100), u64(20), u64(30),
  Buffer.alloc(16, 0x55), u32(1), text('user'), u32(1), text('read'),
]);
const digest = hash(claims);
const signature = sign(null, digest, privateKey);
assert.equal(key.toString('hex'), '03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8');
assert.equal(digest.toString('hex'), '55d41ea4b6d7f9ee02726c30241be325048c560a90261ebb8822c3745a0af057');
assert.equal(signature.toString('hex'), 'c8e4c35496f1397a54faa16f8c438d3e7f238edea6bb289314fdbf32c33e1722e4ae146258e6abd7bcff480ba69567fb584149e38dbf0571d2ab80f9ebccf403');
assert(verify(null, digest, publicKey, signature));
const caller = Buffer.concat([createHash('sha224').update(Buffer.concat([
  Buffer.from('302a300506032b6570032100', 'hex'), browserKey,
])).digest(), Buffer.of(2)]);
const rawSignature = sign(null, claims, privateKey);
assert(!verify(null, digest, publicKey, rawSignature));
// Adversarial scalar encoding S+L, built as bytes for an independent rejection.
const noncanonical = Buffer.from(signature);
const order = Buffer.from('edd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010', 'hex');
let carry = 0;
for (let i = 0; i < 32; i++) {
  const sum = noncanonical[32 + i] + order[i] + carry;
  noncanonical[32 + i] = sum & 255;
  carry = sum >>> 8;
}
const expected = new Map([
  ['claims', claims], ['digest', digest], ['public-key', key], ['key-binding', hash(key)],
  ['signature', signature], ['caller', caller], ['raw-claims-signature', rawSignature],
  ['noncanonical-signature', noncanonical], ['other-public-key', otherPublicKey],
]);
if (process.argv[2] === '--print') {
  for (const [name, bytes] of expected) console.log(`${name}\t${bytes.toString('hex')}`);
} else {
  const saved = new Map(readFileSync(new URL('fixtures/ed25519.tsv', import.meta.url), 'utf8')
    .trim().split('\n').map(line => line.split('\t')));
  assert.equal(saved.size, expected.size);
  for (const [name, bytes] of expected) assert.equal(saved.get(name), bytes.toString('hex'), name);
  console.log('Verified the reference claims/digest/signature, caller and adversarial fixtures with Node/OpenSSL');
}
