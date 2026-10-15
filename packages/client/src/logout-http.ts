import { HttpAgent } from '@icp-sdk/core/agent';
import { IDL } from '@icp-sdk/core/candid';
import { LogoutRequest } from './generated/protocol.did.js';
import { logoutResultFromBytes } from './codec.js';
import { ClientError } from './contracts.js';
import type { LogoutTransport } from './logout.js';

export interface HttpLogoutOptions {
  /** Protected gateway configuration, never obtained from a credential. */
  host: string;
  /** Consumer-owned endpoint implementing the shared empty request/result. */
  methodName: string;
  /** Explicitly trusted local/test root; omitted uses the SDK's IC root. Never fetched. */
  rootKey?: Uint8Array;
  fetch?: typeof fetch;
  dispose: LogoutTransport['dispose'];
}

/** SDK update transport. The journal owns retries, identity leases and disposal. */
export function httpLogoutTransport(options: HttpLogoutOptions): LogoutTransport {
  const host = new URL(options.host);
  if (!['http:', 'https:'].includes(host.protocol) || host.username || host.password
    || host.search || host.hash || !/^[a-zA-Z_][a-zA-Z0-9_]{0,127}$/.test(options.methodName)
    || (options.rootKey !== undefined && options.rootKey.length === 0)) throw new ClientError('configuration');
  const methodName = options.methodName;
  const rootKey = options.rootKey?.slice();
  const fetcher = options.fetch ?? globalThis.fetch.bind(globalThis);
  const dispose = options.dispose;
  return {
    dispose,
    async revoke(target, identity, signal) {
      signal.throwIfAborted();
      if (target.isAnonymous() || identity.getPrincipal().isAnonymous()) throw new ClientError('configuration');
      let submitted = false;
      // A fresh agent must use this attempt's lease-guarded identity, including
      // read-state polling. Never cache an agent carrying a previous lease.
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
      const result = await agent.update(target, {
        methodName, arg: IDL.encode([LogoutRequest], [{}]),
        // One submission per durable attempt; no synchronous-endpoint fallback.
        callSync: false,
      }, {
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
      // HttpAgent.update verifies certificate, request ID, target and freshness.
      // HTTP acceptance alone is never a revocation acknowledgement.
      return logoutResultFromBytes(result.reply);
    },
  };
}
