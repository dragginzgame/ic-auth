// Private adapter for apps/qualification/sessions.rs, not a production endpoint.
import assert from 'node:assert/strict';
import { setTimeout as delay } from 'node:timers/promises';
import { IDBFactory } from 'fake-indexeddb';
import { HttpAgent, type Identity } from '@icp-sdk/core/agent';
import { IDL } from '@icp-sdk/core/candid';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import type { Principal } from '@icp-sdk/core/principal';
import { PortableCredential as CredentialType } from '../../src/generated/protocol.did.js';
import { IndexedDbLogoutStore, PortableAdmissionClient, PortableLogoutClient, httpLogoutTransport,
  type PortableCredential } from '../../src/index.js';

type SessionView = { caller: Principal; issuer: Principal; subject: Principal; expires_at_ns: bigint };
const View = IDL.Record({ caller: IDL.Principal, issuer: IDL.Principal, subject: IDL.Principal, expires_at_ns: IDL.Nat64 });
const Result = IDL.Variant({ Ok: View, Err: IDL.Text });
const Input = IDL.Record({ credential: CredentialType, scopes: IDL.Vec(IDL.Text), ttl_secs: IDL.Opt(IDL.Nat64) });

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
    caller: Identity, signal: AbortSignal): Promise<{ Ok?: SessionView; Err?: string }> {
    signal.throwIfAborted();
    let submitted = false;
    const agent = await HttpAgent.create({ host, identity: caller, rootKey: rootKey.slice(),
      retryTimes: 0, shouldFetchRootKey: false, shouldSyncTime: false,
      fetch: async (input, init) => {
        signal.throwIfAborted();
        const path = new URL(input instanceof Request ? input.url : input.toString()).pathname;
        if (path === `/api/v2/canister/${target}/call`) {
          assert.equal(submitted, false); submitted = true; submissions.push(methodName);
        } else assert.equal(path, `/api/v3/canister/${target}/read_state`);
        const response = await fetch(input, { ...init, signal });
        signal.throwIfAborted();
        return response;
      },
    });
    const result = await agent.update(target, { methodName, arg, callSync: false },
      { preSignReadStateRequest: false, strategy: async () => {
        signal.throwIfAborted();
        await delay(250, undefined, { signal });
      } });
    signal.throwIfAborted();
    assert.ok(result.reply.length <= 4_096);
    return IDL.decode([Result], result.reply)[0] as { Ok?: SessionView; Err?: string };
  }
  const store = await IndexedDbLogoutStore.open(new IDBFactory(), 'testkit-admission');
  let issued = 0;
  let disposed = 0;
  const admitted = new Map<string, SessionView>();
  const request = (value: PortableCredential) => IDL.encode([Input], [{ credential: value, scopes: ['read'], ttl_secs: [] }]);
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
        const result = await update(target, 'establish_portable_session', request(proof), caller, signal);
        assert.ok(result.Ok, result.Err);
        assert.equal(result.Ok.caller.toText(), identity.getPrincipal().toText());
        assert.equal(result.Ok.issuer.toText(), issuer.toText());
        assert.equal(result.Ok.subject.toText(), subject.toText());
        assert.ok(result.Ok.expires_at_ns > clock.nowNs());
        const previous = admitted.get(target.toText());
        if (previous) assert.deepEqual(result.Ok, previous); // Exact retry cannot extend the lease.
        admitted.set(target.toText(), result.Ok);
      },
    });
    const wrong = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10));
    assert.ok((await update(receiver, 'establish_portable_session', request(credential), wrong, signal)).Err);
    const guarded = await client.establish(issuer);
    await client.establish(receiver);
    await client.establish(receiver); // Explicit retry reuses the original credential.
    assert.equal(issued, 1);
    assert.equal(admitted.size, 2);
    for (const target of [issuer, receiver]) {
      const read = await update(target, 'portable_session_read', IDL.encode([], []), guarded, signal);
      assert.deepEqual(read.Ok, admitted.get(target.toText())); // No proof in application arguments.
    }
    assert.equal(submissions.filter(method => method === 'establish_portable_session').length, 4);
    assert.equal(submissions.filter(method => method === 'portable_session_read').length, 2);
    await journal.begin('sdk-admission', clock.nowNs() + 70_000_000_000n);
    const result = await journal.cleanup('sdk-admission', identity);
    assert.equal(result.phase, 'finished');
    assert.equal(result.issuanceUnknown, true);
    assert.equal(disposed, 1);
    assert.ok(result.targets.every(target => target.attempts === 1 && target.confirmedUntilNs !== undefined));
    const before = submissions.length;
    await assert.rejects(update(receiver, 'portable_session_read', IDL.encode([], []), guarded, signal));
    assert.equal(submissions.length, before); // Durable logout refuses before network dispatch.
    for (const target of [issuer, receiver]) {
      // Even retaining the raw test key cannot bypass the receiver's revocation.
      assert.ok((await update(target, 'portable_session_read', IDL.encode([], []), identity, signal)).Err);
    }
    console.log('SDK admission qualified: one prepared proof; two receivers; caller mismatch rejected; exact retry; proof-free reads; durable logout; retained-key refusal');
  } finally { store.close(); }
}
