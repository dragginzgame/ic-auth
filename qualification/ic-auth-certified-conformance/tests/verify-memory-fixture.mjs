// Independent certified lifecycle claims. No Auth/Platform encoder or message helper.
import { createHash, createPrivateKey, createPublicKey } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const cat = (...parts) => Buffer.concat(parts), text = s => Buffer.from(s);
const rep = (n, size = 32) => Buffer.alloc(size, n);
const u32 = n => { const b = Buffer.alloc(4); b.writeUInt32BE(n); return b; };
const u64 = n => { const b = Buffer.alloc(8); b.writeBigUInt64BE(BigInt(n)); return b; };
const sized = b => cat(u32(b.length), b), sha = b => createHash('sha256').update(b).digest();
const read = name => new Map(readFileSync(new URL(`fixtures/${name}.tsv`, import.meta.url), 'utf8').trim().split('\n').map(line => {
  const [key, value] = line.split('\t');
  return [key, Buffer.from(value, 'hex')];
}));
const original = read('certified'), other = read('certified-other-issuer');
const grants = cat(u32(1), sized(text('app')), u32(2), sized(text('read')), sized(text('write')));
const records = {};
for (const [name, seed, issuer, slot, generation, issued, expires, nonce] of [
  ['initial', null, 2, 1, 7, 140, 180, 6],
  ['other-device', 0x51, 2, 2, 7, 140, 180, 7],
  ['other-issuer', 0x52, 3, 1, 7, 140, 180, 8],
  ['early-renewal', null, 2, 1, 7, 150, 180, 12],
  ['replacement', 0x53, 2, 1, 8, 180, 189, 9],
  ['renewal', null, 2, 1, 7, 180, 189, 10],
  ['late-old', null, 2, 1, 7, 181, 189, 11],
]) {
  const privateKey = createPrivateKey({ format: 'der', type: 'pkcs8', key: cat(
    Buffer.from('302e020100300506032b657004220420', 'hex'),
    seed === null ? Buffer.from(Array.from({ length: 32 }, (_, i) => i)) : rep(seed),
  ) });
  const der = createPublicKey(privateKey).export({ format: 'der', type: 'spki' });
  const binding = (issuer === 2 ? original : other).get('certificate_hash');
  const claims = cat(text('IC-AUTH-PORTABLE-CREDENTIAL\0'), Buffer.from([0, 2, 0]),
    rep(3), rep(4), sized(rep(issuer, 10)), binding, der.subarray(-32),
    sized(rep(5, 10)), Buffer.of(slot), u64(generation), u64(100), u64(190),
    u64(issued), u64(expires), rep(nonce, 16), grants);
  const caller = cat(createHash('sha224').update(der).digest(), Buffer.of(2));
  if (name === 'initial') {
    assert.deepEqual(claims, original.get('claims'));
    assert.deepEqual(caller, original.get('caller'));
  }
  records[`${name}-claims`] = claims;
  records[`${name}-digest`] = sha(claims);
  records[`${name}-caller`] = caller;
}
const output = Object.entries(records).map(([key, value]) => `${key}\t${value.toString('hex')}\n`).join('');
const target = new URL('fixtures/certified-memory.tsv', import.meta.url);
if (process.argv.includes('--generate')) writeFileSync(target, output);
else assert.equal(readFileSync(target, 'utf8'), output);
console.log('Verified seven independent certified lifecycle claims and browser identities.');
