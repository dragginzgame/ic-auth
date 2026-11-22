// Independently spell the current fixture layout using Postcard primitives.
// No Auth/Wire encoder, generated schema, legacy source or crypto is executed.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
const u = value => {
  let n = BigInt(value);
  const bytes = [];
  do { bytes.push(Number(n & 127n) | (n > 127n ? 128 : 0)); n >>= 7n; } while (n);
  return bytes;
};
const blob = bytes => [u(bytes.length), bytes];
const str = value => blob([...Buffer.from(value)]);
const hash = value => Array(32).fill(value);
const max = u(0xffffffffffffffffn);
const audience = [0, hash(1), hash(2)];
const grants = [1, str('project_instance'), 2, str('write'), str('read')];
const ed = [blob([0, 128, 255]), 1, blob(Array.from({ length: 64 }, (_, i) => i))];
// Certified credential: claims; proof tag; certificate; batch header; leaf;
// witness; signature; issuer evidence. Values are intentionally not valid proofs.
const certified = [
  blob([128, 0, 255, 1]), 0,
  blob([3]), blob([4]), 0, hash(5), 0, hash(6),
  u(127), u(128), u(129), u(130), audience, grants,
  0, u(257), blob([7]), hash(8), u(131), u(132), hash(9), hash(10),
  u(133), u(134), 0, str('root-key'), hash(11), u(135),
  blob([12]), blob([13]), u(136), 0, hash(14), 0, hash(15), u(137),
  audience, grants, u(138), u(139), u(140), hash(16),
  2, 0, hash(17), 1, hash(18),
  0, str('signature-key'), 2, blob([]), blob([0, 128, 255]), blob([19, 20]), blob([21, 22, 23]),
  blob([24, 25, 26, 27]), blob([28, 29]),
];
export const frames = new Map();
function add(name, method, response, payload = []) {
  const id = Buffer.alloc(8);
  id.writeBigUInt64LE(method);
  const frame = Buffer.from([0x49, 0x57, ...id, Number(response), ...payload.flat(Infinity)]);
  assert(!frames.has(name));
  frames.set(name, frame.toString('hex'));
}
const read = 0x415554480013n, logout = 0x415554480014n;
const credential = 0x415554480001n, establish = 0x415554480012n;
add('read-request', read, false);
add('logout-request', logout, false);
add('read-empty', read, true, [0, blob([]), blob([]), blob([]), 0, 0]);
add('read-max', read, true, [0, blob(Array(29).fill(1)), blob([2]), blob([3]), max, 1, blob(Array(16).fill(0xab))]);
for (const [tag, name] of ['MissingOrExpired', 'AuthorityUnavailable', 'ScopeRejected', 'Revoked', 'Unavailable'].entries()) {
  add('read-' + name, read, true, [1, tag]);
}
for (const [tag, name] of ['Revoked', 'Unconfirmed'].entries()) {
  add('logout-' + name + '-zero', logout, true, [tag, 0]);
  add('logout-' + name + '-max', logout, true, [tag, max]);
}
add('logout-InvalidCaller', logout, true, [2]);
for (const [tag, name] of ['Capacity', 'Storage', 'Clock', 'ResetPending'].entries()) {
  add('logout-' + name, logout, true, [3, tag]);
}
for (const [kind, payload] of [['ed25519', ed], ['certified', certified]]) {
  add('credential-' + kind + '-request', credential, false, payload);
  add('credential-' + kind + '-response', credential, true, payload);
}
add('credential-empty', credential, false, [0, 1, 0]);
add('establish-none', establish, false, [ed, 3, str('write'), str('read'), str('write'), 0]);
add('establish-some', establish, false, [certified, 3, str(''), str('Read'), str('read::write'), 1, max]);
add('establish-success', establish, true, [0, blob([30]), blob([31]), blob([32]), max, 0]);
for (const [tag, name] of ['NotYetValid', 'Expired', 'InvalidCredential', 'AuthorityUnavailable', 'BeforeReset',
  'ScopeRejected', 'ReplayConflict', 'Revoked', 'Capacity', 'Throttled', 'Unavailable'].entries()) {
  add('establish-' + name, establish, true, [1, tag]);
}
const retained = new Map(['caller-frames.tsv', 'admission-frames.tsv'].flatMap(name =>
  readFileSync(new URL('fixtures/' + name, import.meta.url), 'utf8').trim().split('\n').map(line => line.split('\t'))));
assert.equal(frames.size, 37);
assert.equal(retained.size, frames.size);
for (const [name, hex] of frames) assert.equal(retained.get(name), hex, name);
console.log('Independently assembled all 37 current Wire fixtures');
