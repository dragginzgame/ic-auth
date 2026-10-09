import type { Identity } from '@icp-sdk/core/agent';
import type { Principal } from '@icp-sdk/core/principal';
import type {
  DelegatedRoleGrant, DelegatedToken, DelegatedTokenPrepareRequest,
  DelegatedTokenPrepareResponse, DelegationAudience,
} from './generated/protocol.did.js';

/** Obtained from the login owner, never from a token or issuer reply. */
export interface Session {
  identity: Identity;
  /** Globally fresh for every login/reconnect/invalidation; never reuse it. */
  generation: string;
}

export interface SessionSource {
  current(): Session | undefined;
  connect(signal: AbortSignal): Promise<Session>;
  /** Atomically compare the observed generation before invalidating it. */
  invalidate(expectedGeneration: string | undefined): Promise<boolean>;
}

/** Consumer configuration, independent of submitted token claims. */
export interface TokenScope {
  issuer: Principal;
  audience: DelegationAudience;
  grants: DelegatedRoleGrant[];
  ext: [] | [Uint8Array];
  network: string;
  policyId: string;
}

export type IssuerFailure =
  | { kind: 'ttlUnavailable' }
  | { kind: 'sessionInvalid'; detail?: unknown }
  | { kind: 'rejected'; detail?: unknown };
export type PrepareResult =
  | { kind: 'prepared'; value: DelegatedTokenPrepareResponse }
  | IssuerFailure;
export type RetrieveResult =
  | { kind: 'ready'; value: DelegatedToken }
  | { kind: 'pending' }
  | Exclude<IssuerFailure, { kind: 'ttlUnavailable' }>;

export interface IssuerOperations {
  prepare(request: DelegatedTokenPrepareRequest, signal: AbortSignal): Promise<PrepareResult>;
  /** Lookup or qualified exact-id replay only; never allocate a replacement. */
  reconcile(request: DelegatedTokenPrepareRequest, signal: AbortSignal): Promise<PrepareResult | { kind: 'unresolved' }>;
  retrieve(claimsHash: Uint8Array, signal: AbortSignal): Promise<RetrieveResult>;
}

/** Bytes use the generated Candid codecs, including bigints and principals. */
export type StoredToken =
  | { kind: 'prepare'; request: Uint8Array; candidate: number; deadlineNs: bigint }
  | { kind: 'retrieve'; request: Uint8Array; prepared: Uint8Array; deadlineNs: bigint }
  | { kind: 'ready'; token: Uint8Array };
export interface StoreSnapshot {
  revision: bigint;
  entry: StoredToken | undefined;
}
export interface TokenStore {
  read(key: string): Promise<StoreSnapshot>;
  /** Atomic per-key compare-and-swap; copy bytes and retain unknown effects. */
  write(key: string, expectedRevision: bigint, entry: StoredToken | undefined): Promise<boolean>;
}

export interface ClientOptions {
  sessions: SessionSource;
  store: TokenStore;
  issuer(session: Session, scope: TokenScope): IssuerOperations;
  nowNs(): bigint;
  /** Resolve after the delay; reject when aborted. No implicit downloads. */
  wait(ms: number, signal: AbortSignal): Promise<void>;
  requestId(): Uint8Array;
  ttlCandidatesNs: readonly bigint[];
  requestTtlNs: bigint;
  operationTtlNs: bigint;
  renewBeforeNs: bigint;
  callTimeoutMs: number;
  pollIntervalMs: number;
  maxPollAttempts: number;
  maxMaterialBytes: number;
}

export type ClientErrorCode =
  | 'configuration' | 'unauthenticated' | 'stale_generation' | 'ttl_exhausted'
  | 'prepare_uncertain' | 'retrieval_expired' | 'polling_exhausted' | 'transport'
  | 'issuer_rejected' | 'invalid_material' | 'storage_conflict' | 'timeout'
  | 'storage_capacity';

export class ClientError extends Error {
  constructor(public readonly code: ClientErrorCode, options?: ErrorOptions) {
    super(code, options);
    this.name = 'ClientError';
  }
}
