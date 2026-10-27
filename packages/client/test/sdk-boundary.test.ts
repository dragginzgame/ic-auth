import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

test('the client SDK gate rejects aliased dependencies and nested direct imports', async () => {
  const { checkIcSdkBoundary } = await import(
    new URL('../../../../scripts/ci/check-ic-sdk-boundary.mjs', import.meta.url).href);
  const evidence = new URL('../../../../target/browser-client/', import.meta.url);
  const root = mkdtempSync(fileURLToPath(new URL('sdk-boundary.', evidence)));
  const client = `${root}/packages/client`;
  mkdirSync(`${client}/test/nested`, { recursive: true });
  const manifest = JSON.parse(readFileSync(new URL('../../package.json', import.meta.url), 'utf8'));
  writeFileSync(`${client}/package.json`, JSON.stringify(manifest));
  await checkIcSdkBoundary([client]);

  const sdk = '@icp-sdk' + '/core';
  manifest.dependencies.shadow = `npm:${sdk}@6.1.1`;
  writeFileSync(`${client}/package.json`, JSON.stringify(manifest));
  await assert.rejects(checkIcSdkBoundary([client]));

  delete manifest.dependencies.shadow;
  writeFileSync(`${client}/package.json`, JSON.stringify(manifest));
  writeFileSync(`${client}/test/nested/consumer.ts`,
    ['import { Principal } from ', JSON.stringify(`${sdk}/principal`), ';'].join(''));
  await assert.rejects(checkIcSdkBoundary([client]));
});
