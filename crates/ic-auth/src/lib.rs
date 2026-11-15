//! Authentication protocol operations without ambient caller, time or storage.
//!
//! Canonical claims and unsigned request identity establish structure only.
//! The optional verifier authenticates a proof against explicit host inputs;
//! neither protocol operations nor verification authorize application operations.

pub mod claims;
pub mod delegation_encoding;
pub mod session_request;

#[cfg(feature = "certified-issuer-verification")]
pub mod issuer_signature;

#[cfg(feature = "ed25519-verification")]
pub mod verification;
