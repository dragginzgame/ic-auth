//! Passive application authentication common contracts.
//! Adapted from Canic; see docs/design/canic-source-review.md.

use crate::{AudienceId, AuthRole};
use serde::{Deserialize, Serialize};

//
// DelegationAudience
//

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DelegationAudience {
    Fleet(AudienceId),
}

//
// DelegatedRoleGrant
//

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DelegatedRoleGrant {
    pub target: AuthRole,
    pub scopes: Vec<String>,
}

//
// AuthRequestMetadata
//

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuthRequestMetadata {
    pub request_id: [u8; 32],
    pub ttl_ns: u64,
}
