import { ClientError } from './contracts.js';

// Private lifecycle owner. Stores retain schemas, error projection and mutations.
export function strictTransaction<T>(
  open: () => IDBTransaction,
  timeoutMs: number,
  storageFailure: (cause: unknown) => unknown,
  execute: (tx: IDBTransaction, result: (value: T) => void, fail: (cause: unknown) => void) => void,
): Promise<T> {
  return new Promise((resolve, reject) => {
    let tx: IDBTransaction;
    try { tx = open(); } catch (cause) { reject(storageFailure(cause)); return; }
    let completed = false;
    let failed = false;
    let value: T;
    let failure: unknown;
    const fail = (cause: unknown) => {
      failed = true;
      failure = cause;
      try { tx.abort(); } catch { clearTimeout(timer); reject(cause); }
    };
    const timer = setTimeout(() => fail(new ClientError('timeout')), timeoutMs);
    tx.onabort = () => { clearTimeout(timer); reject(failed ? failure : storageFailure(tx.error)); };
    tx.oncomplete = () => {
      clearTimeout(timer);
      if (failed) reject(failure);
      else if (!completed) reject(new ClientError('storage_corrupt'));
      else resolve(value);
    };
    if (tx.durability !== 'strict') { fail(new ClientError('storage_unavailable')); return; }
    // Request success stages a result; only transaction completion publishes it.
    // Native request errors keep their default abort behavior.
    try { execute(tx, result => { completed = true; value = result; }, fail); }
    catch (cause) { fail(cause); }
  });
}
