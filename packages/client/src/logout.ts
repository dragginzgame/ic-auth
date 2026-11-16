import type { Identity } from '@tokostudios/ic-platform/agent';
import { Principal } from '@tokostudios/ic-platform/principal';
import { AttemptError, runSigningAttempt } from './attempt.ts';
import { JournalError, journalOwner, type LoginJournal, type CleanupClaim, type JournalStatus } from './journal.ts';
import { encodeRequest, decodeResponse, maximumResponseBytes, type Response } from './generated/logout.js';

/** The host owns endpoint wiring and authentication of this exact update reply.
 * Use the temporary browser identity; HTTP acceptance is not an acknowledgement.
 */
export type LogoutTransport = (target: Principal, identity: Identity, request: Uint8Array,
  signal: AbortSignal) => Promise<Uint8Array>;
export interface CleanupOptions {
  journal: LoginJournal;
  generation: string;
  browser: Identity;
  revoke: LogoutTransport;
  signal?: AbortSignal;
  /** Local scheduling after a typed nonconfirmation, 500–1,499 ms; defaults to 1,000. */
  retryAfterMs?: number;
}
export type CleanupStep = Exclude<CleanupClaim, { kind: 'attempt' }>
  | { readonly kind: 'settled'; readonly canister: string; readonly response: Response; readonly status: JournalStatus };

/** Schedule already committed logout until confirmation or the original deadline.
 * Errors stop this run without erasing uncertainty; invoke again to resume retained
 * leases/counts. Completion is not proof that every destination acknowledged logout.
 */
export async function cleanupUntilFinished(options: CleanupOptions): Promise<JournalStatus> {
  const captured = { ...options };
  const caller = captured.browser.getPrincipal().toText();
  const check = () => {
    if (captured.signal?.aborted) throw new AttemptError('cancelled');
    if (captured.browser.getPrincipal().toText() !== caller) throw new AttemptError('identity_changed');
  };
  for (;;) {
    check();
    const step = await cleanupOnce(captured);
    check();
    if (step.kind === 'finished') return step.status;
    if (step.kind === 'wait') await waitForCleanup(step.delayMs, captured.signal);
    // A timer is only a wake-up hint. The next journal claim checks time, deadline,
    // attempt limits and other tabs' progress before another effect is possible.
  }
}

function waitForCleanup(delayMs: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const finish = () => { signal?.removeEventListener('abort', abort); resolve(); };
    const abort = () => {
      clearTimeout(timer);
      signal?.removeEventListener('abort', abort);
      reject(new AttemptError('cancelled'));
    };
    const timer = setTimeout(finish, delayMs);
    signal?.addEventListener('abort', abort, { once: true });
    if (signal?.aborted) abort();
  });
}

/** Resume already committed logout with at most one effect. No loop or implicit local logout. */
export async function cleanupOnce(options: CleanupOptions): Promise<CleanupStep> {
  const { generation, browser, revoke, signal, retryAfterMs = 1_000 } = options;
  const journal = journalOwner(options.journal);
  const caller = browser.getPrincipal().toText();
  if (!Number.isInteger(retryAfterMs) || retryAfterMs < 500 || retryAfterMs >= 1_500)
    throw new JournalError('configuration');
  const check = () => {
    if (signal?.aborted) throw new AttemptError('cancelled');
    if (browser.getPrincipal().toText() !== caller) throw new AttemptError('identity_changed');
  };
  check();
  const claim = await journal.claimCleanup(generation, Principal.fromText(caller));
  check();
  if (claim.kind !== 'attempt') return claim;
  const { ticket } = claim;
  const current = async () => {
    check();
    await journal.assertCleanup(generation, Principal.fromText(caller), ticket);
    check();
  };
  // Any interruption leaves the committed count/lease in place. Only a complete,
  // authenticated typed response reaches settlement; a later caller resumes explicitly.
  const response = await runSigningAttempt({ identity: browser, target: Principal.fromText(ticket.canister),
    assertCurrent: current, timeoutMs: ticket.timeoutMs, ...(signal ? { signal } : {}),
  }, async (identity, abort) => {
    const reply = await revoke(Principal.fromText(ticket.canister), identity, encodeRequest(null), abort);
    try {
      if (!(reply instanceof Uint8Array) || reply.length > maximumResponseBytes) throw new Error('logout reply cap');
      const decoded = decodeResponse(reply);
      if (decoded.tag === 'Revoked' && decoded.value === 0n) throw new Error('invalid acknowledgement');
      // Decoded tags/u64 values are detached before the asynchronous postflight guard.
      return decoded;
    } catch (cause) { throw new JournalError('invalid_material', { cause }); }
  });
  check();
  const status = await journal.completeCleanup(generation, Principal.fromText(caller), ticket,
    response.tag === 'Revoked' ? { kind: 'revoked', untilNs: response.value }
      : { kind: 'unconfirmed', retryAfterMs });
  check();
  return { kind: 'settled', canister: ticket.canister, response, status };
}
