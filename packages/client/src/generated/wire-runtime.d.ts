export declare const schemaFormat = "ic-wire-json";
export declare class WireError extends Error {
    readonly code: 'limit' | 'malformed' | 'identity' | 'value';
    name: string;
    constructor(code: 'limit' | 'malformed' | 'identity' | 'value');
}
type Field = {
    name: string;
    value: Schema;
};
export type Schema = ({
    k: 'unit' | 'bool';
} | {
    k: 'u' | 'i';
    bits: number;
    min?: number;
    max?: number;
} | {
    k: 'float';
    bits: number;
    min?: number;
    max?: number;
} | {
    k: 'bytes';
    min?: number;
    max?: number;
} | {
    k: 'text';
    max?: number;
} | {
    k: 'packed';
    max?: number;
    word: 'bool';
    nonzero?: never;
} | {
    k: 'packed';
    max?: number;
    word: 'i64';
    nonzero?: true;
} | {
    k: 'packed';
    max?: number;
    word: Exclude<keyof typeof packedArrays, 'i64'>;
    nonzero?: never;
} | {
    k: 'list';
    item: Schema;
    max?: number;
} | {
    k: 'array';
    item: Schema;
    max: number;
} | {
    k: 'tuple';
    items: readonly Schema[];
} | {
    k: 'result';
    ok: Schema;
    err: Schema;
} | {
    k: 'option' | 'newtype';
    item: Schema;
} | {
    k: 'record';
    fields: readonly Field[];
} | {
    k: 'enum';
    variants: readonly (Field & {
        id: number;
    })[];
} | {
    k: 'ref';
    name: string;
}) & {
    check?: string;
};
export type Checks = Readonly<Record<string, (value: unknown) => void>>;
declare const packedArrays: {
    readonly i8: Int8ArrayConstructor;
    readonly i16: Int16ArrayConstructor;
    readonly u16: Uint16ArrayConstructor;
    readonly i32: Int32ArrayConstructor;
    readonly u32: Uint32ArrayConstructor;
    readonly i64: BigInt64ArrayConstructor;
    readonly u64: BigUint64ArrayConstructor;
    readonly hash16: Uint8ArrayConstructor;
    readonly hash32: Uint8ArrayConstructor;
    readonly 'finite-f32': Float32ArrayConstructor;
    readonly 'finite-f64': Float64ArrayConstructor;
};
export declare function createPayloadCodec(metadata: unknown, registry?: unknown, checks?: Checks): {
    encode(value: unknown): Uint8Array;
    decode(bytes: Uint8Array): unknown;
};
export declare function createCodec(metadata: unknown, registry?: unknown, checks?: Checks): {
    encodeRequest: (value: unknown) => Uint8Array<ArrayBufferLike>;
    decodeRequest: (bytes: Uint8Array) => unknown;
    encodeResponse: (value: unknown) => Uint8Array<ArrayBufferLike>;
    decodeResponse: (bytes: Uint8Array) => unknown;
};
export {};
