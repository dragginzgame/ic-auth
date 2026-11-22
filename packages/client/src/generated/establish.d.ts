export type Request = Wire_SessionInput_137958e57484ead0;
export type Response = Wire_Result_1863c47b0e9cf3b4;
export type Wire_AdmissionRefusal_3851fab250e1d64a = /* id 0 */ {
    readonly tag: "NotYetValid";
    readonly value: null;
} | /* id 1 */ {
    readonly tag: "Expired";
    readonly value: null;
} | /* id 2 */ {
    readonly tag: "InvalidCredential";
    readonly value: null;
} | /* id 3 */ {
    readonly tag: "AuthorityUnavailable";
    readonly value: null;
} | /* id 4 */ {
    readonly tag: "BeforeReset";
    readonly value: null;
} | /* id 5 */ {
    readonly tag: "ScopeRejected";
    readonly value: null;
} | /* id 6 */ {
    readonly tag: "ReplayConflict";
    readonly value: null;
} | /* id 7 */ {
    readonly tag: "Revoked";
    readonly value: null;
} | /* id 8 */ {
    readonly tag: "Capacity";
    readonly value: null;
} | /* id 9 */ {
    readonly tag: "Throttled";
    readonly value: null;
} | /* id 10 */ {
    readonly tag: "Unavailable";
    readonly value: null;
};
export type Wire_AudienceId_8bc0869c74aa09df = {
    readonly "canonical_network_id": Wire__u8__32__a6c3ffa4fc6f7348;
    readonly "fleet_id": Wire__u8__32__a6c3ffa4fc6f7348;
};
export type Wire_Box_a1fbf0d4f9e63efd = Wire_CertifiedProof_cb152bc3b4d909a5;
export type Wire_Bytes_2710d3471b0c3eff = globalThis.Uint8Array;
export type Wire_Bytes_443e9b8691dbc496 = globalThis.Uint8Array;
export type Wire_Bytes_ceafa818703233a0 = globalThis.Uint8Array;
export type Wire_Bytes_e915c1e9b65fd36a = globalThis.Uint8Array;
export type Wire_Bytes_f12ccc1883af8860 = globalThis.Uint8Array;
export type Wire_Bytes_f1304e1883b29b1d = globalThis.Uint8Array;
export type Wire_Bytes_f48f109af4454406 = globalThis.Uint8Array;
export type Wire_CertifiedProof_cb152bc3b4d909a5 = {
    readonly "delegation": Wire_DelegationProof_d277fb44d3d3cee4;
    readonly "issuer_signature": Wire_IcCanisterSignatureProofV1_fb2c8c3cba723804;
};
export type Wire_ChainKeyAlgorithm_fdf5366e13224129 = {
    readonly tag: "EcdsaSecp256k1";
    readonly value: null;
};
export type Wire_ChainKeyBatchHeaderV1_890d40df66ef9aac = {
    readonly "schema_version": Wire_u16_4d24d3193e7ef3bf;
    readonly "root_canister_id": Wire_Principal_f14095eb1cc0831f;
    readonly "batch_id": Wire__u8__32__a6c3ffa4fc6f7348;
    readonly "proof_epoch": Wire_u64_4d35d3193e8d66f2;
    readonly "registry_epoch": Wire_u64_4d35d3193e8d66f2;
    readonly "registry_hash": Wire__u8__32__a6c3ffa4fc6f7348;
    readonly "tree_root": Wire__u8__32__a6c3ffa4fc6f7348;
    readonly "not_before_ns": Wire_u64_4d35d3193e8d66f2;
    readonly "expires_at_ns": Wire_u64_4d35d3193e8d66f2;
    readonly "algorithm": Wire_ChainKeyAlgorithm_fdf5366e13224129;
    readonly "key_id": Wire_ChainKeyKeyId_f6b46c2384ff67e6;
    readonly "derivation_path_hash": Wire__u8__32__a6c3ffa4fc6f7348;
    readonly "key_version": Wire_u64_4d35d3193e8d66f2;
};
export type Wire_ChainKeyBatchWitnessStepV1_97703821142072f8 = /* id 0 */ {
    readonly tag: "LeftSibling";
    readonly value: Wire__u8__32__a6c3ffa4fc6f7348;
} | /* id 1 */ {
    readonly tag: "RightSibling";
    readonly value: Wire__u8__32__a6c3ffa4fc6f7348;
};
export type Wire_ChainKeyBatchWitnessV1_fb79323bfd2eb396 = {
    readonly "steps": Wire_List_88a3f28a9d5eb54f;
};
export type Wire_ChainKeyDelegationCertV1_18b7b19e40f7af45 = {
    readonly "root_canister_id": Wire_Principal_f14095eb1cc0831f;
    readonly "issuer_canister_id": Wire_Principal_f14095eb1cc0831f;
    readonly "proof_epoch": Wire_u64_4d35d3193e8d66f2;
    readonly "issuer_proof_algorithm": Wire_IssuerProofAlgorithm_d7884866b5fb688c;
    readonly "issuer_proof_binding_hash": Wire__u8__32__a6c3ffa4fc6f7348;
    readonly "issuer_proof_binding": Wire_IssuerProofBinding_9fbd1ad54af544a0;
    readonly "max_token_ttl_ns": Wire_u64_4d35d3193e8d66f2;
    readonly "audience": Wire_DelegationAudience_0d4dcf94cc0737b0;
    readonly "grants": Wire_List_e1113faecdb92277;
    readonly "not_before_ns": Wire_u64_4d35d3193e8d66f2;
    readonly "expires_at_ns": Wire_u64_4d35d3193e8d66f2;
    readonly "registry_epoch": Wire_u64_4d35d3193e8d66f2;
    readonly "registry_hash": Wire__u8__32__a6c3ffa4fc6f7348;
};
export type Wire_ChainKeyKeyId_f6b46c2384ff67e6 = {
    readonly "name": Wire_Text_61e787441a2cf39e;
};
export type Wire_ChainKeyRootSignatureV1_346836fc3be657c3 = {
    readonly "algorithm": Wire_ChainKeyAlgorithm_fdf5366e13224129;
    readonly "key_id": Wire_ChainKeyKeyId_f6b46c2384ff67e6;
    readonly "derivation_path": Wire_List_2b22a9a48c73aca8;
    readonly "public_key": Wire_Bytes_f12ccc1883af8860;
    readonly "signature": Wire_Bytes_f1304e1883b29b1d;
};
export type Wire_DelegatedRoleGrant_c99b5891c7e6eb29 = {
    readonly "target": Wire_Text_4419a861e8a2bb65;
    readonly "scopes": Wire_List_53c7dfbc6ff800b1;
};
export type Wire_DelegationAudience_0d4dcf94cc0737b0 = {
    readonly tag: "Fleet";
    readonly value: Wire_AudienceId_8bc0869c74aa09df;
};
export type Wire_DelegationCert_d2ca0c4475a5aef4 = {
    readonly "root_pid": Wire_Principal_f14095eb1cc0831f;
    readonly "issuer_pid": Wire_Principal_f14095eb1cc0831f;
    readonly "issuer_proof_alg": Wire_IssuerProofAlgorithm_d7884866b5fb688c;
    readonly "issuer_proof_binding_hash": Wire__u8__32__a6c3ffa4fc6f7348;
    readonly "issuer_proof_binding": Wire_IssuerProofBinding_9fbd1ad54af544a0;
    readonly "issued_at_ns": Wire_u64_4d35d3193e8d66f2;
    readonly "not_before_ns": Wire_u64_4d35d3193e8d66f2;
    readonly "expires_at_ns": Wire_u64_4d35d3193e8d66f2;
    readonly "max_token_ttl_ns": Wire_u64_4d35d3193e8d66f2;
    readonly "aud": Wire_DelegationAudience_0d4dcf94cc0737b0;
    readonly "grants": Wire_List_e1113faecdb92277;
};
export type Wire_DelegationProof_d277fb44d3d3cee4 = {
    readonly "cert": Wire_DelegationCert_d2ca0c4475a5aef4;
    readonly "root_proof": Wire_RootProof_7e06807663ae99ca;
};
export type Wire_IcCanisterSignatureProofV1_fb2c8c3cba723804 = {
    readonly "signature_cbor": Wire_Bytes_2710d3471b0c3eff;
    readonly "public_key_der": Wire_Bytes_443e9b8691dbc496;
};
export type Wire_IcChainKeyBatchSignatureProofV1_4325affd9c98c289 = {
    readonly "header": Wire_ChainKeyBatchHeaderV1_890d40df66ef9aac;
    readonly "delegation_cert": Wire_ChainKeyDelegationCertV1_18b7b19e40f7af45;
    readonly "issuer_witness": Wire_ChainKeyBatchWitnessV1_fb79323bfd2eb396;
    readonly "signature": Wire_ChainKeyRootSignatureV1_346836fc3be657c3;
};
export type Wire_IssuerProofAlgorithm_d7884866b5fb688c = {
    readonly tag: "IcCanisterSignatureV1";
    readonly value: null;
};
export type Wire_IssuerProofBinding_9fbd1ad54af544a0 = {
    readonly tag: "IcCanisterSignatureV1";
    readonly value: Wire__u8__32__a6c3ffa4fc6f7348;
};
export type Wire_List_2b22a9a48c73aca8 = globalThis.ReadonlyArray<Wire_Bytes_443e9b8691dbc496>;
export type Wire_List_41ed0dbc65b4a6ef = globalThis.ReadonlyArray<Wire_Text_4419a861e8a2bb65>;
export type Wire_List_53c7dfbc6ff800b1 = globalThis.ReadonlyArray<Wire_Text_4419a861e8a2bb65>;
export type Wire_List_88a3f28a9d5eb54f = globalThis.ReadonlyArray<Wire_ChainKeyBatchWitnessStepV1_97703821142072f8>;
export type Wire_List_e1113faecdb92277 = globalThis.ReadonlyArray<Wire_DelegatedRoleGrant_c99b5891c7e6eb29>;
export type Wire_Option_8177596c9bc78c33 = null | {
    readonly some: Wire_u64_4d35d3193e8d66f2;
};
export type Wire_Option_a6592b9dcb478bd1 = null | {
    readonly some: Wire_Bytes_f48f109af4454406;
};
export type Wire_PortableCredential_67340e188f64605a = {
    readonly "claims": Wire_Bytes_e915c1e9b65fd36a;
    readonly "proof": Wire_PortableProof_2968b956605d919b;
};
export type Wire_PortableProof_2968b956605d919b = /* id 0 */ {
    readonly tag: "Certified";
    readonly value: Wire_Box_a1fbf0d4f9e63efd;
} | /* id 1 */ {
    readonly tag: "Ed25519";
    readonly value: Wire_Bytes_f1304e1883b29b1d;
};
export type Wire_Principal_f14095eb1cc0831f = Wire_Bytes_ceafa818703233a0;
export type Wire_Result_1863c47b0e9cf3b4 = {
    readonly ok: Wire_SessionView_d83548b41b1a7e2a;
} | {
    readonly err: Wire_AdmissionRefusal_3851fab250e1d64a;
};
export type Wire_RootProof_7e06807663ae99ca = {
    readonly tag: "IcChainKeyBatchSignatureV1";
    readonly value: Wire_IcChainKeyBatchSignatureProofV1_4325affd9c98c289;
};
export type Wire_SessionInput_137958e57484ead0 = {
    readonly "credential": Wire_PortableCredential_67340e188f64605a;
    readonly "scopes": Wire_List_41ed0dbc65b4a6ef;
    readonly "ttl_secs": Wire_Option_8177596c9bc78c33;
};
export type Wire_SessionView_d83548b41b1a7e2a = {
    readonly "caller": Wire_Principal_f14095eb1cc0831f;
    readonly "issuer": Wire_Principal_f14095eb1cc0831f;
    readonly "subject": Wire_Principal_f14095eb1cc0831f;
    readonly "expires_at_ns": Wire_u64_4d35d3193e8d66f2;
    readonly "ext": Wire_Option_a6592b9dcb478bd1;
};
export type Wire_Text_4419a861e8a2bb65 = string;
export type Wire_Text_61e787441a2cf39e = string;
export type Wire__u8__32__a6c3ffa4fc6f7348 = globalThis.ReadonlyArray<Wire_u8_08c48207b56753d8>;
export type Wire_u16_4d24d3193e7ef3bf = number;
export type Wire_u64_4d35d3193e8d66f2 = bigint;
export type Wire_u8_08c48207b56753d8 = number;
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
export declare const methodName = "establish_portable_session";
export declare const encodeRequest: (value: Request) => globalThis.Uint8Array, decodeRequest: (bytes: globalThis.Uint8Array) => Request, encodeResponse: (value: Response) => globalThis.Uint8Array, decodeResponse: (bytes: globalThis.Uint8Array) => Response;
