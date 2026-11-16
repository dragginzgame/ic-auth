import { HttpAgent, createAgentOptions } from '@tokostudios/ic-platform/agent';
import { Principal } from '@tokostudios/ic-platform/principal';
import { JournalError } from './journal.ts';
import type { AdmissionTransport } from './admission.ts';
import type { LogoutTransport } from './logout.ts';

export interface HttpUpdateOptions {
  /** Protected gateway origin and trust root, never supplied by credential fields. */
  host: URL;
  rootKey: Uint8Array;
  /** Consumer-owned endpoint implementing the selected Wire contract. */
  methodName: string;
  fetch?: typeof fetch;
}

/** One raw update adapter for admission or logout. The caller owns codec and attempt lifetime. */
export function httpUpdateTransport(options: HttpUpdateOptions): AdmissionTransport & LogoutTransport {
  const { methodName } = options;
  if (!(options.host instanceof URL) || !['http:', 'https:'].includes(options.host.protocol)
    || options.host.username || options.host.password || options.host.pathname !== '/'
    || options.host.search || options.host.hash || !/^[A-Za-z_][A-Za-z0-9_]{0,127}$/.test(methodName)
    || !(options.rootKey instanceof Uint8Array) || options.rootKey.length === 0)
    throw new JournalError('configuration');
  const host = new URL(options.host.href), root = new Uint8Array(options.rootKey);
  const fetcher = options.fetch ?? globalThis.fetch.bind(globalThis);
  return async (target, identity, request, signal) => {
    signal.throwIfAborted();
    if (target.isAnonymous() || identity.getPrincipal().isAnonymous() || !(request instanceof Uint8Array))
      throw new JournalError('configuration');
    const canister = Principal.fromText(target.toText()), arg = new Uint8Array(request);
    const callPath = `/api/v2/canister/${canister.toText()}/call`;
    const statePath = `/api/v3/canister/${canister.toText()}/read_state`;
    let submitted = false;
    const agent = HttpAgent.createSync(createAgentOptions({
      host, rootKey: root, identity, retryTimes: 0, verifyQuerySignatures: true, shouldSyncTime: false,
      fetch: async (input, init) => {
        signal.throwIfAborted();
        const url = new URL(input instanceof Request ? input.url : input.toString());
        if (url.origin !== host.origin || url.search || url.hash || init?.method !== 'POST')
          throw new JournalError('configuration');
        if (url.pathname === callPath && !submitted) submitted = true;
        else if (url.pathname !== statePath || !submitted) throw new JournalError('configuration');
        const response = await fetcher(input, { ...init, signal, redirect: 'error', credentials: 'omit' });
        signal.throwIfAborted();
        return response;
      },
    }));
    const result = await agent.update(canister, { methodName, arg, callSync: false }, {
      // Recheck the journal-backed identity for every read-state signature.
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
    return new Uint8Array(result.reply);
  };
}
