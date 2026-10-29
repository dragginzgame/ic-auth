//! Portable Ed25519 issuer-signature verification, without signing or custody.
//!
//! This primitive authenticates claims against a host-selected key. It does not
//! establish TEE assurance, enroll that key, or admit a session. The admission
//! engine must still enforce current enrollment provenance, revocation, replay,
//! atomic storage and its shared work budget. Common claims checks run here.

use crate::admission_budget::{AdmissionBudgetError, SessionAdmissionBudget};
use crate::portable::{
    ContextualClaims, DecodedPortableClaims, PortableClaimsContext, PortableClaimsError,
    PortableEncodingError, VerifiedPortableClaims,
};
use ic_auth_protocol_types::PortableMechanism;
use ic_platform::ed25519::{Scalar, Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Rejection of a portable issuer proof; not a session-admission result.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PortableSignatureError {
    #[error(transparent)]
    Budget(#[from] AdmissionBudgetError),
    #[error(transparent)]
    Encoding(#[from] PortableEncodingError),
    #[error(transparent)]
    Claims(#[from] PortableClaimsError),
    #[error("invalid or weak Ed25519 public key")]
    InvalidPublicKey,
    #[error("Ed25519 signature must contain exactly 64 bytes")]
    MalformedSignature,
    #[error("invalid Ed25519 signature")]
    InvalidSignature,
}

/// Verify ordinary strict Ed25519 over the canonical claims' SHA-256 digest.
///
/// Context and `public_key` MUST come from current protected host authority,
/// with the actual runtime caller and clock, never from the proof alone.
/// Common claims checks precede cryptography; signature validity cannot override
/// future time, reset, authority expiry, caller mismatch or grant escalation.
/// This function charges the host budget before signature verification. Returned claims remain
/// inadmissible until the shared session engine completes all other checks.
///
/// Structural, mechanism and protected-binding checks precede cryptography.
/// Dalek's strict verifier rejects small-order keys/R points. An explicit
/// upstream canonical-scalar check also rejects noncanonical S if a consumer
/// enables Dalek's legacy compatibility feature. This is not Ed25519ph.
/// Use one persistent receiver-wide budget for both proof mechanisms. Failed
/// cryptography consumes credit. The host must preserve charges on normal errors
/// and subsequent storage failure; trapping the message would roll them back.
pub fn verify_ed25519_claims(
    budget: &mut SessionAdmissionBudget,
    encoded_claims: &[u8],
    signature: &[u8],
    context: &PortableClaimsContext<'_>,
    public_key: &[u8; 32],
) -> Result<VerifiedPortableClaims, PortableSignatureError> {
    verify_decoded(
        budget,
        DecodedPortableClaims::decode(encoded_claims)?,
        signature,
        context,
        public_key,
    )
}

pub(crate) fn verify_decoded(
    budget: &mut SessionAdmissionBudget,
    decoded: DecodedPortableClaims,
    signature: &[u8],
    context: &PortableClaimsContext<'_>,
    public_key: &[u8; 32],
) -> Result<VerifiedPortableClaims, PortableSignatureError> {
    if context.authority.mechanism != PortableMechanism::Ed25519 {
        return Err(PortableClaimsError::MechanismMismatch.into());
    }
    verify_prepared(
        budget,
        decoded.validate_context(context)?,
        signature,
        public_key,
    )
}

pub(crate) fn verify_prepared(
    budget: &mut SessionAdmissionBudget,
    prepared: ContextualClaims<'_, '_>,
    signature: &[u8],
    public_key: &[u8; 32],
) -> Result<VerifiedPortableClaims, PortableSignatureError> {
    let (
        DecodedPortableClaims {
            claims,
            fingerprint,
            ..
        },
        context,
    ) = prepared.into_parts();
    if context.authority.mechanism != PortableMechanism::Ed25519 {
        return Err(PortableClaimsError::MechanismMismatch.into());
    }
    let key_id: [u8; 32] = Sha256::digest(public_key).into();
    if context.authority.key_binding_id != key_id {
        return Err(PortableClaimsError::KeyMismatch.into());
    }
    let key = VerifyingKey::from_bytes(public_key)
        .map_err(|_| PortableSignatureError::InvalidPublicKey)?;
    if key.is_weak() {
        return Err(PortableSignatureError::InvalidPublicKey);
    }
    let signature =
        Signature::from_slice(signature).map_err(|_| PortableSignatureError::MalformedSignature)?;
    if bool::from(Scalar::from_canonical_bytes(*signature.s_bytes()).is_none()) {
        return Err(PortableSignatureError::InvalidSignature);
    }
    budget.consume(context.authority.now_ns)?;
    key.verify_strict(&fingerprint, &signature)
        .map_err(|_| PortableSignatureError::InvalidSignature)?;
    Ok(VerifiedPortableClaims::new(claims, fingerprint, context))
}
