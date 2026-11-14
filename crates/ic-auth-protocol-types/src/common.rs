//! Passive application authentication common contracts.
//! Adapted from Canic; see docs/design/canic-source-review.md.

use crate::{AudienceId, AuthRole, ScopeLabel};
use serde::{Deserialize, Serialize};

//
// DelegationAudience
//

/// Claimed application audience; verification compares it with protected policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DelegationAudience {
    /// One network-qualified fleet identity, without fleet membership information.
    Fleet(AudienceId),
}

//
// DelegatedRoleGrant
//

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// Checked labels only; grant count, strict ordering and authority remain
/// engine/host policy. Serialized labels retain their exact text representation.
pub struct DelegatedRoleGrant {
    /// Role whose permissions this grant describes; interpretation is host-owned.
    pub target: AuthRole,
    /// Exact scope labels granted to the role, without implicit wildcard expansion.
    pub scopes: Vec<ScopeLabel>,
}

//
// AuthRequestMetadata
//

/// Caller-provided request correlation and requested lifetime, not authorization.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuthRequestMetadata {
    /// Opaque request identifier; uniqueness and retry policy belong to the host.
    pub request_id: [u8; 32],
    /// Requested lifetime in nanoseconds, subject to the host's admitted limits.
    pub ttl_ns: u64,
}
