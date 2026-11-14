//! Portable certified proofs using the existing Root and IC signature verifiers.
//! No session, replay, enrollment installation or certification-root effects.

use crate::admission_budget::{AdmissionBudgetError, SessionAdmissionBudget};
use crate::{
    canister_signature::{
        CanisterSignatureError, CanisterSignaturePolicy, verify_canister_signature,
    },
    canonical::CanonicalAuthError,
    certificate_issuance::grants_are_subset,
    delegation::{
        DelegationProofError, DelegationProofVerificationContext,
        DelegationProofVerificationLimits, PreparedDelegationProof, RootKeyPolicy,
        rules::check_proof_size,
    },
    portable::{
        ContextualClaims, DecodedPortableClaims, PortableClaimsContext, PortableClaimsError,
        PortableEncodingError, VerifiedPortableClaims,
    },
};
use ic_auth_protocol_types::{
    DelegationAudience, DelegationProof, IcCanisterSignatureProofV1, IssuerProofBinding,
    PortableMechanism,
};
use thiserror::Error;

/// IC signature framing domain, distinct from the old delegated-token message.
/// Producers sign `domain_separated_message(domain, SHA256(canonical_claims))`.
pub const CERTIFIED_SIGNATURE_DOMAIN: &[u8] = b"ic-auth-portable-credential";

/// Host-selected bounds. Canic adapters retain their configured values; there
/// are no implicit clock allowances or limits selected from submitted proofs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CertifiedProofLimits {
    /// Upper bound on root-authorized issuer certificate lifetime.
    pub max_cert_ttl_ns: u64,
    /// Upper bound on the issuer's authorized maximum token lifetime.
    pub max_token_ttl_ns: u64,
    /// Inclusive maximum age of the issuer's authenticated IC certificate.
    pub max_certificate_age_ns: u64,
    /// Budget for vector/string bytes plus one byte per variable entry, checked
    /// before canonical allocation or cryptographic work. Not a Candid wire limit.
    pub max_variable_bytes: usize,
    /// Separate bound on issuer signature CBOR before parsing.
    pub max_issuer_signature_bytes: usize,
    /// Separate bound on the root proof's Merkle path length.
    pub max_witness_steps: usize,
}

impl From<CertifiedProofLimits> for DelegationProofVerificationLimits {
    fn from(limits: CertifiedProofLimits) -> Self {
        Self {
            max_cert_ttl_ns: limits.max_cert_ttl_ns,
            max_token_ttl_ns: limits.max_token_ttl_ns,
            max_future_skew_ns: 0,
            max_variable_bytes: limits.max_variable_bytes,
            max_witness_steps: limits.max_witness_steps,
        }
    }
}

/// Protected portable, Root and network authority for a single proof check.
/// All fields must come from runtime/configuration, never the submitted proof.
pub struct PortableCertifiedContext<'a> {
    /// Protected caller, issuer enrollment, time and admission bounds.
    pub claims: PortableClaimsContext<'a>,
    /// Independently enrolled root authority for delegation verification.
    pub root_key: &'a RootKeyPolicy,
    /// Protected raw IC network BLS trust anchor, not proof-supplied key material.
    pub ic_root_public_key_raw: &'a [u8],
    /// Bounded proof policy. This profile always refuses future validity starts.
    pub limits: CertifiedProofLimits,
}

