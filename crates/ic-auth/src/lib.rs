//! Pure application-token encoding and optional signature/token/session machinery.
//!
//! Hashing untrusted material does not authenticate it. Applications must not
//! admit tokens by hashing them or checking just one signature. Use the optional
//! token verifier with protected host inputs. The optional session engine uses
//! one explicit host transaction for admission and replay consumption.

pub mod canonical;

#[cfg(feature = "canister-signature-preparation")]
pub mod signature_store;

#[cfg(feature = "canister-signature-verification")]
pub mod canister_signature;

#[cfg(feature = "token-verification")]
pub mod token;

#[cfg(feature = "sessions")]
pub mod session;
