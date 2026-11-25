import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { HttpAgent, IC_ROOT_KEY, inspectIngress } from '@tokostudios/ic-platform/agent';
import { Ed25519KeyIdentity } from '@tokostudios/ic-platform/identity';
import { Principal } from '@tokostudios/ic-platform/principal';
import { invalidCertificateFixture, readStateResponseFixture } from '@tokostudios/ic-platform/testing';
import { httpUpdateTransport } from '../src/index.ts';
import { runSigningAttempt } from '../src/attempt.ts';

const rootKey = Buffer.from(IC_ROOT_KEY, 'hex');
const host = () => new URL('https://gateway.invalid');
const browser = () => Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
const target = () => Principal.fromUint8Array(Uint8Array.of(1));
const controller = () => new AbortController();
function fixture(file: string, name: string): Uint8Array {
  const line = readFileSync(new URL(`../../../crates/ic-auth-wire/tests/fixtures/${file}`, import.meta.url), 'utf8')
    .split('\n').find(line => line.startsWith(`${name}\t`))!;
  return new Uint8Array(Buffer.from(line.split('\t')[1]!, 'hex'));
}
const logout = fixture('caller-frames.tsv', 'logout-request');
const admission = fixture('admission-frames.tsv', 'establish-none');

test('real SDK signs exact admission/logout input and request-bound read-state; HTTP acceptance never confirms', async () => {
  for (const [methodName, frame] of [['admit_auth', admission], ['logout_auth', logout]] as const) {
    const identity = browser(), canister = target(), abort = controller();
    const paths: string[] = [];
    let requestId: Uint8Array | undefined;
    const update = httpUpdateTransport({ host: host(), rootKey, methodName, fetch: async (input, init) => {
      const path = new URL(input.toString()).pathname; paths.push(path);
      assert.equal(init?.signal, abort.signal);
      assert.equal(init?.redirect, 'error'); assert.equal(init?.credentials, 'omit');
      const ingress = inspectIngress(init!.body as Uint8Array);
      assert.equal(ingress.sender.toText(), identity.getPrincipal().toText());
      assert.deepEqual(ingress.senderPublicKey, new Uint8Array(identity.getPublicKey().toDer()));
      assert.equal(ingress.senderSignature?.length, 64);
      if (ingress.requestType === 'call') {
        assert.equal(ingress.canisterId.toText(), canister.toText());
        assert.equal(ingress.methodName, methodName); assert.deepEqual(ingress.arg, frame);
        requestId = ingress.requestId;
        return new Response(null, { status: 202 });
      }
      assert.equal(ingress.requestType, 'read_state');
      assert(ingress.requestType === 'read_state');
      assert.deepEqual(ingress.paths[0], [new TextEncoder().encode('request_status'), requestId]);
      return new Response(new Uint8Array(readStateResponseFixture(invalidCertificateFixture())).buffer);
    } });
    await assert.rejects(update(canister, identity, frame, abort.signal));
    assert.deepEqual(paths, [`/api/v2/canister/${canister}/call`, `/api/v3/canister/${canister}/read_state`]);
  }
});

test('HTTP failures and lost responses never resubmit an update', async () => {
  for (const status of [429, 503, undefined]) {
    let calls = 0;
    const update = httpUpdateTransport({ host: host(), rootKey, methodName: 'contract', fetch: async () => {
      calls++;
      if (status === undefined) throw new Error('connection lost after send');
      return new Response(null, { status });
    } });
    await assert.rejects(update(target(), browser(), logout, controller().signal));
    assert.equal(calls, 1);
  }
});

test('aborting before submission or after acceptance prevents later polling', async () => {
  const abort = controller(); let calls = 0;
  const update = httpUpdateTransport({ host: host(), rootKey, methodName: 'contract', fetch: async () => {
    calls++; abort.abort(); return new Response(null, { status: 202 });
  } });
  await assert.rejects(update(target(), browser(), logout, abort.signal));
  assert.equal(calls, 1);
  await assert.rejects(update(target(), browser(), logout, abort.signal));
  assert.equal(calls, 1);
});

test('journal invalidation after submission blocks a freshly signed read-state request', async () => {
  let active = true, calls = 0;
  const update = httpUpdateTransport({ host: host(), rootKey, methodName: 'contract', fetch: async () => {
    calls++; active = false; return new Response(null, { status: 202 });
  } });
  await assert.rejects(runSigningAttempt({ identity: browser(), target: target(), assertCurrent: async () => {
    if (!active) throw new Error('journal invalidated');
  } }, (identity, signal) => update(target(), identity, logout, signal)));
  assert.equal(calls, 1);
});

