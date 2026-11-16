import type { Principal } from '@tokostudios/ic-platform/principal';
import { decodeRequest, maximumConfiguredBytes } from './generated/credential.js';

export type JournalFailure = 'configuration' | 'storage_unavailable' | 'storage_corrupt'
  | 'login_exists' | 'not_current' | 'clock_changed' | 'destination_limit'
  | 'account_required' | 'issuance_unrecoverable' | 'sequence_exhausted'
  | 'issuance_conflict' | 'invalid_material' | 'credential_expired';
export class JournalError extends Error {
  readonly code: JournalFailure;
  constructor(code: JournalFailure, options?: ErrorOptions) {
    super(code, options);
    this.name = 'JournalError';
    this.code = code;
  }
}
export interface JournalClock { readonly id: string; nowNs(): bigint }
export interface LoginStart { generation: string; caller: Principal; issuer: Principal }
export interface JournalStatus {
  readonly generation: string;
  readonly phase: 'active' | 'cleanup' | 'finished';
  readonly deadlineNs: bigint;
  readonly issuanceUnknown: boolean;
  readonly sequence?: bigint;
  readonly destinations: readonly string[];
}
export interface CredentialClaimOptions { readonly accountAvailable: boolean; readonly refreshThrough?: bigint }
export type CredentialClaim =
  | { readonly kind: 'saved'; readonly sequence: bigint; readonly bytes: Uint8Array; readonly untilNs: bigint }
  | { readonly kind: 'issue'; readonly sequence: bigint; readonly signer: 'account' }
  | { readonly kind: 'renew'; readonly sequence: bigint; readonly signer: 'browser' }
  | { readonly kind: 'reconcile'; readonly sequence: bigint; readonly signer: 'account' | 'browser' };
interface Target { canister: string; attempts: number; nextNs: bigint; confirmedUntilNs?: bigint }
interface RecordState {
  generation: string; caller: string; issuer: string; clockId: string; observedNs: bigint;
  targets: Target[]; issuanceUnknown: boolean; issuance?: { sequence: bigint; untilNs?: bigint };
  phase: 'active' | 'cleanup' | 'finished'; deadlineNs: bigint; busyUntilNs: bigint;
}
const FORMAT = 'ic-auth-login-cleanup-journal-v1';
const CLEANUP_NS = 300_000_000_000n;
const ATTEMPT_NS = 5_000_000_000n;
const text = (v: unknown): v is string => typeof v === 'string' && v.length > 0 && v.length <= 128;
const u64 = (v: unknown): v is bigint => typeof v === 'bigint' && v >= 0n && v <= 0xffff_ffff_ffff_ffffn;
const object = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null;
const fields = (v: Record<string, unknown>, names: string[]) =>
  Object.keys(v).length === names.length && names.every(name => Object.hasOwn(v, name));
function refuse(code: JournalFailure): never { throw new JournalError(code); }

// This is the frozen private v1 metadata profile. Evidence bytes have a separate key.
function validate(v: unknown): asserts v is RecordState {
  if (!object(v) || !fields(v, ['generation', 'caller', 'issuer', 'clockId', 'observedNs',
    'targets', 'issuanceUnknown', 'phase', 'deadlineNs', 'busyUntilNs',
    ...(v.issuance === undefined ? [] : ['issuance'])])
    || !text(v.generation) || !text(v.caller) || !text(v.issuer) || !text(v.clockId)
    || !u64(v.observedNs) || !u64(v.deadlineNs) || !u64(v.busyUntilNs)
    || (v.phase !== 'active' && v.phase !== 'cleanup' && v.phase !== 'finished')
    || typeof v.issuanceUnknown !== 'boolean' || !Array.isArray(v.targets)
    || v.targets.length < 1 || v.targets.length > 64) refuse('storage_corrupt');
  if (v.issuance !== undefined) {
    const op = v.issuance;
    if (!object(op) || !fields(op, ['sequence', ...(op.untilNs === undefined ? [] : ['untilNs'])])
      || !u64(op.sequence) || !v.issuanceUnknown
      || (op.untilNs !== undefined && (!u64(op.untilNs) || op.untilNs === 0n || v.phase !== 'active')))
      refuse('storage_corrupt');
  }
  const seen = new Set<string>();
  for (const t of v.targets) {
    if (!object(t) || !fields(t, ['canister', 'attempts', 'nextNs',
      ...(t.confirmedUntilNs === undefined ? [] : ['confirmedUntilNs'])])
      || !text(t.canister) || seen.has(t.canister) || !u64(t.nextNs)
      || !Number.isSafeInteger(t.attempts) || Number(t.attempts) < 0 || Number(t.attempts) > 8
      || (t.confirmedUntilNs !== undefined && !u64(t.confirmedUntilNs))
      || (v.phase === 'active' && (t.attempts !== 0 || t.confirmedUntilNs !== undefined)))
      refuse('storage_corrupt');
    seen.add(t.canister);
  }
  if (!seen.has(v.issuer) || (v.phase === 'active'
    ? v.deadlineNs !== 0n || v.busyUntilNs !== 0n : v.deadlineNs === 0n)) refuse('storage_corrupt');
}
function status(record: RecordState): JournalStatus {
  return {
    generation: record.generation, phase: record.phase, deadlineNs: record.deadlineNs,
    issuanceUnknown: record.issuanceUnknown,
    ...(record.issuance === undefined ? {} : { sequence: record.issuance.sequence }),
    destinations: record.targets.map(t => t.canister),
  };
}

