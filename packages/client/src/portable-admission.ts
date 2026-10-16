import { boundedCall } from './bounded-call.js';
import type { Identity } from '@icp-sdk/core/agent';
import type { Principal } from '@icp-sdk/core/principal';
import { ClientError } from './contracts.js';
import { credentialBytes, credentialFromBytes } from './codec.js';
import type { PortableCredential } from './generated/protocol.did.js';
import type { PortableLogoutClient } from './logout.js';

export interface AdmissionCredential {
  credential: PortableCredential;
  /** Qualified adapter maps the original proof expiry conservatively to the journal clock.
   * Recovery MUST retain this deadline, never extend it from the time of retrieval.
   */
  admissionUntilNs: bigint;
}

export interface PortableAdmissionTransport {
  /** Bound to one issuer, login and immutable requested grant set. Persist exact
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

/** One portable proof across receivers for an already started browser login.
 * No endpoint installation, automatic renewal, retained credential format or app-call retry.
 */
export class PortableAdmissionClient {
  private cached: { bytes: Uint8Array; until: bigint } | undefined;
  private credentialFlight: Promise<{ bytes: Uint8Array; until: bigint }> | undefined;
  private readonly guarded: Identity;
  private readonly caller: string;

  constructor(private readonly journal: PortableLogoutClient, private readonly generation: string,
    identity: Identity, private readonly issuer: Principal,
    private readonly transport: PortableAdmissionTransport,
    private readonly timeoutMs = 5_000, private readonly maximumBytes = 131_072) {
    if (issuer.isAnonymous() || !Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > 5_000
      || !Number.isSafeInteger(maximumBytes) || maximumBytes < 1 || maximumBytes > 1_048_576) throw new ClientError('configuration');
    this.caller = identity.getPrincipal().toText();
    this.guarded = journal.guardedIdentity(generation, identity);
  }

  private async active(): Promise<bigint> {
    const state = await this.journal.status(this.generation);
    if (state.phase !== 'active' || state.caller !== this.caller || state.issuer !== this.issuer.toText()) {
      this.cached = undefined;
      throw new ClientError('stale_generation');
    }
    return state.observedNs;
  }

  private async call<T>(operation: (signal: AbortSignal, identity: Identity) => Promise<T>): Promise<T> {
    await this.active();
    return boundedCall(this.timeoutMs, async signal => {
      const check = () => { if (signal.aborted) throw new ClientError('timeout'); };
      const identity: Identity = {
        getPrincipal: () => this.guarded.getPrincipal(),
        transformRequest: async request => {
          check();
          const signed = await this.guarded.transformRequest(request);
          check();
          return signed;
        },
      };
      const result = await operation(signal, identity);
      await this.active();
      check();
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
      const result = await this.call((signal, identity) => first
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
    const value = await this.credential();
    await this.journal.destination(this.generation, target, value.until);
    if (value.until <= await this.active()) throw new ClientError('retrieval_expired');
    await this.call((signal, identity) => this.transport.admit(target, identity,
      credentialFromBytes(value.bytes, this.maximumBytes), signal));
    return this.guarded;
  }
}
