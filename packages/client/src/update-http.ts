import { HttpAgent, type Identity } from '@icp-sdk/core/agent';
import type { Principal } from '@icp-sdk/core/principal';
import { ClientError } from './contracts.js';

export interface HttpUpdateOptions {
  /** Protected gateway configuration, never obtained from a credential. */
  host: string;
  /** Consumer-owned endpoint implementing the selected Wire contract. */
  methodName: string;
  /** Explicitly trusted local/test root; omitted uses the SDK's IC root. Never fetched. */
  rootKey?: Uint8Array;
  fetch?: typeof fetch;
}

/** Private single-attempt transport. Callers own codecs, identity leases and retries. */
export function httpUpdateTransport(options: HttpUpdateOptions) {
  const host = new URL(options.host);
  if (!['http:', 'https:'].includes(host.protocol) || host.username || host.password
    || host.search || host.hash || !/^[a-zA-Z_][a-zA-Z0-9_]{0,127}$/.test(options.methodName)
    || (options.rootKey !== undefined && options.rootKey.length === 0)) throw new ClientError('configuration');
  const methodName = options.methodName;
  const rootKey = options.rootKey?.slice();
  const fetcher = options.fetch ?? globalThis.fetch.bind(globalThis);
  return async (target: Principal, identity: Identity, arg: Uint8Array, signal: AbortSignal): Promise<Uint8Array> => {
    signal.throwIfAborted();
    if (target.isAnonymous() || identity.getPrincipal().isAnonymous()) throw new ClientError('configuration');
    let submitted = false;
    // Each attempt uses its current signing capability, including read-state.
    const agent = await HttpAgent.create({
      host: host.href, identity, retryTimes: 0,
      shouldFetchRootKey: false, shouldSyncTime: false,
      ...(rootKey ? { rootKey: rootKey.slice() } : {}),
      fetch: async (input, init) => {
        signal.throwIfAborted();
        const path = new URL(input instanceof Request ? input.url : input.toString()).pathname;
        if (path === `/api/v2/canister/${target.toText()}/call`) {
          if (submitted) throw new ClientError('transport');
          submitted = true;
        } else if (path !== `/api/v3/canister/${target.toText()}/read_state`) {
          throw new ClientError('transport');
        }
        const response = await fetcher(input, { ...init, signal });
        signal.throwIfAborted();
        return response;
      },
    });
    signal.throwIfAborted();
    const result = await agent.update(target, { methodName, arg, callSync: false }, {
      preSignReadStateRequest: false,
      strategy: async () => {
        signal.throwIfAborted();
        await new Promise<void>((resolve, reject) => {
          const abort = () => { clearTimeout(timer); reject(signal.reason); };
          const timer = setTimeout(() => { signal.removeEventListener('abort', abort); resolve(); }, 250);
          signal.addEventListener('abort', abort, { once: true });
        });
        signal.throwIfAborted();
      },
    });
    signal.throwIfAborted();
    // The SDK authenticates certificate, request ID, target and freshness.
    return result.reply;
  };
}
