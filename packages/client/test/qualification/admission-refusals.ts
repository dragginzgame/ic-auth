// Actual authenticated SDK replies from the private receiver, with no issuer operation.
import assert from 'node:assert/strict';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { Principal } from '@icp-sdk/core/principal';
import { AdmissionError, credentialBytes, credentialFromBytes, httpAdmissionTransport } from '../../src/index.js';
import { httpUpdateTransport } from '../../src/update-http.js';
import * as read from '../../src/generated/read.js';

const [host, targetText, staleText, rootHex, futureHex, expiredHex, invalidHex] = process.argv.slice(2);
assert.ok(host && targetText && staleText && rootHex && futureHex && expiredHex && invalidHex);
const target = Principal.fromText(targetText);
const stale = Principal.fromText(staleText);
const identity = Ed25519KeyIdentity.generate(new Uint8Array(32).fill(71));
const rootKey = Uint8Array.from(Buffer.from(rootHex, 'hex'));
const decode = (hex: string) => credentialFromBytes(Uint8Array.from(Buffer.from(hex, 'hex')), 131_072);
const future = decode(futureHex);
const original = credentialBytes(future, 131_072);
let submissions = 0;
const admit = httpAdmissionTransport({ host, rootKey, methodName: 'establish_portable_session', scopes: ['read'],
  fetch: async (input, init) => {
    if (new URL(input instanceof Request ? input.url : input.toString()).pathname.endsWith('/call')) submissions++;
    return fetch(input, init);
  } });
// Every attempt has an independent bounded lease; the test owns explicit retry.
const attempt = (receiver: Principal, credential: typeof future) => admit(receiver, identity, credential, AbortSignal.timeout(5_000));
await assert.rejects(attempt(target, future), (error: unknown) => error instanceof AdmissionError && error.refusal === 'NotYetValid');
assert.equal(submissions, 1);
const update = httpUpdateTransport({ host, rootKey, methodName: 'portable_session_read' });
const session = async () => read.decodeResponse(await update(target, identity, read.encodeRequest(null), AbortSignal.timeout(5_000)));
assert.ok('err' in await session(), 'future-start refusal must not install a session');
for (const [receiver, credential, refusal] of [
  [target, decode(expiredHex), 'Expired'],
  [target, decode(invalidHex), 'InvalidCredential'],
  [stale, future, 'AuthorityUnavailable'],
] as const) {
  await assert.rejects(attempt(receiver, credential), (error: unknown) => error instanceof AdmissionError && error.refusal === refusal);
}
assert.equal(submissions, 4, 'typed refusals do not trigger transport retries');
const deadline = performance.now() + 15_000;
for (;;) {
  try { await attempt(target, future); break; }
  catch (error) {
    if (!(error instanceof AdmissionError) || error.refusal !== 'NotYetValid' || performance.now() >= deadline) throw error;
    await new Promise(resolve => setTimeout(resolve, 250));
  }
  assert.deepEqual(credentialBytes(future, 131_072), original);
}
const established = await session();
assert.ok('ok' in established);
await attempt(target, future);
assert.deepEqual(await session(), established, 'same-credential retry must retain the original session lease');
assert.deepEqual(credentialBytes(future, 131_072), original);
console.log('SDK typed admission qualified: future-start refusal without session; expired/signature/authority refusals remain distinct; identical credential retried explicitly; original lease retained');
