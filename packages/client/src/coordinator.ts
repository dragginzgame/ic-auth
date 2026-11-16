import type { Identity } from '@tokostudios/ic-platform/agent';
import { Principal } from '@tokostudios/ic-platform/principal';
import { AdmissionError, admitCredential, type AdmissionReceipt, type EstablishSessionOptions } from './admission.ts';
import { AttemptError, runSigningAttempt } from './attempt.ts';
import { journalOwner, JournalError } from './journal.ts';
import { retrieveCredential, type CredentialEvidence } from './issuance.ts';

export class ReadinessError extends Error {
  readonly code: 'operation_limit' | 'lease_expired';
  constructor(code: 'operation_limit' | 'lease_expired') { super(code); this.name = 'ReadinessError'; this.code = code; }
}
class ExpiredEvidence extends Error {
  readonly sequence: bigint;
  constructor(cause: unknown, sequence: bigint) { super('expired evidence', { cause }); this.sequence = sequence; }
}
export interface SessionCoordinatorOptions extends Omit<EstablishSessionOptions, 'target' | 'refreshThrough'> {
  /** Conservative exclusive refresh deadline in the journal clock, never an inferred offset. */
  refreshAt(receipt: AdmissionReceipt, target: Principal): bigint;
}
type Hint = { receipt: AdmissionReceipt; sequence: bigint; refreshAt: bigint };
type Flight<T> = { refreshThrough: bigint | undefined; promise: Promise<T> };
const detached = (receipt: AdmissionReceipt): AdmissionReceipt => ({ ...receipt,
  caller: new Uint8Array(receipt.caller), issuer: new Uint8Array(receipt.issuer), subject: new Uint8Array(receipt.subject) });
const covers = (flight: bigint | undefined, requested: bigint | undefined) => requested === undefined
  || (flight !== undefined && flight >= requested);

/** Reuse one coordinator per login/tab. Only hints and outstanding promises live here;
 * all issuance, retained evidence, logout and clock decisions stay with the journal.
 */
export class SessionCoordinator {
  readonly #options: SessionCoordinatorOptions;
  readonly #caller: string;
  readonly #account: string | undefined;
  readonly #issuer: string;
  readonly #journal;
  readonly #hints = new Map<string, Hint>();
  readonly #admissions = new Map<string, Flight<Hint>>();
  readonly #readiness = new Map<string, Promise<Hint>>();
  #credential: Promise<CredentialEvidence> | undefined;
  #outstanding = 0;

