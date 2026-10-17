//! Passive portable authentication and delegation contracts, independent of runtime, storage and Candid.
//! `Principal` re-exports the canonical narrow `ic_principal` type.
//! Transport schemas belong to endpoint adapters, outside these domain contracts.
//!
//! Decoding these values does not verify a proof or establish authority. Hosts
//! must obtain expected audience, caller, time and issuer policy independently.
//! Application credentials are not IC ingress delegations.

mod common;
mod identifiers;
mod logout;
mod portable;
mod proof;

pub use common::*;
pub use ic_principal::Principal;
pub use identifiers::{AudienceId, AuthRole, CanonicalId, IdentifierError};
pub use logout::{LogoutRequest, LogoutResult, LogoutUnavailable};
pub use portable::{PortableClaims, PortableCredential, PortableMechanism, PortableProof};
pub use proof::*;
