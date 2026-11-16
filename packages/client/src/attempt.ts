/** A temporary SDK signing capability for one journal-guarded operation. */
import type { Identity } from '@tokostudios/ic-platform/agent';
import { Principal } from '@tokostudios/ic-platform/principal';

type SigningRequest = Parameters<Identity['transformRequest']>[0];
export type AttemptFailure = 'invalid_timeout' | 'closed' | 'cancelled' | 'timeout'
  | 'wrong_destination' | 'identity_changed';

/** Local capability refusal; this never establishes whether a remote effect happened. */
export class AttemptError extends Error {
  readonly code: AttemptFailure;
  constructor(code: AttemptFailure) {
    super(`Signing attempt refused: ${code}`);
    this.name = 'AttemptError';
    this.code = code;
  }
}

export interface SigningAttemptOptions {
  /** Existing Platform SDK identity: account for initial issue, browser for receiver/renewal. */
  identity: Identity;
  /** Protected destination, captured before asynchronous work. */
  target: Principal;
  /** Must recheck the owning journal's current generation/lease; rejection propagates. */
  assertCurrent: () => Promise<void>;
  /** Local cancellation, for example after durable logout invalidation commits. */
  signal?: AbortSignal;
  /** Positive whole milliseconds, at most the reference's five-second ceiling. */
  timeoutMs?: number;
}

/**
 * Run exactly one operation with a temporary SDK identity. No automatic retry,
 * key custody, authority creation, remote acknowledgement or application replay.
 * The signer closes on every exit. Signatures already returned cannot be recalled.
 */
export async function runSigningAttempt<T>(
  options: SigningAttemptOptions,
  operation: (identity: Identity, signal: AbortSignal) => Promise<T>,
): Promise<T> {
  const { identity, target, assertCurrent, signal: external, timeoutMs = 5_000 } = options;
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0 || timeoutMs > 5_000) {
    throw new AttemptError('invalid_timeout');
  }
  const destination = target.toText();
  const caller = identity.getPrincipal().toText();
  const controller = new AbortController();
  let closed = false;
  let failure: AttemptError | undefined;
  let rejectCancellation!: (error: AttemptError) => void;
  const cancellation = new Promise<never>((_, reject) => { rejectCancellation = reject; });
  const stop = (code: 'cancelled' | 'timeout') => {
    if (closed) return;
    closed = true;
    failure = new AttemptError(code);
    controller.abort();
    rejectCancellation(failure);
  };
  const abort = () => stop('cancelled');
  const checkIdentity = () => {
    if (closed) throw failure ?? new AttemptError('closed');
    if (identity.getPrincipal().toText() !== caller) throw new AttemptError('identity_changed');
  };
  const checkDestination = (request: SigningRequest) => {
    // Polling an already submitted request has no canister_id. It still requires
    // the same live generation, principal and individual attempt capability.
    if (request.body.request_type === 'read_state') return;
    const requested = request.body.canister_id;
    if (!(requested instanceof Principal) || requested.toText() !== destination) {
      throw new AttemptError('wrong_destination');
    }
  };
  const check = async (request?: SigningRequest) => {
    checkIdentity();
    if (request) checkDestination(request);
    await assertCurrent();
    // Inputs and authority can change while a journal read is suspended.
    checkIdentity();
    if (request) checkDestination(request);
  };
  const scoped: Identity = {
    getPrincipal: () => { checkIdentity(); return Principal.fromText(caller); },
    transformRequest: async request => {
      await check(request);
      const signed = await identity.transformRequest(request);
      await check(request);
      return signed;
    },
  };
  const timer = setTimeout(() => stop('timeout'), timeoutMs);
  external?.addEventListener('abort', abort, { once: true });
  if (external?.aborted) abort();
  try {
    return await Promise.race([
      cancellation,
      (async () => {
        await check();
        const result = await operation(scoped, controller.signal);
        await check();
        return result;
      })(),
    ]);
  } finally {
    closed = true;
    clearTimeout(timer);
    external?.removeEventListener('abort', abort);
    controller.abort();
  }
}
