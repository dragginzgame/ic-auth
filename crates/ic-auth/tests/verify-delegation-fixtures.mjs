// Independent field assembly and Node/OpenSSL SHA-256. No Auth code is loaded.
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
const cat = (...parts) => Buffer.concat(parts);
const text = value => Buffer.from(value);
const repeat = (value, length = 32) => Buffer.alloc(length, value);
const u32 = value => { const b = Buffer.alloc(4); b.writeUInt32BE(value); return b; };
const u64 = value => { const b = Buffer.alloc(8); b.writeBigUInt64BE(BigInt(value)); return b; };
const bytes = value => cat(u32(value.length), value);
const sha = value => createHash('sha256').update(value).digest();
const root = bytes(repeat(1, 29)), issuer = bytes(repeat(3, 29));
const audience = cat(Buffer.from([1]), Buffer.from('402b3681453fb9cfa356b6ea7abde2de53cc4c665caaf438535bddc1e1679f60', 'hex'), repeat(2));
const grants = cat(u32(1), bytes(text('project_instance')), u32(2), bytes(text('read')), bytes(text('write')));
const binding = sha(cat(text('canic-issuer-proof-binding-v1'), issuer, Buffer.from([1, 1]), repeat(8)));
const certificate = cat(text('CANIC-AUTH\0'), Buffer.from([1]), root, issuer, Buffer.from([1]), binding, Buffer.from([1]), repeat(8), u64(100), u64(100), u64(200), u64(60), audience, grants);
const leaf = cat(root, issuer, u64(2), Buffer.from([1]), repeat(35), Buffer.from([1]), repeat(36), u64(60), audience, grants, u64(100), u64(200), u64(3), repeat(32));
const header = cat(Buffer.from([0, 1]), root, repeat(31), u64(2), u64(3), repeat(32), repeat(33), u64(100), u64(200), Buffer.from([1]), bytes(text('test_key_1')), repeat(34), u64(4));
const leafHash = sha(cat(Buffer.from([0]), text('CANIC_ROOT_DELEGATION_CHAIN_KEY_ISSUER_LEAF_V1'), bytes(leaf)));
const headerHash = sha(cat(text('CANIC_ROOT_DELEGATION_CHAIN_KEY_BATCH_V1'), bytes(header)));
const pathHash = sha(cat(text('CANIC_ROOT_DELEGATION_CHAIN_KEY_DERIVATION_PATH_V1'), u32(2), bytes(text('canic')), bytes(text('delegation'))));
// These three expectations are retained literally from pinned reference tests.
assert.equal(leafHash.toString('hex'), 'e57d2e09d2f7433593e783c4279a2f66b4e6288a0f24184d4c70a6f556165ed8');
assert.equal(headerHash.toString('hex'), 'e786c7ba82f4faf3fefc968c039ae6fc2d3459d777e4e9e7f560362d21122cc0');
assert.equal(pathHash.toString('hex'), 'fe51a87b988d221227b134c48f36787e891a902dcb5d48ea5f94cff8bfed5a16');
const records = { certificate, certificate_hash: sha(certificate), issuer_binding_hash: binding, leaf_hash: leafHash, batch_header_hash: headerHash, derivation_path_hash: pathHash };
const expected = Object.entries(records).map(([key, value]) => `${key}\t${value.toString('hex')}\n`).join('');
assert.equal(readFileSync(new URL('./fixtures/delegation.tsv', import.meta.url), 'utf8'), expected);
console.log('Six delegation records agree with independent Node/OpenSSL construction.');
