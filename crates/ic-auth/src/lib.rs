//! Authentication protocol operations without ambient caller, time or storage.
//!
//! Canonical claims and unsigned request identity establish structure only.
//! The optional verifier authenticates a proof against explicit host inputs;
//! neither protocol operations nor verification authorize application operations.

pub mod claims;
pub mod delegation_encoding;
mod grant_policy;
#[cfg(any(feature = "receiver-memory", test))]
mod receiver_policy;
pub mod session_request;

#[cfg(all(
    feature = "receiver-memory",
    not(any(feature = "ed25519-verification", feature = "certified-verification"))
))]
compile_error!("receiver-memory requires ed25519-verification or certified-verification");

#[cfg(all(
    feature = "receiver-memory",
    any(feature = "ed25519-verification", feature = "certified-verification")
))]
pub mod receiver_memory;

#[cfg(feature = "certified-issuer-verification")]
pub mod issuer_signature;

#[cfg(feature = "root-delegation-verification")]
pub mod root_delegation;

#[cfg(any(feature = "ed25519-verification", feature = "certified-verification"))]
pub mod verification;

#[cfg(any(feature = "ed25519-verification", feature = "certified-verification"))]
mod work_budget;
