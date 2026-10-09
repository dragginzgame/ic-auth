import { ClientError, type StoredToken, type StoreSnapshot, type TokenStore } from './contracts.js';

export interface IndexedDbTokenStoreOptions {
  factory: IDBFactory;
  /** A dedicated application-owned database; never an existing business DB. */
  databaseName: string;
  maximumEntries: number;
  maximumEntryBytes: number;
  maximumKeyBytes: number;
  timeoutMs: number;
}

const FORMAT = 'ic-auth-token-store-v1';
const TOKENS = 'tokens';
const METADATA = 'metadata';
const U64_MAX = (1n << 64n) - 1n;
const u64 = (value: unknown): value is bigint => typeof value === 'bigint' && value >= 0n && value <= U64_MAX;
const bounded = (value: number, maximum: number) => Number.isSafeInteger(value) && value >= 1 && value <= maximum;
const object = (value: unknown): value is Record<string, unknown> => typeof value === 'object' && value !== null;
const fields = (value: Record<string, unknown>, names: string[]) => Object.keys(value).length === names.length && names.every(name => Object.hasOwn(value, name));
interface Metadata {
  format: typeof FORMAT;
  maximumEntries: number;
  maximumEntryBytes: number;
  maximumKeyBytes: number;
  revision: bigint;
}

function metadata(value: unknown, options: IndexedDbTokenStoreOptions): Metadata {
  if (!object(value) || !fields(value, ['format', 'maximumEntries', 'maximumEntryBytes', 'maximumKeyBytes', 'revision'])
    || value.format !== FORMAT || !u64(value.revision)) throw new ClientError('storage_corrupt');
  if (value.maximumEntries !== options.maximumEntries || value.maximumEntryBytes !== options.maximumEntryBytes
    || value.maximumKeyBytes !== options.maximumKeyBytes) throw new ClientError('configuration');
  return { format: FORMAT, maximumEntries: options.maximumEntries, maximumEntryBytes: options.maximumEntryBytes,
    maximumKeyBytes: options.maximumKeyBytes, revision: value.revision };
}

function entry(value: unknown, maximum: number): value is StoredToken | undefined {
  if (value === undefined) return true;
  if (!object(value)) return false;
  const bytes = (item: unknown): item is Uint8Array => item instanceof Uint8Array && item.byteLength > 0 && item.byteLength <= maximum;
  switch (value.kind) {
    case 'prepare': return fields(value, ['kind', 'request', 'candidate', 'deadlineNs']) && bytes(value.request)
      && typeof value.candidate === 'number' && Number.isSafeInteger(value.candidate) && value.candidate >= 0 && value.candidate < 16
      && u64(value.deadlineNs) && value.deadlineNs > 0n;
    case 'retrieve': return fields(value, ['kind', 'request', 'prepared', 'deadlineNs']) && bytes(value.request) && bytes(value.prepared)
      && value.request.byteLength + value.prepared.byteLength <= maximum && u64(value.deadlineNs) && value.deadlineNs > 0n;
    case 'ready': return fields(value, ['kind', 'token']) && bytes(value.token);
    default: return false;
  }
}

function snapshot(value: unknown, meta: Metadata): StoreSnapshot {
  if (value === undefined) return { revision: 0n, entry: undefined };
  if (!object(value) || !fields(value, ['revision', 'entry']) || !u64(value.revision) || value.revision === 0n
    || value.revision > meta.revision || !entry(value.entry, meta.maximumEntryBytes)) throw new ClientError('storage_corrupt');
  return { revision: value.revision, entry: value.entry };
}

function storageFailure(cause: unknown): ClientError {
  if (cause instanceof ClientError) return cause;
  return new ClientError(object(cause) && cause.name === 'QuotaExceededError' ? 'storage_capacity' : 'storage_unavailable', { cause });
}

