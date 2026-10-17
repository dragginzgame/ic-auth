// Generated from IC Wire schema; edit the Rust contract, not this file.
import { createCodec } from './wire-runtime.js';
export type Request = null;
export type Response = { readonly tag: "Revoked"; readonly value: bigint } | { readonly tag: "InvalidCaller"; readonly value: null } | { readonly tag: "Unavailable"; readonly value: { readonly tag: "Capacity"; readonly value: null } | { readonly tag: "Storage"; readonly value: null } | { readonly tag: "Clock"; readonly value: null } | { readonly tag: "ResetPending"; readonly value: null } };
export const methodName = "logout_portable_session";
const codec = createCodec({"id":"71834742030340","request":{"k":"unit"},"requestBytes":11,"requestDepth":4,"requestNodes":8,"response":{"k":"enum","variants":[{"id":0,"name":"Revoked","value":{"bits":64,"k":"u"}},{"id":1,"name":"InvalidCaller","value":{"k":"unit"}},{"id":2,"name":"Unavailable","value":{"k":"enum","variants":[{"id":0,"name":"Capacity","value":{"k":"unit"}},{"id":1,"name":"Storage","value":{"k":"unit"}},{"id":2,"name":"Clock","value":{"k":"unit"}},{"id":3,"name":"ResetPending","value":{"k":"unit"}}]}}]},"responseBytes":26,"responseDepth":8,"responseNodes":128});
export const encodeRequest = (value: Request): Uint8Array => codec.encodeRequest(value);
export const decodeRequest = (bytes: Uint8Array): Request => codec.decodeRequest(bytes) as Request;
export const encodeResponse = (value: Response): Uint8Array => codec.encodeResponse(value);
export const decodeResponse = (bytes: Uint8Array): Response => codec.decodeResponse(bytes) as Response;
