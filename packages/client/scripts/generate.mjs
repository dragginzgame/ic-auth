// Regenerate the codecs used by this package from Rust-owned schemas.
import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
const root = fileURLToPath(new URL('../../../', import.meta.url));
const source = join(root, 'target/client-wire/typescript');
const compiled = join(root, 'target/client-wire/javascript');
const destination = fileURLToPath(new URL('../src/generated/', import.meta.url));
const modules = ['credential', 'establish', 'logout', 'wire-runtime'];
const check = process.argv[2] === '--check';
if (process.argv.length > (check ? 3 : 2)) throw new Error('Usage: node scripts/generate.mjs [--check]');
execFileSync('cargo', ['run', '--locked', '-p', 'ic-auth-wire', '--features', 'typescript',
  '--example', 'generate', '--', source], { cwd: root, stdio: 'inherit' });
execFileSync(process.execPath, [fileURLToPath(new URL('../node_modules/typescript/bin/tsc', import.meta.url)),
  '--target', 'ES2023', '--module', 'ESNext', '--strict', '--declaration', '--outDir', compiled,
  ...modules.map(name => join(source, `${name}.ts`))], { cwd: root, stdio: 'inherit' });
mkdirSync(destination, { recursive: true });
for (const name of modules.flatMap(name => [`${name}.js`, `${name}.d.ts`])) {
  const bytes = readFileSync(join(compiled, name));
  if (check) {
    if (!bytes.equals(readFileSync(join(destination, name)))) throw new Error(`Regenerate client codec: ${name}`);
  } else writeFileSync(join(destination, name), bytes);
}
console.log(check ? 'Client codecs match Rust generator' : 'Generated client codecs');
