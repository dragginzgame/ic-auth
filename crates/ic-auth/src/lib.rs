//! Pure application-token encoding and optional token/IC signature verification.
//!
//! Hashing untrusted material does not authenticate it. Applications must not
//! admit tokens by hashing them or checking just one signature. Use the optional
//! token verifier with protected host inputs; session admission and replay
//! consumption remain separate host operations.

pub mod canonical;

#[cfg(feature = "canister-signature-verification")]
pub mod canister_signature;

#[cfg(feature = "token-verification")]
pub mod token;
