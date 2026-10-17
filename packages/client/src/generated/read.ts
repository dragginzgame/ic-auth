// Generated from IC Wire schema; edit the Rust contract, not this file.
import { createCodec } from './wire-runtime.js';
export type Request = null;
export type Response = { readonly ok: { readonly "caller": Uint8Array; readonly "issuer": Uint8Array; readonly "subject": Uint8Array; readonly "expires_at_ns": bigint } } | { readonly err: string };
export const methodName = "portable_session_read";
const codec = createCodec({"id":"71834742030339","request":{"k":"unit"},"requestBytes":11,"requestDepth":4,"requestNodes":8,"response":{"err":{"k":"text","max":256},"k":"result","ok":{"fields":[{"id":1,"name":"caller","value":{"k":"bytes","max":29}},{"id":2,"name":"issuer","value":{"k":"bytes","max":29}},{"id":3,"name":"subject","value":{"k":"bytes","max":29}},{"id":4,"name":"expires_at_ns","value":{"bits":64,"k":"u"}}],"k":"record"}},"responseBytes":278,"responseDepth":8,"responseNodes":128});
export const encodeRequest = (value: Request): Uint8Array => codec.encodeRequest(value);
export const decodeRequest = (bytes: Uint8Array): Request => codec.decodeRequest(bytes) as Request;
export const encodeResponse = (value: Response): Uint8Array => codec.encodeResponse(value);
export const decodeResponse = (bytes: Uint8Array): Response => codec.decodeResponse(bytes) as Response;
