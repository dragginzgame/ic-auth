//! Fixture transport schema extracted from the independent Canic wire tests.
//! Text identifiers are validated by the domain Deserialize implementation when
//! entering Candid<T>; these projections do not authenticate material.
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum DelegationAudience {
    Fleet(AudienceId),
}

/// Exact network-qualified audience, with existing protocol field labels.
/// The host owns the meaning and trusted source of both identifiers; this type
/// contains no fleet topology, membership, derivation or installation policy.
#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AudienceId {
    pub canonical_network_id: String,
    pub fleet_id: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DelegatedRoleGrant {
    pub target: String,
    pub scopes: Vec<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AuthRequestMetadata {
    pub request_id: [u8; 32],
    pub ttl_ns: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DelegatedTokenPrepareRequest {
    pub metadata: Option<AuthRequestMetadata>,
    pub aud: DelegationAudience,
    pub grants: Vec<DelegatedRoleGrant>,
    pub ttl_ns: u64,
    pub ext: Option<Vec<u8>>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DelegatedTokenClaims {
    pub presenter: Principal,
    pub subject: Principal,
    pub issuer_pid: Principal,
    pub cert_hash: [u8; 32],
    pub issued_at_ns: u64,
    pub expires_at_ns: u64,
    pub aud: DelegationAudience,
    pub grants: Vec<DelegatedRoleGrant>,
    pub nonce: [u8; 16],
    pub ext: Option<Vec<u8>>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DelegatedTokenPrepareResponse {
    pub claims: DelegatedTokenClaims,
    pub claims_hash: [u8; 32],
    pub retrieval_expires_at_ns: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DelegatedTokenGetRequest {
    pub claims_hash: [u8; 32],
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum RootProof {
    IcChainKeyBatchSignatureV1(IcChainKeyBatchSignatureProofV1),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum IssuerProof {
    IcCanisterSignatureV1(IcCanisterSignatureProofV1),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IcCanisterSignatureProofV1 {
    pub signature_cbor: Vec<u8>,
    pub public_key_der: Vec<u8>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ChainKeyAlgorithm {
    EcdsaSecp256k1,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ChainKeyKeyId {
    pub name: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IcChainKeyBatchSignatureProofV1 {
    pub header: ChainKeyBatchHeaderV1,
    pub delegation_cert: ChainKeyDelegationCertV1,
    pub issuer_witness: ChainKeyBatchWitnessV1,
    pub signature: ChainKeyRootSignatureV1,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ChainKeyBatchHeaderV1 {
    pub schema_version: u16,
    pub root_canister_id: Principal,
    pub batch_id: [u8; 32],
    pub proof_epoch: u64,
    pub registry_epoch: u64,
    pub registry_hash: [u8; 32],
    pub tree_root: [u8; 32],
    pub not_before_ns: u64,
    pub expires_at_ns: u64,
    pub algorithm: ChainKeyAlgorithm,
    pub key_id: ChainKeyKeyId,
    pub derivation_path_hash: [u8; 32],
    pub key_version: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ChainKeyDelegationCertV1 {
    pub root_canister_id: Principal,
    pub issuer_canister_id: Principal,
    pub proof_epoch: u64,
    pub issuer_proof_algorithm: IssuerProofAlgorithm,
    pub issuer_proof_binding_hash: [u8; 32],
    pub issuer_proof_binding: IssuerProofBinding,
    pub max_token_ttl_ns: u64,
    pub audience: DelegationAudience,
    pub grants: Vec<DelegatedRoleGrant>,
    pub not_before_ns: u64,
    pub expires_at_ns: u64,
    pub registry_epoch: u64,
    pub registry_hash: [u8; 32],
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ChainKeyRootSignatureV1 {
    pub algorithm: ChainKeyAlgorithm,
    pub key_id: ChainKeyKeyId,
    pub derivation_path: Vec<Vec<u8>>,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ChainKeyBatchWitnessV1 {
    pub steps: Vec<ChainKeyBatchWitnessStepV1>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ChainKeyBatchWitnessStepV1 {
    LeftSibling([u8; 32]),
    RightSibling([u8; 32]),
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum IssuerProofAlgorithm {
    IcCanisterSignatureV1,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum IssuerProofBinding {
    IcCanisterSignatureV1 { seed_hash: [u8; 32] },
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DelegationCert {
    pub root_pid: Principal,
    pub issuer_pid: Principal,
    pub issuer_proof_alg: IssuerProofAlgorithm,
    pub issuer_proof_binding_hash: [u8; 32],
    pub issuer_proof_binding: IssuerProofBinding,
    pub issued_at_ns: u64,
    pub not_before_ns: u64,
    pub expires_at_ns: u64,
    pub max_token_ttl_ns: u64,
    pub aud: DelegationAudience,
    pub grants: Vec<DelegatedRoleGrant>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DelegationProof {
    pub cert: DelegationCert,
    pub root_proof: RootProof,
}

#[derive(CandidType, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DelegatedToken {
    pub claims: DelegatedTokenClaims,
    pub proof: DelegationProof,
    pub issuer_proof: IssuerProof,
}

/// Unauthenticated portable credential transport. `claims` contains the exact
/// canonical signed bytes, not a Candid encoding of `PortableClaims`.
/// Hosts must bound outer decoding and verify against protected enrollment.
#[derive(CandidType, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PortableCredential {
    pub claims: Vec<u8>,
    pub proof: PortableProof,
}

/// Transport selects proof material, never trusted authority or signing custody.
/// Ed25519 signatures are exactly 64 bytes at verification; issuer keys are not
/// carried here. Certified proofs reuse the existing Root and IC proof owners.
#[derive(CandidType, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PortableProof {
    Certified {
        delegation: Box<DelegationProof>,
        issuer_signature: IcCanisterSignatureProofV1,
    },
    Ed25519 {
        signature: Vec<u8>,
    },
}

/// The host authenticates the caller and selects its installed trust domain.
#[derive(Clone, Copy, Debug, Default, CandidType, Deserialize, Serialize, Eq, PartialEq)]
pub struct LogoutRequest {}

/// Acknowledges only this destination's committed revocation, never global logout.
#[derive(Clone, Copy, Debug, CandidType, Deserialize, Serialize, Eq, PartialEq)]
pub enum LogoutResult {
    Revoked { deny_until_ns: u64 },
    InvalidCaller,
    Unavailable(LogoutUnavailable),
}

/// No successful revocation is implied; transport failures also remain unknown.
#[derive(Clone, Copy, Debug, CandidType, Deserialize, Serialize, Eq, PartialEq)]
pub enum LogoutUnavailable {
    Capacity,
    Storage,
    Clock,
    ResetPending,
}
