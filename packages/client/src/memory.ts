import { ClientError, type StoredToken, type StoreSnapshot, type TokenStore } from './contracts.js';

/** Volatile reference only. Durable/browser storage owns atomic CAS and custody. */
export class MemoryTokenStore implements TokenStore {
  readonly #entries = new Map<string, StoreSnapshot>();
  #revision = 0n;
  constructor(private readonly maximumEntries: number) {
    if (!Number.isSafeInteger(maximumEntries) || maximumEntries < 1 || maximumEntries > 1024) {
      throw new ClientError('configuration');
    }
  }
  async read(key: string): Promise<StoreSnapshot> {
    return structuredClone(this.#entries.get(key) ?? { revision: 0n, entry: undefined });
  }
  async write(key: string, expected: bigint, entry: StoredToken | undefined): Promise<boolean> {
    if ((this.#entries.get(key)?.revision ?? 0n) !== expected) return false;
    if (!this.#entries.has(key) && this.#entries.size >= this.maximumEntries) {
      throw new ClientError('storage_capacity');
    }
    // Keep tombstone revisions: deletion must not create an ABA race.
    this.#entries.set(key, structuredClone({ revision: ++this.#revision, entry }));
    return true;
  }
}
