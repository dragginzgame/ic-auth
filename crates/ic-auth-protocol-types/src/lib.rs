//! Passive authentication values. Valid shape never establishes authority.
//!
//! Labels and claim fields are unauthenticated data. Canonical encoding belongs
//! to Auth; transport, proof verification, host policy and persistence are separate.

mod claims;
mod role;
mod scope;

pub use claims::{AudienceId, CanonicalId, DelegatedRoleGrant, PortableClaims, PortableMechanism};
pub use ic_platform_types::Principal;
pub use role::{AuthRole, RoleError};
pub use scope::{ScopeLabel, ScopeLabelError};