test('gateway, root and endpoint configuration are owned before attempts and each SDK gets separate root bytes', async t => {
  const observed: { host: string | undefined; root: number[]; retries: number | undefined; fetchRoot: boolean | undefined; sync: boolean | undefined }[] = [];
  const stop = new Error('stop before SDK IO');
  t.mock.method(HttpAgent, 'createSync', (options: Parameters<typeof HttpAgent.createSync>[0]) => {
    assert(options?.rootKey);
    observed.push({ host: options.host, root: [...options.rootKey], retries: options.retryTimes,
      fetchRoot: options.shouldFetchRootKey, sync: options.shouldSyncTime });
    options.rootKey.fill(7); throw stop;
  });
  for (const backing of [Buffer.from([99, 1, 2, 3, 99]), Uint8Array.of(99, 1, 2, 3, 99)]) {
    const options = { host: host(), rootKey: backing.subarray(1, 4), methodName: 'contract' };
    const update = httpUpdateTransport(options);
    options.host.hostname = 'changed.invalid'; backing.fill(8);
    for (let i = 0; i < 2; i++) await assert.rejects(update(target(), browser(), logout, controller().signal), e => e === stop);
  }
  assert.deepEqual(observed, Array(4).fill({ host: host().origin, root: [1, 2, 3], retries: 0, fetchRoot: false, sync: false }));
});

test('destination and raw request bytes cannot change across signing awaits', async () => {
  const canister = target(), original = canister.toText(), request = new Uint8Array(logout);
  const paths: string[] = [];
  const update = httpUpdateTransport({ host: host(), rootKey, methodName: 'contract', fetch: async (input, init) => {
    paths.push(new URL(input.toString()).pathname);
    const ingress = inspectIngress(init!.body as Uint8Array);
    assert(ingress.requestType === 'call'); assert.deepEqual(ingress.arg, logout);
    return new Response(null, { status: 503 });
  } });
  const pending = update(canister, browser(), request, controller().signal);
  canister.toUint8Array()[0] = 2; request.fill(0);
  await assert.rejects(pending);
  assert.deepEqual(paths, [`/api/v2/canister/${original}/call`]);
});

test('attempt timeout fences a late HTTP acceptance even when fetch ignores cancellation', async () => {
  let calls = 0, resume!: () => void;
  const update = httpUpdateTransport({ host: host(), rootKey, methodName: 'contract', fetch: async () => {
    calls++; return new Promise(resolve => { resume = () => resolve(new Response(null, { status: 202 })); });
  } });
  await assert.rejects(runSigningAttempt({ identity: browser(), target: target(), timeoutMs: 100,
    assertCurrent: async () => {} }, (identity, signal) => update(target(), identity, logout, signal)), { code: 'timeout' });
  assert.equal(calls, 1); resume();
  await new Promise(resolve => setTimeout(resolve, 0));
  assert.equal(calls, 1);
});

test('invalid protected configuration and anonymous callers refuse before HTTP', async () => {
  for (const bad of ['https://user:pass@gateway.invalid', 'https://gateway.invalid/path',
    'https://gateway.invalid/?key=x', 'https://gateway.invalid/#part', 'file:///', 'http://gateway.invalid', 'http://127.0.0.1.invalid', 'http://[::2]']) {
    assert.throws(() => httpUpdateTransport({ host: new URL(bad), rootKey, methodName: 'contract' }), { code: 'configuration' });
  }
  for (const local of ['localhost', '127.25.0.1', '[::1]']) {
    httpUpdateTransport({ host: new URL(`http://${local}:8080`), rootKey, methodName: 'contract' });
  }
  for (const methodName of ['', 'auth/logout', 'a'.repeat(129)]) {
    assert.throws(() => httpUpdateTransport({ host: host(), rootKey, methodName }), { code: 'configuration' });
  }
  assert.throws(() => httpUpdateTransport({ host: host(), rootKey: new Uint8Array(), methodName: 'contract' }), { code: 'configuration' });
  let calls = 0;
  const update = httpUpdateTransport({ host: host(), rootKey, methodName: 'contract', fetch: async () => {
    calls++; throw new Error('unexpected IO');
  } });
  await assert.rejects(update(Principal.anonymous(), browser(), logout, controller().signal), { code: 'configuration' });
  const identity = { getPrincipal: () => Principal.anonymous(), transformRequest: browser().transformRequest };
  await assert.rejects(update(target(), identity, logout, controller().signal), { code: 'configuration' });
  assert.equal(calls, 0);
});
