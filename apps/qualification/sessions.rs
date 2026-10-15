//! Private controller-installed portable-session qualification contracts.
use candid::{CandidType, Principal};
use ic_auth_protocol_types::{AudienceId, AuthRole, DelegatedRoleGrant, PortableCredential};
use serde::Deserialize;
#[derive(Clone, CandidType, Deserialize)]
pub struct ReceiverConfig {
    pub audience: AudienceId,
    pub issuer: Principal,
    pub public_key: [u8; 32],
    pub role: AuthRole,
    pub grants: Vec<DelegatedRoleGrant>,
    pub valid_from_ns: u64,
    pub valid_until_ns: u64,
    pub enrollment_until_ns: u64,
}
#[derive(Clone, CandidType, Deserialize)]
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
