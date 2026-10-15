import { IDL } from '@icp-sdk/core/candid';
import {
  DelegatedToken, DelegatedTokenClaims, DelegatedTokenPrepareRequest,
  DelegatedTokenPrepareResponse, PortableCredential, LogoutResult,
} from './generated/protocol.did.js';
import { ClientError } from './contracts.js';

export function encode(type: IDL.Type, value: unknown, maximum: number): Uint8Array {
  try {
    const bytes = IDL.encode([type], [value]);
    if (bytes.byteLength > maximum) throw new ClientError('invalid_material');
    return bytes;
  } catch (cause) {
    throw new ClientError('invalid_material', { cause });
  }
}

// The SDK decoder returns untyped values. Each boundary below pairs the exact
// generated runtime descriptor with its generated TS type, without hand-written
// DTOs, legacy JSON readers or coercions between different service declarations.
function decode<T>(type: IDL.Type, bytes: Uint8Array, maximum: number): T {
  if (bytes.byteLength > maximum) throw new ClientError('invalid_material');
  try {
    // Guard the public input-stream read boundary on this decode only. The
    // selected SDK's optimized blob decoder calls read(length) without checking
    // availability; its stream otherwise silently returns a truncated slice.
    // Inherit the generated descriptor without mutating it or SDK prototypes.
    const boundedType: IDL.Type = Object.create(type);
    boundedType.decodeValue = (pipe, wireType) => {
      const read = pipe.read.bind(pipe);
      pipe.read = (length: number) => {
        if (!Number.isSafeInteger(length) || length < 0 || length > pipe.byteLength) {
          throw new ClientError('invalid_material');
        }
        return read(length);
      };
      return type.decodeValue(pipe, wireType);
    };
    const value = IDL.decode([boundedType], bytes)[0];
    if (!type.covariant(value)) throw new ClientError('invalid_material');
    // Runtime admission above uses this exact generated descriptor. This is the
    // single typed boundary over the SDK's untyped decode result, not a service
    // declaration conversion or a bypass of field validation.
    return value as T;
  } catch (cause) {
    throw new ClientError('invalid_material', { cause });
  }
}

export const tokenBytes = (value: DelegatedToken, maximum: number) => encode(DelegatedToken, value, maximum);
export const tokenFromBytes = (bytes: Uint8Array, maximum: number) => decode<DelegatedToken>(DelegatedToken, bytes, maximum);
export const requestBytes = (value: DelegatedTokenPrepareRequest, maximum: number) => encode(DelegatedTokenPrepareRequest, value, maximum);
export const requestFromBytes = (bytes: Uint8Array, maximum: number) => decode<DelegatedTokenPrepareRequest>(DelegatedTokenPrepareRequest, bytes, maximum);
export const preparedBytes = (value: DelegatedTokenPrepareResponse, maximum: number) => encode(DelegatedTokenPrepareResponse, value, maximum);
export const preparedFromBytes = (bytes: Uint8Array, maximum: number) => decode<DelegatedTokenPrepareResponse>(DelegatedTokenPrepareResponse, bytes, maximum);
export const claimsBytes = (value: DelegatedTokenClaims, maximum: number) => encode(DelegatedTokenClaims, value, maximum);
export function equalBytes(left: Uint8Array, right: Uint8Array): boolean {
  return left.length === right.length && left.every((byte, index) => byte === right[index]);
}

/** Bounded transport only: preserves opaque signed bytes, never verifies claims. */
export const credentialBytes = (value: PortableCredential, maximum: number) => encode(PortableCredential, value, maximum);
export const credentialFromBytes = (bytes: Uint8Array, maximum: number) => decode<PortableCredential>(PortableCredential, bytes, maximum);

/** Decode only after the transport authenticates the update reply. */
export const logoutResultFromBytes = (bytes: Uint8Array) => decode<LogoutResult>(LogoutResult, bytes, 1_024);
