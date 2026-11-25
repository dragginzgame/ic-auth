import { test } from 'node:test';
import assert from 'node:assert/strict';
import { requestIdOf, Endpoint, SubmitRequestType, ReadRequestType, Expiry, type Identity } from '@tokostudios/ic-platform/agent';
import { Ed25519KeyIdentity } from '@tokostudios/ic-platform/identity';
import { Principal } from '@tokostudios/ic-platform/principal';
import { AttemptError, runSigningAttempt, type AttemptFailure } from '../src/attempt.ts';

const target = () => Principal.fromUint8Array(Uint8Array.of(1));
const other = () => Principal.fromUint8Array(Uint8Array.of(2));
const identity = () => Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
const current = async () => {};
const refuses = (code: AttemptFailure) => (error: unknown) => error instanceof AttemptError && error.code === code;
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(done => { resolve = done; });
  return { promise, resolve };
}
function request(signer: Identity, destination = target()): Parameters<Identity['transformRequest']>[0] {
  return { endpoint: Endpoint.Call, request: { method: 'POST' }, body: {
    request_type: SubmitRequestType.Call, canister_id: destination, method_name: 'auth',
    arg: new Uint8Array(), sender: signer.getPrincipal(),
    ingress_expiry: Expiry.fromJSON('{"__expiry__":"100000000000"}'),
  } };
}

test('real SDK signing, destination binding and permanent disposal on success', async () => {
  const base = identity();
  let saved!: Identity;
  let signal!: AbortSignal;
  const signed = await runSigningAttempt({ identity: base, target: target(), assertCurrent: current }, async (signer, abort) => {
    saved = signer; signal = abort;
    assert.equal(signer.getPrincipal().toText(), base.getPrincipal().toText());
    await assert.rejects(signer.transformRequest(request(base, other())), refuses('wrong_destination'));
    return signer.transformRequest(request(base));
  });
  assert(signed !== null && typeof signed === 'object' && 'body' in signed);
  const body = signed.body;
  assert(body !== null && typeof body === 'object' && 'sender_sig' in body);
  assert(body.sender_sig instanceof Uint8Array);
  assert.equal(body.sender_sig.length, 64);
  assert.equal(signal.aborted, true);
  await assert.rejects(saved.transformRequest(request(base)), refuses('closed'));
});

test('transport failure propagates unchanged, closes once and never retries', async () => {
  const base = identity();
  const failure = new Error('uncertain transport outcome');
  let calls = 0;
  let saved!: Identity;
  await assert.rejects(runSigningAttempt({ identity: base, target: target(), assertCurrent: current }, async signer => {
    saved = signer; calls++; throw failure;
  }), error => error === failure);
  assert.equal(calls, 1);
  await assert.rejects(saved.transformRequest(request(base)), refuses('closed'));
});

test('invalidation while a real signer is suspended prevents its signature escaping', async () => {
  const base = identity(), entered = deferred<void>(), resume = deferred<void>();
  const invalidation = new Error('generation no longer active');
  let active = true;
  let exposed = false;
  const delayed: Identity = { getPrincipal: () => base.getPrincipal(), transformRequest: async value => {
    entered.resolve(); await resume.promise; return base.transformRequest(value);
  } };
  const attempt = runSigningAttempt({ identity: delayed, target: target(), assertCurrent: async () => { if (!active) throw invalidation; } }, async signer => {
    const result = await signer.transformRequest(request(base)); exposed = true; return result;
  });
  await entered.promise; active = false; resume.resolve();
  await assert.rejects(attempt, error => error === invalidation);
  assert.equal(exposed, false);
});

test('invalidation during transport completion discards its late result', async () => {
  const entered = deferred<void>(), resume = deferred<number>();
  const invalidation = new Error('durable logout committed');
  let active = true;
  const attempt = runSigningAttempt({ identity: identity(), target: target(), assertCurrent: async () => { if (!active) throw invalidation; } }, async () => {
    entered.resolve(); return resume.promise;
  });
  await entered.promise; active = false; resume.resolve(42);
  await assert.rejects(attempt, error => error === invalidation);
});

test('a destination changed during the journal check is refused before signing', async () => {
  const base = identity(), entered = deferred<void>(), resume = deferred<void>();
  let checks = 0, signatures = 0;
  const counted: Identity = { getPrincipal: () => base.getPrincipal(), transformRequest: value => {
    signatures++; return base.transformRequest(value);
  } };
  const value = request(base);
  const attempt = runSigningAttempt({ identity: counted, target: target(), assertCurrent: async () => {
    if (++checks === 2) { entered.resolve(); await resume.promise; }
  } }, signer => signer.transformRequest(value));
  await entered.promise;
  if (value.body.request_type !== 'call') throw new Error('expected SDK call');
  value.body.canister_id = other(); resume.resolve();
  await assert.rejects(attempt, refuses('wrong_destination'));
  assert.equal(signatures, 0);
});

