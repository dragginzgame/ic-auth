// Private adapter for qualification-fixtures receiver configuration, not a production endpoint.
import assert from 'node:assert/strict';
import { IDBFactory } from 'fake-indexeddb';
import type { Identity } from '@tokostudios/ic-platform/agent';
import { httpUpdateTransport } from '../../src/update-http.js';
import * as sessionRead from '../../src/generated/read.js';
import { Ed25519KeyIdentity } from '@tokostudios/ic-platform/identity';
import { Principal } from '@tokostudios/ic-platform/principal';
type SessionResult = sessionRead.Response;
type SessionView = Extract<SessionResult, { ok: unknown }>['ok'];
import { IndexedDbLogoutStore, PortableAdmissionClient, PortableLogoutClient, httpAdmissionTransport, httpLogoutTransport,
  AdmissionError, credentialBytes, type PortableCredential } from '../../src/index.js';

export async function qualifyAdmission(host: string, issuer: Principal, receiver: Principal,
  rootKey: Uint8Array, origin: bigint, credential: PortableCredential, subject: Principal, admissionUntilNs: bigint,
  peerCredential: PortableCredential): Promise<void> {
  const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
  const account = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(81));
  assert.equal(account.getPrincipal().toText(), subject.toText());
  assert.notEqual(account.getPrincipal().toText(), identity.getPrincipal().toText());
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
    return sessionRead.decodeResponse(reply);
  }
  const factory = new IDBFactory();
  let store = await IndexedDbLogoutStore.open(factory, 'testkit-admission');
  const peerStore = await IndexedDbLogoutStore.open(factory, 'testkit-admission-peer');
  assert.ok(admissionUntilNs > origin);
  const original = { credential, admissionUntilNs }; // Native issuer supplies the immutable original deadline.
  const originalFrame = credentialBytes(credential, 131_072);
  let issued = 0;
  let reconciled = 0;
  let lookupReady = false;
  let disposed = 0;
  let accountProbes = 0;
  const admitted = new Map<string, SessionView>();
  const admit = httpAdmissionTransport({ host, methodName: 'establish_portable_session', rootKey,
    scopes: ['read'], fetch: async (input, init) => {
      const path = new URL(input instanceof Request ? input.url : input.toString()).pathname;
      if (path.endsWith('/call')) submissions.push('establish_portable_session');
      return fetch(input, init);
    } });
  const signal = new AbortController().signal;
  try {
    const reopenJournal = () => new PortableLogoutClient(store, clock, httpLogoutTransport({ host,
      methodName: 'logout_portable_session', rootKey,
      dispose: async generation => { assert.equal(generation, 'sdk-admission'); disposed++; } }), () => 0);
    let journal = reopenJournal();
    await journal.start('sdk-admission', identity, issuer);
    const probeAccount = async (caller: Identity, signal: AbortSignal) => {
      assert.equal(caller.getPrincipal().toText(), account.getPrincipal().toText());
      const result = await update(issuer, 'portable_session_read', sessionRead.encodeRequest(null), caller, signal);
      assert.ok('err' in result, 'account identity has no browser session');
      accountProbes++;
    };
    const clientFor = () => new PortableAdmissionClient(journal, 'sdk-admission', { browser: identity, account }, issuer, {
      issue: async (caller, signal) => {
        await probeAccount(caller, signal);
        issued++;
        assert.equal((await journal.status('sdk-admission')).issuanceUnknown, true);
        throw new Error('injected lost issuer reply');
      },
      // Injected issuer lookup only: this does not qualify a production endpoint.
      reconcile: async (caller, signal) => {
        await probeAccount(caller, signal);
        reconciled++;
        assert.equal(issued, 1);
        if (!lookupReady) throw new Error('original issuer result unavailable');
        return original; // Preserve exact credential and the original deadline.
      },
      admit: async (target, caller, proof, signal) => {
        assert.deepEqual(credentialBytes(proof, 131_072), originalFrame);
        assert.ok((await journal.status('sdk-admission')).targets.some(t => t.canister === target.toText()));
        await admit(target, caller, proof, signal);
        const result = await update(target, 'portable_session_read', sessionRead.encodeRequest(null), caller, signal);
        assert.ok('ok' in result, 'err' in result ? result.err.tag : undefined);
        assert.equal(Principal.fromUint8Array(result.ok.caller).toText(), identity.getPrincipal().toText());
        assert.equal(Principal.fromUint8Array(result.ok.issuer).toText(), issuer.toText());
        assert.equal(Principal.fromUint8Array(result.ok.subject).toText(), subject.toText());
        assert.ok(result.ok.expires_at_ns > clock.nowNs());
        const previous = admitted.get(target.toText());
        if (previous) assert.deepEqual(result.ok, previous); // Exact retry cannot extend the lease.
        admitted.set(target.toText(), result.ok);
      },
    });
    const peerIdentity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(10));
    await assert.rejects(admit(receiver, peerIdentity, credential, signal),
      (error: unknown) => error instanceof AdmissionError && error.refusal === 'InvalidCredential');
    let client = clientFor();
    const beforeRecovery = submissions.length;
    await assert.rejects(client.establish(issuer), /injected lost issuer reply/);
    assert.equal((await journal.status('sdk-admission')).issuanceUnknown, true);
    store.close();
    store = await IndexedDbLogoutStore.open(factory, 'testkit-admission');
    journal = reopenJournal();
    client = clientFor();
    await assert.rejects(client.establish(issuer), /original issuer result unavailable/);
    assert.equal(submissions.length, beforeRecovery + accountProbes);
    assert.equal((await journal.status('sdk-admission')).issuanceUnknown, true);
    assert.equal(issued, 1);
    lookupReady = true;
    const guarded = await client.establish(issuer);
    await client.establish(receiver);
    await client.establish(receiver); // Explicit retry reuses the original credential.
    assert.equal(issued, 1);
    assert.equal(reconciled, 2);
    assert.equal(admitted.size, 2);
    assert.ok((await journal.status('sdk-admission')).targets.every(target => target.admissionUntilNs === admissionUntilNs));
    for (const target of [issuer, receiver]) {
      const read = await update(target, 'portable_session_read', sessionRead.encodeRequest(null), guarded, signal);
      assert.ok('ok' in read, 'err' in read ? read.err.tag : undefined);
      assert.deepEqual(read.ok, admitted.get(target.toText())); // No proof in application arguments.
    }
    await probeAccount(account, signal); // Even after browser admission, the account signer has no session.
    assert.equal(accountProbes, 4);
    assert.equal(submissions.filter(method => method === 'establish_portable_session').length, 4);
    assert.equal(submissions.filter(method => method === 'portable_session_read').length, 5 + accountProbes);
    // A second device authenticates the same account at the issuer but owns its
    // own browser caller, journal and receiver sessions. Issuance is injected.
    let peerIssued = 0;
    let peerDisposed = 0;
    const peerJournal = new PortableLogoutClient(peerStore, clock, httpLogoutTransport({ host,
      methodName: 'logout_portable_session', rootKey,
      dispose: async generation => { assert.equal(generation, 'sdk-admission-peer'); peerDisposed++; } }), () => 0);
    await peerJournal.start('sdk-admission-peer', peerIdentity, issuer);
    const peerClient = new PortableAdmissionClient(peerJournal, 'sdk-admission-peer', { browser: peerIdentity, account }, issuer, {
      issue: async (caller, signal) => {
        await probeAccount(caller, signal);
        peerIssued++;
        return { credential: peerCredential, admissionUntilNs };
      },
      reconcile: async () => { throw new Error('unexpected peer reconciliation'); },
      admit,
    });
    const peerGuarded = await peerClient.establish(issuer);
    await peerClient.establish(receiver);
    assert.equal(peerIssued, 1);
    assert.ok((await peerJournal.status('sdk-admission-peer')).targets.every(target => target.admissionUntilNs === admissionUntilNs));
    const peerSessions = new Map<string, SessionView>();
    for (const target of [issuer, receiver]) {
      const peerRead = await update(target, 'portable_session_read', sessionRead.encodeRequest(null), peerGuarded, signal);
      assert.ok('ok' in peerRead);
      assert.equal(Principal.fromUint8Array(peerRead.ok.caller).toText(), peerIdentity.getPrincipal().toText());
      assert.equal(Principal.fromUint8Array(peerRead.ok.issuer).toText(), issuer.toText());
      assert.equal(Principal.fromUint8Array(peerRead.ok.subject).toText(), subject.toText());
      assert.ok(peerRead.ok.expires_at_ns > clock.nowNs());
      peerSessions.set(target.toText(), peerRead.ok);
      const firstRead = await update(target, 'portable_session_read', sessionRead.encodeRequest(null), guarded, signal);
      assert.ok('ok' in firstRead);
      assert.deepEqual(firstRead.ok, admitted.get(target.toText())); // No device overwrite or lease extension.
    }
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
      const peerRead = await update(target, 'portable_session_read', sessionRead.encodeRequest(null), peerGuarded, signal);
      assert.ok('ok' in peerRead);
      assert.deepEqual(peerRead.ok, peerSessions.get(target.toText())); // First-device logout is isolated.
    }
    await peerClient.establish(receiver); // Exact retry still works for the unaffected device.
    assert.equal(peerIssued, 1);
    await peerJournal.begin('sdk-admission-peer', clock.nowNs() + 70_000_000_000n);
    const peerResult = await peerJournal.cleanup('sdk-admission-peer', peerIdentity);
    assert.equal(peerResult.phase, 'finished');
    assert.equal(peerResult.issuanceUnknown, true);
    assert.equal(peerDisposed, 1);
    assert.ok(peerResult.targets.every(target => target.attempts === 1 && target.confirmedUntilNs !== undefined));
    for (const target of [issuer, receiver]) {
      assert.ok('err' in await update(target, 'portable_session_read', sessionRead.encodeRequest(null), peerIdentity, signal));
      assert.ok('err' in await update(target, 'portable_session_read', sessionRead.encodeRequest(null), identity, signal));
    }
    const afterPeerLogout = submissions.length;
    await assert.rejects(update(receiver, 'portable_session_read', sessionRead.encodeRequest(null), peerGuarded, signal));
    assert.equal(submissions.length, afterPeerLogout);
    console.log('SDK admission qualified: distinct account/browser signers over authenticated reads; injected issuer lost reply; reopened journal; unresolved lookup retained; original proof recovered without reissuance; two receivers and two devices for one account; no session overwrite; independent durable logout; retained-key refusal');
  } finally { peerStore.close(); store.close(); }
}
