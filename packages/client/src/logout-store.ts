import { strictTransaction } from './idb-transaction.js';
import { ClientError } from './contracts.js';

/** Non-secret cleanup state. This store never holds a signing key or token. */
export interface LogoutTarget {
  canister: string;
  admissionUntilNs: bigint;
  attempts: number;
  nextNs: bigint;
  confirmedUntilNs?: bigint;
}
export interface LogoutJournal {
  generation: string;
  caller: string;
  issuer: string;
  clockId: string;
  observedNs: bigint;
  busyUntilNs: bigint;
  phase: 'active' | 'cleanup' | 'finished';
  deadlineNs: bigint;
  issuanceUnknown: boolean;
  targets: LogoutTarget[];
}

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
      const fail = (cause: unknown) => { abandoned = true; clearTimeout(timer); reject(new ClientError('storage_unavailable', { cause })); };
      const timer = setTimeout(() => fail(new Error('open timeout')), 5_000);
      let request: IDBOpenDBRequest;
      try { request = factory.open(databaseName, 1); } catch (cause) { fail(cause); return; }
      request.onblocked = () => fail(new Error('blocked'));
      request.onerror = () => fail(request.error);
      request.onupgradeneeded = event => {
        if (abandoned || event.oldVersion !== 0) { request.transaction?.abort(); return; }
        request.result.createObjectStore('journal').put(FORMAT, 'format');
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
  transact<T>(operation: (record: LogoutJournal | undefined) => { record: LogoutJournal; value: T } | undefined): Promise<T | undefined> {
    return strictTransaction<T | undefined>(
      () => this.db.transaction('journal', 'readwrite', { durability: 'strict' }),
      5_000, cause => new ClientError('storage_unavailable', { cause }),
      (tx, result, fail) => {
        const bucket = tx.objectStore('journal');
        if (bucket.keyPath !== null || bucket.autoIncrement || bucket.indexNames.length) throw new ClientError('storage_corrupt');
        const profile = bucket.get('format');
        profile.onsuccess = () => {
          if (profile.result !== FORMAT) { fail(new ClientError('storage_corrupt')); return; }
          const request = bucket.get('login');
          request.onsuccess = () => {
            try {
              if (request.result !== undefined) validate(request.result);
              const change = operation(request.result);
              if (change) { validate(change.record); bucket.put(change.record, 'login'); }
              result(change?.value);
            } catch (cause) { fail(cause); }
          };
        };
      },
    );
  }
}
