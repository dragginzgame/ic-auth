import { ClientError } from './contracts.js';

/** Private connection lifecycle; callers own schema creation and error projection. */
export function openDatabase(open: () => IDBOpenDBRequest, timeoutMs: number,
  failure: (cause: unknown) => unknown,
  initialize: (request: IDBOpenDBRequest, event: IDBVersionChangeEvent) => void): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    let abandoned = false;
    const fail = (cause: unknown) => { abandoned = true; clearTimeout(timer); reject(failure(cause)); };
    const timer = setTimeout(() => fail(new ClientError('timeout')), timeoutMs);
    let request: IDBOpenDBRequest;
    try { request = open(); } catch (cause) { fail(cause); return; }
    request.onblocked = () => fail(new ClientError('storage_unavailable'));
    request.onerror = () => fail(request.error);
    request.onupgradeneeded = event => {
      if (abandoned) { request.transaction?.abort(); return; }
      try { initialize(request, event); }
      catch (cause) {
        // A failed abort must not swallow the original schema refusal.
        try { request.transaction?.abort(); } catch { /* Already inactive. */ }
        fail(cause);
      }
    };
    request.onsuccess = () => {
      clearTimeout(timer);
      if (abandoned) request.result.close(); else resolve(request.result);
    };
  });
}
