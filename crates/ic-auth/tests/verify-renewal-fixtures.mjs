// Independent renewal identity frames; no Auth/Wire encoder or Rust hash helper.
// Contract: ic-auth e52a40be75dd9809d0528189d84b43fe96952ad9,
// persistent/issuer/model.rs::RenewRequest and prepare.rs::hash, with Wire's
// 11-byte IW/request header and Postcard field encodings. Principal is bytes,
// login_id is a fixed u8 array, and role/scope labels are transparent strings.
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import assert from 'node:assert/strict';

const cat = (...parts) => Buffer.concat(parts);
const uint = value => {
  let n = BigInt(value);
  assert(n >= 0n && n <= (1n << 64n) - 1n);
  const bytes = [];
  do { const low = Number(n & 127n); n >>= 7n; bytes.push(low | (n ? 128 : 0)); } while (n);
  return Buffer.from(bytes);
};
const sized = bytes => cat(uint(bytes.length), bytes);
const text = s => sized(Buffer.from(s));
const header = Buffer.alloc(11);
header.write('IW'); header.writeBigUInt64LE(0x41555448495201n, 2);
const cases = [
  ['base', Buffer.of(2), 1n, Buffer.alloc(32, 3), 1n, [['user', ['read']]]],
  ['next', Buffer.of(2), 1n, Buffer.alloc(32, 3), 2n, [['user', ['read']]]],
  ['wide', Buffer.alloc(29, 255), 128n, Buffer.from(Array.from({length: 32}, (_, i) => i)),
    (1n << 64n) - 1n, [['app', ['read', 'write']], ['user', ['read']]]],
];
let output = '';
for (const [name, account, epoch, login, sequence, grants] of cases) {
  const frame = cat(header, sized(account), uint(epoch), login, uint(sequence), uint(grants.length),
    ...grants.flatMap(([role, scopes]) => [text(role), uint(scopes.length), ...scopes.map(text)]), uint(0));
  output += `${name}-frame\t${frame.toString('hex')}\n`;
  output += `${name}-id\t${createHash('sha256').update(frame).digest('hex')}\n`;
}
const target = new URL('fixtures/issuer-renewal.tsv', import.meta.url);
if (process.argv.includes('--generate')) writeFileSync(target, output);
else assert.equal(readFileSync(target, 'utf8'), output);
console.log('Verified three independent renewal identity frames and SHA-256 operation IDs.');
