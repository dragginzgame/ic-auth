//! Pure application-token encoding and optional IC canister-signature verification.
//!
//! Hashing untrusted material does not authenticate it. Applications must not
//! admit tokens using this package until complete proof verification is supplied.
//! Signature verification alone does not check token grants, expiry or authority.

pub mod canonical;

#[cfg(feature = "canister-signature-verification")]
pub mod canister_signature;
