import { boundedCall, MAX_NETWORK_CALL_MS as CALL_MS } from './bounded-call.js';
import { checkedIdentity, checkDestination } from './checked-identity.js';
import { Principal } from '@icp-sdk/core/principal';
import type { Identity } from '@icp-sdk/core/agent';
import type { LogoutResult } from './codec.js';
import { ClientError } from './contracts.js';
import { IndexedDbLogoutStore, MAX_ATTEMPTS, MAX_TARGETS, type LogoutJournal } from './logout-store.js';

const MS = 1_000_000n;
const MAX_CLEANUP_NS = 300_000n * MS;

function cleanupComplete(record: LogoutJournal, now: bigint): boolean {
  return now >= record.deadlineNs || record.targets.every(target => target.confirmedUntilNs !== undefined);
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
    const now = this.clock.nowNs();
    await this.store.transact(previous => {
      // Never overwrite cleanup or its uncertainty to make room for another login.
      if (previous) throw new ClientError('storage_conflict');
      return { record: { generation, caller: caller.toText(), issuer: issuer.toText(), clockId: this.clock.id,
        observedNs: now, busyUntilNs: 0n, phase: 'active', deadlineNs: 0n, issuanceUnknown: false,
        targets: [{ canister: issuer.toText(), admissionUntilNs: now, attempts: 0, nextNs: now }] }, value: undefined };
    });
  }

  private async update<T>(generation: string, operation: (record: LogoutJournal, now: bigint) => T): Promise<T> {
    const value = await this.store.transact(record => {
      if (!record || record.generation !== generation) throw new ClientError('stale_generation');
      const now = this.clock.nowNs();
      if (record.clockId !== this.clock.id || now < record.observedNs) throw new ClientError('configuration');
      record.observedNs = now;
      const value = operation(record, now);
      return { record, value };
    });
    return value as T;
  }

  /** Commit BEFORE dispatching admission; expiry is conservatively mapped by the admitted clock. */
  async destination(generation: string, target: Principal, admissionUntilNs: bigint): Promise<void> {
    await this.update(generation, (record, now) => {
      if (record.phase !== 'active') throw new ClientError('stale_generation');
      if (target.isAnonymous() || admissionUntilNs < now || admissionUntilNs + BigInt(CALL_MS) * MS >= now + MAX_CLEANUP_NS) throw new ClientError('configuration');
      const existing = record.targets.find(item => item.canister === target.toText());
      if (existing) existing.admissionUntilNs = existing.admissionUntilNs > admissionUntilNs ? existing.admissionUntilNs : admissionUntilNs;
      else {
        if (record.targets.length === MAX_TARGETS) throw new ClientError('storage_capacity');
        record.targets.push({ canister: target.toText(), admissionUntilNs, attempts: 0, nextNs: now });
      }
    });
  }

  /** Exactly one caller may initiate issuance. Others must reconcile, never reissue. */
  async claimIssuance(generation: string): Promise<boolean> {
    return this.update(generation, record => {
      if (record.phase !== 'active') throw new ClientError('stale_generation');
      const first = !record.issuanceUnknown;
      record.issuanceUnknown = true;
      return first;
    });
  }

  async status(generation: string): Promise<LogoutJournal> {
    return this.update(generation, record => structuredClone(record));
  }

  /** Give ordinary SDK agents this wrapper, never the retained cleanup identity.
   * Already returned signatures/accepted requests cannot be recalled by this gate.
   */
  guardedIdentity(generation: string, identity: Identity): Identity {
    const principal = identity.getPrincipal();
    const check = async (request: Parameters<Identity['transformRequest']>[0]) => {
      await this.update(generation, record => {
        if (record.phase !== 'active' || record.caller !== principal.toText()
          || identity.getPrincipal().toText() !== record.caller) throw new ClientError('stale_generation');
        checkDestination(request, target => record.targets.some(t => t.canister === target));
      });
    };
    return checkedIdentity(identity, () => principal, check);
  }

  /** Durable invalidation before any logout I/O. Repeating cannot extend the deadline. */
  async begin(generation: string, deadlineNs: bigint): Promise<void> {
    await this.update(generation, (record, now) => {
      if (record.phase !== 'active') return;
      if (deadlineNs <= now || deadlineNs > now + MAX_CLEANUP_NS
        || record.targets.some(target => target.admissionUntilNs + BigInt(CALL_MS) * MS >= deadlineNs)) throw new ClientError('configuration');
      record.phase = 'cleanup';
      record.deadlineNs = deadlineNs;
    });
  }

  /** Resume only an already committed logout. Missing keys never count as confirmation. */
  async cleanup(generation: string, identity: Identity | undefined): Promise<LogoutJournal> {
    for (;;) {
      const current = await this.update(generation, (record, now) => {
        if (record.phase === 'cleanup' && cleanupComplete(record, now)) record.phase = 'finished';
        return structuredClone(record);
      });
      if (current.phase === 'active') throw new ClientError('configuration');
      if (current.phase === 'finished') { await this.transport.dispose(generation); return current; }
      if (identity && identity.getPrincipal().toText() !== current.caller) throw new ClientError('configuration');
      // A tab without a local handle must not cancel another tab's worker or
      // delete its shared key. Completion depends on remote evidence or deadline.
      if (!identity) return current;
      // Persist a claim before dispatch. One live dispatch lease across tabs;
      // a crash consumes the attempt and the claim expires after its call bound.
      const claim = await this.update(generation, (record, now) => {
        if (record.phase !== 'cleanup') return undefined;
        if (cleanupComplete(record, now)) {
          record.phase = 'finished'; return undefined;
        }
        if (record.busyUntilNs > now) return undefined;
        const target = record.targets.find(t => t.confirmedUntilNs === undefined && t.attempts < MAX_ATTEMPTS && t.nextNs <= now);
        if (!target) return undefined;
        if (target.attempts === MAX_ATTEMPTS - 1 && now <= target.admissionUntilNs) {
          target.nextNs = target.admissionUntilNs + 1n; return undefined;
        }
        target.attempts++;
        target.nextNs = now + BigInt(CALL_MS) * MS;
        record.busyUntilNs = target.nextNs;
        return { lease: record.busyUntilNs, canister: target.canister, attempt: target.attempts, timeoutMs: Math.max(1, Math.min(CALL_MS, Number((record.deadlineNs - now) / MS))) };
      });
      if (!claim) {
        const status = await this.status(generation);
        if (status.phase === 'finished') continue;
        let next = status.targets.filter(t => t.confirmedUntilNs === undefined && t.attempts < MAX_ATTEMPTS)
          .reduce((time, t) => t.nextNs < time ? t.nextNs : time, status.deadlineNs);
        if (status.busyUntilNs > next) next = status.busyUntilNs < status.deadlineNs ? status.busyUntilNs : status.deadlineNs;
        await this.clock.wait(Math.max(1, Math.min(1_000, Number((next - this.clock.nowNs()) / MS))));
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
                || record.caller !== identity.getPrincipal().toText()) throw new ClientError('stale_generation');
            });
            if (signal.aborted) throw new ClientError('stale_generation');
          };
          const cleanupIdentity = checkedIdentity(identity, () => identity.getPrincipal(), async request => {
            await checkLease();
            checkDestination(request, target => target === claim.canister);
          }, checkLease);
          await checkLease();
          return this.transport.revoke(Principal.fromText(claim.canister), cleanupIdentity, signal);
        });
      } catch { /* Lost replies and typed failures never acknowledge revocation. */ }
      const jitter = this.random();
      if (!Number.isFinite(jitter) || jitter < 0 || jitter >= 1) throw new ClientError('configuration');
      await this.update(generation, (record, now) => {
        if (record.phase !== 'cleanup' || now >= record.deadlineNs) return;
        const target = record.targets.find(t => t.canister === claim.canister)!;
        if (target.attempts !== claim.attempt || record.busyUntilNs !== claim.lease) return;
        record.busyUntilNs = 0n;
        if (response && response.tag === 'Revoked' && typeof response.value === 'bigint'
          && response.value > 0n && response.value <= (1n << 64n) - 1n) {
          target.confirmedUntilNs = response.value;
        } else target.nextNs = now + BigInt(500 + Math.floor(jitter * 1_000)) * MS;
      });
    }
  }
}
