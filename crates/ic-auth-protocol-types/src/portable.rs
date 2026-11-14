//! Unauthenticated semantic claims for the schema-2 portable credential domain.
//! The canonical codec lives in `ic-auth`; these are not a second wire encoding.

use crate::{AudienceId, DelegatedRoleGrant, Principal};

/// Claimed proof mechanism. Only protected enrollment can authorize its use.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PortableMechanism {
    /// Root-delegated issuer proof backed by an IC canister signature.
    Certified,
    /// Ed25519 signature checked against an independently enrolled issuer key.
    Ed25519,
}

/// Decoded signed material, not verified identity, enrollment or permission.
/// Login windows survive renewal unchanged; admission expiry is distinct from
/// both the absolute login deadline and the receiver's session lease.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortableClaims {
    /// Mechanism bound into the canonical signed claims.
    pub mechanism: PortableMechanism,
    /// Intended network-qualified application audience.
    pub audience: AudienceId,
    /// Claimed issuer, which must match protected receiver enrollment.
    pub issuer: Principal,
    /// Exact identifier of the issuer key binding selected by protected policy.
    pub key_binding_id: [u8; 32],
    /// Raw Ed25519 browser key whose self-authenticating Principal must be caller.
    pub browser_public_key: [u8; 32],
    /// Issuer-asserted account identity, distinct from the browser ingress identity.
    pub account_subject: Principal,
    /// Issuer-assigned bounded device slot, scoped by audience/issuer/account.
    pub device_slot: u8,
    /// Nonzero issuer-owned fence. Renewal retains it; replacement advances it.
    pub device_generation: u64,
    /// Original login start in nanoseconds; renewal must retain this value.
    pub login_started_at_ns: u64,
    /// Absolute login deadline in nanoseconds; renewal must not extend it.
    pub login_deadline_ns: u64,
    /// This credential's issuance time in nanoseconds.
    pub issued_at_ns: u64,
    /// Exclusive admission deadline for this credential, not the session lease.
    pub admission_expires_at_ns: u64,
    /// Issuer-assigned identity for a distinct credential within this login.
    /// Exact issuance retries retain it. All-zero bytes are permitted; the full
    /// signed claims digest, rather than this field alone, identifies replay.
    pub nonce: [u8; 16],
    /// Claimed role/scope grants, subject to canonical bounds and issuer authority.
    pub grants: Vec<DelegatedRoleGrant>,
}

/// Unauthenticated portable credential transport. `claims` contains the exact
/// canonical signed bytes, not a Candid encoding of `PortableClaims`.
/// Hosts must bound outer decoding and verify against protected enrollment.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortableCredential {
    /// Exact canonical claims bytes covered by the proof; decoding is not verification.
    pub claims: Vec<u8>,
    /// Proof material whose variant must agree with the signed mechanism.
    pub proof: PortableProof,
}

/// Transport selects proof material, never trusted authority or signing custody.
/// Ed25519 signatures are exactly 64 bytes at verification; issuer keys are not
/// carried here. Certified proofs reuse the existing Root and IC proof owners.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PortableProof {
    /// Root delegation plus the delegated issuer's IC canister signature.
    Certified {
        /// Root proof of the issuer's bounded delegated authority.
        delegation: Box<crate::DelegationProof>,
        /// Issuer signature over the canonical claims bytes.
        issuer_signature: crate::IcCanisterSignatureProofV1,
    },
    /// Signature under an issuer key supplied independently by protected policy.
    Ed25519 {
        /// Raw Ed25519 signature; verification requires exactly 64 bytes.
        signature: Vec<u8>,
    },
}