test('source principal and target are captured before the first await', async () => {
  const base = identity(), entered = deferred<void>(), resume = deferred<void>();
  let principal = base.getPrincipal();
  const changing: Identity = { getPrincipal: () => principal, transformRequest: value => base.transformRequest(value) };
  const attempt = runSigningAttempt({ identity: changing, target: target(), assertCurrent: async () => {
    entered.resolve(); await resume.promise;
  } }, async () => assert.fail('changed principal reached dispatch'));
  await entered.promise; principal = other(); resume.resolve();
  await assert.rejects(attempt, refuses('identity_changed'));

  const destination = target();
  const original = destination.toText();
  const pending = runSigningAttempt({ identity: base, target: destination, assertCurrent: current }, async signer => {
    await assert.rejects(signer.transformRequest(request(base, destination)), refuses('wrong_destination'));
    return signer.transformRequest(request(base, Principal.fromText(original)));
  });
  destination.toUint8Array()[0] = 2;
  await pending;
});

test('cancellation stops a suspended signer even if it ignores its signal', async () => {
  const base = identity(), entered = deferred<void>(), resume = deferred<void>(), finished = deferred<void>();
  const abort = new AbortController();
  let exposed = false;
  const delayed: Identity = { getPrincipal: () => base.getPrincipal(), transformRequest: async value => {
    entered.resolve(); await resume.promise; return base.transformRequest(value);
  } };
  const attempt = runSigningAttempt({ identity: delayed, target: target(), assertCurrent: current, signal: abort.signal }, async signer => {
    try { await signer.transformRequest(request(base)); exposed = true; }
    finally { finished.resolve(); }
  });
  await entered.promise; abort.abort();
  await assert.rejects(attempt, refuses('cancelled'));
  resume.resolve(); await finished.promise;
  assert.equal(exposed, false);
});

test('timeout bounds journal/transport work and permanently closes retained signers', async t => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const base = identity(), entered = deferred<void>(), resume = deferred<number>();
  let saved!: Identity;
  const attempt = runSigningAttempt({ identity: base, target: target(), assertCurrent: current }, async signer => {
    saved = signer; entered.resolve(); return resume.promise;
  });
  await entered.promise; t.mock.timers.tick(5_000);
  await assert.rejects(attempt, refuses('timeout'));
  resume.resolve(42);
  await assert.rejects(saved.transformRequest(request(base)), refuses('timeout'));

  const blocked = runSigningAttempt({ identity: base, target: target(), assertCurrent: () => new Promise(() => {}) }, async () => assert.fail('journal did not finish'));
  t.mock.timers.tick(5_000);
  await assert.rejects(blocked, refuses('timeout'));
});

test('invalid bounds and pre-cancellation refuse without dispatch', async () => {
  const base = identity();
  for (const timeoutMs of [0, -1, 0.5, 5_001, Infinity, NaN]) {
    await assert.rejects(runSigningAttempt({ identity: base, target: target(), assertCurrent: current, timeoutMs }, async () => assert.fail()), refuses('invalid_timeout'));
  }
  await assert.rejects(runSigningAttempt({ identity: base, target: target(), assertCurrent: current, signal: AbortSignal.abort() }, async () => assert.fail()), refuses('cancelled'));
});


test('read-state polling requires an update signed by this attempt and a current generation', async () => {
  const base = identity();
  let active = true;
  const invalidation = new Error('logout during polling');
  await runSigningAttempt({ identity: base, target: target(), assertCurrent: async () => {
    if (!active) throw invalidation;
  } }, async signer => {
    const poll: Parameters<Identity['transformRequest']>[0] = {
      endpoint: Endpoint.ReadState, request: { method: 'POST' },
      body: { request_type: ReadRequestType.ReadState, sender: base.getPrincipal(),
        ingress_expiry: Expiry.fromJSON('{"__expiry__":"100000000000"}'), paths: [] },
    };
    await assert.rejects(signer.transformRequest(poll), refuses('wrong_destination'));
    const update = request(base);
    const path = [new TextEncoder().encode('request_status'), requestIdOf({ ...update.body })];
    poll.body.paths = [path];
    await assert.rejects(signer.transformRequest(poll), refuses('wrong_destination'));
    await signer.transformRequest(update);
    poll.body.paths = [path, path];
    await assert.rejects(signer.transformRequest(poll), refuses('wrong_destination'));
    poll.body.paths = [[path[0]!, new Uint8Array(32)]];
    await assert.rejects(signer.transformRequest(poll), refuses('wrong_destination'));
    poll.body.paths = [path];
    await signer.transformRequest(poll);
    const next = request(base);
    assert(next.body.request_type === 'call');
    next.body.method_name = 'next';
    await signer.transformRequest(next);
    await assert.rejects(signer.transformRequest(poll), refuses('wrong_destination'));
    poll.body.paths = [[path[0]!, requestIdOf(next.body)]];
    active = false;
    await assert.rejects(signer.transformRequest(poll), error => error === invalidation);
    // Completion must still fail: a previously signed poll is not live authority.
  }).then(() => assert.fail('invalidated result escaped'), error => assert.equal(error, invalidation));
});
