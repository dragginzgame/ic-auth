import assert from 'node:assert/strict';
import { accessSync, constants, copyFileSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import { isAbsolute, resolve } from 'node:path';
import { createServer } from 'node:http';
import { pathToFileURL } from 'node:url';
import { checkIcSdkBoundary } from './check-ic-sdk-boundary.mjs';

/** Install only the packed artifact; no source-relative imports or linked client. */
export async function qualifyClientPackage({ root, source, directory, packed, command, browser = false }) {
  const consumer = resolve(directory, 'consumer');
  mkdirSync(consumer);
  const manifest = JSON.parse(readFileSync(resolve(source, 'package.json'), 'utf8'));
  const archive = resolve(directory, packed.filename);
  const dependencies = { [manifest.name]: `file:${archive}`, ...manifest.dependencies };
  const consumerManifest = { name: 'ic-auth-packed-consumer', version: '0.0.0', private: true,
    type: 'module', dependencies };
  const reviewed = JSON.parse(readFileSync(resolve(source, 'package-lock.json'), 'utf8'));
  const lock = structuredClone(reviewed);
  lock.name = consumerManifest.name;
  lock.packages[''] = consumerManifest;
  lock.packages[`node_modules/${manifest.name}`] = { version: packed.version,
    resolved: `file:${archive}`, integrity: packed.integrity, dependencies: manifest.dependencies };
  writeFileSync(resolve(consumer, 'package.json'), JSON.stringify(consumerManifest, null, 2));
  writeFileSync(resolve(consumer, 'package-lock.json'), JSON.stringify(lock, null, 2));
  // Seed the existing qualified external selections, then let npm prune the
  // disposable consumer lock. Preparation never changes the producer lock.
  command('npm', ['install', '--offline', '--package-lock-only', '--ignore-scripts', '--no-audit', '--no-fund'], consumer);
  const selected = JSON.parse(readFileSync(resolve(consumer, 'package-lock.json'), 'utf8'));
  for (const [path, entry] of Object.entries(selected.packages)) {
    if (path === '' || path === `node_modules/${manifest.name}`) continue;
    assert.equal(entry.version, reviewed.packages[path]?.version, `Unreviewed selection: ${path}`);
    assert.equal(entry.resolved, reviewed.packages[path]?.resolved, `Changed external location: ${path}`);
    assert.equal(entry.integrity, reviewed.packages[path]?.integrity, `Changed external source: ${path}`);
  }
  command('npm', ['ci', '--offline', '--ignore-scripts', '--no-audit', '--no-fund'], consumer);
  const installed = resolve(consumer, 'node_modules', manifest.name);
  assert.equal(readFileSync(resolve(installed, 'LICENSE.md'), 'utf8'), readFileSync(resolve(root, 'LICENSE.md'), 'utf8'));
  copyFileSync(resolve(source, 'fixtures/consumer.ts'), resolve(consumer, 'consumer.ts'));
  writeFileSync(resolve(consumer, 'tsconfig.json'), JSON.stringify({ compilerOptions: {
    target: 'ES2022', module: 'NodeNext', moduleResolution: 'NodeNext', strict: true,
    noUncheckedIndexedAccess: true, exactOptionalPropertyTypes: true, skipLibCheck: false,
    types: [], lib: ['ES2022', 'DOM', 'DOM.Iterable'], outDir: 'compiled',
  }, include: ['consumer.ts'] }, null, 2));
  await checkIcSdkBoundary([resolve(consumer, 'package.json'), resolve(consumer, 'consumer.ts')]);
  command('node', [resolve(source, 'node_modules/typescript/lib/tsc.js'), '-p', 'tsconfig.json'], consumer);
  const { smoke } = await import(pathToFileURL(resolve(consumer, 'compiled/consumer.js')).href);
  const { IDBFactory } = await import(pathToFileURL(resolve(source, 'node_modules/fake-indexeddb/build/esm/index.js')).href);
  assert.deepEqual(await smoke(new IDBFactory()), { issues: 1, admissions: 1, revocations: 2 });
  // Verify the export map actually refuses internal paths in the installed copy.
  writeFileSync(resolve(consumer, 'private-import.mjs'), `import '${manifest.name}/dist/src/logout-store.js';\n`);
  assert.throws(() => command('node', ['private-import.mjs'], consumer),
    error => String(error.cause?.stderr).includes('ERR_PACKAGE_PATH_NOT_EXPORTED'));
  const { build } = await import(pathToFileURL(resolve(source, 'node_modules/esbuild/lib/main.js')).href);
  const bundle = await build({ absWorkingDir: consumer, entryPoints: ['consumer.ts'],
    outfile: 'browser.js', bundle: true, platform: 'browser', format: 'esm', target: 'es2022',
    metafile: true, logLevel: 'silent' });
  assert(Object.keys(bundle.metafile.inputs).some(path => path.includes(`node_modules/${manifest.name}/dist/src/`)));
  assert(Object.keys(bundle.metafile.inputs).every(path => !path.includes('packages/client/src/')),
    'Consumer bundle must resolve the archive, not producer sources');
  if (browser) await qualifyPackedBrowser({ root, source, consumer, directory, command });
  writeFileSync(resolve(directory, 'consumer-proof.json'), JSON.stringify({
    externalSelections: 'producer-lock-subset', declarations: 'strict',
    smoke: 'injected-transports-and-fake-indexeddb', privateImports: 'refused', bundle: 'browser-es2022',
    ...(browser ? { chromium: 'browser-proof.json' } : {}),
  }, null, 2) + '\n');
}

/** The already installed archive and public fixture, with real IndexedDB. */
async function qualifyPackedBrowser({ root, source, consumer, directory, command }) {
  const selected = process.env.IC_AUTH_BROWSER_EXECUTABLE;
  assert(selected && isAbsolute(selected), 'Select an absolute prepared Chromium executable');
  const executablePath = realpathSync(selected);
  accessSync(executablePath, constants.X_OK);
  const { chromium } = await import(pathToFileURL(resolve(source, 'node_modules/playwright-core/index.mjs')).href);
  const javascript = readFileSync(resolve(consumer, 'browser.js'));
  const server = createServer((request, response) => {
    if (request.method === 'GET' && request.url === '/') {
      response.setHeader('content-type', 'text/html'); response.end('<!doctype html><title>Packed Auth consumer</title>');
    } else if (request.method === 'GET' && request.url === '/browser.js') {
      response.setHeader('content-type', 'text/javascript'); response.end(javascript);
    } else { response.writeHead(404); response.end(); }
  });
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  const temporaryRoot = mkdtempSync('/tmp/ic-auth-packed-browser-');
  const previousTemporaryRoot = process.env.TMPDIR;
  let browser, timeout;
  try {
    // Chromium singleton sockets require a short path, independently of retained evidence.
    process.env.TMPDIR = temporaryRoot;
    browser = await chromium.launch({ executablePath, headless: true, args: ['--no-sandbox'] });
    const page = await browser.newPage();
    await page.goto(`http://127.0.0.1:${server.address().port}/`);
    const counts = await Promise.race([
      page.evaluate(async () => {
        const { smoke } = await import('/browser.js');
        return smoke(indexedDB);
      }),
      new Promise((_, reject) => { timeout = setTimeout(() => reject(Error('Packed browser smoke timed out')), 30_000); }),
    ]);
    assert.deepEqual(counts, { issues: 1, admissions: 1, revocations: 2 });
    const hash = path => command('bash', [resolve(root, 'scripts/dev/run-host-file.sh'),
      'hash-file', path, '536870912']).trim();
    writeFileSync(resolve(directory, 'browser-proof.json'), JSON.stringify({
      browser: browser.version(), executable: executablePath, executableSha256: hash(executablePath),
      bundleSha256: hash(resolve(consumer, 'browser.js')),
      smoke: 'installed-archive-public-api-and-real-indexeddb-reopen', counts,
      transports: 'injected',
    }, null, 2) + '\n');
  } finally {
    clearTimeout(timeout);
    if (previousTemporaryRoot === undefined) delete process.env.TMPDIR;
    else process.env.TMPDIR = previousTemporaryRoot;
    try { await browser?.close(); }
    finally { await new Promise(resolve => server.close(resolve)); rmSync(temporaryRoot, { recursive: true, force: true }); }
  }
}
