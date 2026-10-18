//! Controller input is a bounded transport projection, never trusted enrollment.
use ic_auth_wire::model::{AudienceId, DelegatedRoleGrant};
use ic_wire::{Bytes, List, Text, Wire};

#[derive(Clone, Debug, Eq, PartialEq, Wire)]
pub struct ReceiverConfig {
    #[wire(id = 1)]
    pub audience: AudienceId,
    #[wire(id = 2)]
    pub issuer: Bytes<29>,
    #[wire(id = 3)]
    pub authority: ReceiverAuthority,
    #[wire(id = 4)]
    pub role: Text<64>,
    #[wire(id = 5)]
    pub grants: List<DelegatedRoleGrant, 16>,
    #[wire(id = 6)]
    pub valid_from_ns: u64,
    #[wire(id = 7)]
    pub valid_until_ns: u64,
    #[wire(id = 8)]
    pub enrollment_until_ns: u64,
    #[wire(id = 9)]
    pub verification_burst: u32,
    #[wire(id = 10)]
    pub verification_refill_ns: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Wire)]
pub enum ReceiverAuthority {
    #[wire(id = 0)]
    Ed25519([u8; 32]),
    #[wire(id = 1)]
    Certified(CertifiedAuthority),
}
#[derive(Clone, Debug, Eq, PartialEq, Wire)]
pub struct CertifiedAuthority {
    #[wire(id = 1)]
    pub binding: [u8; 32],
    #[wire(id = 2)]
    pub root: RootEnrollment,
    #[wire(id = 3)]
    pub network_key: Bytes<96>,
}
#[derive(Clone, Debug, Eq, PartialEq, Wire)]
pub struct RootEnrollment {
    #[wire(id = 1)]
    pub root_canister_id: Bytes<29>,
    #[wire(id = 12)]
    pub algorithm: ic_auth_wire::model::ChainKeyAlgorithm,
    #[wire(id = 2)]
    pub key_name: Text<128>,
    #[wire(id = 3)]
    pub derivation_path_hash: [u8; 32],
    #[wire(id = 4)]
    pub public_key: Bytes<65>,
    #[wire(id = 5)]
    pub key_version: u64,
    #[wire(id = 6)]
    pub min_accepted_key_version: u64,
    #[wire(id = 7)]
    pub min_accepted_proof_epoch: u64,
    #[wire(id = 8)]
    pub min_accepted_registry_epoch: u64,
    #[wire(id = 9)]
    pub valid_from_ns: u64,
    #[wire(id = 10)]
    pub accept_until_ns: u64,
    #[wire(id = 11)]
    pub max_revocation_latency_ns: u64,
}
#[derive(Debug, Eq, PartialEq, Wire)]
pub struct SessionProfile {
    #[wire(id = 1)]
    pub record_bytes: u64,
    #[wire(id = 2)]
    pub decode_instructions: u64,
    #[wire(id = 3)]
    pub authorization_instructions: u64,
}
#[derive(Debug, Eq, PartialEq, Wire)]
pub struct AdmissionProfile {
    #[wire(id = 1)]
    pub reply: Bytes<4096>,
    #[wire(id = 2)]
    pub instructions: u64,
}
