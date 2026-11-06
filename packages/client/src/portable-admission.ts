import { boundedCall, MAX_NETWORK_CALL_MS } from './bounded-call.js';
import { checkedIdentity, checkDestination } from './checked-identity.js';
import type { Identity } from '@tokostudios/ic-platform/agent';
import { Principal } from '@tokostudios/ic-platform/principal';
import { AdmissionError, ClientError } from './contracts.js';
import { credentialBytes, credentialFromBytes } from './codec.js';
import { defaultMaximumBytes, maximumConfiguredBytes } from './generated/credential.js';
import type { PortableCredential } from './codec.js';
import type { PortableLogoutClient } from './logout.js';
import { MAX_TARGETS } from './logout-store.js';
import { isU64 } from './u64.js';
import type { Response as EstablishResponse } from './generated/establish.js';

/** Authenticated receiver reply, not reusable authorization. Times are in that
 * receiver's clock domain; scheduling must use the host's admitted mapping. */
export type AdmissionReceipt = Extract<EstablishResponse, { ok: unknown }>['ok'];

/** Host-owned conservative refresh time in the journal clock domain. The
 * receipt is detached; the mapping is scheduling policy, never authority. */
export type SessionRefreshTime = (receipt: AdmissionReceipt) => bigint;

interface CredentialState { bytes: Uint8Array; until: bigint; sequence: bigint }

export interface AdmissionCredential {
  credential: PortableCredential;
  /** Qualified adapter maps the original proof expiry conservatively to the journal clock.
   * Recovery MUST retain this deadline, never extend it from the time of retrieval.
   */
  admissionUntilNs: bigint;
}

export interface PortableAdmissionTransport {
  /** The signer is the account identity, not the receiver's browser identity.
   * Bound to one issuer, account caller, browser login and immutable requested grant set. Persist exact
   * intent before dispatch; a lost reply must remain recoverable by reconcile.
   */
  issue(identity: Identity, signal: AbortSignal): Promise<AdmissionCredential>;
  /** Lookup only: never issue, renew or interpret missing/pruned evidence as no effect. */
  reconcile(identity: Identity, sequence: bigint, signal: AbortSignal): Promise<AdmissionCredential>;
  /** Browser-key authenticated renewal of the original immutable login window.
   * Persist this sequence's exact result at the issuer; never create a new login.
   */
  renew(identity: Identity, sequence: bigint, signal: AbortSignal): Promise<AdmissionCredential>;
  /** Authenticate the update reply and resolve only on successful session admission.
   * Receivers perform complete proof, authority, replay and scope checks.
   */
  admit(target: Principal, identity: Identity, credential: PortableCredential, signal: AbortSignal): Promise<AdmissionReceipt>;
}

/** Separate account authentication at the issuer from fresh receiver ingress. */
export interface PortableAdmissionIdentities {
  browser: Identity;
  account?: Identity;
}

/** One portable proof across receivers for an already started browser login.
 * Expired completed proofs renew via the browser binding. No application-call retry.
 */
export class PortableAdmissionClient {
  private credentialFlight: { refreshThrough: bigint | undefined; promise: Promise<CredentialState> } | undefined;
  private readonly admissions = new Map<string, { sequence: bigint; promise: Promise<Identity> }>();
  private readonly receipts = new Map<string, { sequence: bigint; receipt: AdmissionReceipt }>();
  private pending = 0;
  private readonly guarded: Identity;
  private readonly caller: string;
  private readonly accountCaller: string | undefined;
  private readonly issuer: string;
  private readonly identities: PortableAdmissionIdentities;

  constructor(private readonly journal: PortableLogoutClient, private readonly generation: string,
    identities: PortableAdmissionIdentities, issuer: Principal,
    private readonly transport: PortableAdmissionTransport,
    private readonly timeoutMs = MAX_NETWORK_CALL_MS, private readonly maximumBytes = defaultMaximumBytes) {
    if (issuer.isAnonymous() || identities.account?.getPrincipal().isAnonymous()
      || !Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > MAX_NETWORK_CALL_MS
      || !Number.isSafeInteger(maximumBytes) || maximumBytes < 1 || maximumBytes > maximumConfiguredBytes) throw new ClientError('configuration');
    this.issuer = issuer.toText();
    this.identities = { browser: identities.browser, ...(identities.account ? { account: identities.account } : {}) };
    this.caller = identities.browser.getPrincipal().toText();
    this.accountCaller = identities.account?.getPrincipal().toText();
    this.guarded = journal.guardedIdentity(generation, identities.browser);
  }

