import { Principal } from '@icp-sdk/core/principal';
import * as establish from './generated/establish.js';
import { AdmissionError, ClientError } from './contracts.js';
import { httpUpdateTransport, type HttpUpdateOptions } from './update-http.js';
import type { PortableAdmissionTransport } from './portable-admission.js';

export interface HttpAdmissionOptions extends HttpUpdateOptions {
  /** Requested receiver scopes, fixed when the adapter is constructed. */
  scopes: readonly string[];
  ttlSecs?: bigint;
}

/** One authenticated Wire admission attempt. Issuance and recovery stay with the issuer adapter. */
export function httpAdmissionTransport(options: HttpAdmissionOptions): PortableAdmissionTransport['admit'] {
  const update = httpUpdateTransport(options);
  const scopes = [...options.scopes];
  const ttl = options.ttlSecs === undefined ? null : { some: options.ttlSecs };
  return async (target, identity, credential, signal) => {
    signal.throwIfAborted();
    let request: Uint8Array;
    try {
      request = establish.encodeRequest({ credential, scopes, ttl_secs: ttl });
    } catch (cause) { throw new ClientError('invalid_material', { cause }); }
    const reply = await update(target, identity, request, signal);
    let result: establish.Response;
    try {
      result = establish.decodeResponse(reply);
      if ('ok' in result && Principal.fromUint8Array(result.ok.caller).toText() !== identity.getPrincipal().toText()) {
        throw new Error('admission caller mismatch');
      }
    } catch (cause) { throw new ClientError('invalid_material', { cause }); }
    if ('err' in result) throw new AdmissionError(result.err.tag);
  };
}
