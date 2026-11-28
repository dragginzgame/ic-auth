// Generated IC Wire client; no transport or execution.
import { schemaFormat as $format, createCodec as $create } from "./wire-runtime.js";
if ($format !== "ic-wire-sparse-v1")
    throw new globalThis.Error('IC Wire runtime format mismatch');
const $types = globalThis.JSON.parse("{\"FrameRequest\":{\"k\":\"ref\",\"name\":\"Wire_Message_9c9260c865802609\"},\"FrameResponse\":{\"k\":\"ref\",\"name\":\"Wire_Message_f3cef0b603e3440f\"},\"Wire_LogoutResult_983ac2b3d5afb6ea\":{\"k\":\"enum\",\"variants\":[{\"name\":\"Revoked\",\"id\":0,\"value\":{\"k\":\"ref\",\"name\":\"Wire_u64_4d35d3193e8d66f2\"}},{\"name\":\"Unconfirmed\",\"id\":1,\"value\":{\"k\":\"ref\",\"name\":\"Wire_u64_4d35d3193e8d66f2\"}},{\"name\":\"InvalidCaller\",\"id\":2,\"value\":{\"k\":\"unit\"}},{\"name\":\"Unavailable\",\"id\":3,\"value\":{\"k\":\"ref\",\"name\":\"Wire_LogoutUnavailable_6f86044fa22ff487\"}}],\"projected\":true},\"Wire_LogoutUnavailable_6f86044fa22ff487\":{\"k\":\"enum\",\"variants\":[{\"name\":\"Capacity\",\"id\":0,\"value\":{\"k\":\"unit\"}},{\"name\":\"Storage\",\"id\":1,\"value\":{\"k\":\"unit\"}},{\"name\":\"Clock\",\"id\":2,\"value\":{\"k\":\"unit\"}},{\"name\":\"ResetPending\",\"id\":3,\"value\":{\"k\":\"unit\"}}],\"projected\":true},\"Wire_Message_9c9260c865802609\":{\"k\":\"extension-tuple\",\"items\":[{\"k\":\"ref\",\"name\":\"Wire____07e11f07b4a6665a\"}],\"extensions\":[],\"projected\":true},\"Wire_Message_f3cef0b603e3440f\":{\"k\":\"extension-tuple\",\"items\":[{\"k\":\"ref\",\"name\":\"Wire_LogoutResult_983ac2b3d5afb6ea\"}],\"extensions\":[],\"projected\":true},\"Wire____07e11f07b4a6665a\":{\"k\":\"unit\"},\"Wire_u64_4d35d3193e8d66f2\":{\"k\":\"u\",\"bits\":64}}");
export const codec = (() => {
    const $codec = $create({ id: '71834742030612', request: { k: 'ref', name: "FrameRequest" }, response: { k: 'ref', name: "FrameResponse" }, requestBytes: 12, requestNodes: 32, requestDepth: 8, responseBytes: 4096, responseNodes: 128, responseDepth: 8 }, $types);
    return globalThis.Object.freeze({
        encodeRequest: (value) => $codec.encodeRequest(value), decodeRequest: (bytes) => $codec.decodeRequest(bytes),
        encodeResponse: (value) => $codec.encodeResponse(value), decodeResponse: (bytes) => $codec.decodeResponse(bytes),
    });
})();
export const methodName = "logout_portable_session";
export const contract = globalThis.Object.freeze({ ...codec, methodName: 'contract', mode: 'update',
    encodeRequest: (value) => codec.encodeRequest([value]), decodeRequest: (bytes) => codec.decodeRequest(bytes)[0],
    encodeResponse: (value) => codec.encodeResponse([value]), decodeResponse: (bytes) => codec.decodeResponse(bytes)[0] });
export const { encodeRequest, decodeRequest, encodeResponse, decodeResponse } = contract;
export const maximumResponseBytes = 1024;
