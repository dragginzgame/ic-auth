//! Private controller-installed portable-session qualification contracts.
use candid::{CandidType, Principal};
use ic_auth_protocol_types::{AudienceId, AuthRole, DelegatedRoleGrant, PortableCredential};
use ic_auth_qualification_contracts::Candid;
use serde::Deserialize;
#[derive(Clone, CandidType, Deserialize)]
pub struct ReceiverConfig {
    pub audience: Candid<AudienceId>,
    pub issuer: Principal,
    pub authority: ReceiverAuthority,
    pub role: Candid<AuthRole>,
    pub grants: Candid<Vec<DelegatedRoleGrant>>,
    pub valid_from_ns: u64,
    pub valid_until_ns: u64,
    pub enrollment_until_ns: u64,
    pub verification_burst: u32,
    pub verification_refill_ns: u64,
}
#[derive(Clone)]
pub struct SessionInput {
    pub credential: PortableCredential,
    pub scopes: Vec<String>,
    pub ttl_secs: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct SessionView {
    pub caller: Principal,
    pub issuer: Principal,
    pub subject: Principal,
    pub expires_at_ns: u64,
}
#[derive(Debug, CandidType, Deserialize)]
pub struct SessionProfile {
    pub record_bytes: u64,
    pub decode_instructions: u64,
    pub authorization_instructions: u64,
}

/// Controller-installed test authority; never selected from submitted proofs.
#[derive(Clone, CandidType, Deserialize)]
pub enum ReceiverAuthority {
    Ed25519([u8; 32]),
    Certified {
        binding: [u8; 32],
        root: Box<RootEnrollment>,
        network_key: Vec<u8>,
    },
}

impl ReceiverAuthority {
    pub fn mechanism(&self) -> ic_auth_protocol_types::PortableMechanism {
        match self {
            Self::Ed25519(_) => ic_auth_protocol_types::PortableMechanism::Ed25519,
            Self::Certified { .. } => ic_auth_protocol_types::PortableMechanism::Certified,
        }
    }
    pub fn binding(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        match self {
            Self::Ed25519(key) => Sha256::digest(key).into(),
            Self::Certified { binding, .. } => *binding,
        }
    }
}

// Explicit fixture DTO: keep Candid/storage derivation outside the pure library.
#[derive(Clone, CandidType, Deserialize)]
pub struct RootEnrollment {
    pub root_canister_id: Principal,
    pub algorithm: Candid<ic_auth_protocol_types::ChainKeyAlgorithm>,
    pub key_id: Candid<ic_auth_protocol_types::ChainKeyKeyId>,
    pub derivation_path_hash: [u8; 32],
    pub public_key: Vec<u8>,
    pub key_version: u64,
    pub min_accepted_key_version: u64,
    pub min_accepted_proof_epoch: u64,
    pub min_accepted_registry_epoch: u64,
    pub valid_from_ns: u64,
    pub accept_until_ns: u64,
    pub max_revocation_latency_ns: u64,
}
impl From<&ic_auth::delegation::RootKeyPolicy> for RootEnrollment {
    fn from(policy: &ic_auth::delegation::RootKeyPolicy) -> Self {
        Self {
            root_canister_id: policy.root_canister_id,
            algorithm: policy.algorithm.into(),
            key_id: policy.key_id.clone().into(),
            derivation_path_hash: policy.derivation_path_hash,
            public_key: policy.public_key.clone(),
            key_version: policy.key_version,
            min_accepted_key_version: policy.min_accepted_key_version,
            min_accepted_proof_epoch: policy.min_accepted_proof_epoch,
            min_accepted_registry_epoch: policy.min_accepted_registry_epoch,
            valid_from_ns: policy.valid_from_ns,
            accept_until_ns: policy.accept_until_ns,
            max_revocation_latency_ns: policy.max_revocation_latency_ns,
        }
    }
}
impl RootEnrollment {
    pub fn policy(&self) -> ic_auth::delegation::RootKeyPolicy {
        ic_auth::delegation::RootKeyPolicy {
            root_canister_id: self.root_canister_id,
            algorithm: self.algorithm.0,
            key_id: self.key_id.0.clone(),
            derivation_path_hash: self.derivation_path_hash,
            public_key: self.public_key.clone(),
            key_version: self.key_version,
            min_accepted_key_version: self.min_accepted_key_version,
            min_accepted_proof_epoch: self.min_accepted_proof_epoch,
            min_accepted_registry_epoch: self.min_accepted_registry_epoch,
            valid_from_ns: self.valid_from_ns,
            accept_until_ns: self.accept_until_ns,
            max_revocation_latency_ns: self.max_revocation_latency_ns,
        }
    }
}

/// Instruction window covers the real admission implementation, excluding Candid
/// request decoding and response encoding. Cycle deltas include the whole update.
#[derive(Debug, CandidType, Deserialize)]
pub struct AdmissionProfile {
    pub result: Result<SessionView, String>,
    pub instructions: u64,
}
