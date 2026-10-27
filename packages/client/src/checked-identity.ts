import type { Identity } from '@tokostudios/ic-platform/agent';
import { Principal } from '@tokostudios/ic-platform/principal';
import { ClientError } from './contracts.js';

type SigningRequest = Parameters<Identity['transformRequest']>[0];

/** Mechanics only: each owner supplies its active-login or cleanup-lease policy.
 * Recheck after signing because asynchronous signers can outlive their authority.
 */
export function checkedIdentity(identity: Identity, getPrincipal: Identity['getPrincipal'],
  before: (request: SigningRequest) => Promise<void>,
  after: (request: SigningRequest) => Promise<void> = before): Identity {
  return {
    getPrincipal,
    transformRequest: async request => {
      await before(request);
      const signed = await identity.transformRequest(request);
      await after(request);
      return signed;
    },
  };
}

/** Read-state signing is checked by the owning lease, not a call destination. */
export function checkDestination(request: SigningRequest, accepts: (target: string) => boolean): void {
  if (request.body.request_type === 'read_state') return;
  const target = request.body.canister_id;
  if (!(target instanceof Principal) || !accepts(target.toText())) throw new ClientError('configuration');
}
