//! Private Candid contracts for controller-installed session cost qualification.
use candid::{CandidType, Principal};
use ic_auth_protocol_types::{AudienceId, AuthRole};
use serde::Deserialize;

#[derive(Clone, CandidType, Deserialize)]
pub struct SessionProfileInput {
    pub record: Option<Vec<u8>>,
    pub audience: AudienceId,
    pub role: AuthRole,
    pub allowed_scopes: Vec<String>,
    pub required_scope: String,
    pub root: Principal,
    pub public_key: Vec<u8>,
    pub path_hash: [u8; 32],
    pub valid_from_ns: u64,
    pub accept_until_ns: u64,
    pub generation: u64,
    pub enabled: bool,
}

#[derive(Debug, CandidType, Deserialize)]
pub struct SessionProfileResult {
    pub record_bytes: u64,
    pub decode_instructions: u64,
    pub authorization_instructions: u64,
    pub total_instructions: u64,
    pub decision: String,
}
