import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { Identity } from '@icp-sdk/core/agent';
import { internetIdentityExample } from '../examples/internet-identity.js';
import { deferred, harness, principal } from './support.js';

test('existing II identity signs the injected agent; logout prevents a late login', async () => {
  const h = harness(); const identity = principal(3);
  let observed: Promise<string> | undefined;
  const client = internetIdentityExample(async () => identity, { host: 'https://icp-api.io' }, (agent, scope) => {
    observed = agent.getPrincipal().then(pid => pid.toText());
    return h.options.issuer({ identity, generation: 'fixture' }, scope);
  }, h.options);
  await client.token(h.configured);
  assert.equal(await observed, identity.getPrincipal().toText());

  const late = deferred<Identity>();
  const cancelled = internetIdentityExample(() => late.promise, { host: 'https://icp-api.io' }, () => { throw new Error('not called'); }, h.options);
  const connect = cancelled.connect();
  assert.equal(await cancelled.logout(), true);
  late.resolve(identity);
  await assert.rejects(connect, /login cancelled/);
});
