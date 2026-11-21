//! Passive schema-2 fields. Construction does not establish canonicality or trust.

use crate::{AuthRole, Principal, ScopeLabel};

///
/// CanonicalId
///
/// Exactly 32 identity bytes; every byte pattern is permitted.
///

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "stable-records",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct CanonicalId([u8; 32]);

impl CanonicalId {
    /// Preserve an identity's bytes without assigning authority to their source.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Borrow the exact signed representation.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

///
/// AudienceId
///
/// Claimed network/application context, compared later with protected enrollment.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "stable-records",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct AudienceId {
    /// Claimed network identity.
    pub canonical_network_id: CanonicalId,
    /// Claimed application identity, retaining the existing protocol field name.
    pub fleet_id: CanonicalId,
}

///
/// PortableMechanism
///
/// A claimed mechanism, never permission to select a verifier or enroll a key.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "stable-records",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum PortableMechanism {
    /// Root-delegated certified issuer proof (signed tag 0).
    Certified,
    /// Independently enrolled Ed25519 issuer key (signed tag 1).
    Ed25519,
}

///
/// DelegatedRoleGrant
///
/// Syntactically valid labels whose counts, order and authority remain unchecked.
///

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "stable-records",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DelegatedRoleGrant {
    /// Role to which these scopes apply; roles must not be combined implicitly.
    pub target: AuthRole,
    /// Claimed scope labels in their supplied order.
    pub scopes: Vec<ScopeLabel>,
}

///
/// PortableClaims
///
/// Unauthenticated schema-2 credential fields.
/// The codec checks structure, not signature validity, time relationships,
/// enrollment, caller binding, revocation or permission. Even a successfully
/// decoded value cannot authorize a session.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortableClaims {
    /// Mechanism committed by the signed bytes.
    pub mechanism: PortableMechanism,
    /// Claimed receiving network and application.
    pub audience: AudienceId,
    /// Claimed issuer; verification must match protected enrollment.
    pub issuer: Principal,
    /// Claimed key-binding identity, not key material or authority.
    pub key_binding_id: [u8; 32],
    /// Raw browser Ed25519 key; possession and caller binding are not checked here.
    pub browser_public_key: [u8; 32],
    /// Issuer-qualified subject; canonical claims exclude the anonymous principal.
    pub account_subject: Principal,
    /// Issuer-assigned slot, canonically bounded to 0–7.
    pub device_slot: u8,
    /// Nonzero slot generation, unchanged by renewal.
    pub device_generation: u64,
    /// Original login start, in nanoseconds.
    pub login_started_at_ns: u64,
    /// Original absolute login deadline, in nanoseconds.
    pub login_deadline_ns: u64,
    /// Credential issuance time, in nanoseconds.
    pub issued_at_ns: u64,
    /// Exclusive fresh-admission expiry, distinct from the session lease.
    pub admission_expires_at_ns: u64,
    /// Credential nonce; all-zero bytes remain valid structural input.
    pub nonce: [u8; 16],
    /// Signed grants in strict role/scope order; never normalize them on decode.
    pub grants: Vec<DelegatedRoleGrant>,
}
