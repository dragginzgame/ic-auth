// Generated IC Wire client; no transport or execution.
import { schemaFormat as $format, createCodec as $create } from "./wire-runtime.js";
if ($format !== "ic-wire-json")
    throw new globalThis.Error('IC Wire runtime format mismatch');
const $types = globalThis.JSON.parse("{\"Request\":{\"k\":\"ref\",\"name\":\"Wire____07e11f07b4a6665a\"},\"Response\":{\"k\":\"ref\",\"name\":\"Wire_LogoutResult_983ac2b3d5afb6ea\"},\"Wire_LogoutResult_983ac2b3d5afb6ea\":{\"k\":\"enum\",\"variants\":[{\"name\":\"Revoked\",\"id\":0,\"value\":{\"k\":\"ref\",\"name\":\"Wire_u64_4d35d3193e8d66f2\"}},{\"name\":\"Unconfirmed\",\"id\":1,\"value\":{\"k\":\"ref\",\"name\":\"Wire_u64_4d35d3193e8d66f2\"}},{\"name\":\"InvalidCaller\",\"id\":2,\"value\":{\"k\":\"unit\"}},{\"name\":\"Unavailable\",\"id\":3,\"value\":{\"k\":\"ref\",\"name\":\"Wire_LogoutUnavailable_6f86044fa22ff487\"}}]},\"Wire_LogoutUnavailable_6f86044fa22ff487\":{\"k\":\"enum\",\"variants\":[{\"name\":\"Capacity\",\"id\":0,\"value\":{\"k\":\"unit\"}},{\"name\":\"Storage\",\"id\":1,\"value\":{\"k\":\"unit\"}},{\"name\":\"Clock\",\"id\":2,\"value\":{\"k\":\"unit\"}},{\"name\":\"ResetPending\",\"id\":3,\"value\":{\"k\":\"unit\"}}]},\"Wire____07e11f07b4a6665a\":{\"k\":\"unit\"},\"Wire_u64_4d35d3193e8d66f2\":{\"k\":\"u\",\"bits\":64}}");
export const codec = (() => {
    const $codec = $create({ id: '71834742030356', request: { k: 'ref', name: "Request" }, response: { k: 'ref', name: "Response" }, requestBytes: 11, requestNodes: 8, requestDepth: 4, responseBytes: 4096, responseNodes: 128, responseDepth: 8 }, $types);
    return globalThis.Object.freeze({
        encodeRequest: (value) => $codec.encodeRequest(value), decodeRequest: (bytes) => $codec.decodeRequest(bytes),
        encodeResponse: (value) => $codec.encodeResponse(value), decodeResponse: (bytes) => $codec.decodeResponse(bytes),
    });
})();
export const contract = globalThis.Object.freeze({ ...codec, methodName: 'contract', mode: 'update' });
export const methodName = "logout_portable_session";
export const { encodeRequest, decodeRequest, encodeResponse, decodeResponse } = contract;
export const maximumResponseBytes = 1024;
