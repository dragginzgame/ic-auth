import * as credential from './generated/credential.js';
import * as logout from './generated/logout.js';
import { ClientError } from './contracts.js';
export type PortableCredential = credential.Request;
export type PortableProof = PortableCredential['proof'];
export type LogoutResult = logout.Response;
export type LogoutUnavailable = Extract<LogoutResult, { tag: 'Unavailable' }>['value'];
export type LogoutRequest = logout.Request;
function bounded<T>(maximum: number, operation: () => T): T {
  if (!Number.isSafeInteger(maximum) || maximum < 1) throw new ClientError('invalid_material');
  try { return operation(); } catch (cause) { throw new ClientError('invalid_material', { cause }); }
}
export const credentialBytes = (value: PortableCredential, maximum: number): Uint8Array => bounded(maximum, () => {
  const bytes = credential.encodeRequest(value);
  if (bytes.length > maximum) throw new ClientError('invalid_material');
  return bytes;
});
export const credentialFromBytes = (bytes: Uint8Array, maximum: number): PortableCredential => bounded(maximum, () => {
  if (bytes.length > maximum) throw new ClientError('invalid_material');
  // Buffer.slice() aliases storage; the shared runtime accepts Uint8Array views.
  // Give it an ordinary owned Uint8Array after admission so decoded blobs detach.
  return credential.decodeRequest(new Uint8Array(bytes));
});
export const logoutResultFromBytes = (bytes: Uint8Array): LogoutResult => bounded(1024, () => {
  if (bytes.length > 1024) throw new ClientError('invalid_material');
  return logout.decodeResponse(new Uint8Array(bytes));
});
