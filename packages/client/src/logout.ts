import { boundedCall, MAX_NETWORK_CALL_MS as CALL_MS } from './bounded-call.js';
import { checkedIdentity, checkDestination } from './checked-identity.js';
import { Principal } from '@tokostudios/ic-platform/principal';
import type { Identity } from '@tokostudios/ic-platform/agent';
import type { LogoutResult } from './codec.js';
import { ClientError } from './contracts.js';
import { IndexedDbLogoutStore, MAX_ATTEMPTS, MAX_TARGETS, type LogoutJournal, type LogoutTarget } from './logout-store.js';

const MS = 1_000_000n;
const MAX_CLEANUP_NS = 300_000n * MS;

function cleanupComplete(record: LogoutJournal & { phase: 'cleanup' | 'finished' }, now: bigint): boolean {
  return now >= record.deadlineNs || record.targets.every(target => target.confirmedUntilNs !== undefined);
}

// One eligibility rule for both dispatch and waiting. The final attempt cannot
// run until admission has expired, even when its persisted backoff is earlier.
function nextAttemptNs(target: LogoutTarget): bigint | undefined {
  if (target.confirmedUntilNs !== undefined || target.attempts >= MAX_ATTEMPTS) return undefined;
  return target.attempts === MAX_ATTEMPTS - 1 && target.nextNs <= target.admissionUntilNs
    ? target.admissionUntilNs + 1n : target.nextNs;
}

// Synchronous journal transitions; callers commit them through update before I/O.
// One live lease across tabs. A crash consumes its attempt; the lease expires
// after the call bound. Keep the last attempt for after admission expiry.
function claimCleanup(record: LogoutJournal, now: bigint) {
  const idle = { record, value: undefined };
  if (record.phase !== 'cleanup') return idle;
  if (cleanupComplete(record, now)) {
    return { record: { ...record, phase: 'finished' as const }, value: undefined };
  }
  if (record.busyUntilNs > now) return idle;
  const target = record.targets.find(t => {
    const next = nextAttemptNs(t);
    return next !== undefined && next <= now;
  });
  if (!target) return idle;
  target.attempts++;
  target.nextNs = now + BigInt(CALL_MS) * MS;
  record.busyUntilNs = target.nextNs;
  return { record, value: { lease: record.busyUntilNs, canister: target.canister, attempt: target.attempts, timeoutMs: Math.max(1, Math.min(CALL_MS, Number((record.deadlineNs - now) / MS))) } };
}

function cleanupWaitMs(status: LogoutJournal, now: bigint): number {
  let next = status.targets.reduce((time, target) => {
    const next = nextAttemptNs(target);
    return next !== undefined && next < time ? next : time;
  }, status.deadlineNs);
  if (status.busyUntilNs > next) next = status.busyUntilNs < status.deadlineNs ? status.busyUntilNs : status.deadlineNs;
  return Math.max(1, Math.min(1_000, Number((next - now) / MS)));
}

function settleCleanup(record: LogoutJournal, now: bigint,
  claim: NonNullable<ReturnType<typeof claimCleanup>['value']>, response: LogoutResult | undefined, jitter: number): void {
  if (record.phase !== 'cleanup' || now >= record.deadlineNs) return;
  const target = record.targets.find(t => t.canister === claim.canister)!;
  if (target.attempts !== claim.attempt || record.busyUntilNs !== claim.lease) return;
  record.busyUntilNs = 0n;
  if (response && response.tag === 'Revoked' && typeof response.value === 'bigint'
    && response.value > 0n && response.value <= (1n << 64n) - 1n) {
    target.confirmedUntilNs = response.value;
  } else target.nextNs = now + BigInt(500 + Math.floor(jitter * 1_000)) * MS;
}

export interface LogoutClock {
  /** Stable across reloads; adapter must bound IC clock uncertainty and rollback. */
  id: string;
  nowNs(): bigint;
  wait(ms: number): Promise<void>;
}
export interface LogoutTransport {
  /** Must authenticate as this exact identity; target and empty request are fixed. */
  revoke(target: Principal, identity: Identity, signal: AbortSignal): Promise<LogoutResult>;
  /** Remove retained signing material/usable credentials; must be idempotent. */
  dispose(generation: string): Promise<void>;
}

