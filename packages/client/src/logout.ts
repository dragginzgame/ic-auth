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

/** Public identity of the exact host-owned login key; contains no signing material. */
export interface LoginKeyBinding {
  readonly generation: string;
  readonly caller: string;
}
export interface DisposeLoginKeyOptions {
  journal: LoginJournal;
  generation: string;
  caller: Principal;
  /** Atomically delete only this binding, or succeed if already absent. Must never
   * remove a replacement key. Repeated/concurrent invocation is possible; the host
   * owns custody and idempotency. Inspect retained uncertainty before choosing to delete.
   */
  dispose(binding: LoginKeyBinding, status: JournalStatus): Promise<void>;
  signal?: AbortSignal;
}

/** Dispose through the host only after terminal cleanup. No private key is needed
 * to resume after a lost disposal reply. The journal and uncertainty are retained.
 * Cancellation fences invocation; once invoked, await the host's actual outcome
 * rather than claiming its external deletion was cancelled or rolled back.
 */
export async function disposeLoginKey(options: DisposeLoginKeyOptions): Promise<JournalStatus> {
  const { generation, dispose, signal } = options;
  const caller = options.caller.toText();
  const journal = journalOwner(options.journal);
  if (typeof dispose !== 'function') throw new JournalError('configuration');
  if (signal?.aborted) throw new AttemptError('cancelled');
  const terminal = await journal.disposalStatus(generation, Principal.fromText(caller));
  if (signal?.aborted) throw new AttemptError('cancelled');
  // Callback input is detached from both the journal and our returned evidence.
  await dispose(Object.freeze({ generation, caller }), {
    ...terminal, destinations: terminal.destinations.map(destination => ({ ...destination })),
  });
  return terminal;
}

/** Schedule already committed logout until confirmation or the original deadline.
 * Destination failures retain uncertainty and allow other due destinations to proceed.
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
    const step = await cleanupStep(captured);
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
  const step = await cleanupStep(options);
  if (step.kind === 'failed') throw step.cause;
  return step;
}

async function cleanupStep(options: CleanupOptions): Promise<CleanupStep | { kind: 'failed'; cause: unknown }> {
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
  // Only transport and reply failures are destination-specific. Journal, clock,
  // cancellation and identity failures stop scheduling.
  try {
    const outcome = await runSigningAttempt({ identity: browser, target: Principal.fromText(ticket.canister),
      assertCurrent: current, timeoutMs: ticket.timeoutMs, ...(signal ? { signal } : {}),
    }, async (identity, abort) => {
      let reply: Uint8Array;
      try {
        reply = await revoke(Principal.fromText(ticket.canister), identity, encodeRequest(null), abort);
      } catch (cause) {
        if (cause instanceof JournalError || cause instanceof AttemptError) throw cause;
        return { failed: true, cause } as const;
      }
      try {
        if (!(reply instanceof Uint8Array) || reply.length > maximumResponseBytes) throw new Error('logout reply cap');
        const decoded = decodeResponse(reply);
        if (decoded.tag === 'Revoked' && decoded.value === 0n) throw new Error('invalid acknowledgement');
        return { failed: false, decoded } as const;
      } catch (cause) { return { failed: true, cause: new JournalError('invalid_material', { cause }) } as const; }
    });
    check();
    const response = outcome.failed ? undefined : outcome.decoded;
    const status = await journal.completeCleanup(generation, Principal.fromText(caller), ticket,
      response?.tag === 'Revoked' ? { kind: 'revoked', untilNs: response.value }
        : { kind: 'unconfirmed', retryAfterMs });
    check();
    if (outcome.failed) return { kind: 'failed', cause: outcome.cause };
    return { kind: 'settled', canister: ticket.canister, response: outcome.decoded, status };
  } catch (cause) {
    check();
    // Expired or superseded tickets cannot settle. Reclaim from durable state
    // on the next iteration, where generation, caller and phase are checked again.
    if ((cause instanceof AttemptError && cause.code === 'timeout')
      || (cause instanceof JournalError && cause.code === 'not_current')) return { kind: 'failed', cause };
    throw cause;
  }
}