  constructor(options: SessionCoordinatorOptions) {
    this.#journal = journalOwner(options.journal);
    this.#caller = options.browser.getPrincipal().toText();
    this.#account = options.account?.getPrincipal().toText();
    this.#issuer = options.issuer.toText();
    if (options.issuer.isAnonymous() || !Array.isArray(options.scopes) || typeof options.refreshAt !== 'function')
      throw new JournalError('configuration');
    this.#options = { ...options, issuer: Principal.fromText(this.#issuer), scopes: [...options.scopes],
      transport: { issue: options.transport.issue.bind(options.transport), renew: options.transport.renew.bind(options.transport),
        reconcile: options.transport.reconcile.bind(options.transport) } };
  }

  /** Force one admission; typed expiry is not retried. Concurrent equal requests share it. */
  establish(target: Principal): Promise<AdmissionReceipt> {
    return this.#operation(target, async destination => this.#receipt(await this.#admit(destination), false));
  }

  /** Return a checked, detached scheduling receipt. It is never application authority.
   * Invoke readiness again before a new application operation; this method executes no such operation.
   */
  ensureSession(target: Principal): Promise<AdmissionReceipt> {
    return this.#operation(target, async destination => this.#receipt(await this.#ready(destination), true));
  }

  /** Prepare readiness, then invoke one application operation with a temporary browser
   * signer. Every signature checks the captured deadline and durable logout generation.
   * Expiry or uncertainty after invocation refuses; it never renews or replays the operation.
   */
  call<T>(target: Principal, operation: (identity: Identity, signal: AbortSignal) => Promise<T>): Promise<T> {
    return this.#operation(target, async destination => {
      const hint = await this.#ready(destination);
      return runSigningAttempt({ identity: this.#options.browser, target: Principal.fromText(destination),
        ...(this.#options.signal ? { signal: this.#options.signal } : {}),
        assertCurrent: async () => {
          if (await this.#now() >= hint.refreshAt) throw new ReadinessError('lease_expired');
        },
      }, operation);
    });
  }

  #ready(destination: string): Promise<Hint> {
    let pending = this.#readiness.get(destination);
    if (!pending) {
      pending = this.#prepare(destination).finally(() => { this.#readiness.delete(destination); });
      this.#readiness.set(destination, pending);
    }
    return pending;
  }

  async #receipt(result: Hint, readiness: boolean): Promise<AdmissionReceipt> {
    if (await this.#now() >= result.refreshAt && readiness) throw new ReadinessError('lease_expired');
    return detached(result.receipt);
  }

  async #operation<T>(target: Principal, run: (destination: string) => Promise<T>): Promise<T> {
    if (this.#outstanding === 64) throw new ReadinessError('operation_limit');
    if (target.isAnonymous()) throw new JournalError('configuration');
    const destination = target.toText();
    this.#outstanding++;
    try {
      await this.#now();
      return await run(destination);
    } catch (error) { throw error instanceof ExpiredEvidence ? error.cause : error;
    } finally { this.#outstanding--; }
  }

  #identity() {
    const o = this.#options;
    if (o.signal?.aborted) throw new AttemptError('cancelled');
    if (o.browser.getPrincipal().toText() !== this.#caller || o.account?.getPrincipal().toText() !== this.#account)
      throw new AttemptError('identity_changed');
  }
  async #now(): Promise<bigint> {
    this.#identity();
    const now = await this.#journal.readinessTime(this.#options.generation,
      Principal.fromText(this.#caller), Principal.fromText(this.#issuer));
    this.#identity();
    return now;
  }

  async #evidence(refreshThrough?: bigint): Promise<CredentialEvidence> {
    const active = this.#credential;
    if (active) {
      const result = await active;
      if (refreshThrough === undefined || result.sequence > refreshThrough) return result;
      // A stricter refresh waits for original work; it must never race a new claim against it.
      return this.#evidence(refreshThrough);
    }
    const promise = retrieveCredential({ ...this.#options,
      ...(refreshThrough === undefined ? {} : { refreshThrough }) });
    this.#credential = promise.finally(() => { this.#credential = undefined; });
    return this.#credential;
  }

  async #admit(destination: string, refreshThrough?: bigint): Promise<Hint> {
    const active = this.#admissions.get(destination);
    if (active) {
      const result = await active.promise;
      if (covers(active.refreshThrough, refreshThrough) || (refreshThrough !== undefined && result.sequence > refreshThrough)) return result;
      return this.#admit(destination, refreshThrough);
    }
    const promise = (async () => {
      this.#hints.delete(destination);
      const credential = await this.#evidence(refreshThrough);
      let receipt: AdmissionReceipt;
      try { receipt = await admitCredential({ ...this.#options, target: Principal.fromText(destination) }, credential); }
      catch (error) {
        if ((error instanceof AdmissionError && error.refusal === 'Expired')
          || (error instanceof JournalError && error.code === 'credential_expired'))
          throw new ExpiredEvidence(error, credential.sequence);
        throw error;
      }
      await this.#now();
      const refreshAt = this.#options.refreshAt(detached(receipt), Principal.fromText(destination));
      if (typeof refreshAt !== 'bigint' || refreshAt < 0n || refreshAt > 0xffff_ffff_ffff_ffffn)
        throw new JournalError('configuration');
      const result = { receipt, sequence: credential.sequence, refreshAt };
      this.#hints.set(destination, result);
      return result;
    })().finally(() => { this.#admissions.delete(destination); });
    this.#admissions.set(destination, { refreshThrough, promise });
    return promise;
  }

  async #prepare(destination: string): Promise<Hint> {
    const now = await this.#now();
    const hint = this.#hints.get(destination);
    if (hint && now < hint.refreshAt) return hint;
    let refreshThrough = hint?.sequence;
    for (let attempt = 0; attempt < 2; attempt++) {
      try {
        const result = await this.#admit(destination, refreshThrough);
        if (await this.#now() >= result.refreshAt) throw new ReadinessError('lease_expired');
        return result;
      } catch (error) {
        if (attempt !== 0 || !(error instanceof ExpiredEvidence)) throw error;
        // Only completed evidence can renew. A pending/unknown sequence stays lookup-only
        // in claimCredential, regardless of this scheduling watermark.
        refreshThrough = error.sequence;
      }
    }
    throw new ReadinessError('lease_expired');
  }
}
