//! Portable authentication, delegation proofs and optional local session machinery.
//!
//! Hashing untrusted material does not authenticate it. Applications must not
//! admit credentials by hashing them or checking just one signature. Application
//! authentication admits a credential through the optional session engine, then checks
//! the caller-bound session for each operation with current protected host inputs.
//! Admission uses one explicit host transaction for verification and replay
//! consumption. Standalone proof verification establishes no session or resource grant.

pub mod admission_budget;
pub mod canonical;
pub mod certificate_issuance;
pub mod chain_key_batch;
mod limits;
pub mod portable;

#[cfg(any(
    feature = "portable-signature-verification",
    feature = "certified-proof-verification"
))]
pub mod portable_credential;
pub mod session_request;

#[cfg(feature = "portable-signature-verification")]
pub mod portable_signature;

#[cfg(feature = "chain-key-verification")]
pub mod chain_key_signature;

#[cfg(feature = "canister-signature-preparation")]
pub mod signature_store;

#[cfg(feature = "canister-signature-verification")]
pub mod canister_signature;

#[cfg(feature = "chain-key-verification")]
pub mod delegation;

#[cfg(feature = "certified-proof-verification")]
pub mod portable_certified;

#[cfg(feature = "sessions")]
pub mod session;

#[cfg(feature = "sessions")]
mod storage_codec;
