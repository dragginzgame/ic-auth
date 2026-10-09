import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { Principal } from '@icp-sdk/core/principal';
import { MemoryTokenStore, TokenSessionClient, type ClientOptions, type Session, type SessionSource, type TokenScope, type IssuerOperations, type DelegatedToken, type DelegatedTokenPrepareRequest } from '../src/index.js';

export const second = 1_000_000_000n;
export const principal = (seed: number) => Ed25519KeyIdentity.generate(new Uint8Array(32).fill(seed));
export const scope = (): TokenScope => ({
  issuer: Principal.fromUint8Array(new Uint8Array([1, 1])),
  audience: { Fleet: { canonical_network_id: 'a'.repeat(64), fleet_id: 'b'.repeat(64) } },
  network: 'local', policyId: 'policy-1', grants: [{ target: 'users', scopes: ['read'] }], ext: [],
});
export class Sessions implements SessionSource {
  value: Session | undefined = { identity: principal(1), generation: 'login-1' };
  invalidations: (string | undefined)[] = [];
  current() { return this.value; }
  async connect(_signal: AbortSignal) { if (!this.value) throw new Error('login required'); return this.value; }
  async invalidate(expected: string | undefined) {
    this.invalidations.push(expected);
    if (this.value?.generation !== expected) return false;
    this.value = undefined;
    return true;
  }
}
export function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(yes => { resolve = yes; });
  return { promise, resolve };
}
export function wait(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const abort = () => { clearTimeout(timer); reject(new Error('aborted')); };
    const timer = setTimeout(() => { signal.removeEventListener('abort', abort); resolve(); }, ms);
    if (signal.aborted) abort(); else signal.addEventListener('abort', abort, { once: true });
  });
}
/** Unsigned transport fixtures; these do not qualify token cryptography. */
export function material(request: DelegatedTokenPrepareRequest, session: Session, configured: TokenScope, now: bigint): DelegatedToken {
  const pid = session.identity.getPrincipal();
  const binding = { IcCanisterSignatureV1: { seed_hash: new Uint8Array(32) } };
  const algorithm = { IcCanisterSignatureV1: null };
  return {
    claims: { aud: request.aud, grants: request.grants, ext: request.ext, cert_hash: new Uint8Array(32), nonce: new Uint8Array(16),
      subject: pid, presenter: pid, issuer_pid: configured.issuer, issued_at_ns: now, expires_at_ns: now + request.ttl_ns },
    issuer_proof: { IcCanisterSignatureV1: { signature_cbor: new Uint8Array([1]), public_key_der: new Uint8Array([2]) } },
    proof: {
      cert: { aud: request.aud, grants: request.grants, root_pid: configured.issuer, issuer_pid: configured.issuer,
        issuer_proof_alg: algorithm, issuer_proof_binding: binding, issuer_proof_binding_hash: new Uint8Array(32),
        issued_at_ns: now, not_before_ns: now, expires_at_ns: now + request.ttl_ns, max_token_ttl_ns: request.ttl_ns },
      root_proof: { IcChainKeyBatchSignatureV1: {
        header: { root_canister_id: configured.issuer, registry_epoch: 1n, proof_epoch: 1n, registry_hash: new Uint8Array(32),
          algorithm: { EcdsaSecp256k1: null }, key_id: { name: 'test' }, key_version: 1n, schema_version: 1,
          batch_id: new Uint8Array(32), derivation_path_hash: new Uint8Array(32), tree_root: new Uint8Array(32),
          not_before_ns: now, expires_at_ns: now + request.ttl_ns },
        delegation_cert: { root_canister_id: configured.issuer, issuer_canister_id: configured.issuer, audience: request.aud,
          grants: request.grants, max_token_ttl_ns: request.ttl_ns, issuer_proof_binding: binding,
          issuer_proof_binding_hash: new Uint8Array(32), issuer_proof_algorithm: algorithm,
          registry_epoch: 1n, proof_epoch: 1n, registry_hash: new Uint8Array(32), not_before_ns: now, expires_at_ns: now + request.ttl_ns },
        issuer_witness: { steps: [] }, signature: { algorithm: { EcdsaSecp256k1: null }, key_id: { name: 'test' },
          derivation_path: [], signature: new Uint8Array(64), public_key: new Uint8Array(33) },
      } },
    },
  };
}
export function harness(changes: Partial<ClientOptions> = {}) {
  const sessions = new Sessions();
  const store = new MemoryTokenStore(128);
  const state = { now: second, prepares: 0, retrieves: 0, reconciles: 0, ids: 0 };
  const requests: DelegatedTokenPrepareRequest[] = [];
  const issued = new Map<string, DelegatedToken>();
  const configured = scope();
  const issuer: ClientOptions['issuer'] = (session, fixed) => {
    const prepare: IssuerOperations['prepare'] = async request => {
      state.prepares++; requests.push(request);
      const token = material(request, session, fixed, state.now);
      issued.set(session.generation, token);
      return { kind: 'prepared', value: { claims: token.claims, claims_hash: new Uint8Array(32), retrieval_expires_at_ns: state.now + second } };
    };
    return {
      prepare,
      reconcile: async () => {
        state.reconciles++;
        const token = issued.get(session.generation);
        return token ? { kind: 'prepared', value: { claims: token.claims, claims_hash: new Uint8Array(32), retrieval_expires_at_ns: state.now + second } } : { kind: 'unresolved' };
      },
      retrieve: async () => { state.retrieves++; return { kind: 'ready', value: issued.get(session.generation)! }; },
    };
  };
  const options: ClientOptions = {
    sessions, store, issuer, nowNs: () => state.now, wait,
    requestId: () => new Uint8Array(32).fill(++state.ids),
    ttlCandidatesNs: [10n * second, 5n * second], requestTtlNs: second,
    operationTtlNs: 2n * second, renewBeforeNs: second,
    callTimeoutMs: 500, pollIntervalMs: 1, maxPollAttempts: 3, maxMaterialBytes: 65_536,
    ...changes,
  };
  return { client: new TokenSessionClient(options), options, sessions, store, state, requests, configured, issued };
}
