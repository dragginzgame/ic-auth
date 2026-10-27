import { ClientError } from './contracts.js';

/** Non-secret cleanup state. This store never holds a signing key or token. */
export interface LogoutTarget {
  canister: string;
  admissionUntilNs: bigint;
  attempts: number;
  nextNs: bigint;
  confirmedUntilNs?: bigint;
}
interface JournalCommon {
  generation: string;
  caller: string;
  issuer: string;
  clockId: string;
  observedNs: bigint;
  issuanceUnknown: boolean;
  targets: LogoutTarget[];
}

/** Active logins have neither a cleanup deadline nor a dispatch lease. The
 * flat record shape is also the existing persisted profile; no conversion or
 * migration is performed when narrowing its phase in memory.
 */
export type LogoutJournal = JournalCommon & (
  | { phase: 'active'; deadlineNs: 0n; busyUntilNs: 0n }
  | { phase: 'cleanup' | 'finished'; deadlineNs: bigint; busyUntilNs: bigint }
);

const FORMAT = 'ic-auth-logout-journal-v1';
const U64_MAX = (1n << 64n) - 1n;
export const MAX_TARGETS = 64;
export const MAX_ATTEMPTS = 8;
const object = (x: unknown): x is Record<string, unknown> => typeof x === 'object' && x !== null;
const fields = (x: Record<string, unknown>, names: string[]) => Object.keys(x).length === names.length && names.every(name => Object.hasOwn(x, name));
const text = (x: unknown): x is string => typeof x === 'string' && x.length > 0 && x.length <= 128;
const time = (x: unknown): x is bigint => typeof x === 'bigint' && x >= 0n && x <= U64_MAX;

function validate(value: unknown): asserts value is LogoutJournal {
  if (!object(value) || !fields(value, ['generation', 'caller', 'issuer', 'clockId', 'observedNs', 'busyUntilNs', 'phase', 'deadlineNs', 'issuanceUnknown', 'targets']) || !text(value.generation) || !text(value.caller) || !text(value.issuer)
    || !text(value.clockId) || !time(value.observedNs) || !time(value.busyUntilNs) || !time(value.deadlineNs)
    || !['active', 'cleanup', 'finished'].includes(String(value.phase))
    || typeof value.issuanceUnknown !== 'boolean' || !Array.isArray(value.targets)
    || value.targets.length < 1 || value.targets.length > MAX_TARGETS) throw new ClientError('storage_corrupt');
  const seen = new Set<string>();
  for (const target of value.targets) {
    if (!object(target) || !fields(target, ['canister', 'admissionUntilNs', 'attempts', 'nextNs', ...(target.confirmedUntilNs === undefined ? [] : ['confirmedUntilNs'])]) || !text(target.canister) || seen.has(target.canister)
      || !time(target.admissionUntilNs) || !time(target.nextNs)
      || !Number.isSafeInteger(target.attempts) || Number(target.attempts) < 0 || Number(target.attempts) > MAX_ATTEMPTS
      || (target.confirmedUntilNs !== undefined && !time(target.confirmedUntilNs))) throw new ClientError('storage_corrupt');
    seen.add(target.canister);
  }
  const deadline = value.deadlineNs;
  if ((value.phase === 'active' && (value.deadlineNs !== 0n || value.busyUntilNs !== 0n
    || value.targets.some(t => t.attempts !== 0 || t.confirmedUntilNs !== undefined)))
    || (value.phase !== 'active' && (value.deadlineNs === 0n || value.targets.some(t => t.admissionUntilNs >= deadline)))
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
  transact<T>(operation: (record: LogoutJournal | undefined) => { record: LogoutJournal; value: T }): Promise<T>;
  transact<T>(operation: (record: LogoutJournal | undefined) => { record: LogoutJournal; value: T } | undefined): Promise<T | undefined>;
  transact<T>(operation: (record: LogoutJournal | undefined) => { record: LogoutJournal; value: T } | undefined): Promise<T | undefined> {
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
            try {
              if (request.result !== undefined) validate(request.result);
              const change = operation(request.result);
              if (change) { validate(change.record); bucket.put(change.record, 'login'); }
              // Request success stages only; strict commit publishes.
              outcome = { phase: 'staged', value: change?.value };
            } catch (cause) { fail(cause); }
          };
        };
      } catch (cause) { fail(cause); }
    });
  }
}
