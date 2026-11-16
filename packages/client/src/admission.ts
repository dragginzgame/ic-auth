import type { Identity } from '@tokostudios/ic-platform/agent';
import { Principal } from '@tokostudios/ic-platform/principal';
import { AttemptError, runSigningAttempt } from './attempt.ts';
import { JournalError, journalOwner } from './journal.ts';
import { retrieveCredential, type CredentialRequestOptions } from './issuance.ts';
import { decodeRequest } from './generated/credential.js';
import { encodeRequest, decodeResponse, type Response } from './generated/establish.js';

/** Detached receiver-clock metadata, never reusable session authority. */
export type AdmissionReceipt = Extract<Response, { ok: unknown }>['ok'];
export type AdmissionRefusal = Extract<Response, { err: unknown }>['err']['tag'];
export class AdmissionError extends Error {
  readonly code = 'admission_refused';
  readonly refusal: AdmissionRefusal;
  constructor(refusal: AdmissionRefusal) {
    super(`Session admission refused: ${refusal}`);
    this.name = 'AdmissionError';
    this.refusal = refusal;
  }
}
/** Host owns endpoint wiring and authenticating the update reply for this target/request.
 * Use the supplied temporary SDK identity; HTTP acceptance alone is not a reply.
 */
export type AdmissionTransport = (target: Principal, identity: Identity, request: Uint8Array,
  signal: AbortSignal) => Promise<Uint8Array>;
export interface EstablishSessionOptions extends CredentialRequestOptions {
  target: Principal;
  scopes: readonly string[];
  ttlSecs?: bigint;
  admit: AdmissionTransport;
}

/** Explicit admission: acquire exact evidence, inventory the receiver, call once. */
export async function establishSession(options: EstablishSessionOptions): Promise<AdmissionReceipt> {
  const { generation, browser, issuer, target, scopes, ttlSecs, admit, signal } = options;
  const journal = journalOwner(options.journal);
  const caller = browser.getPrincipal().toText(), issuerText = issuer.toText(), destination = target.toText();
  if (target.isAnonymous() || !Array.isArray(scopes)) throw new JournalError('configuration');
  const requestedScopes = [...scopes];
  const credential = await retrieveCredential({ ...options, issuer: Principal.fromText(issuerText) });
  let request: Uint8Array;
  try {
    request = encodeRequest({ credential: decodeRequest(credential.bytes), scopes: requestedScopes,
      ttl_secs: ttlSecs === undefined ? null : { some: ttlSecs } });
  } catch (cause) { throw new JournalError('invalid_material', { cause }); }
  // Strict commit precedes any receiver dispatch, even when its reply is lost.
  await journal.addDestination(generation, Principal.fromText(destination), credential.untilNs);
  const current = async () => {
    const check = () => {
      if (signal?.aborted) throw new AttemptError('cancelled');
      if (browser.getPrincipal().toText() !== caller) throw new AttemptError('identity_changed');
    };
    check();
    await journal.assertAdmission(generation, Principal.fromText(caller), Principal.fromText(issuerText),
      Principal.fromText(destination), credential.sequence);
    check();
  };
  return runSigningAttempt({ identity: browser, target: Principal.fromText(destination),
    assertCurrent: current, ...(signal ? { signal } : {}),
  }, async (identity, abort) => {
    const reply = await admit(Principal.fromText(destination), identity, request, abort);
    let result: Response;
    try {
      // Supplier decoding checks frame bounds before allocation and detaches byte fields.
      result = decodeResponse(reply);
      if ('ok' in result && (Principal.fromUint8Array(result.ok.caller).toText() !== caller
        || Principal.fromUint8Array(result.ok.issuer).toText() !== issuerText || result.ok.expires_at_ns === 0n))
        throw new Error('admission binding or expiry mismatch');
    } catch (cause) { throw new JournalError('invalid_material', { cause }); }
    if ('err' in result) throw new AdmissionError(result.err.tag);
    // Already detached before the attempt's asynchronous postflight guard.
    return result.ok;
  });
}