// Structural Wire admission only; the receiver still authenticates the signed proof.
function validateFrame(bytes: unknown, code: 'invalid_material' | 'storage_corrupt'): asserts bytes is Uint8Array {
  if (!(bytes instanceof Uint8Array) || bytes.length < 1 || bytes.length > maximumConfiguredBytes) refuse(code);
  try { decodeRequest(bytes); } catch (cause) { throw new JournalError(code, { cause }); }
}

/** One dedicated DB, no reset/migration, signing keys or network effects. */
export class LoginJournal {
  readonly #db: IDBDatabase;
  readonly #clock: JournalClock;
  private constructor(db: IDBDatabase, clock: JournalClock) {
    this.#db = db;
    // Capture the clock identity; a later caller mutation cannot rebind this handle.
    this.#clock = { id: clock.id, nowNs: () => clock.nowNs() };
    db.onversionchange = () => db.close();
  }

  static async open(factory: IDBFactory, name: string, clock: JournalClock): Promise<LoginJournal> {
    if (!text(name) || !text(clock.id)) refuse('configuration');
    const capturedClock = { id: clock.id, nowNs: () => clock.nowNs() };
    const db = await new Promise<IDBDatabase>((resolve, reject) => {
      let abandoned = false;
      const fail = (cause: unknown) => {
        abandoned = true;
        clearTimeout(timer);
        reject(new JournalError('storage_unavailable', { cause }));
      };
      const timer = setTimeout(() => fail(new Error('open timeout')), 5_000);
      let request: IDBOpenDBRequest;
      try { request = factory.open(name, 1); } catch (cause) { fail(cause); return; }
      request.onblocked = () => fail(new Error('open blocked'));
      request.onerror = () => fail(request.error);
      request.onupgradeneeded = event => {
        if (abandoned) { request.transaction?.abort(); return; }
        try {
          if (event.oldVersion !== 0) refuse('storage_corrupt');
          request.result.createObjectStore('journal').put(FORMAT, 'format');
        } catch (cause) { request.transaction?.abort(); fail(cause); }
      };
      request.onsuccess = () => {
        clearTimeout(timer);
        if (abandoned) request.result.close(); else resolve(request.result);
      };
    });
    const journal = new LoginJournal(db, capturedClock);
    try {
      if (db.objectStoreNames.length !== 1 || !db.objectStoreNames.contains('journal')) refuse('storage_corrupt');
      await journal.inspect();
      return journal;
    } catch (cause) { journal.close(); throw cause; }
  }

