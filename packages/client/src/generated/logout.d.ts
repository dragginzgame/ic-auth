export type FrameRequest = Wire_Message_9c9260c865802609;
export type FrameResponse = Wire_Message_f3cef0b603e3440f;
export type Wire_LogoutResult_983ac2b3d5afb6ea = /* id 0 */ {
    readonly tag: "Revoked";
    readonly value: Wire_u64_4d35d3193e8d66f2;
} | /* id 1 */ {
    readonly tag: "Unconfirmed";
    readonly value: Wire_u64_4d35d3193e8d66f2;
} | /* id 2 */ {
    readonly tag: "InvalidCaller";
    readonly value: null;
} | /* id 3 */ {
    readonly tag: "Unavailable";
    readonly value: Wire_LogoutUnavailable_6f86044fa22ff487;
};
export type Wire_LogoutUnavailable_6f86044fa22ff487 = /* id 0 */ {
    readonly tag: "Capacity";
    readonly value: null;
} | /* id 1 */ {
    readonly tag: "Storage";
    readonly value: null;
} | /* id 2 */ {
    readonly tag: "Clock";
    readonly value: null;
} | /* id 3 */ {
    readonly tag: "ResetPending";
    readonly value: null;
};
export type Wire_Message_9c9260c865802609 = readonly [Wire____07e11f07b4a6665a];
export type Wire_Message_f3cef0b603e3440f = readonly [Wire_LogoutResult_983ac2b3d5afb6ea];
export type Wire____07e11f07b4a6665a = null;
export type Wire_u64_4d35d3193e8d66f2 = bigint;
export declare const codec: Readonly<{
    encodeRequest: (value: FrameRequest) => globalThis.Uint8Array;
    decodeRequest: (bytes: globalThis.Uint8Array) => FrameRequest;
    encodeResponse: (value: FrameResponse) => globalThis.Uint8Array;
    decodeResponse: (bytes: globalThis.Uint8Array) => FrameResponse;
}>;
export type Request = FrameRequest[0];
export type Response = FrameResponse[0];
export declare const methodName = "logout_portable_session";
export declare const contract: Readonly<{
    methodName: "contract";
    mode: 'update';
    encodeRequest: (value: Request) => Uint8Array<ArrayBufferLike>;
    decodeRequest: (bytes: Uint8Array) => Request;
    encodeResponse: (value: Response) => Uint8Array<ArrayBufferLike>;
    decodeResponse: (bytes: Uint8Array) => Response;
}>;
export declare const encodeRequest: (value: Request) => Uint8Array<ArrayBufferLike>, decodeRequest: (bytes: Uint8Array) => Request, encodeResponse: (value: Response) => Uint8Array<ArrayBufferLike>, decodeResponse: (bytes: Uint8Array) => Response;
export declare const maximumResponseBytes = 1024;
