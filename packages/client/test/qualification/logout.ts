// Actual SDK update transport against Testkit's live, signed receiver fixture.
// Rust admits the logout-only device; the SDK admits another fresh device.
// Rust independently checks both devices' revocation after this child exits.
import assert from 'node:assert/strict';
import { IDBFactory } from 'fake-indexeddb';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { Principal } from '@icp-sdk/core/principal';
import { Endpoint, SubmitRequestType, Expiry } from '@icp-sdk/core/agent';
import { IndexedDbLogoutStore, PortableLogoutClient, httpLogoutTransport, ClientError, credentialFromBytes } from '../../src/index.js';
import { qualifyAdmission } from './admission.js';

const [host, issuerText, receiverText, callerText, rootHex, nowText, credentialHex, subjectText, admissionUntilText, peerCredentialHex] = process.argv.slice(2);
assert.ok(host && issuerText && receiverText && callerText && rootHex && nowText && credentialHex && subjectText && admissionUntilText && peerCredentialHex);
const issuer = Principal.fromText(issuerText);
const receiver = Principal.fromText(receiverText);
const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(4));
assert.equal(identity.getPrincipal().toText(), callerText);
const origin = BigInt(nowText);
const started = performance.now();
const clock = { id: 'testkit-local-clock',
  nowNs: () => origin + BigInt(Math.ceil((performance.now() - started) * 1_000_000)),
  wait: (ms: number) => new Promise<void>(resolve => setTimeout(resolve, ms)) };
await qualifyAdmission(host, issuer, receiver, Uint8Array.from(Buffer.from(rootHex, 'hex')), origin,
  credentialFromBytes(Uint8Array.from(Buffer.from(credentialHex, 'hex')), 131_072), Principal.fromText(subjectText), BigInt(admissionUntilText),
  credentialFromBytes(Uint8Array.from(Buffer.from(peerCredentialHex, 'hex')), 131_072));
const submissions: string[] = [];
let disposed = 0;
const transport = httpLogoutTransport({ host, methodName: 'logout_portable_session',
  rootKey: Uint8Array.from(Buffer.from(rootHex, 'hex')),
  fetch: async (input, init) => {
    const url = new URL(input instanceof Request ? input.url : input.toString());
    if (url.pathname.endsWith('/call')) submissions.push(url.pathname);
    return fetch(input, init);
  },
  dispose: async generation => { assert.equal(generation, 'testkit-login'); disposed++; },
});
const store = await IndexedDbLogoutStore.open(new IDBFactory(), 'testkit-logout');
try {
  const client = new PortableLogoutClient(store, clock, transport, () => 0);
  await client.start('testkit-login', identity, issuer);
  const until = clock.nowNs() + 60_000_000_000n;
  await client.destination('testkit-login', receiver, until);
  assert.equal(await client.claimIssuance('testkit-login'), true); // Conservatively retain an unknown issuance.
  const guarded = client.guardedIdentity('testkit-login', identity);
  await client.begin('testkit-login', clock.nowNs() + 70_000_000_000n);
  const result = await client.cleanup('testkit-login', identity);
  assert.equal(result.phase, 'finished');
  assert.equal(result.issuanceUnknown, true);
  assert.equal(result.targets.length, 2);
  for (const target of result.targets) {
    assert.equal(target.attempts, 1);
    assert.ok(target.confirmedUntilNs !== undefined && target.confirmedUntilNs > origin);
  }
  assert.deepEqual(submissions.sort(), [issuer, receiver]
    .map(target => `/api/v2/canister/${target.toText()}/call`).sort());
  assert.equal(disposed, 1);
  await assert.rejects(guarded.transformRequest({
    endpoint: Endpoint.Call, request: { method: 'POST' },
    body: { request_type: SubmitRequestType.Call, canister_id: receiver,
      method_name: 'portable_session_read', arg: new Uint8Array(), sender: identity.getPrincipal(),
      ingress_expiry: Expiry.fromJSON('{"__expiry__":"100000000000"}') },
  }), (error: unknown) => error instanceof ClientError && error.code === 'stale_generation');
  console.log('SDK logout qualified: two certified receiver replies; one update each; durable invalidation; unknown issuance retained; signer refused; disposal completed');
} finally { store.close(); }
