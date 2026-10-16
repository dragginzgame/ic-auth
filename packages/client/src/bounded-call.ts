import { ClientError } from './contracts.js';

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
