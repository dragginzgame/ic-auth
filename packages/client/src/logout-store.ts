import { ClientError } from './contracts.js';
import { isU64 as time } from './u64.js';
import { maximumConfiguredBytes } from './generated/credential.js';

/** Caller-bound admission evidence and cleanup state; never a signing key. */
export interface LogoutTarget {
  canister: string;
  admissionUntilNs: bigint;
  attempts: number;
  nextNs: bigint;
  confirmedUntilNs?: bigint;
}
/** A retained intent has no expiry or evidence. Completed metadata always has
 * its original expiry; evidence transactions also load the separate frame. */
type Issuance =
  | { sequence: bigint; untilNs?: never; bytes?: never }
  | { sequence: bigint; untilNs: bigint; bytes?: Uint8Array };

interface JournalCommon {
  generation: string;
  caller: string;
  issuer: string;
  clockId: string;
  observedNs: bigint;
  issuanceUnknown: boolean;
  issuance?: Issuance;
  targets: LogoutTarget[];
}

/** Active logins have neither a cleanup deadline nor a dispatch lease. The
 * flat record shape is also the existing persisted profile; no conversion or
 * migration is performed when narrowing its phase in memory.
 */
export type LogoutJournal = JournalCommon & (
  | { phase: 'active'; deadlineNs: 0n; busyUntilNs: 0n }
  | { phase: 'cleanup'; deadlineNs: bigint; busyUntilNs: bigint }
  | { phase: 'finished'; deadlineNs: bigint; busyUntilNs: bigint }
);

const FORMAT = 'ic-auth-local-logout-journal-v1';
export const MAX_TARGETS = 64;
export const MAX_ATTEMPTS = 8;
const object = (x: unknown): x is Record<string, unknown> => typeof x === 'object' && x !== null;
const fields = (x: Record<string, unknown>, names: string[]) => Object.keys(x).length === names.length && names.every(name => Object.hasOwn(x, name));
const text = (x: unknown): x is string => typeof x === 'string' && x.length > 0 && x.length <= 128;

function validate(value: unknown, evidence = false): asserts value is LogoutJournal {
  if (!object(value) || !fields(value, ['generation', 'caller', 'issuer', 'clockId', 'observedNs', 'busyUntilNs', 'phase', 'deadlineNs', 'issuanceUnknown', 'targets', ...(value.issuance === undefined ? [] : ['issuance'])]) || !text(value.generation) || !text(value.caller) || !text(value.issuer)
    || !text(value.clockId) || !time(value.observedNs) || !time(value.busyUntilNs) || !time(value.deadlineNs)
    || !['active', 'cleanup', 'finished'].includes(String(value.phase))
    || typeof value.issuanceUnknown !== 'boolean' || !Array.isArray(value.targets)
    || value.targets.length < 1 || value.targets.length > MAX_TARGETS) throw new ClientError('storage_corrupt');
  if (value.issuance !== undefined) {
    const op = value.issuance;
    if (!object(op) || !time(op.sequence) || !value.issuanceUnknown
      || !fields(op, ['sequence', ...(op.untilNs === undefined ? [] : ['untilNs']), ...(op.bytes === undefined ? [] : ['bytes'])])
      || (op.untilNs !== undefined && (!time(op.untilNs) || op.untilNs === 0n || (evidence && op.bytes === undefined)))
      || (value.phase !== 'active' && op.untilNs !== undefined)
      || (op.bytes !== undefined && (!(op.bytes instanceof Uint8Array) || op.bytes.length < 1
        || op.bytes.length > maximumConfiguredBytes || !time(op.untilNs) || op.untilNs === 0n))) throw new ClientError('storage_corrupt');
  }
  const seen = new Set<string>();
  for (const target of value.targets) {
    if (!object(target) || !fields(target, ['canister', 'admissionUntilNs', 'attempts', 'nextNs', ...(target.confirmedUntilNs === undefined ? [] : ['confirmedUntilNs'])]) || !text(target.canister) || seen.has(target.canister)
      || !time(target.admissionUntilNs) || !time(target.nextNs)
      || !Number.isSafeInteger(target.attempts) || Number(target.attempts) < 0 || Number(target.attempts) > MAX_ATTEMPTS
      || (target.confirmedUntilNs !== undefined && !time(target.confirmedUntilNs))) throw new ClientError('storage_corrupt');
    seen.add(target.canister);
  }
  if ((value.phase === 'active' && (value.deadlineNs !== 0n || value.busyUntilNs !== 0n
    || value.targets.some(t => t.attempts !== 0 || t.confirmedUntilNs !== undefined)))
    || (value.phase !== 'active' && value.deadlineNs === 0n)
    || !seen.has(value.issuer)) throw new ClientError('storage_corrupt');
}

/** One login per dedicated DB. No automatic reset, migration, eviction or key custody. */
export class IndexedDbLogoutStore {
  private constructor(private readonly db: IDBDatabase) {
    db.onversionchange = () => db.close();
  }

