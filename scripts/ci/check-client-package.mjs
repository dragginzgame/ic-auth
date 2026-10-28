import assert from 'node:assert/strict';
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { checkIcSdkBoundary } from './check-ic-sdk-boundary.mjs';

/** Install only the packed artifact; no source-relative imports or linked client. */
export async function qualifyClientPackage({ source, directory, packed, command }) {
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
  assert.equal(readFileSync(resolve(installed, 'LICENSE'), 'utf8'), readFileSync(resolve(source, 'LICENSE'), 'utf8'));
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
  writeFileSync(resolve(directory, 'consumer-proof.json'), JSON.stringify({
    externalSelections: 'producer-lock-subset', declarations: 'strict',
    smoke: 'injected-transports-and-fake-indexeddb', privateImports: 'refused', bundle: 'browser-es2022',
  }, null, 2) + '\n');
}
