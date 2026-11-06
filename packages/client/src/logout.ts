import { boundedCall, MAX_NETWORK_CALL_MS as CALL_MS } from './bounded-call.js';
import { checkedIdentity, checkDestination } from './checked-identity.js';
import { Principal } from '@tokostudios/ic-platform/principal';
import type { Identity } from '@tokostudios/ic-platform/agent';
import type { LogoutResult } from './codec.js';
import { ClientError } from './contracts.js';
import { isU64 } from './u64.js';
import { credentialFromBytes } from './codec.js';
import { maximumConfiguredBytes } from './generated/credential.js';
import { IndexedDbLogoutStore, MAX_ATTEMPTS, MAX_TARGETS, type LogoutJournal, type LogoutTarget } from './logout-store.js';

const MS = 1_000_000n;
const MAX_CLEANUP_NS = 300_000n * MS;

function cleanupComplete(record: LogoutJournal & { phase: 'cleanup' | 'finished' }, now: bigint): boolean {
  return now >= record.deadlineNs || record.targets.every(target => target.confirmedUntilNs !== undefined);
}

// One local eligibility rule for dispatch and waiting; remote clock values
// cannot establish that a previously issued credential is no longer usable.
function nextAttemptNs(target: LogoutTarget): bigint | undefined {
  if (target.confirmedUntilNs !== undefined || target.attempts >= MAX_ATTEMPTS) return undefined;
  return target.nextNs;
}

// Synchronous journal transitions; callers commit them through update before I/O.
// One live lease across tabs. A crash consumes its attempt; the lease expires
// after the call bound. Unconfirmed replies never become confirmation by waiting.
function claimCleanup(record: LogoutJournal & { phase: 'cleanup' }, now: bigint) {
  const idle = { record, value: undefined };
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
  if (response && response.tag === 'Revoked' && isU64(response.value) && response.value > 0n) {
    target.confirmedUntilNs = response.value;
  } else target.nextNs = now + BigInt(500 + Math.floor(jitter * 1_000)) * MS;
}

export interface LogoutClock {
  /** Stable local scheduling clock across reloads; reject rollback. No remote expiry guarantee. */
  id: string;
  nowNs(): bigint;
  wait(ms: number): Promise<void>;
}
export interface CredentialClaim {
  sequence: bigint;
  first: boolean;
  saved?: { bytes: Uint8Array; until: bigint };
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
    }, true);
  }

  private update<T>(generation: string, operation: (record: LogoutJournal, now: bigint) => { record: LogoutJournal; value: T }, evidence = false): Promise<T> {
    return this.store.transact(record => {
      if (!record || record.generation !== generation) throw new ClientError('stale_generation');
      const now = this.clock.nowNs();
      if (record.clockId !== this.clock.id || now < record.observedNs) throw new ClientError('configuration');
      record.observedNs = now;
      return operation(record, now);
    }, evidence);
  }

  /** Commit BEFORE dispatching admission; expiry is a local scheduling hint only. */
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

  /** Persist one intent before issuance. Pending sequences remain lookup-only.
   * Completed evidence can advance on proof expiry or when readiness requests
   * refresh through the sequence used by an expired receiver lease. */
  async claimCredential(generation: string, accountAvailable = true, refreshThrough?: bigint): Promise<CredentialClaim> {
    if (refreshThrough !== undefined && !isU64(refreshThrough)) throw new ClientError('configuration');
    return this.update<CredentialClaim>(generation, (record, now) => {
      if (record.phase !== 'active') throw new ClientError('stale_generation');
      const op = record.issuance;
      if (!accountAvailable && (!op || (op.sequence === 0n && !op.bytes))) throw new ClientError('account_required');
      if (op?.bytes && op.untilNs! > now && (refreshThrough === undefined || op.sequence > refreshThrough)) return { record, value: {
        sequence: op.sequence, first: false, saved: { bytes: new Uint8Array(op.bytes), until: op.untilNs! },
      } };
      if (op && !op.bytes) return { record, value: { sequence: op.sequence, first: false } };
      const sequence = op ? op.sequence + 1n : 0n;
      if (!isU64(sequence)) throw new ClientError('storage_capacity');
      record.issuanceUnknown = true;
      record.issuance = { sequence };
      return { record, value: { sequence, first: true } };
    }, true);
  }

  /** Retain exact returned evidence before admission. Conflicting replies never
   * replace the committed result or extend its original deadline. */
  async completeCredential(generation: string, sequence: bigint, bytes: Uint8Array, untilNs: bigint): Promise<void> {
    if (bytes.length < 1 || bytes.length > maximumConfiguredBytes) throw new ClientError('invalid_material');
    const owned = new Uint8Array(bytes);
    credentialFromBytes(owned, maximumConfiguredBytes);
    await this.update(generation, record => {
      if (record.phase !== 'active') throw new ClientError('stale_generation');
      const op = record.issuance;
      if (!op || op.sequence !== sequence || !isU64(untilNs) || untilNs === 0n) throw new ClientError('retrieval_expired');
      if (op.bytes && (op.untilNs !== untilNs || op.bytes.length !== owned.length
        || op.bytes.some((byte, index) => byte !== owned[index]))) throw new ClientError('storage_conflict');
      record.issuance = { sequence, bytes: owned, untilNs };
      return { record, value: undefined };
    }, true);
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
      if (deadlineNs <= now || deadlineNs > now + MAX_CLEANUP_NS) throw new ClientError('configuration');
      if (record.issuance) {
        delete record.issuance.bytes;
        delete record.issuance.untilNs;
      }
      return { record: { ...record, phase: 'cleanup', deadlineNs }, value: undefined };
    });
  }

  /** Resume only an already committed logout. Missing keys never count as confirmation. */
  async cleanup(generation: string, identity: Identity | undefined): Promise<LogoutJournal> {
    for (;;) {
      const current = await this.update<
        { kind: 'active' } | { kind: 'return'; journal: LogoutJournal }
        | { kind: 'wait'; milliseconds: number }
        | { kind: 'dispatch'; claim: NonNullable<ReturnType<typeof claimCleanup>['value']>; signer: Identity }
      >(generation, (record, now) => {
        const next = record.phase === 'cleanup' && cleanupComplete(record, now)
          ? { ...record, phase: 'finished' as const } : record;
        if (next.phase === 'active') return { record: next, value: { kind: 'active' } };
        if (next.phase === 'finished' || !identity) {
          return { record: next, value: { kind: 'return', journal: structuredClone(next) } };
        }
        if (identity.getPrincipal().toText() !== next.caller) throw new ClientError('configuration');
        // Select waiting or persist the attempt/lease in this same isolated
        // transaction. Signing still rechecks the committed lease after awaits.
        const { record: claimed, value: claim } = claimCleanup(next, now);
        return claim
          ? { record: claimed, value: { kind: 'dispatch', claim, signer: identity } }
          : { record: claimed, value: { kind: 'wait', milliseconds: cleanupWaitMs(claimed, now) } };
      });
      if (current.kind === 'active') throw new ClientError('configuration');
      if (current.kind === 'return') {
        if (current.journal.phase === 'finished') await this.transport.dispose(generation);
        return current.journal;
      }
      if (current.kind === 'wait') {
        await this.clock.wait(current.milliseconds);
        continue;
      }
      const { signer, claim } = current;
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
