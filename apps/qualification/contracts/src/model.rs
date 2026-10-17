//! Private controller and IC certificate qualification projections only.
use candid::CandidType;
use serde::{Deserialize, Serialize};

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
