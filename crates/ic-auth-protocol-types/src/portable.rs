//! Unauthenticated semantic claims for the 0.6 portable credential domain.
//! The canonical codec lives in `ic-auth`; these are not a second wire encoding.

use crate::{AudienceId, DelegatedRoleGrant, Principal};

/// Claimed proof mechanism. Only protected enrollment can authorize its use.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PortableMechanism {
    Certified,
    Ed25519,
}

/// Decoded signed material, not verified identity, enrollment or permission.
/// Login windows survive renewal unchanged; admission expiry is distinct from
/// both the absolute login deadline and the receiver's session lease.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortableClaims {
    pub mechanism: PortableMechanism,
    pub audience: AudienceId,
    pub issuer: Principal,
    pub key_binding_id: [u8; 32],
    pub browser_public_key: [u8; 32],
    pub account_subject: Principal,
    pub login_started_at_ns: u64,
    pub login_deadline_ns: u64,
    pub issued_at_ns: u64,
    pub admission_expires_at_ns: u64,
    pub nonce: [u8; 16],
    pub grants: Vec<DelegatedRoleGrant>,
}

/// Unauthenticated portable credential transport. `claims` contains the exact
/// canonical signed bytes, not a Candid encoding of `PortableClaims`.
/// Hosts must bound outer decoding and verify against protected enrollment.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortableCredential {
    pub claims: Vec<u8>,
    pub proof: PortableProof,
}

/// Transport selects proof material, never trusted authority or signing custody.
/// Ed25519 signatures are exactly 64 bytes at verification; issuer keys are not
/// carried here. Certified proofs reuse the existing Root and IC proof owners.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PortableProof {
    Certified {
        delegation: Box<crate::DelegationProof>,
        issuer_signature: crate::IcCanisterSignatureProofV1,
    },
    Ed25519 {
        signature: Vec<u8>,
    },
}
