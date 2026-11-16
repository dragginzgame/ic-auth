import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { issuanceCases } from './issuance.mjs';
import { transportCases } from './transport.mjs';
import { admissionCases } from './admission.mjs';
import { cleanupCases } from './cleanup.mjs';
import { schedulerCases } from './scheduler.mjs';
import { logoutCases } from './logout.mjs';
import { readinessCases } from './readiness.mjs';
import { httpCases } from './http.mjs';
import { installedHttpCases } from './installed-http.mjs';
import { installedFlowCases } from './installed-flow.mjs';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { build } from 'esbuild';
import { chromium } from 'playwright-core';

const referenceFrames = new Map(readFileSync(new URL('../../../../crates/ic-auth-wire/tests/fixtures/admission-frames.tsv', import.meta.url), 'utf8')
  .trim().split('\n').map(line => line.split('\t')));
const credentialFrames = Object.fromEntries(['ed25519', 'certified'].map(name =>
  [name, Array.from(Buffer.from(referenceFrames.get(`credential-${name}-request`), 'hex'))]));
for (const [name, hex] of referenceFrames) {
  if (name.startsWith('establish-')) credentialFrames[name] = Array.from(Buffer.from(hex, 'hex'));
}
for (const line of readFileSync(new URL('../../../../crates/ic-auth-wire/tests/fixtures/caller-frames.tsv', import.meta.url), 'utf8').trim().split('\n')) {
  const [name, hex] = line.split('\t');
  if (name.startsWith('logout-')) credentialFrames[name] = Array.from(Buffer.from(hex, 'hex'));
}
const suite = process.env.IC_AUTH_CLIENT_SUITE ?? 'all';
if (!['all', 'journal', 'issuance', 'transport', 'admission', 'cleanup', 'logout', 'http', 'readiness', 'scheduler', 'installed-http', 'installed-flow'].includes(suite)) throw new Error('Unknown IC_AUTH_CLIENT_SUITE');
const executablePath = process.env.IC_AUTH_BROWSER_EXECUTABLE;
assert(executablePath, 'Set IC_AUTH_BROWSER_EXECUTABLE to an installed Chromium executable; this gate never downloads one.');
const bundle = await build({ entryPoints: ['tests/browser/entry.ts'], bundle: true,
  format: 'iife', platform: 'browser', write: false });