/// Proof refusal, never a session-admission result.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum PortableCertifiedError {
    #[error(transparent)]
    /// The shared work budget refuses this verification attempt.
    Budget(#[from] AdmissionBudgetError),
    #[error("portable certified context or delegation binding mismatch")]
    /// Delegation or mechanism does not match the protected portable authority.
    BindingMismatch,
    #[error(transparent)]
    /// Claims fail protected identity, grant or lifetime checks.
    Claims(#[from] PortableClaimsError),
    #[error(transparent)]
    /// Claims fail bounded canonical encoding admission.
    Encoding(#[from] PortableEncodingError),
    #[error(transparent)]
    /// Delegation labels or canonical field encoding are invalid.
    Canonical(#[from] CanonicalAuthError),
    #[error(transparent)]
    /// Root delegation verification failed under the enrolled authority.
    Delegation(#[from] DelegationProofError),
    #[error(transparent)]
    /// Issuer canister signature or certificate verification failed.
    Signature(#[from] CanisterSignatureError),
}

/// Verify bounded canonical claims, protected context, Root delegation and the
/// issuer's IC signature. The two existing cryptographic owners are reused.
///
/// Success authenticates claims only at these protected inputs. Current enrolled
/// authority provenance, replay/revocation and atomic session
/// admission remain mandatory. Outer decoding must also be bounded by the host.
/// Use one persistent receiver-wide budget for both proof mechanisms. Failed
/// cryptography consumes credit. The host must preserve charges on normal errors
/// and subsequent storage failure; trapping the message would roll them back.
pub fn verify_certified_claims(
    budget: &mut SessionAdmissionBudget,
    encoded_claims: &[u8],
    delegation: &DelegationProof,
    issuer_signature: &IcCanisterSignatureProofV1,
    context: &PortableCertifiedContext<'_>,
) -> Result<VerifiedPortableClaims, PortableCertifiedError> {
    check_bounds(encoded_claims.len(), delegation, issuer_signature, context)?;
    verify_decoded(
        budget,
        DecodedPortableClaims::decode(encoded_claims)?,
        delegation,
        issuer_signature,
        context,
    )
}

pub(crate) fn check_bounds(
    encoded_len: usize,
    delegation: &DelegationProof,
    issuer_signature: &IcCanisterSignatureProofV1,
    context: &PortableCertifiedContext<'_>,
) -> Result<(), PortableCertifiedError> {
    let limits = context.limits;
    if context.claims.authority.mechanism != PortableMechanism::Certified {
        return Err(PortableCertifiedError::BindingMismatch);
    }
    // Aggregate variable-material cap precedes codec allocation and cryptography.
    let mut remaining = limits.max_variable_bytes.min(131_072);
    for length in [
        encoded_len,
        issuer_signature.signature_cbor.len(),
        issuer_signature.public_key_der.len(),
    ] {
        remaining = remaining
            .checked_sub(length)
            .ok_or(DelegationProofError::InputTooLarge)?;
    }
    if issuer_signature.signature_cbor.len() > limits.max_issuer_signature_bytes {
        return Err(DelegationProofError::InputTooLarge.into());
    }
    check_proof_size(delegation, limits.max_witness_steps, &mut remaining)?;
    Ok(())
}

// Callers admit aggregate proof bounds before this verification step.
pub(crate) fn verify_decoded(
    budget: &mut SessionAdmissionBudget,
    decoded: DecodedPortableClaims,
    delegation: &DelegationProof,
    issuer_signature: &IcCanisterSignatureProofV1,
    context: &PortableCertifiedContext<'_>,
) -> Result<VerifiedPortableClaims, PortableCertifiedError> {
    verify_prepared(
        budget,
        decoded.validate_context(&context.claims)?,
        delegation,
        issuer_signature,
        context,
    )
}

pub(crate) fn verify_prepared(
    budget: &mut SessionAdmissionBudget,
    checked: ContextualClaims<'_, '_>,
    delegation: &DelegationProof,
    issuer_signature: &IcCanisterSignatureProofV1,
    context: &PortableCertifiedContext<'_>,
) -> Result<VerifiedPortableClaims, PortableCertifiedError> {
    let limits = context.limits;
    let (
        DecodedPortableClaims {
            claims,
            fingerprint,
            ..
        },
        claims_context,
        browser_caller,
    ) = checked.into_parts();
    let prepared = PreparedDelegationProof::new(delegation)?;
    let cert = &delegation.cert;
    if prepared.certificate_hash() != claims.key_binding_id
        || cert.issuer_pid != claims_context.authority.issuer
        || cert.aud != DelegationAudience::Fleet(*claims_context.authority.audience)
        || !grants_are_subset(&claims.grants, &cert.grants)
        || claims.issued_at_ns < cert.not_before_ns
        || claims.admission_expires_at_ns > cert.expires_at_ns
        || claims.admission_expires_at_ns - claims.issued_at_ns > cert.max_token_ttl_ns
        || claims_context.authority.authority_not_before_ns < cert.not_before_ns
        || claims_context.authority.authority_expires_at_ns
            > cert.expires_at_ns.min(context.root_key.accept_until_ns)
    {
        return Err(PortableCertifiedError::BindingMismatch);
    }
    budget.consume_verification(
        claims_context.authority.now_ns,
        ic_auth_protocol_types::PortableMechanism::Certified,
    )?;
    prepared.verify(&DelegationProofVerificationContext {
        expected_issuer: claims_context.authority.issuer,
        root_key: context.root_key,
        now_ns: claims_context.authority.now_ns,
        limits: limits.into(),
    })?;
    let IssuerProofBinding::IcCanisterSignatureV1 { seed_hash } = cert.issuer_proof_binding;
    // This profile has one fixed domain and a 32-byte digest. Preserve the IC
    // length/domain/payload framing without allocating the general Vec message.
    const DOMAIN_LEN: usize = CERTIFIED_SIGNATURE_DOMAIN.len();
    const { assert!(DOMAIN_LEN <= u8::MAX as usize) };
    let mut message = [0; 1 + DOMAIN_LEN + 32];
    message[0] = DOMAIN_LEN as u8;
    message[1..1 + DOMAIN_LEN].copy_from_slice(CERTIFIED_SIGNATURE_DOMAIN);
    message[1 + DOMAIN_LEN..].copy_from_slice(&fingerprint);
    verify_canister_signature(
        &message,
        issuer_signature,
        &CanisterSignaturePolicy {
            signing_canister: claims_context.authority.issuer,
            seed_hash,
            ic_root_public_key_raw: context.ic_root_public_key_raw,
            now_ns: claims_context.authority.now_ns,
            max_certificate_age_ns: limits.max_certificate_age_ns,
            max_future_skew_ns: 0,
            max_signature_bytes: limits.max_issuer_signature_bytes,
            max_message_bytes: message.len(),
        },
    )?;
    Ok(VerifiedPortableClaims::new(
        claims,
        fingerprint,
        browser_caller,
        claims_context,
    ))
}
