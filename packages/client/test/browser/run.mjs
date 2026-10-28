// Explicit, opt-in browser qualification. No browser download or user profile.
import { chromium } from 'playwright-core';
import { build } from 'esbuild';
import { createServer } from 'node:http';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { access, readFile, mkdtemp } from 'node:fs/promises';
import { constants } from 'node:fs';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const executablePath = process.env.IC_AUTH_BROWSER_EXECUTABLE;
assert.ok(executablePath, 'Select an explicitly prepared Chromium executable with IC_AUTH_BROWSER_EXECUTABLE');
await access(executablePath, constants.X_OK);
const args = process.argv.slice(2);
assert.equal(args.length, 9, 'native fixture must supply gateway, receivers, root and original credentials');
const [gateway, issuer, receiver, root, now, credential, subject, until, peerCredential] = args;
const backend = new URL(gateway);
assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(backend.hostname), 'qualification gateway must be local');
const clientRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const bundle = await build({ entryPoints: [resolve(clientRoot, 'test/browser/qualification.ts')],
  absWorkingDir: clientRoot, bundle: true, platform: 'browser', format: 'iife',
  globalName: 'authFixture', write: false });
const javascript = bundle.outputFiles[0].contents;
const server = createServer(async (request, response) => {
  try {
    const path = new URL(request.url, 'http://localhost').pathname;
    if (path === '/') { response.setHeader('content-type', 'text/html'); response.end('<script src="/bundle.js"></script>'); return; }
    if (path === '/bundle.js') { response.setHeader('content-type', 'text/javascript'); response.end(javascript); return; }
    if (path === '/favicon.ico') { response.writeHead(204); response.end(); return; }
    const match = /^\/api\/v[23]\/canister\/([^/]+)\/(call|read_state)$/.exec(path);
    if (request.method !== 'POST' || !match || ![issuer, receiver].includes(match[1])) { response.writeHead(404); response.end(); return; }
    const chunks = []; let bytes = 0;
    for await (const chunk of request) { bytes += chunk.length; if (bytes > 1_048_576) throw Error('bounded test proxy request exceeded'); chunks.push(chunk); }
    const reply = await fetch(new URL(path, backend), { method: 'POST', body: Buffer.concat(chunks),
      headers: { 'content-type': 'application/cbor' }, signal: AbortSignal.timeout(20_000) });
    response.writeHead(reply.status, { 'content-type': reply.headers.get('content-type') ?? 'application/cbor' });
    response.end(Buffer.from(await reply.arrayBuffer()));
  } catch (error) { response.writeHead(502); response.end(String(error)); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
let browser;
// Chromium's singleton socket has a short Unix path limit. Keep its transient
// socket/profile root short even when the retained qualification path is long.
const browserTemporaryRoot = await mkdtemp('/tmp/ic-auth-browser-');
process.env.TMPDIR = browserTemporaryRoot;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ['--no-sandbox'] });
  const context = await browser.newContext();
  const origin = `http://127.0.0.1:${server.address().port}`;
  const page = async () => { const result = await context.newPage(); await result.goto(origin); await result.waitForFunction(() => !!globalThis.authFixture); return result; };
  const a = await page(), b = await page();
  const name = `qualification-${crypto.randomUUID()}`;
  await a.evaluate(name => authFixture.open(name, true), name);
  await b.evaluate(name => authFixture.open(name), name);
  const claims = await Promise.all([a.evaluate(() => authFixture.issuance()), b.evaluate(() => authFixture.issuance())]);
  assert.equal(claims.filter(Boolean).length, 1, 'cross-tab issuance must be unique');
  const active = await b.evaluate(() => authFixture.state());
  await a.evaluate(() => authFixture.abortQueuedWrite());
  assert.deepEqual(await b.evaluate(() => authFixture.state()), active, 'aborted queued native write was published');
  await a.evaluate(() => authFixture.refuseRelaxedCommit());
  assert.deepEqual(await b.evaluate(() => authFixture.state()), active, 'relaxed native transaction was accepted');
  await a.evaluate(() => authFixture.delayedSign());
  await a.waitForFunction(() => authFixture.signatureWaiting());
  await b.evaluate(() => authFixture.begin());
  assert.equal(await a.evaluate(() => authFixture.finishSignature()), true, 'late real SDK signature must be rejected');
  await assert.rejects(a.evaluate(() => authFixture.sign()));
  const before = await b.evaluate(() => authFixture.state());
  await a.evaluate(() => authFixture.abort());
  assert.deepEqual(await b.evaluate(() => authFixture.state()), before, 'aborted strict transaction changed state');
  await a.evaluate(() => { authFixture.network(true, false); void authFixture.cleanup().catch(() => {}); });
  await a.waitForFunction(() => authFixture.counts().attempts === 1);
  const claimed = await b.evaluate(() => authFixture.state());
  assert.equal(claimed.targets[0].attempts, 1);
  const cdp = await context.newCDPSession(a);
  const crashed = new Promise(resolve => a.once('crash', resolve));
  void cdp.send('Page.crash').catch(() => {});
  await Promise.race([crashed, new Promise((_, reject) => setTimeout(() => reject(Error('renderer crash timed out')), 10_000))]);
  await a.close();
  assert.equal((await b.evaluate(() => authFixture.observer())).phase, 'cleanup');
  assert.equal((await b.evaluate(() => authFixture.counts())).disposed, 0, 'keyless observer erased custody');
  await b.evaluate(() => authFixture.time('6000000001'));
  const resumed = await b.evaluate(() => authFixture.cleanup());
  assert.equal(resumed.phase, 'finished');
  assert.equal(resumed.deadlineNs, before.deadlineNs);
  assert.equal(resumed.issuanceUnknown, true);
  assert.equal(resumed.targets[0].attempts, 2, 'crashed attempt was not retained');
  assert.ok(resumed.targets.every(target => target.confirmedUntilNs !== undefined));
  await b.evaluate(() => authFixture.close());
  await assert.rejects(b.evaluate(() => authFixture.state()), 'closed storage must refuse');
  await b.close();
  const reopened = await page();
  await reopened.evaluate(() => authFixture.time('6000000001'));
  await reopened.evaluate(name => authFixture.open(name), name);
  assert.deepEqual(await reopened.evaluate(() => authFixture.state()), resumed);
  await assert.rejects(reopened.evaluate(() => authFixture.sign()));
  await assert.rejects(reopened.evaluate(name => authFixture.open(name, true), name), 'existing uncertainty must never be overwritten');
  await reopened.close();
  console.log('PASS actual Chromium tabs: strict commit; queued-write abort; relaxed-durability refusal; unique issuance; real late signing; renderer crash; retained lease/attempt/deadline/uncertainty; keyless observer; storage refusal; reopen');

  const unavailable = await page();
  await unavailable.evaluate(name => authFixture.open(name, true), `${name}-unavailable`);
  await unavailable.evaluate(() => authFixture.issuance());
  await unavailable.evaluate(() => authFixture.begin());
  await unavailable.evaluate(() => authFixture.network(false, true));
  const expired = await unavailable.evaluate(() => authFixture.cleanup());
  assert.equal(expired.phase, 'finished');
  assert.equal(expired.deadlineNs, 11_000_000_000n);
  assert.equal(expired.issuanceUnknown, true);
  assert.ok(expired.targets.every(target => target.confirmedUntilNs === undefined), 'outage fabricated acknowledgement');
  assert.ok((await unavailable.evaluate(() => authFixture.counts())).attempts > 0);
  await assert.rejects(unavailable.evaluate(() => authFixture.sign()));
  await unavailable.close();
  console.log('PASS actual Chromium outage: controlled deadline expiry retains unknown issuance and unconfirmed destinations; signing stays invalid');

  for (const phase of ['issuance', 'admission']) {
    const contended = await page();
    await contended.evaluate(({ name, phase }) => authFixture.admissionContention(name, phase),
      { name: `${name}-${phase}`, phase });
    await contended.close();
  }
  console.log('PASS actual Chromium admission contention: bounded queued work; logout during blocked issuance/admission; fixed deadline; no false remote acknowledgement');

  const live = await page();
  const result = await live.evaluate(input => authFixture.sdk(input), { issuer, receiver, root, now, credential, peerCredential, subject, until });
  console.log('PASS actual browser SDK: pinned IC root; distinct account/browser signers; original issuance recovery; two receivers/devices; retry lease unchanged; lost logout reply; authenticated acknowledgements; retained-key refusal', JSON.stringify(result));
  console.log(JSON.stringify({ browser: browser.version(), platform: process.platform,
    executablePath, browserTemporaryRoot,
    executableSha256: createHash('sha256').update(await readFile(executablePath)).digest('hex'),
    bundleSha256: createHash('sha256').update(javascript).digest('hex'),
    issuer: 'injected native credential and lookup; not a production issuer', controlledLifecycleClock: true }));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
