import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';

const evidence = new URL('../../../../target/browser-client/', import.meta.url);
const keys = ['requestBytes', 'requestNodes', 'requestDepth', 'responseBytes', 'responseNodes', 'responseDepth'];

test('actual generated codec settings match Rust policy and refuse directional budget drift', async () => {
  const expected = JSON.parse(readFileSync(new URL('codec-policy.json', evidence), 'utf8')) as Record<string, Record<string, number>>;
  const directory = mkdtempSync(fileURLToPath(new URL('policy.', evidence)));
  writeFileSync(`${directory}/package.json`, '{"type":"module"}');
  writeFileSync(`${directory}/capture.js`, 'export const seen = []; export const state = { field: null };');
  const capture = await import(pathToFileURL(`${directory}/capture.js`).href) as {
    seen: Record<string, unknown>[]; state: { field: string | null };
  };
  const actualRuntime = new URL('../src/generated/wire-runtime.js', import.meta.url).href;
  // Observe configuration at the supported createCodec boundary and delegate to
  // the actual codec. Alter only this isolated output's effective configuration.
  writeFileSync(`${directory}/wire-runtime.js`, `
    import { createCodec as actual } from ${JSON.stringify(actualRuntime)};
    import { seen, state } from './capture.js';
    export function createCodec(metadata, ...args) {
      if (state.field) metadata[state.field]++;
      seen.push(structuredClone(metadata));
      return actual(metadata, ...args);
    }
  `);
  const check = () => {
    assert.equal(capture.seen.length, 4);
    for (const metadata of capture.seen) {
      const policy = expected[String(metadata.id)];
      assert.ok(policy, 'method identity must belong to the Rust owner');
      assert.deepEqual(Object.fromEntries(keys.map(key => [key, metadata[key]])), policy);
    }
  };
  for (const [index, field] of [null, ...keys, null].entries()) {
    capture.seen.length = 0;
    capture.state.field = field ?? null;
    for (const name of ['credential', 'establish', 'read', 'logout']) {
      writeFileSync(`${directory}/${name}.js`, readFileSync(new URL(`../src/generated/${name}.js`, import.meta.url)));
      await import(`${pathToFileURL(`${directory}/${name}.js`).href}?case=${index}`);
    }
    if (field === null) check(); else assert.throws(check, assert.AssertionError);
  }
});
