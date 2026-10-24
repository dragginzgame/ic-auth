import { ClientError } from './contracts.js';

/** Admission's maximum in-flight duration, also reserved by logout cleanup. */
export const MAX_NETWORK_CALL_MS = 5_000;

/** One transport attempt. The owner retains generation, lease and retry policy. */
export async function boundedCall<T>(timeoutMs: number, operation: (signal: AbortSignal) => Promise<T>): Promise<T> {
  const controller = new AbortController();
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([
      Promise.resolve().then(() => operation(controller.signal)),
      new Promise<never>((_, reject) => {
        timer = setTimeout(() => { controller.abort(); reject(new ClientError('timeout')); }, timeoutMs);
      }),
    ]);
  } finally { clearTimeout(timer); controller.abort(); }
}