/** Portable login journal and bounded cleanup; not an issuer or ordinary-call adapter. */
export class PortableLogoutClient {
  constructor(private readonly store: IndexedDbLogoutStore, private readonly clock: LogoutClock,
    private readonly transport: LogoutTransport, private readonly random: () => number = Math.random) {}

  async start(generation: string, identity: Identity, issuer: Principal): Promise<void> {
    const caller = identity.getPrincipal();
    const bytes = caller.toUint8Array();
    if (bytes.length !== 29 || bytes[28] !== 2 || issuer.isAnonymous()) throw new ClientError('configuration');
    const callerText = caller.toText();
    const issuerText = issuer.toText();
    const clockId = this.clock.id;
    const now = this.clock.nowNs();
    await this.store.transact(previous => {
      // Never overwrite cleanup or its uncertainty to make room for another login.
      if (previous) throw new ClientError('storage_conflict');
      return { record: { generation, caller: callerText, issuer: issuerText, clockId,
        observedNs: now, busyUntilNs: 0n, phase: 'active', deadlineNs: 0n, issuanceUnknown: false,
        targets: [{ canister: issuerText, admissionUntilNs: now, attempts: 0, nextNs: now }] }, value: undefined };
    });
  }

  private update<T>(generation: string, operation: (record: LogoutJournal, now: bigint) => { record: LogoutJournal; value: T }): Promise<T> {
    return this.store.transact(record => {
      if (!record || record.generation !== generation) throw new ClientError('stale_generation');
      const now = this.clock.nowNs();
      if (record.clockId !== this.clock.id || now < record.observedNs) throw new ClientError('configuration');
      record.observedNs = now;
      return operation(record, now);
    });
  }

  /** Commit BEFORE dispatching admission; expiry is conservatively mapped by the admitted clock. */
  async destination(generation: string, target: Principal, admissionUntilNs: bigint): Promise<void> {
    const key = target.toText();
    const anonymous = target.isAnonymous();
    await this.update(generation, (record, now) => {
      if (record.phase !== 'active') throw new ClientError('stale_generation');
      if (anonymous || admissionUntilNs < now || admissionUntilNs + BigInt(CALL_MS) * MS >= now + MAX_CLEANUP_NS) throw new ClientError('configuration');
      const existing = record.targets.find(item => item.canister === key);
      if (existing) existing.admissionUntilNs = existing.admissionUntilNs > admissionUntilNs ? existing.admissionUntilNs : admissionUntilNs;
      else {
        if (record.targets.length === MAX_TARGETS) throw new ClientError('storage_capacity');
        record.targets.push({ canister: key, admissionUntilNs, attempts: 0, nextNs: now });
      }
      return { record, value: undefined };
    });
  }

  /** Exactly one caller may initiate issuance. Others must reconcile, never reissue. */
  async claimIssuance(generation: string): Promise<boolean> {
    return this.update(generation, record => {
      if (record.phase !== 'active') throw new ClientError('stale_generation');
      const first = !record.issuanceUnknown;
      record.issuanceUnknown = true;
      return { record, value: first };
    });
  }

  async status(generation: string): Promise<LogoutJournal> {
    return this.update(generation, record => ({ record, value: structuredClone(record) }));
  }

  /** Small detached projection; admission still checks phase and identity.
   * Keep the strict transaction and clock observation without copying targets.
   */
  async admissionStatus(generation: string): Promise<Pick<LogoutJournal, 'phase' | 'caller' | 'issuer' | 'observedNs'>> {
    return this.update(generation, record => ({ record, value: {
      phase: record.phase, caller: record.caller, issuer: record.issuer, observedNs: record.observedNs,
    } }));
  }

