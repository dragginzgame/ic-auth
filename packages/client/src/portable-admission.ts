import { boundedCall, MAX_NETWORK_CALL_MS } from './bounded-call.js';
import { checkedIdentity, checkDestination } from './checked-identity.js';
import type { Identity } from '@icp-sdk/core/agent';
import { Principal } from '@icp-sdk/core/principal';
import { ClientError } from './contracts.js';
import { credentialBytes, credentialFromBytes } from './codec.js';
import type { PortableCredential } from './codec.js';
import type { PortableLogoutClient } from './logout.js';

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
  reconcile(identity: Identity, signal: AbortSignal): Promise<AdmissionCredential>;
  /** Authenticate the update reply and resolve only on successful session admission.
   * Receivers perform complete proof, authority, replay and scope checks.
   */
  admit(target: Principal, identity: Identity, credential: PortableCredential, signal: AbortSignal): Promise<void>;
}

/** Separate account authentication at the issuer from fresh receiver ingress. */
export interface PortableAdmissionIdentities {
  browser: Identity;
  account: Identity;
}

/** One portable proof across receivers for an already started browser login.
 * No endpoint installation, automatic renewal, retained credential format or app-call retry.
 */
export class PortableAdmissionClient {
  private cached: { bytes: Uint8Array; until: bigint } | undefined;
  private credentialFlight: Promise<{ bytes: Uint8Array; until: bigint }> | undefined;
  private readonly admissions = new Map<string, Promise<Identity>>();
  private readonly guarded: Identity;
  private readonly caller: string;
  private readonly accountCaller: string;
  private readonly identities: PortableAdmissionIdentities;

  constructor(private readonly journal: PortableLogoutClient, private readonly generation: string,
    identities: PortableAdmissionIdentities, private readonly issuer: Principal,
    private readonly transport: PortableAdmissionTransport,
    private readonly timeoutMs = MAX_NETWORK_CALL_MS, private readonly maximumBytes = 131_072) {
    if (issuer.isAnonymous() || identities.account.getPrincipal().isAnonymous()
      || !Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > MAX_NETWORK_CALL_MS
      || !Number.isSafeInteger(maximumBytes) || maximumBytes < 1 || maximumBytes > 1_048_576) throw new ClientError('configuration');
    this.identities = { browser: identities.browser, account: identities.account };
    this.caller = identities.browser.getPrincipal().toText();
    this.accountCaller = identities.account.getPrincipal().toText();
    this.guarded = journal.guardedIdentity(generation, identities.browser);
  }

  private async active(): Promise<bigint> {
    const state = await this.journal.status(this.generation);
    if (state.phase !== 'active' || state.caller !== this.caller || state.issuer !== this.issuer.toText()) {
      this.cached = undefined;
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

  private async credential(): Promise<{ bytes: Uint8Array; until: bigint }> {
    const now = await this.active();
    if (this.cached) {
      if (this.cached.until <= now) throw new ClientError('retrieval_expired');
      return this.cached;
    }
    if (this.credentialFlight) return this.credentialFlight;
    const flight = (async () => {
      const first = await this.journal.claimIssuance(this.generation);
      const result = await this.call(this.identities.account, this.accountCaller, this.issuer.toText(), (signal, identity) => first
        ? this.transport.issue(identity, signal) : this.transport.reconcile(identity, signal));
      // Copy returned bytes before another asynchronous boundary. Transport decoding
      // is not authentication; the receiver remains the verifier.
      const until = result.admissionUntilNs;
      const bytes = credentialBytes(result.credential, this.maximumBytes);
      const now = await this.active();
      if (typeof until !== 'bigint' || until <= now || until > (1n << 64n) - 1n) throw new ClientError('retrieval_expired');
      this.cached = { bytes, until };
      return this.cached;
    })();
    this.credentialFlight = flight;
    try { return await flight; }
    finally { if (this.credentialFlight === flight) this.credentialFlight = undefined; }
  }

  /** Explicit first-contact/re-admission operation. Returned identity also checks
   * durable logout state at signing; ordinary application calls use receiver guards.
   */
  async establish(target: Principal): Promise<Identity> {
    if (target.isAnonymous()) throw new ClientError('configuration');
    // SDK Principals expose mutable backing bytes. Own the destination before
    // any await and keep transport objects separate from signing authority.
    const key = target.toText();
    const value = await this.credential();
    await this.journal.destination(this.generation, Principal.fromText(key), value.until);
    if (value.until <= await this.active()) throw new ClientError('retrieval_expired');
    // Only committed journal destinations reach this map, keeping it within the
    // journal's target bound. Completed calls are not cached as session authority.
    const previous = this.admissions.get(key);
    if (previous) return previous;
    const flight = this.call(this.identities.browser, this.caller, key, (signal, identity) => this.transport.admit(Principal.fromText(key), identity,
      credentialFromBytes(value.bytes, this.maximumBytes), signal)).then(() => this.guarded);
    this.admissions.set(key, flight);
    try { return await flight; }
    finally { if (this.admissions.get(key) === flight) this.admissions.delete(key); }
  }
}
