import { LoginJournal as PublicJournal, retrieveCredential, establishSession, cleanupOnce, httpUpdateTransport, AdmissionError } from '../../src/index.ts';
import { Journal as LoginJournal, type CleanupAttempt } from '../../src/journal.ts';
import { runSigningAttempt } from '../../src/attempt.ts';
import { Principal } from '@tokostudios/ic-platform/principal';
import { Ed25519KeyIdentity } from '@tokostudios/ic-platform/identity';
import { Endpoint, SubmitRequestType, Expiry, IC_ROOT_KEY, inspectIngress, type Identity } from '@tokostudios/ic-platform/agent';
import { invalidCertificateFixture, readStateResponseFixture } from '@tokostudios/ic-platform/testing';

// Real SDK signing paused at an explicit boundary while a second tab commits logout.
async function pauseSigning(journal: LoginJournal, ticket?: CleanupAttempt) {
  const base = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(9));
  const target = Principal.fromUint8Array(Uint8Array.of(1));
  let resume!: () => void, entered!: () => void;
  const gate = new Promise<void>(resolve => { resume = resolve; });
  const ready = new Promise<void>(resolve => { entered = resolve; });
  let escaped = false;
  const delayed: Identity = {
    getPrincipal: () => base.getPrincipal(),
    transformRequest: async request => { entered(); await gate; return base.transformRequest(request); },
  };
  const result = runSigningAttempt({ identity: delayed, target,
    assertCurrent: () => ticket
      ? journal.assertCleanup('login', base.getPrincipal(), ticket)
      : journal.assertActive('login', base.getPrincipal(), target),
  }, async signer => {
    await signer.transformRequest({ endpoint: Endpoint.Call, request: { method: 'POST' }, body: {
      request_type: SubmitRequestType.Call, canister_id: target, method_name: 'auth',
      arg: new Uint8Array(), sender: base.getPrincipal(),
      ingress_expiry: Expiry.fromJSON('{"__expiry__":"100000000000"}'),
    } });
    escaped = true;
  }).then(() => ({ code: 'accepted', escaped }), (error: unknown) => ({
    code: error instanceof Error && 'code' in error ? String(error.code) : String(error), escaped,
  }));
  await Promise.race([ready, result.then(() => { throw new Error('attempt ended before signing'); })]);
  return { resume, result };
}
async function signCall(identity: Identity, target = Principal.fromUint8Array(Uint8Array.of(1)), arg = new Uint8Array(), methodName = 'issuer_fixture') {
  return identity.transformRequest({ endpoint: Endpoint.Call, request: { method: 'POST' }, body: {
    request_type: SubmitRequestType.Call, canister_id: target, method_name: methodName,
    arg, sender: identity.getPrincipal(),
    ingress_expiry: Expiry.fromJSON('{"__expiry__":"100000000000"}'),
  } });
}
// Independent fixture packing follows the retained establish-success frame, not the encoder.
function admissionReply(caller: Principal, issuer: Principal, expires = 1n): Uint8Array {
  const bytes = [0x49, 0x57, 0x12, 0, 0x48, 0x54, 0x55, 0x41, 0, 0, 1, 0];
  for (const principal of [caller, issuer, Principal.fromUint8Array(Uint8Array.of(4))]) {
    const raw = principal.toUint8Array(); bytes.push(raw.length, ...raw);
  }
  do { const next = Number(expires & 127n); expires >>= 7n; bytes.push(next | (expires ? 128 : 0)); } while (expires);
  return Uint8Array.from(bytes);
}
Object.assign(globalThis, { authTest: { LoginJournal, PublicJournal, Principal, Ed25519KeyIdentity, pauseSigning, retrieveCredential, establishSession,
  cleanupOnce, runSigningAttempt, httpUpdateTransport, IC_ROOT_KEY, inspectIngress, invalidCertificateFixture, readStateResponseFixture,
  AdmissionError, signCall, admissionReply } });