  /** Give ordinary SDK agents this wrapper, never the retained cleanup identity.
   * Already returned signatures/accepted requests cannot be recalled by this gate.
   */
  guardedIdentity(generation: string, identity: Identity): Identity {
    const caller = identity.getPrincipal().toText();
    const check = async (request: Parameters<Identity['transformRequest']>[0]) => {
      await this.update(generation, record => {
        if (record.phase !== 'active' || record.caller !== caller
          || identity.getPrincipal().toText() !== record.caller) throw new ClientError('stale_generation');
        checkDestination(request, target => record.targets.some(t => t.canister === target));
        return { record, value: undefined };
      });
    };
    return checkedIdentity(identity, () => Principal.fromText(caller), check);
  }

  /** Durable invalidation before any logout I/O. Repeating cannot extend the deadline. */
  async begin(generation: string, deadlineNs: bigint): Promise<void> {
    await this.update(generation, (record, now) => {
      if (record.phase !== 'active') return { record, value: undefined };
      if (deadlineNs <= now || deadlineNs > now + MAX_CLEANUP_NS
        || record.targets.some(target => target.admissionUntilNs + BigInt(CALL_MS) * MS >= deadlineNs)) throw new ClientError('configuration');
      return { record: { ...record, phase: 'cleanup', deadlineNs }, value: undefined };
    });
  }

  /** Resume only an already committed logout. Missing keys never count as confirmation. */
  async cleanup(generation: string, identity: Identity | undefined): Promise<LogoutJournal> {
    for (;;) {
      const current = await this.update<
        { kind: 'active' } | { kind: 'return'; journal: LogoutJournal }
        | { kind: 'continue'; caller: string; signer: Identity }
      >(generation, (record, now) => {
        const next = record.phase === 'cleanup' && cleanupComplete(record, now)
          ? { ...record, phase: 'finished' as const } : record;
        if (next.phase === 'active') return { record: next, value: { kind: 'active' } };
        if (next.phase === 'finished' || !identity) {
          return { record: next, value: { kind: 'return', journal: structuredClone(next) } };
        }
        return { record: next, value: { kind: 'continue', caller: next.caller, signer: identity } };
      });
      if (current.kind === 'active') throw new ClientError('configuration');
      if (current.kind === 'return') {
        if (current.journal.phase === 'finished') await this.transport.dispose(generation);
        return current.journal;
      }
      const signer = current.signer;
      if (signer.getPrincipal().toText() !== current.caller) throw new ClientError('configuration');
      // A tab without a local signer returns its detached journal above. It
      // must not cancel another worker or dispose shared signing material.
      // Persist the attempt and cross-tab dispatch lease before network I/O.
      const claim = await this.update(generation, claimCleanup);
      if (!claim) {
        const waitMs = await this.update(generation, (record, now) => ({ record,
          value: record.phase === 'cleanup' ? cleanupWaitMs(record, now) : undefined }));
        if (waitMs === undefined) continue;
        await this.clock.wait(waitMs);
        continue;
      }
      let response: LogoutResult | undefined;
      try {
        response = await boundedCall(claim.timeoutMs, async signal => {
          const checkLease = async () => {
            // A completed attempt stays closed even if a later claim receives
            // the same timestamp from a coarse clock.
            if (signal.aborted) throw new ClientError('stale_generation');
            await this.update(generation, (record, now) => {
              if (record.phase !== 'cleanup' || record.busyUntilNs !== claim.lease
                || now >= claim.lease || now >= record.deadlineNs
                || record.caller !== signer.getPrincipal().toText()) throw new ClientError('stale_generation');
              return { record, value: undefined };
            });
            if (signal.aborted) throw new ClientError('stale_generation');
          };
          const cleanupIdentity = checkedIdentity(signer, () => signer.getPrincipal(), async request => {
            await checkLease();
            checkDestination(request, target => target === claim.canister);
          }, checkLease);
          await checkLease();
          return this.transport.revoke(Principal.fromText(claim.canister), cleanupIdentity, signal);
        });
      } catch { /* Lost replies and typed failures never acknowledge revocation. */ }
      const jitter = this.random();
      if (!Number.isFinite(jitter) || jitter < 0 || jitter >= 1) throw new ClientError('configuration');
      await this.update(generation, (record, now) => ({ record, value: settleCleanup(record, now, claim, response, jitter) }));
    }
  }
}
