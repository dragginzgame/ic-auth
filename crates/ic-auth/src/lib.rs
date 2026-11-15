//! Authentication protocol operations without ambient caller, time or storage.
//!
//! Canonical claims and unsigned request identity establish structure only.
//! They do not verify credentials or authorize application operations.

pub mod claims;
pub mod session_request;
