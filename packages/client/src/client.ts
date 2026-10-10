import { Principal } from '@icp-sdk/core/principal';
import { IDL } from '@icp-sdk/core/candid';
import {
  ClientError, type ClientOptions, type IssuerFailure, type Session,
  type TokenScope,
} from './contracts.js';
import { DelegatedRoleGrant, DelegationAudience, type DelegatedToken, type DelegatedTokenPrepareRequest, type DelegatedTokenPrepareResponse } from './generated/protocol.did.js';
import { claimsBytes, encode, equalBytes, preparedBytes, preparedFromBytes, requestBytes, requestFromBytes, tokenBytes, tokenFromBytes } from './codec.js';

const U64_MAX = (1n << 64n) - 1n;
const label = (value: string) => value.length > 0 && value.length <= 128;
const positiveU64 = (value: bigint) => value > 0n && value <= U64_MAX;
interface CapturedSession { session: Session; principal: string }

/** Transport/cache machinery only. Servers still authenticate and authorize every token. */
export class TokenSessionClient {
  readonly #options: ClientOptions;
  readonly #flights = new Map<string, Promise<DelegatedToken>>();
  #connecting: Promise<Session> | undefined;
  #lastNow = 0n;

  constructor(options: ClientOptions) {
    const ttls = [...options.ttlCandidatesNs];
    if (ttls.length === 0 || ttls.length > 16 || !ttls.every(positiveU64)
      || ttls.some((ttl, index) => index > 0 && ttl >= ttls[index - 1]!)
      || !positiveU64(options.requestTtlNs) || !positiveU64(options.operationTtlNs)
      || options.renewBeforeNs < 0n || options.renewBeforeNs >= ttls[ttls.length - 1]!
      || !Number.isSafeInteger(options.callTimeoutMs) || options.callTimeoutMs < 1 || options.callTimeoutMs > 60_000
      || !Number.isSafeInteger(options.pollIntervalMs) || options.pollIntervalMs < 1 || options.pollIntervalMs > 60_000
      || !Number.isSafeInteger(options.maxPollAttempts) || options.maxPollAttempts < 1 || options.maxPollAttempts > 128
      || !Number.isSafeInteger(options.maxMaterialBytes) || options.maxMaterialBytes < 1 || options.maxMaterialBytes > 1_048_576) {
      throw new ClientError('configuration');
    }
    this.#options = { ...options, ttlCandidatesNs: ttls };
  }