  static async open(factory: IDBFactory, databaseName: string): Promise<IndexedDbLogoutStore> {
    if (!text(databaseName)) throw new ClientError('configuration');
    const db = await new Promise<IDBDatabase>((resolve, reject) => {
      let abandoned = false;
      const fail = (cause: unknown) => {
        abandoned = true;
        clearTimeout(timer);
        reject(new ClientError('storage_unavailable', { cause }));
      };
      const timer = setTimeout(() => fail(new ClientError('timeout')), 5_000);
      let request: IDBOpenDBRequest;
      try { request = factory.open(databaseName, 1); } catch (cause) { fail(cause); return; }
      request.onblocked = () => fail(new ClientError('storage_unavailable'));
      request.onerror = () => fail(request.error);
      request.onupgradeneeded = event => {
        if (abandoned) { request.transaction?.abort(); return; }
        try {
          if (event.oldVersion !== 0) throw new ClientError('storage_corrupt');
          request.result.createObjectStore('journal').put(FORMAT, 'format');
        } catch (cause) {
          try { request.transaction?.abort(); } catch { /* Already inactive. */ }
          fail(cause);
        }
      };
      request.onsuccess = () => {
        clearTimeout(timer);
        if (abandoned) request.result.close(); else resolve(request.result);
      };
    });
    const store = new IndexedDbLogoutStore(db);
    try {
      if (db.objectStoreNames.length !== 1 || !db.objectStoreNames.contains('journal')) throw new ClientError('storage_corrupt');
      await store.transact(() => undefined);
      return store;
    } catch (cause) { store.close(); throw cause; }
  }

  close(): void { this.db.close(); }

  /** Synchronous mutation, isolated across tabs; resolves only after strict commit. */
  transact<T>(operation: (record: LogoutJournal | undefined) => { record: LogoutJournal; value: T }, evidence?: boolean): Promise<T>;
  transact<T>(operation: (record: LogoutJournal | undefined) => { record: LogoutJournal; value: T } | undefined, evidence?: boolean): Promise<T | undefined>;
  transact<T>(operation: (record: LogoutJournal | undefined) => { record: LogoutJournal; value: T } | undefined, evidence = false): Promise<T | undefined> {
    return new Promise((resolve, reject) => {
      let tx: IDBTransaction;
      try { tx = this.db.transaction('journal', 'readwrite', { durability: 'strict' }); }
      catch (cause) { reject(new ClientError('storage_unavailable', { cause })); return; }
      let outcome: { phase: 'pending' } | { phase: 'staged'; value: T | undefined }
        | { phase: 'failed'; cause: unknown } = { phase: 'pending' };
      const fail = (cause: unknown) => {
        outcome = { phase: 'failed', cause };
        try { tx.abort(); } catch { clearTimeout(timer); reject(cause); }
      };
      const timer = setTimeout(() => fail(new ClientError('timeout')), 5_000);
      tx.onabort = () => {
        clearTimeout(timer);
        reject(outcome.phase === 'failed' ? outcome.cause : new ClientError('storage_unavailable', { cause: tx.error }));
      };
      tx.oncomplete = () => {
        clearTimeout(timer);
        if (outcome.phase === 'failed') reject(outcome.cause);
        else if (outcome.phase === 'pending') reject(new ClientError('storage_corrupt'));
        else resolve(outcome.value);
      };
      if (tx.durability !== 'strict') { fail(new ClientError('storage_unavailable')); return; }
      try {
        const bucket = tx.objectStore('journal');
        if (bucket.keyPath !== null || bucket.autoIncrement || bucket.indexNames.length) throw new ClientError('storage_corrupt');
        const profile = bucket.get('format');
        profile.onsuccess = () => {
          if (outcome.phase === 'failed') return;
          if (profile.result !== FORMAT) { fail(new ClientError('storage_corrupt')); return; }
          const request = bucket.get('login');
          request.onsuccess = () => {
            if (outcome.phase === 'failed') return;
            const apply = (bytes?: unknown) => {
              try {
                const record: unknown = request.result;
                if (record !== undefined) {
                  validate(record);
                  if (record.issuance?.bytes !== undefined) throw new ClientError('storage_corrupt');
                  if (evidence && record.issuance?.untilNs !== undefined) {
                    if (!(bytes instanceof Uint8Array)) throw new ClientError('storage_corrupt');
                    record.issuance.bytes = bytes;
                  } else if (bytes !== undefined) throw new ClientError('storage_corrupt');
                  validate(record, evidence);
                } else if (bytes !== undefined) throw new ClientError('storage_corrupt');
                const change = operation(record);
                if (change) {
                  validate(change.record, evidence);
                  const op = change.record.issuance;
                  const persisted = { ...change.record };
                  if (op) {
                    const { bytes: frame, ...metadata } = op;
                    persisted.issuance = metadata;
                    if (evidence && frame) bucket.put(frame, 'credential');
                  }
                  if (op?.untilNs === undefined) bucket.delete('credential');
                  bucket.put(persisted, 'login');
                }
                // Request success stages only; strict commit publishes both keys.
                outcome = { phase: 'staged', value: change?.value };
              } catch (cause) { fail(cause); }
            };
            if (evidence) {
              const frame = bucket.get('credential');
              frame.onsuccess = () => { if (outcome.phase !== 'failed') apply(frame.result); };
            } else apply();
          };
        };
      } catch (cause) { fail(cause); }
    });
  }
}
