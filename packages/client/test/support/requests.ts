import { Endpoint, SubmitRequestType, Expiry, type Identity } from '@tokostudios/ic-platform/agent';
import type { Principal } from '@tokostudios/ic-platform/principal';

/** Mechanical SDK request shape; destination, signer and method stay explicit. */
export function signingRequest(signer: Identity, target: Principal, methodName: string): Parameters<Identity['transformRequest']>[0] {
  return { endpoint: Endpoint.Call, request: { method: 'POST' },
    body: { request_type: SubmitRequestType.Call, canister_id: target, method_name: methodName,
      arg: new Uint8Array(), sender: signer.getPrincipal(),
      ingress_expiry: Expiry.fromJSON('{"__expiry__":"100000000000"}') } };
}
