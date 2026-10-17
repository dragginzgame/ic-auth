// Real certificate/delegation bytes supplied by the native qualification owner.
import assert from 'node:assert/strict';
import { credentialBytes, credentialFromBytes } from '../../src/index.js';
const hex = process.argv[2];
assert.ok(hex);
const bytes = Uint8Array.from(Buffer.from(hex, 'hex'));
const value = credentialFromBytes(bytes, 131_072);
assert.equal(value.proof.tag, 'Certified');
assert.deepEqual(credentialBytes(value, 131_072), bytes);
process.stdout.write(Buffer.from(credentialBytes(value, 131_072)).toString('hex'));
