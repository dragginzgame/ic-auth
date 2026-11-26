//! Authentication protocol operations without ambient caller, time or storage.
//!
//! Canonical claims and unsigned request identity establish structure only.
//! The optional verifier authenticates a proof against explicit host inputs;
//! neither protocol operations nor verification authorize application operations.

#[cfg(any(
    feature = "issuer-stable",
    feature = "ed25519-verification",
    feature = "certified-verification"
))]
mod browser_identity;
#[cfg(any(feature = "issuer-stable", feature = "certified-issuer-verification"))]
mod certified_message;
pub mod claims;
pub mod delegation_encoding;
mod grant_policy;
#[cfg(any(feature = "receiver-stable", test))]
mod receiver_policy;
pub mod session_request;
#[cfg(any(feature = "receiver-stable", feature = "issuer-stable"))]
pub mod stable_upgrade;

#[cfg(feature = "issuer-stable")]
pub mod issuer_memory;

#[cfg(all(
    feature = "receiver-stable",
    any(feature = "ed25519-verification", feature = "certified-verification")
))]
pub mod receiver_memory;

#[cfg(feature = "certified-issuer-verification")]
pub mod issuer_signature;

#[cfg(feature = "root-delegation-verification")]
pub mod root_delegation;

#[cfg(any(feature = "ed25519-verification", feature = "certified-verification"))]
pub mod verification;

#[cfg(any(
    feature = "issuer-stable",
    feature = "ed25519-verification",
    feature = "certified-verification"
))]
mod work_budget;

#[cfg(all(
    feature = "receiver-stable",
    not(any(feature = "ed25519-verification", feature = "certified-verification"))
))]
compile_error!("receiver-stable requires ed25519-verification or certified-verification");
