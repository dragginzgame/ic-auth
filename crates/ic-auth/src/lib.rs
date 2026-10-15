//! Pure application-token encoding and optional signature/token/session machinery.
//!
//! Hashing untrusted material does not authenticate it. Applications must not
//! admit tokens by hashing them or checking just one signature. Application
//! authentication admits a token through the optional session engine, then checks
//! the caller-bound session for each operation with current protected host inputs.
//! Admission uses one explicit host transaction for verification and replay
//! consumption. Standalone token verification remains a lower-level proof
//! primitive; it does not establish a session or authorize application resources.

pub mod admission_budget;
pub mod canonical;
pub mod certificate_issuance;
pub mod chain_key_batch;
pub mod portable;

#[cfg(any(
    feature = "portable-signature-verification",
    feature = "token-verification"
))]
pub mod portable_credential;
pub mod session_request;
pub mod token_preparation;

#[cfg(feature = "portable-signature-verification")]
pub mod portable_signature;

#[cfg(feature = "chain-key-verification")]
pub mod chain_key_signature;

#[cfg(feature = "canister-signature-preparation")]
pub mod signature_store;

#[cfg(feature = "canister-signature-verification")]
pub mod canister_signature;

#[cfg(feature = "token-verification")]
pub mod token;

#[cfg(feature = "token-verification")]
pub mod portable_certified;

#[cfg(feature = "sessions")]
pub mod session;