/** Dedicated, opt-in storage. Writes resolve only after strict transaction commit. */
export class IndexedDbTokenStore implements TokenStore {
  #closed = false;
  private constructor(private readonly database: IDBDatabase, private readonly options: IndexedDbTokenStoreOptions) {
    database.onversionchange = () => this.close();
    database.onclose = () => { this.#closed = true; };
  }

  static async open(options: IndexedDbTokenStoreOptions): Promise<IndexedDbTokenStore> {
    const fixed = { ...options };
    if (!fixed.factory || typeof fixed.factory.open !== 'function' || fixed.databaseName.length === 0 || fixed.databaseName.length > 128
      || !bounded(fixed.maximumEntries, 1024) || !bounded(fixed.maximumEntryBytes, 2_097_152)
      || !bounded(fixed.maximumKeyBytes, 131_072) || !bounded(fixed.timeoutMs, 60_000)) throw new ClientError('configuration');
    const database = await new Promise<IDBDatabase>((resolve, reject) => {
      let abandoned = false;
      const fail = (cause: unknown) => { abandoned = true; clearTimeout(timer); reject(storageFailure(cause)); };
      const timer = setTimeout(() => fail(new ClientError('timeout')), fixed.timeoutMs);
      let request: IDBOpenDBRequest;
      try { request = fixed.factory.open(fixed.databaseName, 1); }
      catch (cause) { fail(cause); return; }
      request.onblocked = () => fail(new ClientError('storage_unavailable'));
      request.onerror = () => fail(request.error);
      request.onupgradeneeded = event => {
        if (abandoned) { request.transaction?.abort(); return; }
        try {
          if (event.oldVersion !== 0) throw new ClientError('storage_corrupt');
          request.result.createObjectStore(TOKENS);
          request.result.createObjectStore(METADATA).put({ format: FORMAT, maximumEntries: fixed.maximumEntries,
            maximumEntryBytes: fixed.maximumEntryBytes, maximumKeyBytes: fixed.maximumKeyBytes, revision: 0n } satisfies Metadata, 'profile');
        } catch (cause) { request.transaction?.abort(); fail(cause); }
      };
      request.onsuccess = () => {
        clearTimeout(timer);
        if (abandoned) request.result.close(); else resolve(request.result);
      };
    });
    const store = new IndexedDbTokenStore(database, fixed);
    try {
      if (database.objectStoreNames.length !== 2 || !database.objectStoreNames.contains(TOKENS)
        || !database.objectStoreNames.contains(METADATA)) throw new ClientError('storage_corrupt');
      await store.#transaction<void>('readonly', (tx, result, fail) => {
        for (const name of [TOKENS, METADATA]) {
          const bucket = tx.objectStore(name);
          if (bucket.keyPath !== null || bucket.autoIncrement || bucket.indexNames.length !== 0) throw new ClientError('storage_corrupt');
        }
        const request = tx.objectStore(METADATA).get('profile');
        request.onsuccess = () => {
          try { metadata(request.result, fixed); result(); } catch (cause) { fail(cause); }
        };
      });
      return store;
    } catch (cause) { store.close(); throw storageFailure(cause); }
  }

  /** Close this connection without clearing records or cancelling committed work. */
  close(): void { this.#closed = true; this.database.close(); }

  #key(key: string): void {
    if (key.length === 0 || key.length > this.options.maximumKeyBytes
      || new TextEncoder().encode(key).byteLength > this.options.maximumKeyBytes) throw new ClientError('invalid_material');
  }

  #transaction<T>(mode: IDBTransactionMode, execute: (tx: IDBTransaction, result: (value: T) => void, fail: (cause: unknown) => void) => void): Promise<T> {
    if (this.#closed) return Promise.reject(new ClientError('storage_unavailable'));
    return new Promise((resolve, reject) => {
      let tx: IDBTransaction;
      try {
        tx = this.database.transaction([TOKENS, METADATA], mode, { durability: 'strict' });
        if (tx.durability !== 'strict') { tx.abort(); throw new ClientError('storage_unavailable'); }
      } catch (cause) { reject(storageFailure(cause)); return; }
      let completed = false;
      let value: T;
      let failure: unknown;
      const fail = (cause: unknown) => {
        failure = storageFailure(cause);
        try { tx.abort(); }
        catch { clearTimeout(timer); reject(failure); }
      };
      const timer = setTimeout(() => fail(new ClientError('timeout')), this.options.timeoutMs);
      tx.onabort = () => { clearTimeout(timer); reject(failure ?? storageFailure(tx.error)); };
      tx.oncomplete = () => {
        clearTimeout(timer);
        if (failure) reject(failure);
        else if (!completed) reject(new ClientError('storage_corrupt'));
        else resolve(value);
      };
      // Request errors retain the native abort behavior; never resolve on put success.
      try { execute(tx, result => { completed = true; value = result; }, fail); }
      catch (cause) { fail(cause); }
    });
  }

  async read(key: string): Promise<StoreSnapshot> {
    this.#key(key);
    return this.#transaction('readonly', (tx, result, fail) => {
      const profile = tx.objectStore(METADATA).get('profile');
      profile.onsuccess = () => {
        try {
          const meta = metadata(profile.result, this.options);
          const request = tx.objectStore(TOKENS).get(key);
          request.onsuccess = () => {
            try { result(snapshot(request.result, meta)); } catch (cause) { fail(cause); }
          };
        } catch (cause) { fail(cause); }
      };
    });
  }

  async write(key: string, expectedRevision: bigint, material: StoredToken | undefined): Promise<boolean> {
    this.#key(key);
    if (!u64(expectedRevision)) throw new ClientError('invalid_material');
    if (!entry(material, this.options.maximumEntryBytes)) throw new ClientError('invalid_material');
    // Detach synchronously, before transactions wait for another connection.
    const fixed = structuredClone(material);
    return this.#transaction('readwrite', (tx, result, fail) => {
      const profiles = tx.objectStore(METADATA);
      const tokens = tx.objectStore(TOKENS);
      const profile = profiles.get('profile');
      profile.onsuccess = () => {
        try {
          const meta = metadata(profile.result, this.options);
          const request = tokens.get(key);
          request.onsuccess = () => {
            try {
              const saved = snapshot(request.result, meta);
              if (saved.revision !== expectedRevision) { result(false); return; }
              const put = () => {
                if (meta.revision === U64_MAX) { fail(new ClientError('storage_capacity')); return; }
                const revision = meta.revision + 1n;
                profiles.put({ ...meta, revision }, 'profile');
                // Tombstones preserve revision history; no eviction or deletion.
                tokens.put({ revision, entry: fixed } satisfies StoreSnapshot, key);
                result(true);
              };
              if (saved.revision !== 0n) { put(); return; }
              const count = tokens.count();
              count.onsuccess = () => {
                if (count.result >= meta.maximumEntries) fail(new ClientError('storage_capacity')); else put();
              };
            } catch (cause) { fail(cause); }
          };
        } catch (cause) { fail(cause); }
      };
    });
  }
}
