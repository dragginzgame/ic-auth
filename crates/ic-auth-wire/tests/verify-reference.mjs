// Read-only oracle: run the fixed reference browser codecs against retained frames.
// Usage: node .../verify-reference.mjs /path/to/ic-auth [newly-generated-directory]
import { Assert } from 'node:assert';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { admissionCases } from './admission-cases.mjs';

const reference = process.argv[2];
if (!reference) throw new Error('Provide the read-only reference checkout path');
const revision = 'e52a40be75dd9809d0528189d84b43fe96952ad9';
const assert = new Assert({ strict: true, skipPrototype: true });
const root = new URL('../../../', import.meta.url);
const hashes = new Map(readFileSync(new URL('reference.sha256', root), 'utf8')
  .trim().split('\n').map(line => { const [hash, path] = line.split('  '); return [path, hash]; }));
const frames = new Map(['caller-frames.tsv', 'admission-frames.tsv'].flatMap(name =>
  readFileSync(new URL(`fixtures/${name}`, import.meta.url), 'utf8')
    .trim().split('\n').map(line => line.split('\t'))));
const scratch = mkdtempSync(join(tmpdir(), 'ic-auth-wire-oracle-'));
const checked = new Set();
try {
  for (const name of ['read', 'logout', 'credential', 'establish', 'wire-runtime']) {
    const path = `packages/client/src/generated/${name}.ts`;
    const source = execFileSync('git', ['-C', resolve(reference), 'show', `${revision}:${path}`]);
    assert.equal(createHash('sha256').update(source).digest('hex'), hashes.get(path), path);
    const js = stripTypeScriptTypes(source.toString(), { mode: 'transform' });
    writeFileSync(join(scratch, `${name}.js`), js);
  }
  writeFileSync(join(scratch, 'package.json'), '{"type":"module"}\n');
  const read = await import(pathToFileURL(join(scratch, 'read.js')));
  const logout = await import(pathToFileURL(join(scratch, 'logout.js')));
  const credential = await import(pathToFileURL(join(scratch, 'credential.js')));
  const establish = await import(pathToFileURL(join(scratch, 'establish.js')));
  const generated = new Map();
  if (process.argv[3]) {
    const current = join(scratch, 'current');
    mkdirSync(current);
    for (const name of ['read', 'logout', 'credential', 'establish', 'wire-runtime']) {
      const source = readFileSync(join(resolve(process.argv[3]), `${name}.ts`), 'utf8');
      writeFileSync(join(current, `${name}.js`), stripTypeScriptTypes(source, { mode: 'transform' }));
    }
    for (const name of ['read', 'logout', 'credential', 'establish']) {
      generated.set(name, await import(pathToFileURL(join(current, `${name}.js`))));
    }
    for (const [name, previous] of [['read', read], ['logout', logout], ['credential', credential], ['establish', establish]]) {
      assert.equal(generated.get(name).methodName, previous.methodName);
      assert.equal(generated.get(name).contract.methodName, previous.contract.methodName);
      assert.equal(generated.get(name).contract.mode, previous.contract.mode);
    }
    assert.equal(generated.get('credential').defaultMaximumBytes, credential.defaultMaximumBytes);
    assert.equal(generated.get('credential').maximumConfiguredBytes, credential.maximumConfiguredBytes);
    assert.equal(generated.get('logout').maximumResponseBytes, logout.maximumResponseBytes);
  }
  function check(name, codec, value, request = false) {
    const next = generated.get(name.split('-')[0]);
    for (const selected of next ? [codec, next] : [codec]) {
      const encode = request ? selected.encodeRequest : selected.encodeResponse;
      const decode = request ? selected.decodeRequest : selected.decodeResponse;
      assert.equal(Buffer.from(encode(value)).toString('hex'), frames.get(name), name);
      const frame = Buffer.from(frames.get(name), 'hex');
      const padded = Buffer.alloc(frame.length + 8);
      frame.copy(padded, 4);
      const decoded = decode(padded.subarray(4, frame.length + 4));
      padded.fill(255);
      assert.deepEqual(decoded, value, `${name}: decoded subarray must be detached`);
    }
    checked.add(name);
  }
  check('read-request', read, null, true);
  check('logout-request', logout, null, true);
  check('read-empty', read, { ok: {
    caller: new Uint8Array(), issuer: new Uint8Array(), subject: new Uint8Array(), expires_at_ns: 0n,
  } });
  check('read-max', read, { ok: {
    caller: new Uint8Array(29).fill(1), issuer: Uint8Array.of(2), subject: Uint8Array.of(3),
    expires_at_ns: 2n ** 64n - 1n,
  } });
  for (const tag of ['MissingOrExpired', 'AuthorityUnavailable', 'ScopeRejected', 'Revoked', 'Unavailable']) {
    check(`read-${tag}`, read, { err: { tag, value: null } });
  }
  for (const tag of ['Revoked', 'Unconfirmed']) {
    for (const [suffix, value] of [['zero', 0n], ['max', 2n ** 64n - 1n]]) {
      check(`logout-${tag}-${suffix}`, logout, { tag, value });
    }
  }
  check('logout-InvalidCaller', logout, { tag: 'InvalidCaller', value: null });
  for (const tag of ['Capacity', 'Storage', 'Clock', 'ResetPending']) {
    check(`logout-${tag}`, logout, { tag: 'Unavailable', value: { tag, value: null } });
  }
  for (const { name, codec, value, request } of admissionCases(credential, establish)) {
    check(name, codec, value, request);
  }
  assert.equal(checked.size, frames.size);
  const redundant = Buffer.from(frames.get('logout-InvalidCaller').slice(0, -2) + '8200', 'hex');
  assert.deepEqual(logout.decodeResponse(redundant), { tag: 'InvalidCaller', value: null });
  if (generated.size) {
    assert.deepEqual(generated.get('logout').decodeResponse(redundant), { tag: 'InvalidCaller', value: null });
  }
  console.log(`Verified ${checked.size} frames, detached buffers and redundant outer varint against ${revision}${generated.size ? ' and newly generated codecs' : ''}`);
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
