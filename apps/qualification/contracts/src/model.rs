//! Controller input is a bounded transport projection, never trusted enrollment.
use ic_auth_wire::model::{AudienceId, DelegatedRoleGrant};
use ic_wire::{Bytes, List, Text};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReceiverConfig {
    pub audience: AudienceId,
    pub issuer: Bytes<29>,
    pub authority: ReceiverAuthority,
    pub role: Text<64>,
    pub grants: List<DelegatedRoleGrant, 16>,
    pub valid_from_ns: u64,
    pub valid_until_ns: u64,
    pub enrollment_until_ns: u64,
    pub verification_burst: u32,
    pub verification_refill_ns: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ReceiverAuthority {
    Ed25519([u8; 32]),
    Certified(CertifiedAuthority),
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CertifiedAuthority {
    pub binding: [u8; 32],
    pub root: RootEnrollment,
    pub network_key: Bytes<96>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RootEnrollment {
    pub root_canister_id: Bytes<29>,
    pub algorithm: ic_auth_wire::model::ChainKeyAlgorithm,
    pub key_name: Text<128>,
    pub derivation_path_hash: [u8; 32],
    pub public_key: Bytes<65>,
    pub key_version: u64,
    pub min_accepted_key_version: u64,
    pub min_accepted_proof_epoch: u64,
    pub min_accepted_registry_epoch: u64,
    pub valid_from_ns: u64,
    pub accept_until_ns: u64,
    pub max_revocation_latency_ns: u64,
}
#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SessionProfile {
    pub record_bytes: u64,
    pub decode_instructions: u64,
    pub authorization_instructions: u64,
}
#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdmissionProfile {
    pub reply: Bytes<4096>,
    pub instructions: u64,
}

/// Independent component probes after complete verification of the same proof.
/// Component values overlap and are not an additive trace of the full verifier.
#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CertifiedProfile {
    pub credential_instructions: u64,
    pub root_delegation_instructions: u64,
    pub issuer_instructions: u64,
    pub decode_instructions: u64,
    pub tree_hash_instructions: u64,
    pub signing_bls_instructions: u64,
    pub subnet_bls_instructions: u64,
    pub signature_bytes: u64,
    pub certificate_bytes: u64,
    pub subnet_certificate_bytes: u64,
}