  connect(): Promise<Session> {
    const current = this.#options.sessions.current();
    if (current) {
      try { return Promise.resolve(this.#capture(current).session); }
      catch (error) { return Promise.reject(error); }
    }
    if (this.#connecting) return this.#connecting;
    const promise = this.#call(signal => this.#options.sessions.connect(signal))
      .then(session => {
        const captured = this.#capture(session);
        this.#assertSession(captured);
        return session;
      }).finally(() => { if (this.#connecting === promise) this.#connecting = undefined; });
    this.#connecting = promise;
    return promise;
  }

  /** Called by the login/cross-tab owner; does not revoke issued IC delegations. */
  logout(): Promise<boolean> {
    return this.#options.sessions.invalidate(this.#options.sessions.current()?.generation);
  }

  /** Capture generation before the endpoint call that may reject a session. */
  invalidate(seenGeneration: string): Promise<boolean> {
    return this.#options.sessions.invalidate(seenGeneration);
  }

  async token(scope: TokenScope): Promise<DelegatedToken> {
    const captured = this.#capture(await this.connect());
    // Detach caller-owned mutable Candid data before selecting a flight or callback.
    const fixed = this.#scope(scope);
    const key = JSON.stringify([
      captured.session.generation, captured.principal, fixed.issuer.toText(),
      fixed.network, fixed.policyId, fixed.audience.Fleet.canonical_network_id,
      fixed.audience.Fleet.fleet_id, fixed.grants, [...(fixed.ext[0] ?? [])],
    ]);
    const existing = this.#flights.get(key);
    if (existing) {
      const value = await existing;
      this.#assertSession(captured);
      return tokenFromBytes(tokenBytes(value, this.#options.maxMaterialBytes), this.#options.maxMaterialBytes);
    }
    if (this.#flights.size >= 128) throw new ClientError('storage_capacity');
    const promise = this.#obtain(key, captured, fixed)
      .finally(() => { if (this.#flights.get(key) === promise) this.#flights.delete(key); });
    this.#flights.set(key, promise);
    const value = await promise;
    this.#assertSession(captured);
    return tokenFromBytes(tokenBytes(value, this.#options.maxMaterialBytes), this.#options.maxMaterialBytes);
  }

  #capture(session: Session): CapturedSession {
    if (!label(session.generation) || session.identity.getPrincipal().isAnonymous()) {
      throw new ClientError('unauthenticated');
    }
    return { session: { ...session }, principal: session.identity.getPrincipal().toText() };
  }

  #assertSession(captured: CapturedSession): void {
    const current = this.#options.sessions.current();
    if (!current || current.generation !== captured.session.generation
      || current.identity.getPrincipal().toText() !== captured.principal) {
      throw new ClientError('stale_generation');
    }
  }

  #now(): bigint {
    const now = this.#options.nowNs();
    if (now < this.#lastNow || now > U64_MAX || now < 0n) throw new ClientError('configuration');
    this.#lastNow = now;
    return now;
  }

  #scope(scope: TokenScope): TokenScope & { audience: { Fleet: { canonical_network_id: string; fleet_id: string } } } {
    const { Fleet: audience } = scope.audience;
    if (scope.issuer.isAnonymous() || !label(scope.network) || !label(scope.policyId)
      || !/^[0-9a-f]{64}$/.test(audience.canonical_network_id) || !/^[0-9a-f]{64}$/.test(audience.fleet_id)
      || scope.grants.length === 0 || scope.grants.length > 16 || scope.ext.length > 1
      || (scope.ext[0]?.length ?? 0) > 4096) throw new ClientError('configuration');
    let previous = '';
    for (const grant of scope.grants) {
      if (!/^[a-z0-9_:-]+$/.test(grant.target) || grant.target <= previous || grant.scopes.length > 32) {
        throw new ClientError('configuration');
      }
      previous = grant.target;
      if (grant.scopes.some((item, index) => item.length === 0 || item.length > 64
        || (index > 0 && item <= grant.scopes[index - 1]!))) throw new ClientError('configuration');
    }
    return {
      issuer: Principal.fromText(scope.issuer.toText()),
      audience: { Fleet: { ...audience } },
      grants: scope.grants.map(grant => ({ target: grant.target, scopes: [...grant.scopes] })),
      ext: scope.ext[0] ? [scope.ext[0].slice()] : [], network: scope.network, policyId: scope.policyId,
    };
  }

  async #call<T>(operation: (signal: AbortSignal) => Promise<T>, deadline?: bigint): Promise<T> {
    const remaining = deadline === undefined ? this.#options.callTimeoutMs : Number((deadline - this.#now() + 999_999n) / 1_000_000n);
    if (remaining <= 0) throw new ClientError('retrieval_expired');
    const call = new AbortController();
    const timer = new AbortController();
    try {
      return await Promise.race([
        operation(call.signal),
        this.#options.wait(Math.min(remaining, this.#options.callTimeoutMs), timer.signal).then(() => {
          call.abort();
          throw new ClientError('timeout');
        }),
      ]);
    } finally { timer.abort(); }
  }

  async #failure(result: IssuerFailure, captured: CapturedSession): Promise<never> {
    this.#assertSession(captured);
    if (result.kind === 'sessionInvalid') {
      await this.#options.sessions.invalidate(captured.session.generation);
      throw new ClientError('unauthenticated', { cause: result.detail });
    }
    throw new ClientError('issuer_rejected', { cause: 'detail' in result ? result.detail : undefined });
  }

  #checkClaims(token: DelegatedToken | DelegatedTokenPrepareResponse, captured: CapturedSession, request: DelegatedTokenPrepareRequest): void {
    const claims = token.claims;
    if (claims.presenter.toText() !== captured.principal || claims.subject.toText() !== captured.principal
      || claims.cert_hash.length !== 32 || claims.nonce.length !== 16
      || claims.issued_at_ns > this.#now() || claims.expires_at_ns <= this.#now()
      || claims.expires_at_ns <= claims.issued_at_ns || claims.expires_at_ns - claims.issued_at_ns > request.ttl_ns
      || !equalBytes(encode(DelegationAudience, claims.aud, this.#options.maxMaterialBytes), encode(DelegationAudience, request.aud, this.#options.maxMaterialBytes))
      || !equalBytes(IDL.encode([IDL.Vec(DelegatedRoleGrant)], [claims.grants]), IDL.encode([IDL.Vec(DelegatedRoleGrant)], [request.grants]))
      || !equalBytes(IDL.encode([IDL.Opt(IDL.Vec(IDL.Nat8))], [claims.ext]), IDL.encode([IDL.Opt(IDL.Vec(IDL.Nat8))], [request.ext]))) {
      throw new ClientError('invalid_material');
    }
  }

  async #obtain(key: string, captured: CapturedSession, scope: TokenScope): Promise<DelegatedToken> {
    const { store, maxMaterialBytes: maximum } = this.#options;
    const operations = this.#options.issuer(captured.session, scope);
    let candidate = 0;
    let deadline = this.#now() + this.#options.operationTtlNs;
    if (deadline > U64_MAX) throw new ClientError('configuration');
    // CAS contention is bounded separately from issuer polling and TTL backoff.
    for (let contention = 0; contention < 32; contention++) {
      this.#assertSession(captured);
      const snapshot = await this.#call(() => store.read(key));
      this.#assertSession(captured);
      let { entry } = snapshot;
      let firstDispatch = false;
      let reserved: Uint8Array | undefined;
      if (entry?.kind === 'ready') {
        const token = tokenFromBytes(entry.token, maximum);
        const request = { metadata: [], aud: scope.audience, grants: scope.grants, ext: scope.ext, ttl_ns: this.#options.ttlCandidatesNs[0]! } satisfies DelegatedTokenPrepareRequest;
        if (token.claims.issuer_pid.toText() !== scope.issuer.toText()) throw new ClientError('invalid_material');
        if (token.claims.expires_at_ns > this.#now()) {
          this.#checkClaims(token, captured, request);
          if (token.claims.expires_at_ns - this.#now() > this.#options.renewBeforeNs) return token;
        }
        if (!await this.#call(() => store.write(key, snapshot.revision, undefined))) continue;
        continue;
      }
      if (!entry) {
        if (this.#now() >= deadline) throw new ClientError('retrieval_expired');
        const requestId = this.#options.requestId().slice();
        if (requestId.length !== 32) throw new ClientError('configuration');
        entry = {
          kind: 'prepare', candidate, deadlineNs: deadline,
          request: requestBytes({ metadata: [{ request_id: requestId, ttl_ns: this.#options.requestTtlNs }],
            ttl_ns: this.#options.ttlCandidatesNs[candidate]!, aud: scope.audience, grants: scope.grants, ext: scope.ext }, maximum),
        };
        const created = entry;
        if (!await this.#call(() => store.write(key, snapshot.revision, created))) continue;
        firstDispatch = true;
        reserved = created.request;
      }
      const owned = await this.#call(() => store.read(key));
      this.#assertSession(captured);
      if (!owned.entry) continue;
      entry = owned.entry;
      if (entry.kind === 'ready') continue;
      deadline = entry.deadlineNs;
      if (!positiveU64(deadline)) throw new ClientError('invalid_material');
      const request = requestFromBytes(entry.request, maximum);
      const expected = { ...request, aud: scope.audience, grants: scope.grants, ext: scope.ext };
      if (!equalBytes(requestBytes(request, maximum), requestBytes(expected, maximum))
        || request.metadata.length !== 1 || request.metadata[0]!.request_id.length !== 32
        || request.metadata[0]!.ttl_ns !== this.#options.requestTtlNs
        || !this.#options.ttlCandidatesNs.includes(request.ttl_ns)) throw new ClientError('invalid_material');
      firstDispatch = firstDispatch && reserved !== undefined && equalBytes(reserved, entry.request);
      if (this.#now() >= deadline) throw new ClientError('retrieval_expired');
      if (entry.kind === 'prepare') {
        candidate = entry.candidate;
        if (!Number.isSafeInteger(candidate) || candidate < 0
          || this.#options.ttlCandidatesNs[candidate] !== request.ttl_ns) throw new ClientError('invalid_material');
        let result;
        try {
          result = await this.#call(signal => firstDispatch
            ? operations.prepare(request, signal) : operations.reconcile(request, signal), deadline);
        } catch (cause) {
          this.#assertSession(captured);
          throw new ClientError('prepare_uncertain', { cause });
        }
        this.#assertSession(captured);
        if (this.#now() >= deadline) throw new ClientError('retrieval_expired');
        if (result.kind === 'unresolved') throw new ClientError('prepare_uncertain');
        if (result.kind === 'ttlUnavailable') {
          // This is explicitly a definitive no-effect rejection. Unknown replies
          // never arrive here and never select a new request ID or TTL.
          if (!await this.#call(() => store.write(key, owned.revision, undefined))) continue;
          candidate++;
          if (candidate >= this.#options.ttlCandidatesNs.length) throw new ClientError('ttl_exhausted');
          continue;
        }
        if (result.kind !== 'prepared') return this.#failure(result, captured);
        const prepared = result.value;
        preparedBytes(prepared, maximum);
        this.#checkClaims(prepared, captured, request);
        if (prepared.claims.issuer_pid.toText() !== scope.issuer.toText() || prepared.claims_hash.length !== 32
          || prepared.retrieval_expires_at_ns <= this.#now()) throw new ClientError('invalid_material');
        entry = { kind: 'retrieve', request: entry.request,
          prepared: preparedBytes(prepared, maximum), deadlineNs: deadline < prepared.retrieval_expires_at_ns ? deadline : prepared.retrieval_expires_at_ns };
        const retrieving = entry;
        if (!await this.#call(() => store.write(key, owned.revision, retrieving))) continue;
        continue;
      }
      const prepared = preparedFromBytes(entry.prepared, maximum);
      this.#checkClaims(prepared, captured, request);
      if (prepared.claims.issuer_pid.toText() !== scope.issuer.toText() || prepared.claims_hash.length !== 32) throw new ClientError('invalid_material');
      for (let attempt = 0; attempt < this.#options.maxPollAttempts; attempt++) {
        this.#assertSession(captured);
        if (this.#now() >= deadline) throw new ClientError('retrieval_expired');
        let result;
        try { result = await this.#call(signal => operations.retrieve(prepared.claims_hash.slice(), signal), deadline); }
        catch (cause) { this.#assertSession(captured); throw new ClientError('transport', { cause }); }
        this.#assertSession(captured);
        if (this.#now() >= deadline) throw new ClientError('retrieval_expired');
        if (result.kind === 'ready') {
          const bytes = tokenBytes(result.value, maximum);
          this.#checkClaims(result.value, captured, request);
          if (!equalBytes(claimsBytes(result.value.claims, maximum), claimsBytes(prepared.claims, maximum))) throw new ClientError('invalid_material');
          if (!await this.#call(() => store.write(key, owned.revision, { kind: 'ready', token: bytes }))) break;
          this.#assertSession(captured);
          if (this.#now() >= deadline) throw new ClientError('retrieval_expired');
          return tokenFromBytes(bytes, maximum);
        }
        if (result.kind !== 'pending') return this.#failure(result, captured);
        if (attempt + 1 === this.#options.maxPollAttempts) throw new ClientError('polling_exhausted');
        const remainingMs = Number((deadline - this.#now() + 999_999n) / 1_000_000n);
        await this.#options.wait(Math.min(this.#options.pollIntervalMs, Math.max(1, remainingMs)), new AbortController().signal);
      }
    }
    throw new ClientError('storage_conflict');
  }
}
