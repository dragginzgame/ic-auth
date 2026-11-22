// Current generated codecs checked against independently assembled fixture bytes.
// Usage: node .../verify-browser.mjs generated-directory
import { Assert } from 'node:assert';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { admissionCases } from './admission-cases.mjs';
import { frames } from './verify-fixtures.mjs';

if (process.argv.length !== 3) throw new Error('Provide the generated codec directory');
const assert = new Assert({ strict: true, skipPrototype: true });
const scratch = mkdtempSync(join(tmpdir(), 'ic-auth-wire-browser-'));
const checked = new Set();
try {
  for (const name of ['read', 'logout', 'credential', 'establish', 'wire-runtime']) {
    const source = readFileSync(join(resolve(process.argv[2]), name + '.ts'), 'utf8');
    writeFileSync(join(scratch, name + '.js'), stripTypeScriptTypes(source, { mode: 'transform' }));
  }
  writeFileSync(join(scratch, 'package.json'), '{"type":"module"}');
  const [read, logout, credential, establish] = await Promise.all(
    ['read', 'logout', 'credential', 'establish'].map(name => import(pathToFileURL(join(scratch, name + '.js')))));
  for (const [codec, name] of [[read, 'portable_session_read'], [logout, 'logout_portable_session'],
    [credential, 'credential'], [establish, 'establish_portable_session']]) {
    assert.equal(codec.methodName, name);
    // The wrapper's generic export label is distinct from the endpoint name.
    assert.equal(codec.contract.methodName, 'contract');
    assert.equal(codec.contract.mode, 'update');
  }
  assert.equal(credential.defaultMaximumBytes, 131072);
  assert.equal(credential.maximumConfiguredBytes, 1048576);
  assert.equal(logout.maximumResponseBytes, 1024);
  function check(name, codec, value, request = false) {
    const encode = request ? codec.encodeRequest : codec.encodeResponse;
    const decode = request ? codec.decodeRequest : codec.decodeResponse;
    assert.equal(Buffer.from(encode(value)).toString('hex'), frames.get(name), name);
    const frame = Buffer.from(frames.get(name), 'hex');
    const padded = Buffer.alloc(frame.length + 8);
    frame.copy(padded, 4);
    const decoded = decode(padded.subarray(4, frame.length + 4));
    padded.fill(255);
    assert.deepEqual(decoded, value, name + ': decoded subarray must be detached');
    for (const invalid of [frame.subarray(0, frame.length - 1), Buffer.concat([frame, Buffer.of(0)])]) {
      assert.throws(() => decode(invalid), name + ': incomplete/trailing frame');
    }
    checked.add(name);
  }
  check('read-request', read, null, true);
  check('logout-request', logout, null, true);
  check('read-empty', read, { ok: {
    caller: new Uint8Array(), issuer: new Uint8Array(), subject: new Uint8Array(), expires_at_ns: 0n, ext: null,
  } });
  check('read-max', read, { ok: {
    caller: new Uint8Array(29).fill(1), issuer: Uint8Array.of(2), subject: Uint8Array.of(3),
    expires_at_ns: 2n ** 64n - 1n, ext: { some: new Uint8Array(16).fill(0xab) },
  } });
  for (const codec of [read, establish]) {
    const ok = { caller: new Uint8Array(), issuer: new Uint8Array(), subject: new Uint8Array(), expires_at_ns: 1n };
    for (const length of [0, 4096]) {
      const value = { ok: { ...ok, ext: { some: new Uint8Array(length).fill(0xff) } } };
      assert.deepEqual(codec.decodeResponse(codec.encodeResponse(value)), value);
    }
    assert.throws(() => codec.encodeResponse({ ok: { ...ok, ext: { some: new Uint8Array(4097) } } }));
  }
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
  console.log('Verified ' + checked.size + ' current frames, metadata, detached buffers and malformed framing');
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
