export type Request = Wire____07e11f07b4a6665a;
export type Response = Wire_LogoutResult_983ac2b3d5afb6ea;
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
export type Wire____07e11f07b4a6665a = null;
export type Wire_u64_4d35d3193e8d66f2 = bigint;
export declare const codec: Readonly<{
    encodeRequest: (value: Request) => globalThis.Uint8Array;
    decodeRequest: (bytes: globalThis.Uint8Array) => Request;
    encodeResponse: (value: Response) => globalThis.Uint8Array;
    decodeResponse: (bytes: globalThis.Uint8Array) => Response;
}>;
export declare const contract: Readonly<{
    encodeRequest: (value: Request) => globalThis.Uint8Array;
    decodeRequest: (bytes: globalThis.Uint8Array) => Request;
    encodeResponse: (value: Response) => globalThis.Uint8Array;
    decodeResponse: (bytes: globalThis.Uint8Array) => Response;
    methodName: "contract";
    mode: 'update';
}>;
export declare const methodName = "logout_portable_session";
export declare const encodeRequest: (value: Request) => globalThis.Uint8Array, decodeRequest: (bytes: globalThis.Uint8Array) => Request, encodeResponse: (value: Response) => globalThis.Uint8Array, decodeResponse: (bytes: globalThis.Uint8Array) => Response;
export declare const maximumResponseBytes = 1024;