  private async active(): Promise<bigint> {
    const state = await this.journal.admissionStatus(this.generation);
    if (state.phase !== 'active' || state.caller !== this.caller || state.issuer !== this.issuer) {
      throw new ClientError('stale_generation');
    }
    return state.observedNs;
  }

  private async call<T>(signer: Identity, caller: string, target: string,
    operation: (signal: AbortSignal, identity: Identity) => Promise<T>): Promise<T> {
    await this.active();
    return boundedCall(this.timeoutMs, async signal => {
      const check = async () => {
        if (signal.aborted) throw new ClientError('timeout');
        await this.active();
        if (signal.aborted) throw new ClientError('timeout');
        if (signer.getPrincipal().toText() !== caller) throw new ClientError('stale_generation');
      };
      const identity = checkedIdentity(signer, () => Principal.fromText(caller), async request => {
        await check();
        checkDestination(request, destination => destination === target);
      }, check);
      await check();
      const result = await operation(signal, identity);
      await check();
      return result;
    });
  }

  private async credential(refreshThrough?: bigint): Promise<CredentialState> {
    await this.active();
    if (this.credentialFlight) {
      const prior = this.credentialFlight;
      const value = await prior.promise;
      if (refreshThrough === undefined || value.sequence > refreshThrough) return value;
      // A shared flight can complete the old proof needed by another receiver.
      // Resolve it before asking the durable owner for a later sequence.
      const next = this.credentialFlight;
      if (next && next !== prior && next.refreshThrough !== undefined && next.refreshThrough >= refreshThrough) return next.promise;
    }
    const flight = (async () => {
      const claim = await this.journal.claimCredential(this.generation, this.identities.account !== undefined, refreshThrough);
      if (claim.saved) return { ...claim.saved, sequence: claim.sequence };
      const initial = claim.sequence === 0n;
      const signer = initial ? this.identities.account : this.identities.browser;
      const caller = initial ? this.accountCaller : this.caller;
      if (!signer || !caller) throw new ClientError('account_required');
      const result = await this.call(signer, caller, this.issuer, (signal, identity) => !claim.first
        ? this.transport.reconcile(identity, claim.sequence, signal)
        : initial ? this.transport.issue(identity, signal) : this.transport.renew(identity, claim.sequence, signal));
      // Copy returned bytes before another asynchronous boundary. Transport decoding
      // is not authentication; the receiver remains the verifier.
      const until = result.admissionUntilNs;
      const bytes = credentialBytes(result.credential, this.maximumBytes);
      await this.active();
      if (!isU64(until) || until === 0n) throw new ClientError('retrieval_expired');
      await this.journal.completeCredential(this.generation, claim.sequence, bytes, until);
      return { bytes, until, sequence: claim.sequence };
    })();
    const owned = { refreshThrough, promise: flight };
    this.credentialFlight = owned;
    try { return await flight; }
    finally { if (this.credentialFlight === owned) this.credentialFlight = undefined; }
  }

  /** Explicit first-contact/re-admission operation. Returned identity also checks
   * durable logout state at signing; ordinary application calls use receiver guards.
   */
  async establish(target: Principal): Promise<Identity> {
    return this.prepareSession(target);
  }

  /** Prepare before dispatching an application operation or after browser resume.
   * A live lease hint skips proof work. The returned receiver-bound identity
   * repeats readiness before signing each new call/query, including after suspend.
   * The host supplies its admitted receiver-to-journal clock mapping. No timer,
   * application callback or retry of a submitted operation is installed.
   */
  async ensureSession(target: Principal, refreshTime: SessionRefreshTime): Promise<Identity> {
    if (typeof refreshTime !== 'function') throw new ClientError('configuration');
    const key = target.toText();
    await this.prepareSession(Principal.fromText(key), refreshTime);
    const check = (request: Parameters<Identity['transformRequest']>[0]) =>
      checkDestination(request, destination => destination === key);
    return checkedIdentity(this.guarded, () => Principal.fromText(this.caller), async request => {
      check(request);
      // Polling an already submitted call must not trigger a new issuance or
      // admission. The underlying identity still enforces durable logout.
      if (request.body.request_type !== 'read_state') await this.prepareSession(Principal.fromText(key), refreshTime);
      check(request);
    }, async request => { check(request); });
  }

