//! Passive application-token contracts, independent of runtime and storage.
//!
//! Decoding these values does not verify a proof or establish authority. Hosts
//! must obtain expected audience, caller, time and issuer policy independently.
//! These tokens are not IC ingress delegations.

mod common;
mod identifiers;
mod proof;
mod token;

pub use candid::Principal;
pub use common::*;
pub use identifiers::{AudienceId, AuthRole, CanonicalId, IdentifierError};
pub use proof::*;
pub use token::*;
