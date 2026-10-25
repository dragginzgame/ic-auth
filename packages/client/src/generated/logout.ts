// Generated from IC Wire metadata; edit the source contracts.
import { createCodec as $createCodec } from "./wire-runtime.js";
export type Request = null;
export type Response = { readonly tag: "Revoked"; readonly value: bigint } | { readonly tag: "InvalidCaller"; readonly value: null } | { readonly tag: "Unavailable"; readonly value: { readonly tag: "Capacity"; readonly value: null } | { readonly tag: "Storage"; readonly value: null } | { readonly tag: "Clock"; readonly value: null } | { readonly tag: "ResetPending"; readonly value: null } };
const $types = {"Request":{"k":"unit"},"Response":{"k":"enum","variants":[{"id":0,"name":"Revoked","value":{"bits":64,"k":"u"}},{"id":1,"name":"InvalidCaller","value":{"k":"unit"}},{"id":2,"name":"Unavailable","value":{"k":"enum","variants":[{"id":0,"name":"Capacity","value":{"k":"unit"}},{"id":1,"name":"Storage","value":{"k":"unit"}},{"id":2,"name":"Clock","value":{"k":"unit"}},{"id":3,"name":"ResetPending","value":{"k":"unit"}}]}}]}};
export const contract = (() => { const codec = $createCodec({"id":"71834742030340","mode":"update","name":"contract","request":{"k":"ref","name":"Request"},"requestBytes":11,"requestDepth":4,"requestNodes":8,"response":{"k":"ref","name":"Response"},"responseBytes":4096,"responseDepth":8,"responseNodes":128}, $types); return { methodName: "contract", mode: "update" as const, encodeRequest: (value: Request): globalThis.Uint8Array => codec.encodeRequest(value), decodeRequest: (bytes: globalThis.Uint8Array): Request => codec.decodeRequest(bytes) as Request, encodeResponse: (value: Response): globalThis.Uint8Array => codec.encodeResponse(value), decodeResponse: (bytes: globalThis.Uint8Array): Response => codec.decodeResponse(bytes) as Response }; })();

export const methodName = "logout_portable_session";
export const { encodeRequest, decodeRequest, encodeResponse, decodeResponse } = contract;
export const maximumResponseBytes = 1024;
