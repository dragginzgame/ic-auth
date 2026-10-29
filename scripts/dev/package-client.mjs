import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync, mkdtempSync, lstatSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { basename, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { qualifyClientPackage } from '../ci/check-client-package.mjs';

const phase = process.argv[2];
assert(phase === 'prepare' ? process.argv.length === 3
  : ['pack', 'check'].includes(phase) && process.argv.length === 4, 'Select package preparation or a prepared pack/check');
const root = fileURLToPath(new URL('../..', import.meta.url));
const source = resolve(root, 'packages/client');
const parent = resolve(root, 'target/browser-client-packages');
mkdirSync(parent, { recursive: true });
const directory = phase === 'prepare' ? mkdtempSync(resolve(parent, 'package.')) : resolve(process.argv[3]);
let sequence = 0;
function command(file, args, cwd = root, input) {
  const log = resolve(directory, `${phase}-${++sequence}-${basename(file)}.log`);
  try {
    const stdout = execFileSync(file, args, { cwd, input, encoding: 'utf8',
      maxBuffer: 8 * 1024 * 1024, stdio: ['pipe', 'pipe', 'pipe'],
      env: { ...process.env, npm_config_dry_run: 'false', npm_config_update_notifier: 'false' } });
    writeFileSync(log, stdout);
    return stdout;
  } catch (error) {
    writeFileSync(log, String(error.stdout ?? '') + String(error.stderr ?? ''));
    throw new Error(`Client package command failed; retained log: ${log}`, { cause: error });
  }
}
// Seal tracked/new inputs plus the actual compiler source trees, including
// ignored source files. Build outputs and moving sibling inputs are excluded.
const generatorPaths = readFileSync(resolve(root, 'ci/client-generator-inputs.txt'), 'utf8').trimEnd().split('\n');
const paths = [...generatorPaths, 'Makefile', 'LICENSE', 'packages/client', 'scripts/dev/client-tools.sh', 'scripts/dev/client-toolchain.sh',
  'scripts/dev/package-client.sh', 'scripts/dev/package-client.mjs', 'scripts/dev/test-client.sh',
  'scripts/ci/check-client-package.mjs', 'scripts/ci/check-ic-sdk-boundary.mjs'];
function inputPaths() {
  const observed = command('git', ['ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', ...paths])
    .split('\0').filter(Boolean);
  function sources(path) {
    for (const entry of readdirSync(resolve(root, path), { withFileTypes: true })) {
      const child = `${path}/${entry.name}`;
      if (entry.isDirectory()) sources(child);
      else observed.push(child); // Host admission refuses symbolic links.
    }
  }
  for (const path of [...generatorPaths, 'packages/client/src', 'packages/client/fixtures']) {
    if (lstatSync(resolve(root, path)).isDirectory()) sources(path);
    else observed.push(path);
  }
  for (const name of ['package.json', 'package-lock.json', 'README.md', 'LICENSE', 'tsconfig.json', 'tsconfig.package.json', '.nvmrc']) {
    observed.push(`packages/client/${name}`);
  }
  return [...new Set(observed)].sort().join('\0') + '\0';
}
function prepare() {
  const selected = inputPaths();
  const tracked = new Set(command('git', ['ls-files', '-z', '--cached', '--', ...paths]).split('\0'));
  return { selected, revision: command('git', ['rev-parse', 'HEAD']).trim(),
    dirty: selected.split('\0').some(path => path && !tracked.has(path))
      || command('git', ['status', '--porcelain', '--untracked-files=all', '--', ...paths]).length > 0 };
}
const preparation = phase === 'prepare' ? prepare()
  : JSON.parse(readFileSync(resolve(directory, 'preparation.json'), 'utf8'));
const { selected, revision, dirty } = preparation;
const seal = () => command('bash', ['scripts/dev/run-host-file.sh', 'hash-files0', '4194304', '4194304', '4096'], root, selected);
if (phase === 'prepare') {
  writeFileSync(resolve(directory, 'inputs.tsv0'), seal());
  writeFileSync(resolve(directory, 'preparation.json'), JSON.stringify(preparation, null, 2));
  console.log(directory);
} else {
  await packClient();
}

async function packClient() {
  const before = readFileSync(resolve(directory, 'inputs.tsv0'), 'utf8');
  assert.equal(inputPaths(), selected, 'Source input roster changed before package build');
  assert.equal(seal(), before, 'Source inputs changed during contract preparation');
  command('npm', ['--prefix', source, 'run', 'check:sdk']);
  assert.equal(readFileSync(resolve(source, 'LICENSE'), 'utf8'), readFileSync(resolve(root, 'LICENSE'), 'utf8'));
  const staging = resolve(directory, 'staging');
  mkdirSync(staging);
  // Compile to a fresh staging tree: removed modules or stale incremental output
  // from source tests must never leak into a consumer archive.
  command('npm', ['--prefix', source, 'run', 'build:package', '--', '--outDir', resolve(staging, 'dist')]);
  for (const name of ['package.json', 'README.md', 'LICENSE']) copyFileSync(resolve(source, name), resolve(staging, name));
  const manifest = JSON.parse(readFileSync(resolve(source, 'package.json'), 'utf8'));
  const report = JSON.parse(command('npm', ['pack', '--offline', '--ignore-scripts', '--json', '--pack-destination', directory], staging));
  const packed = report[manifest.name];
  assert(packed && packed.name === manifest.name && packed.version === manifest.version);
  assert.equal(manifest.private, true, 'Local delivery must remain private');
  assert(packed.files.every(({ path }) => /^(dist\/src\/.+\.(js|d\.ts)|package\.json|README\.md|LICENSE)$/.test(path)),
    'Unexpected archive payload');
  for (const path of ['dist/src/index.js', 'dist/src/index.d.ts', 'dist/src/generated/wire-runtime.js', 'README.md', 'LICENSE']) {
    assert(packed.files.some(file => file.path === path), `Missing archive entry: ${path}`);
  }
  writeFileSync(resolve(directory, 'npm-pack.json'), JSON.stringify(report, null, 2) + '\n');
  if (phase === 'check') await qualifyClientPackage({ root, source, directory, packed, command });
  assert.equal(inputPaths(), selected, 'Source input roster changed during package preparation');
  assert.equal(seal(), before, 'Source inputs changed during package preparation');
  assert.equal(command('git', ['rev-parse', 'HEAD']).trim(), revision, 'Source revision changed');
  const typescript = JSON.parse(readFileSync(resolve(source, 'node_modules/typescript/package.json'), 'utf8')).version;
  const lock = JSON.parse(readFileSync(resolve(source, 'package-lock.json'), 'utf8'));
  assert.equal(typescript, lock.packages['node_modules/typescript'].version, 'Prepare the locked TypeScript compiler');
  const receipt = {
    package: manifest.name, version: manifest.version,
    source: { revision, dirty, inputs: 'inputs.tsv0' },
    tools: { node: process.versions.node, npm: command('npm', ['--version']).trim(), typescript },
    archive: { filename: packed.filename, integrity: packed.integrity, shasum: packed.shasum },
    qualification: phase === 'check' ? 'packed-consumer' : 'archive-only',
  };
  writeFileSync(resolve(directory, 'delivery.json'), JSON.stringify(receipt, null, 2) + '\n');
  console.log(`Client archive: ${resolve(directory, packed.filename)}`);
  console.log(`Client package qualification: ${receipt.qualification}; source dirty: ${dirty}`);
}
