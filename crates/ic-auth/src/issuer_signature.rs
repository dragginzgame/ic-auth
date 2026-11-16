//! The certified issuer-signature component, without root-delegation admission.
//!
//! This authenticates a fixed-domain claims digest under protected issuer/seed/
//! network inputs. It cannot produce verified claims or admit a credential/session.
//! The complete verifier must first establish root delegation, enforce aggregate
//! proof bounds and charge the receiver-wide budget once for the whole proof.

use ic_auth_protocol_types::Principal;
use ic_platform::{
    canister_sig_creation::CanisterSigPublicKey,
    signature_verification::{CanisterSigTimeError, verify_canister_sig_with_time},
};
use sha2::{Digest, Sha256};
use std::{error::Error, fmt};

pub use crate::certified_message::{CERTIFIED_SIGNATURE_DOMAIN, certified_signature_message};

/// Protected policy for the issuer-signature component of one certified proof.
///
/// Issuer and seed must be independently enrolled or established by authenticated
/// root delegation. The submitted DER key cannot supply either authority.
#[derive(Clone, Copy, Debug)]
pub struct IssuerSignaturePolicy<'a> {
    /// Approved issuer canister.
    pub signing_canister: Principal,
    /// Approved SHA-256 seed hash.
    pub seed_hash: [u8; 32],
    /// Protected raw 96-byte BLS network key, never proof-supplied DER.
    pub ic_root_public_key_raw: &'a [u8],
    /// Trusted host clock in nanoseconds.
    pub now_ns: u64,
    /// Inclusive maximum age of the authenticated signing certificate.
    pub max_certificate_age_ns: u64,
    /// Separate issuer CBOR limit, additionally capped at 131072 bytes.
    pub max_signature_bytes: usize,
}

/// Component refusal; no result here establishes complete credential authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssuerSignatureError {
    /// Signature bytes exceed the host or portable-profile limit before parsing.
    SignatureTooLarge,
    /// DER is malformed or outside the supported 20–129-byte short-form boundary.
    InvalidPublicKey,
    /// DER names another canister.
    CanisterMismatch,
    /// DER seed differs from the protected seed commitment.
    SeedMismatch,
    /// Protected network root is not a raw 96-byte BLS key.
    InvalidRootKeyLength,
    /// CBOR, witness, certificate, delegation chain or BLS authentication failed.
    InvalidSignature,
    /// Authenticated certificate lacks a complete u64 LEB128 time.
    InvalidCertificateTime,
    /// Authenticated signing certificate exceeds the inclusive maximum age.
    CertificateTooOld,
    /// Authenticated certificate is in the future; this profile has zero grace.
    CertificateInFuture,
}

impl fmt::Display for IssuerSignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SignatureTooLarge => "issuer signature exceeds its byte limit",
            Self::InvalidPublicKey => "invalid canister signature DER key",
            Self::CanisterMismatch => "issuer canister mismatch",
            Self::SeedMismatch => "issuer seed mismatch",
            Self::InvalidRootKeyLength => "network root key must contain 96 raw bytes",
            Self::InvalidSignature => "invalid canister signature or certificate",
            Self::InvalidCertificateTime => "invalid authenticated certificate time",
            Self::CertificateTooOld => "issuer certificate is too old",
            Self::CertificateInFuture => "issuer certificate is in the future",
        })
    }
}
impl Error for IssuerSignatureError {}

/// Authenticate the issuer signature and return its authenticated signing time.
///
/// Platform owns DER/CBOR, witness, certificate/delegation and BLS verification.
/// Auth checks protected signer/seed, input bounds and inclusive certificate age
/// with zero future skew. Only the signing certificate is subject to that age
/// window; an authenticated older root delegation certificate is not.
///
/// This verifies neither canonical claims nor root authorization of the issuer's
/// grants. It has no state effects or budget charging; the complete credential
/// verifier must meter the whole proof before calling its cryptographic components.
pub fn verify_issuer_signature(
    claims_digest: [u8; 32],
    signature_cbor: &[u8],
    public_key_der: &[u8],
    policy: &IssuerSignaturePolicy<'_>,
) -> Result<u64, IssuerSignatureError> {
    use IssuerSignatureError as E;
    if signature_cbor.len() > policy.max_signature_bytes.min(131_072) {
        return Err(E::SignatureTooLarge);
    }
    if !(20..=129).contains(&public_key_der.len()) {
        return Err(E::InvalidPublicKey);
    }
    let key = CanisterSigPublicKey::try_from(public_key_der).map_err(|_| E::InvalidPublicKey)?;
    if key.canister_id != policy.signing_canister {
        return Err(E::CanisterMismatch);
    }
    if <[u8; 32]>::from(Sha256::digest(&key.seed)) != policy.seed_hash {
        return Err(E::SeedMismatch);
    }
    if policy.ic_root_public_key_raw.len() != 96 {
        return Err(E::InvalidRootKeyLength);
    }
    let time = verify_canister_sig_with_time(
        &certified_signature_message(claims_digest),
        signature_cbor,
        public_key_der,
        policy.ic_root_public_key_raw,
    )
    .map_err(|error| match error {
        CanisterSigTimeError::Verification(_) => E::InvalidSignature,
        CanisterSigTimeError::InvalidCertificateTime => E::InvalidCertificateTime,
    })?;
    let age = policy
        .now_ns
        .checked_sub(time)
        .ok_or(E::CertificateInFuture)?;
    if age > policy.max_certificate_age_ns {
        return Err(E::CertificateTooOld);
    }
    Ok(time)
}
