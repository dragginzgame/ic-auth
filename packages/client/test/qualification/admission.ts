// Private adapter for apps/qualification/sessions.rs, not a production endpoint.
import assert from 'node:assert/strict';
import { IDBFactory } from 'fake-indexeddb';
import type { Identity } from '@icp-sdk/core/agent';
import { httpUpdateTransport } from '../../src/update-http.js';
import * as establish from '../../src/generated/establish.js';
import * as sessionRead from '../../src/generated/read.js';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { Principal } from '@icp-sdk/core/principal';
type SessionResult = establish.Response;
type SessionView = Extract<SessionResult, { ok: unknown }>['ok'];
import { IndexedDbLogoutStore, PortableAdmissionClient, PortableLogoutClient, httpAdmissionTransport, httpLogoutTransport,
  ClientError, type PortableCredential } from '../../src/index.js';

export async function qualifyAdmission(host: string, issuer: Principal, receiver: Principal,
  rootKey: Uint8Array, origin: bigint, credential: PortableCredential, subject: Principal): Promise<void> {
  const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
  const started = performance.now();
  const clock = { id: 'testkit-admission-clock',
    nowNs: () => origin + BigInt(Math.ceil((performance.now() - started) * 1_000_000)),
    wait: (ms: number) => new Promise<void>(resolve => setTimeout(resolve, ms)) };
  const submissions: string[] = [];
  // A new agent uses the current attempt's signing capability. Never fetch trust,
  // retry an update or use a synchronous-endpoint fallback inside an attempt.
  async function update(target: Principal, methodName: string, arg: Uint8Array,
    caller: Identity, signal: AbortSignal): Promise<SessionResult> {
    const call = httpUpdateTransport({ host, methodName, rootKey,
      fetch: async (input, init) => {
        const path = new URL(input instanceof Request ? input.url : input.toString()).pathname;
        if (path.endsWith('/call')) submissions.push(methodName);
        return fetch(input, init);
      } });
    const reply = await call(target, caller, arg, signal);
    assert.ok(reply.length <= 4_096);
    return methodName === 'portable_session_read' ? sessionRead.decodeResponse(reply) : establish.decodeResponse(reply);
  }
  const store = await IndexedDbLogoutStore.open(new IDBFactory(), 'testkit-admission');
  let issued = 0;
  let disposed = 0;
  const admitted = new Map<string, SessionView>();
  const admit = httpAdmissionTransport({ host, methodName: 'establish_portable_session', rootKey,
    scopes: ['read'], fetch: async (input, init) => {
      const path = new URL(input instanceof Request ? input.url : input.toString()).pathname;
      if (path.endsWith('/call')) submissions.push('establish_portable_session');
      return fetch(input, init);
    } });
  const signal = new AbortController().signal;
  try {
    const journal = new PortableLogoutClient(store, clock, httpLogoutTransport({ host,
      methodName: 'logout_portable_session', rootKey,
      dispose: async generation => { assert.equal(generation, 'sdk-admission'); disposed++; } }), () => 0);
    await journal.start('sdk-admission', identity, issuer);
    const client = new PortableAdmissionClient(journal, 'sdk-admission', identity, issuer, {
      issue: async () => {
        issued++;
        assert.equal((await journal.status('sdk-admission')).issuanceUnknown, true);
        return { credential, admissionUntilNs: origin + 60_000_000_000n };
      },
      reconcile: async () => { throw new Error('fixture has no production issuer lookup'); },
      admit: async (target, caller, proof, signal) => {
        assert.ok((await journal.status('sdk-admission')).targets.some(t => t.canister === target.toText()));
        await admit(target, caller, proof, signal);
        const result = await update(target, 'portable_session_read', sessionRead.encodeRequest(null), caller, signal);
        assert.ok('ok' in result, 'err' in result ? result.err : undefined);
        assert.equal(Principal.fromUint8Array(result.ok.caller).toText(), identity.getPrincipal().toText());
        assert.equal(Principal.fromUint8Array(result.ok.issuer).toText(), issuer.toText());
        assert.equal(Principal.fromUint8Array(result.ok.subject).toText(), subject.toText());
        assert.ok(result.ok.expires_at_ns > clock.nowNs());
        const previous = admitted.get(target.toText());
        if (previous) assert.deepEqual(result.ok, previous); // Exact retry cannot extend the lease.
        admitted.set(target.toText(), result.ok);
      },
    });
    const wrong = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10));
    await assert.rejects(admit(receiver, wrong, credential, signal),
      (error: unknown) => error instanceof ClientError && error.code === 'transport');
    const guarded = await client.establish(issuer);
    await client.establish(receiver);
    await client.establish(receiver); // Explicit retry reuses the original credential.
    assert.equal(issued, 1);
    assert.equal(admitted.size, 2);
    for (const target of [issuer, receiver]) {
      const read = await update(target, 'portable_session_read', sessionRead.encodeRequest(null), guarded, signal);
      assert.ok('ok' in read, 'err' in read ? read.err : undefined);
      assert.deepEqual(read.ok, admitted.get(target.toText())); // No proof in application arguments.
    }
    assert.equal(submissions.filter(method => method === 'establish_portable_session').length, 4);
    assert.equal(submissions.filter(method => method === 'portable_session_read').length, 5);
    await journal.begin('sdk-admission', clock.nowNs() + 70_000_000_000n);
    const result = await journal.cleanup('sdk-admission', identity);
    assert.equal(result.phase, 'finished');
    assert.equal(result.issuanceUnknown, true);
    assert.equal(disposed, 1);
    assert.ok(result.targets.every(target => target.attempts === 1 && target.confirmedUntilNs !== undefined));
    const before = submissions.length;
    await assert.rejects(update(receiver, 'portable_session_read', sessionRead.encodeRequest(null), guarded, signal));
    assert.equal(submissions.length, before); // Durable logout refuses before network dispatch.
    for (const target of [issuer, receiver]) {
      // Even retaining the raw test key cannot bypass the receiver's revocation.
      assert.ok('err' in await update(target, 'portable_session_read', sessionRead.encodeRequest(null), identity, signal));
    }
    console.log('SDK admission qualified: one prepared proof; two receivers; caller mismatch rejected; exact retry; proof-free reads; durable logout; retained-key refusal');
  } finally { store.close(); }
}