  close(): void { this.#db.close(); }

  start(input: LoginStart): Promise<void> {
    const { generation } = input;
    const caller = input.caller.toText(), issuer = input.issuer.toText();
    const bytes = input.caller.toUint8Array();
    if (!text(generation) || bytes.length !== 29 || bytes[28] !== 2 || input.issuer.isAnonymous())
      return Promise.reject(new JournalError('configuration'));
    return this.#transaction((record, credentialPresent) => {
      if (record) refuse('login_exists');
      if (credentialPresent) refuse('storage_corrupt');
      const now = this.#now();
      return { record: { generation, caller, issuer, clockId: this.#clock.id, observedNs: now,
        targets: [{ canister: issuer, attempts: 0, nextNs: now }], issuanceUnknown: false,
        phase: 'active', deadlineNs: 0n, busyUntilNs: 0n }, value: undefined };
    });
  }

  /** Persist a receiver before admission can have an external effect. */
  addDestination(generation: string, target: Principal, admissionUntilNs: bigint): Promise<void> {
    const canister = target.toText();
    if (target.isAnonymous() || !u64(admissionUntilNs)) return Promise.reject(new JournalError('configuration'));
    return this.#transaction(record => {
      const current = this.#current(record, generation);
      if (current.phase !== 'active') refuse('not_current');
      if (admissionUntilNs < current.observedNs || admissionUntilNs + ATTEMPT_NS >= current.observedNs + CLEANUP_NS)
        refuse('configuration');
      if (!current.targets.some(t => t.canister === canister)) {
        if (current.targets.length === 64) refuse('destination_limit');
        current.targets.push({ canister, attempts: 0, nextNs: current.observedNs });
      }
      return { record: current, value: undefined };
    });
  }

  /** Suitable for runSigningAttempt.assertCurrent; never cached across an await. */
  assertActive(generation: string, caller: Principal, target: Principal): Promise<void> {
    const callerText = caller.toText(), targetText = target.toText();
    return this.#transaction(record => {
      const current = this.#current(record, generation);
      if (current.phase !== 'active' || current.caller !== callerText
        || !current.targets.some(t => t.canister === targetText)) refuse('not_current');
      return { record: current, value: undefined };
    });
  }

  /** Bind issuer work to this login, not merely an inventoried receiver. */
  assertIssuance(generation: string, caller: Principal, issuer: Principal, sequence?: bigint): Promise<void> {
    const callerText = caller.toText(), issuerText = issuer.toText();
    return this.#transaction(record => {
      const current = this.#current(record, generation);
      if (current.phase !== 'active' || current.caller !== callerText || current.issuer !== issuerText
        || (sequence !== undefined && current.issuance?.sequence !== sequence)) refuse('not_current');
      return { record: current, value: undefined };
    });
  }

  /** Recheck the inventoried receiver and completed proof at every signing boundary. */
  assertAdmission(generation: string, caller: Principal, issuer: Principal, target: Principal, sequence: bigint): Promise<void> {
    const callerText = caller.toText(), issuerText = issuer.toText(), targetText = target.toText();
    return this.#transaction(record => {
      const current = this.#current(record, generation);
      if (current.phase !== 'active' || current.caller !== callerText || current.issuer !== issuerText
        || current.issuance?.sequence !== sequence || current.issuance.untilNs === undefined
        || !current.targets.some(t => t.canister === targetText)) refuse('not_current');
      if (current.issuance.untilNs <= current.observedNs) refuse('credential_expired');
      return { record: current, value: undefined };
    });
  }

  /** Reserve before dispatch. An existing pending intent is always lookup-only. */
  claimCredential(generation: string, options: CredentialClaimOptions): Promise<CredentialClaim> {
    const { accountAvailable, refreshThrough } = options;
    if (typeof accountAvailable !== 'boolean' || (refreshThrough !== undefined && !u64(refreshThrough)))
      return Promise.reject(new JournalError('configuration'));
    return this.#transaction<CredentialClaim>((record, _present, bytes) => {
      const current = this.#current(record, generation);
      if (current.phase !== 'active') refuse('not_current');
      const op = current.issuance;
      if (!op && current.issuanceUnknown) refuse('issuance_unrecoverable');
      if (op?.untilNs !== undefined && op.untilNs > current.observedNs
        && (refreshThrough === undefined || op.sequence > refreshThrough)) {
        // Evidence-mode transactions validate the paired frame before this callback.
        if (!bytes) refuse('storage_corrupt');
        return { record: current, value: { kind: 'saved', sequence: op.sequence,
          bytes, untilNs: op.untilNs } };
      }
      if (op && op.untilNs === undefined) {
        if (op.sequence === 0n && !accountAvailable) refuse('account_required');
        return { record: current, value: { kind: 'reconcile', sequence: op.sequence,
          signer: op.sequence === 0n ? 'account' : 'browser' } };
      }
      if (!op && !accountAvailable) refuse('account_required');
      const sequence = op ? op.sequence + 1n : 0n;
      if (!u64(sequence)) refuse('sequence_exhausted');
      current.issuanceUnknown = true;
      current.issuance = { sequence };
      return { record: current, deleteCredential: true, value: sequence === 0n
        ? { kind: 'issue', sequence, signer: 'account' }
        : { kind: 'renew', sequence, signer: 'browser' } };
    }, true);
  }

  /** Commit exact evidence before admission; identical retries preserve the first expiry. */
  async completeCredential(generation: string, sequence: bigint, bytes: Uint8Array, untilNs: bigint): Promise<void> {
    if (!u64(sequence) || !u64(untilNs) || untilNs === 0n) refuse('configuration');
    if (!(bytes instanceof Uint8Array) || bytes.length < 1 || bytes.length > maximumConfiguredBytes)
      refuse('invalid_material');
    // Capture before awaiting IndexedDB, including Buffer/subarray inputs.
    const owned = new Uint8Array(bytes);
    validateFrame(owned, 'invalid_material');
    await this.#transaction((record, _present, retained) => {
      const current = this.#current(record, generation);
      if (current.phase !== 'active') refuse('not_current');
      const op = current.issuance;
      if (!op || op.sequence !== sequence) refuse('issuance_conflict');
      if (op.untilNs !== undefined && (op.untilNs !== untilNs || !retained
        || retained.length !== owned.length || retained.some((byte, i) => byte !== owned[i])))
        refuse('issuance_conflict');
      current.issuance = { sequence, untilNs };
      return { record: current, credential: owned, value: undefined };
    }, true);
  }

  /** Resolves only after metadata invalidation and evidence deletion commit together. */
  beginLogout(generation: string, deadlineNs: bigint): Promise<JournalStatus> {
    return this.#transaction(record => {
      const current = this.#current(record, generation);
      if (current.phase === 'active') {
        if (!u64(deadlineNs) || deadlineNs <= current.observedNs || deadlineNs > current.observedNs + CLEANUP_NS)
          refuse('configuration');
        current.phase = 'cleanup';
        current.deadlineNs = deadlineNs;
        if (current.issuance) current.issuance = { sequence: current.issuance.sequence };
      }
      return { record: current, value: status(current), deleteCredential: true };
    });
  }

  /** Bounded metadata projection; does not read or validate a credential frame. */
  inspect(): Promise<JournalStatus | undefined> {
    return this.#transaction(record => ({ value: record && status(record) }));
  }

  #now(): bigint {
    const now = this.#clock.nowNs();
    if (!u64(now)) refuse('clock_changed');
    return now;
  }
  #current(record: RecordState | undefined, generation: string): RecordState {
    if (!record || record.generation !== generation) refuse('not_current');
    const now = this.#now();
    if (record.clockId !== this.#clock.id || now < record.observedNs) refuse('clock_changed');
    record.observedNs = now;
    return record;
  }

  // Private synchronous operations keep all reads/writes in one cross-tab transaction.
  #transaction<T>(operation: (record: RecordState | undefined, credentialPresent: boolean, evidence?: Uint8Array) => {
    record?: RecordState; value: T; deleteCredential?: boolean; credential?: Uint8Array;
  }, loadEvidence = false): Promise<T> {
    return new Promise((resolve, reject) => {
      let tx: IDBTransaction;
      try { tx = this.#db.transaction('journal', 'readwrite', { durability: 'strict' }); }
      catch (cause) { reject(new JournalError('storage_unavailable', { cause })); return; }
      let outcome: { phase: 'pending' } | { phase: 'staged'; value: T }
        | { phase: 'failed'; cause: unknown } = { phase: 'pending' };
      const fail = (cause: unknown) => {
        outcome = { phase: 'failed', cause };
        try { tx.abort(); } catch { clearTimeout(timer); reject(cause); }
      };
      const timer = setTimeout(() => fail(new JournalError('storage_unavailable')), 5_000);
      tx.onabort = () => {
        clearTimeout(timer);
        reject(outcome.phase === 'failed' ? outcome.cause : new JournalError('storage_unavailable', { cause: tx.error }));
      };
      tx.oncomplete = () => {
        clearTimeout(timer);
        if (outcome.phase === 'staged') resolve(outcome.value);
        else reject(outcome.phase === 'failed' ? outcome.cause : new JournalError('storage_corrupt'));
      };
      if (tx.durability !== 'strict') { fail(new JournalError('storage_unavailable')); return; }
      try {
        const bucket = tx.objectStore('journal');
        if (bucket.keyPath !== null || bucket.autoIncrement || bucket.indexNames.length) refuse('storage_corrupt');
        const profile = bucket.get('format'), login = bucket.get('login');
        // count avoids loading potentially large credential bytes during a guard.
        const credential = loadEvidence ? bucket.get('credential') : bucket.count('credential');
        credential.onsuccess = () => {
          if (outcome.phase !== 'pending') return;
          try {
            if (profile.result !== FORMAT) refuse('storage_corrupt');
            const record: unknown = login.result;
            if (record !== undefined) validate(record);
            let evidence: Uint8Array | undefined;
            if (loadEvidence) {
              if (record?.issuance?.untilNs !== undefined) {
                validateFrame(credential.result, 'storage_corrupt');
                evidence = credential.result;
              } else if (credential.result !== undefined) refuse('storage_corrupt');
            }
            const change = operation(record, loadEvidence ? evidence !== undefined : credential.result !== 0, evidence);
            if (change.record) { validate(change.record); bucket.put(change.record, 'login'); }
            if (change.deleteCredential) bucket.delete('credential');
            else if (change.credential) bucket.put(change.credential, 'credential');
            outcome = { phase: 'staged', value: change.value };
          } catch (cause) { fail(cause); }
        };
      } catch (cause) { fail(cause); }
    });
  }
}