  private async prepareSession(target: Principal, refreshTime?: SessionRefreshTime): Promise<Identity> {
    // Bound callers before even opening a journal transaction. Destination
    // deduplication happens later and cannot bound queued issuance/status work.
    if (this.pending >= MAX_TARGETS) throw new ClientError('storage_capacity');
    this.pending++;
    try {
      if (target.isAnonymous()) throw new ClientError('configuration');
      const key = target.toText();
      let refreshThrough: bigint | undefined;
      if (refreshTime && this.receipts.has(key)) {
        const before = await this.active();
        const previous = this.receipts.get(key)!;
        if (this.refreshAt(previous.receipt, refreshTime) > before) return this.guarded;
        refreshThrough = previous.sequence;
      }
      const identity = await this.establishTarget(Principal.fromText(key), refreshThrough, refreshTime !== undefined);
      if (refreshTime) {
        const receipt = this.receipts.get(key)!.receipt;
        const until = this.refreshAt(receipt, refreshTime);
        if (until <= await this.active()) throw new ClientError('lease_expired');
      }
      return identity;
    }
    finally { this.pending--; }
  }

  private refreshAt(receipt: AdmissionReceipt, mapping: SessionRefreshTime): bigint {
    const until = mapping(structuredClone(receipt));
    if (!isU64(until)) throw new ClientError('configuration');
    return until;
  }

  /** Detached scheduling metadata from this coordinator's last admission.
   * Reopening has no retained session authority: establish again before use.
   */
  async sessionReceipt(target: Principal): Promise<AdmissionReceipt | undefined> {
    const key = target.toText();
    await this.active();
    const value = this.receipts.get(key);
    return value && structuredClone(value.receipt);
  }

  private async establishTarget(target: Principal, refreshThrough?: bigint, retryExpired = false): Promise<Identity> {
    if (target.isAnonymous()) throw new ClientError('configuration');
    // SDK Principals expose mutable backing bytes. Own the destination before
    // any await and keep transport objects separate from signing authority.
    const key = target.toText();
    const refreshing = refreshThrough !== undefined;
    for (let attempt = 0; ; attempt++) {
      const value = await this.credential(refreshThrough);
      if (value.until <= await this.active()) {
        // Exact recovery has settled this sequence before a new intent is
        // allowed. Refresh once before any application request can be signed.
        if (retryExpired && attempt === 0) { refreshThrough = value.sequence; continue; }
        throw new ClientError('retrieval_expired');
      }
      await this.journal.destination(this.generation, Principal.fromText(key), value.until);
      if (value.until <= await this.active()) throw new ClientError('retrieval_expired');
      // Only committed journal destinations reach this map, keeping it within the
      // journal's target bound. Completed calls are not cached as session authority.
      let flight = this.admissions.get(key);
      if (!flight) {
        const promise = this.call(this.identities.browser, this.caller, key, (signal, identity) => this.transport.admit(Principal.fromText(key), identity,
          credentialFromBytes(value.bytes, this.maximumBytes), signal)).then(receipt => {
            if (Principal.fromUint8Array(receipt.caller).toText() !== this.caller
              || !isU64(receipt.expires_at_ns) || receipt.expires_at_ns === 0n) throw new ClientError('invalid_material');
            this.receipts.set(key, { sequence: value.sequence, receipt: structuredClone(receipt) });
            return this.guarded;
          });
        flight = { sequence: value.sequence, promise };
        this.admissions.set(key, flight);
      }
      try { return await flight.promise; }
      catch (error) {
        if (!retryExpired || refreshing || attempt !== 0 || !(error instanceof AdmissionError) || error.refusal !== 'Expired') throw error;
        refreshThrough = flight.sequence;
      }
      finally { if (this.admissions.get(key) === flight) this.admissions.delete(key); }
    }
  }
}
