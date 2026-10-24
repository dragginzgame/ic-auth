import { encodeRequest } from './generated/logout.js';
import { logoutResultFromBytes } from './codec.js';
import { httpUpdateTransport, type HttpUpdateOptions } from './update-http.js';
import type { LogoutTransport } from './logout.js';

export interface HttpLogoutOptions extends HttpUpdateOptions {
  dispose: LogoutTransport['dispose'];
}

/** SDK update transport. The journal owns retries, identity leases and disposal. */
export function httpLogoutTransport(options: HttpLogoutOptions): LogoutTransport {
  const update = httpUpdateTransport(options);
  return {
    dispose: options.dispose,
    async revoke(target, identity, signal) {
      const reply = await update(target, identity, encodeRequest(null), signal);
      // HTTP acceptance alone is never a revocation acknowledgement.
      return logoutResultFromBytes(reply);
    },
  };
}
