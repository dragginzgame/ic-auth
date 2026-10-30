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
    let prepared = prepare_claims(DecodedPortableClaims::decode(encoded_claims)?, context)?;
    let binding = Sha256::digest(public_key).into();
    if context.authority.key_binding_id != binding {
        return Err(PortableClaimsError::KeyMismatch.into());
    }
    let key = PreparedIssuerKey::parse(*public_key, binding)?;
    verify_prepared(budget, prepared, signature, &key)
}

/// Immutable Ed25519 issuer key admitted with Platform's parser/weak-key checks.
/// Retain it with protected enrollment and rebuild after configuration decoding.
/// This proves key shape only, never enrollment, possession or proof validity.
/// Current mechanism/binding, authority, clock, grants and revocation still apply.
/// Original bytes are retained unchanged; there is no unchecked deserializer.
/// ```compile_fail
/// use ic_auth::portable_signature::PreparedIssuerKey;
/// fn change(key: &mut PreparedIssuerKey) { key.bytes()[0] = 0; }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedIssuerKey {
    bytes: [u8; 32],
    binding: [u8; 32],
    key: VerifyingKey,
}
impl PreparedIssuerKey {
    pub fn new(bytes: [u8; 32]) -> Result<Self, PortableSignatureError> {
        Self::parse(bytes, Sha256::digest(bytes).into())
    }
    fn parse(bytes: [u8; 32], binding: [u8; 32]) -> Result<Self, PortableSignatureError> {
        let key = VerifyingKey::from_bytes(&bytes)
            .map_err(|_| PortableSignatureError::InvalidPublicKey)?;
        if key.is_weak() {
            return Err(PortableSignatureError::InvalidPublicKey);
        }
        Ok(Self {
            bytes,
            binding,
            key,
        })
    }
    pub fn bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
    pub fn binding(&self) -> [u8; 32] {
        self.binding
    }
}

pub(crate) fn verify_with_key(
    budget: &mut SessionAdmissionBudget,
    encoded_claims: &[u8],
    signature: &[u8],
    context: &PortableClaimsContext<'_>,
    public_key: &PreparedIssuerKey,
) -> Result<VerifiedPortableClaims, PortableSignatureError> {
    verify_prepared(
        budget,
        prepare_claims(DecodedPortableClaims::decode(encoded_claims)?, context)?,
        signature,
        public_key,
    )
}

fn prepare_claims<'a, 'ctx>(
    decoded: DecodedPortableClaims,
    context: &'a PortableClaimsContext<'ctx>,
) -> Result<ContextualClaims<'a, 'ctx>, PortableSignatureError> {
    if context.authority.mechanism != PortableMechanism::Ed25519 {
        return Err(PortableClaimsError::MechanismMismatch.into());
    }
    Ok(decoded.validate_context(context)?)
}

pub(crate) fn verify_prepared(
    budget: &mut SessionAdmissionBudget,
    prepared: ContextualClaims<'_, '_>,
    signature: &[u8],
    public_key: &PreparedIssuerKey,
) -> Result<VerifiedPortableClaims, PortableSignatureError> {
    let (
        DecodedPortableClaims {
            claims,
            fingerprint,
            ..
        },
        context,
        browser_caller,
    ) = prepared.into_parts();
    if context.authority.mechanism != PortableMechanism::Ed25519 {
        return Err(PortableClaimsError::MechanismMismatch.into());
    }
    if context.authority.key_binding_id != public_key.binding {
        return Err(PortableClaimsError::KeyMismatch.into());
    }
    let signature =
        Signature::from_slice(signature).map_err(|_| PortableSignatureError::MalformedSignature)?;
    if bool::from(Scalar::from_canonical_bytes(*signature.s_bytes()).is_none()) {
        return Err(PortableSignatureError::InvalidSignature);
    }
    budget.consume(context.authority.now_ns)?;
    public_key
        .key
        .verify_strict(&fingerprint, &signature)
        .map_err(|_| PortableSignatureError::InvalidSignature)?;
    Ok(VerifiedPortableClaims::new(
        claims,
        fingerprint,
        browser_caller,
        context,
    ))
}