const server = createServer((req, res) => {
  res.setHeader('Content-Type', req.url === '/test.js' ? 'text/javascript' : 'text/html');
  res.end(req.url === '/test.js' ? bundle.outputFiles[0].contents : '<!doctype html><script src="/test.js"></script>');
});
server.listen(0, '127.0.0.1');
await once(server, 'listening');
let browser;
let cases = 0;
let group = 'journal';
async function check(name, run) {
  if (suite !== 'all' && suite !== group) return;
  await run(); console.log(`ok ${++cases} - ${name}`);
}
try {
  browser = await chromium.launch({ executablePath, headless: true });
  console.log(`Native browser: ${browser.version()}`);
  const context = await browser.newContext();
  const pages = await Promise.all([context.newPage(), context.newPage()]);
  const url = `http://127.0.0.1:${server.address().port}`;
  async function prepare(page) {
    await page.goto(url);
    await page.evaluate(credentialFrames => {
      globalThis.frames = Object.fromEntries(Object.entries(credentialFrames).map(([name, bytes]) => [name, new Uint8Array(bytes)]));
      const { Principal, Ed25519KeyIdentity, LoginJournal } = authTest;
      globalThis.caller = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9)).getPrincipal();
      globalThis.issuer = Principal.fromUint8Array(Uint8Array.of(1));
      globalThis.now = 100n;
      globalThis.clockId = 'clock';
      globalThis.open = name => LoginJournal.open(indexedDB, name, { id: clockId, nowNs: () => now });
      globalThis.failure = promise => promise.then(() => 'accepted', e => e.code ?? e.name);
      // Independently specified frozen v1 profile, not serialized by the implementation.
      globalThis.record = () => ({ generation: 'login', caller: caller.toText(), issuer: issuer.toText(),
        clockId: 'clock', observedNs: 100n, targets: [{ canister: issuer.toText(), attempts: 0, nextNs: 100n }],
        issuanceUnknown: false, phase: 'active', deadlineNs: 0n, busyUntilNs: 0n });
      globalThis.raw = async (name, operation, version = 1) => {
        const db = await new Promise((resolve, reject) => {
          const r = indexedDB.open(name, version);
          r.onupgradeneeded = () => r.result.createObjectStore('journal');
          r.onsuccess = () => resolve(r.result); r.onerror = () => reject(r.error);
        });
        try {
          return await new Promise((resolve, reject) => {
            const tx = db.transaction('journal', 'readwrite', { durability: 'strict' });
            const value = operation(tx.objectStore('journal'));
            tx.oncomplete = () => resolve(typeof value === 'function' ? value() : undefined);
            tx.onabort = () => reject(tx.error);
          });
        } finally { db.close(); }
      };
      globalThis.seed = (name, value, bytes) => raw(name, bucket => {
        bucket.put('ic-auth-login-cleanup-journal-v1', 'format');
        if (value !== undefined) bucket.put(value, 'login');
        if (bytes !== undefined) bucket.put(bytes, 'credential');
      });
    }, credentialFrames);
  }
  await Promise.all(pages.map(prepare));
  const [a, b] = pages;

  await check('concurrent tabs serialize login creation and refuse stale authority', async () => {
    await Promise.all(pages.map(p => p.evaluate(async () => { globalThis.journal = await open('tabs'); })));
    const starts = await Promise.all(pages.map(p => p.evaluate(() => failure(journal.start({ generation: 'login', caller, issuer })))));
    assert.deepEqual(starts.sort(), ['accepted', 'login_exists']);
    assert.deepEqual(await a.evaluate(async () => [
      await failure(journal.assertActive('wrong', caller, issuer)),
      await failure(journal.assertActive('login', issuer, issuer)),
      await failure(journal.assertActive('login', caller, authTest.Principal.fromUint8Array(Uint8Array.of(2)))),
      await failure(journal.assertActive('login', caller, issuer)),
    ]), ['not_current', 'not_current', 'not_current', 'accepted']);
  });

  await check('committed logout in another tab prevents a suspended SDK signature escaping', async () => {
    await a.evaluate(async () => { globalThis.signing = await authTest.pauseSigning(journal); });
    assert.equal(await b.evaluate(async () => (await journal.beginLogout('login', 1_000n)).phase), 'cleanup');
    const result = await a.evaluate(async () => { signing.resume(); return signing.result; });
    assert.deepEqual(result, { code: 'not_current', escaped: false });
    await prepare(a);
    assert.equal(await a.evaluate(async () => {
      const j = await open('tabs');
      try { return await failure(j.assertActive('login', caller, issuer)); } finally { j.close(); }
    }), 'not_current');
    await b.evaluate(() => journal.close());
  });

  await check('logout preserves sequence-less uncertainty and retained sequence across reopening', async () => {
    const result = await a.evaluate(async () => {
      const values = [];
      for (const [i, issuance] of [undefined, { sequence: 0n }, { sequence: 7n, untilNs: 2_000n }].entries()) {
        const r = { ...record(), issuanceUnknown: true, ...(issuance === undefined ? {} : { issuance }) };
        await seed(`uncertain-${i}`, r, issuance?.untilNs ? Uint8Array.of(1, 2, 3) : undefined);
        let j = await open(`uncertain-${i}`);
        await j.beginLogout('login', 1_000n); j.close();
        j = await open(`uncertain-${i}`);
        const s = await j.beginLogout('login', 9_000n); j.close();
        const persisted = await raw(`uncertain-${i}`, bucket => {
          const login = bucket.get('login'), bytes = bucket.count('credential');
          return () => ({ login: login.result, evidence: bytes.result });
        });
        values.push({ phase: s.phase, unknown: s.issuanceUnknown, sequence: s.sequence?.toString(),
          deadline: String(s.deadlineNs), evidence: persisted.evidence, issuance: persisted.login.issuance });
      }
      return values;
    });
    assert.deepEqual(result.map(r => [r.phase, r.unknown, r.sequence, r.deadline, r.evidence]), [
      ['cleanup', true, undefined, '1000', 0], ['cleanup', true, '0', '1000', 0], ['cleanup', true, '7', '1000', 0],
    ]);
    assert(result.every(r => r.issuance === undefined || !('untilNs' in r.issuance)));
  });

  await check('abort after a successful queued write rolls back logout and credential deletion', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      await seed('abort', { ...record(), issuanceUnknown: true, issuance: { sequence: 3n, untilNs: 500n } }, Uint8Array.of(9, 8));
      const j = await open('abort');
      const put = IDBObjectStore.prototype.put;
      let intercepted = false;
      IDBObjectStore.prototype.put = function(value, key) {
        const req = put.call(this, value, key);
        if (key === 'login' && value.phase === 'cleanup') {
          intercepted = true;
          req.addEventListener('success', () => this.transaction.abort());
        }
        return req;
      };
      let code;
      try { code = await failure(j.beginLogout('login', 1_000n)); }
      finally { IDBObjectStore.prototype.put = put; j.close(); }
      const state = await raw('abort', bucket => {
        const login = bucket.get('login'), bytes = bucket.get('credential');
        return () => ({ phase: login.result.phase, bytes: Array.from(bytes.result) });
      });
      return { code, intercepted, ...state };
    }), { code: 'storage_unavailable', intercepted: true, phase: 'active', bytes: [9, 8] });
  });

  await check('strict durability is mandatory and ordinary guards never load credential bytes', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const transact = IDBDatabase.prototype.transaction;
      IDBDatabase.prototype.transaction = function(stores, mode) {
        return transact.call(this, stores, mode, { durability: 'relaxed' });
      };
      let refused;
      try { refused = await failure(open('durability')); }
      finally { IDBDatabase.prototype.transaction = transact; }
      const j = await open('abort');
      const get = IDBObjectStore.prototype.get;
      IDBObjectStore.prototype.get = function(key) {
        if (key === 'credential') throw new Error('unexpected evidence read');
        return get.call(this, key);
      };
      let guard;
      try { guard = await failure(j.assertActive('login', caller, issuer)); }
      finally { IDBObjectStore.prototype.get = get; j.close(); }
      return [refused, guard];
    }), ['storage_unavailable', 'accepted']);
  });

  await check('clock regression/rebinding, bounds and detached inspection cannot change authority', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const j = await open('bounds');
      await j.start({ generation: 'login', caller, issuer });
      const results = [];
      now = 99n; results.push(await failure(j.assertActive('login', caller, issuer))); now = 100n;
      results.push(await failure(j.beginLogout('login', 100n)));
      results.push(await failure(j.beginLogout('login', 300_000_000_101n)));
      const other = authTest.Principal.fromUint8Array(Uint8Array.of(2));
      results.push(await failure(j.addDestination('login', other, 295_000_000_100n)));
      await j.addDestination('login', other, 1_000n);
      const view = await j.inspect(); view.destinations.length = 0; view.phase = 'cleanup';
      results.push(await failure(j.assertActive('login', caller, other)));
      for (let i = 3; i <= 64; i++) await j.addDestination('login', authTest.Principal.fromUint8Array(Uint8Array.of(i, 1)), 1_000n);
      results.push(await failure(j.addDestination('login', authTest.Principal.fromUint8Array(Uint8Array.of(65, 1)), 1_000n)));
      j.close(); clockId = 'changed';
      const reopened = await open('bounds');
      results.push(await failure(reopened.assertActive('login', caller, issuer)));
      reopened.close(); clockId = 'clock'; return results;
    }), ['clock_changed', 'configuration', 'configuration', 'configuration', 'accepted', 'destination_limit', 'clock_changed']);
  });

  await check('malformed metadata and foreign profiles refuse without reset or migration', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const corruptions = [
        r => ({ ...r, extra: true }), r => ({ ...r, issuance: { sequence: 0n } }),
        r => ({ ...r, issuanceUnknown: true, issuance: { sequence: 0n, bytes: Uint8Array.of(1) } }),
        r => ({ ...r, targets: [r.targets[0], r.targets[0]] }), r => ({ ...r, observedNs: -1n }),
        r => ({ ...r, phase: 'cleanup' }), r => ({ ...r, deadlineNs: 1n }),
        r => ({ ...r, targets: [{ ...r.targets[0], attempts: 1 }] }),
        r => ({ ...r, targets: [] }), r => ({ ...r, issuance: undefined }),
      ];
      const results = [];
      for (const [i, corrupt] of corruptions.entries()) {
        await seed(`corrupt-${i}`, corrupt(record()));
        results.push(await failure(open(`corrupt-${i}`)));
      }
      await seed('foreign', record());
      await raw('foreign', b => b.put('other-profile', 'format'));
      results.push(await failure(open('foreign')));
      results.push(await raw('foreign', b => { const req = b.get('format'); return () => req.result; }));
      await raw('future', b => b.put('ic-auth-login-cleanup-journal-v1', 'format'), 2);
      results.push(await failure(open('future')));
      await seed('orphan', undefined, Uint8Array.of(1));
      const j = await open('orphan');
      results.push(await failure(j.start({ generation: 'login', caller, issuer }))); j.close();
      return results;
    }), [...Array(11).fill('storage_corrupt'), 'other-profile', 'storage_unavailable', 'storage_corrupt']);
  });
  await check('foreign storage layouts refuse while existing cleanup progress remains intact', async () => {
    assert.deepEqual(await a.evaluate(async () => {
      const results = [];
      for (const shape of ['extra-store', 'key-path', 'auto-increment', 'index']) {
        await new Promise((resolve, reject) => {
          const req = indexedDB.open(shape, 1);
          req.onupgradeneeded = () => {
            const b = req.result.createObjectStore('journal', shape === 'key-path'
              ? { keyPath: 'id' } : { autoIncrement: shape === 'auto-increment' });
            if (shape === 'extra-store') req.result.createObjectStore('extra');
            if (shape === 'index') b.createIndex('extra', 'id');
            if (shape !== 'key-path') b.put('ic-auth-login-cleanup-journal-v1', 'format');
          };
          req.onerror = () => reject(req.error);
          req.onsuccess = () => { req.result.close(); resolve(); };
        });
        results.push(await failure(open(shape)));
      }
      const r = { ...record(), phase: 'finished', deadlineNs: 1_000n, busyUntilNs: 700n,
        issuanceUnknown: true, issuance: { sequence: 6n },
        targets: [{ canister: issuer.toText(), attempts: 8, nextNs: 900n, confirmedUntilNs: 1_000n }] };
      await seed('finished', r);
      const j = await open('finished');
      results.push(await failure(j.start({ generation: 'replacement', caller, issuer })));
      results.push(await failure(j.assertActive('login', caller, issuer)));
      results.push((await j.beginLogout('login', 2_000n)).phase); j.close();
      const retained = await raw('finished', b => { const req = b.get('login'); return () => req.result; });
      results.push(retained.issuanceUnknown && retained.targets[0].attempts === 8
        && retained.targets[0].confirmedUntilNs === 1_000n && retained.busyUntilNs === 700n
        && retained.issuance.sequence === 6n && retained.deadlineNs === 1_000n);
      return results;
    }), [...Array(4).fill('storage_corrupt'), 'login_exists', 'not_current', 'finished', true]);
  });
  group = 'issuance';
  await issuanceCases({ check, a, b, prepare });
  group = 'transport';
  await transportCases({ check, a, b });
  group = 'admission';
  await admissionCases({ check, a, b });
  group = 'cleanup';
  await cleanupCases({ check, a, b, prepare });
  group = 'logout';
  await logoutCases({ check, a, b });
  group = 'scheduler';
  await schedulerCases({ check, a, b });
  group = 'readiness';
  await readinessCases({ check, a, b });
  group = 'http';
  await httpCases({ check, a });
  if (suite === 'installed-http') {
    group = 'installed-http';
    await installedHttpCases({ check, a });
  }
  if (suite === 'installed-flow') {
    group = 'installed-flow';
    await installedFlowCases({ check, a });
  }
  console.log(`${cases} native browser cases passed`);
} finally {
  await browser?.close();
  await new Promise(resolve => server.close(resolve));
}
