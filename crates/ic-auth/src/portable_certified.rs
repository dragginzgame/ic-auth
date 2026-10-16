//! Portable certified proofs using the existing Root and IC signature verifiers.
//! No session, replay, enrollment installation or certification-root effects.

use crate::admission_budget::{AdmissionBudgetError, SessionAdmissionBudget};
use crate::{
    canister_signature::{
        CanisterSignatureError, CanisterSignaturePolicy, domain_separated_message,
        verify_canister_signature,
    },
    canonical::{CanonicalAuthError, cert_hash},
    certificate_issuance::grants_are_subset,
    portable::{
        DecodedPortableClaims, PortableClaimsContext, PortableClaimsError, PortableEncodingError,
        VerifiedPortableClaims, validate_claims_context,
    },
    token::{
        DelegationProofVerificationContext, RootKeyPolicy, TokenVerificationError,
        TokenVerificationLimits, rules::check_proof_size, verify_delegation_proof,
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

/// Protected portable, Root and network authority for a single proof check.
/// All fields must come from runtime/configuration, never the submitted proof.
pub struct PortableCertifiedContext<'a> {
    pub claims: PortableClaimsContext<'a>,
    pub root_key: &'a RootKeyPolicy,
    pub ic_root_public_key_raw: &'a [u8],
    /// Existing bounded proof policy. Future skew MUST be zero for this profile.
    pub limits: TokenVerificationLimits,
}

/// Proof refusal, never a session-admission result.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum PortableCertifiedError {
    #[error(transparent)]
    Budget(#[from] AdmissionBudgetError),
    #[error("portable certified context or delegation binding mismatch")]
    BindingMismatch,
    #[error(transparent)]
    Claims(#[from] PortableClaimsError),
    #[error(transparent)]
    Encoding(#[from] PortableEncodingError),
    #[error(transparent)]
    Canonical(#[from] CanonicalAuthError),
    #[error(transparent)]
    Delegation(#[from] TokenVerificationError),
    #[error(transparent)]
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
    if context.claims.mechanism != PortableMechanism::Certified || limits.max_future_skew_ns != 0 {
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
            .ok_or(TokenVerificationError::InputTooLarge)?;
    }
    if issuer_signature.signature_cbor.len() > limits.max_issuer_signature_bytes {
        return Err(TokenVerificationError::InputTooLarge.into());
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
    let limits = context.limits;
    let DecodedPortableClaims {
        claims,
        fingerprint,
        ..
    } = decoded;
    validate_claims_context(&claims, &context.claims)?;
    let cert = &delegation.cert;
    if cert_hash(cert)? != claims.key_binding_id
        || cert.issuer_pid != context.claims.issuer
        || cert.aud != DelegationAudience::Fleet(*context.claims.audience)
        || !grants_are_subset(&claims.grants, &cert.grants)
        || claims.issued_at_ns < cert.not_before_ns
        || claims.admission_expires_at_ns > cert.expires_at_ns
        || claims.admission_expires_at_ns - claims.issued_at_ns > cert.max_token_ttl_ns
        || context.claims.authority_not_before_ns < cert.not_before_ns
        || context.claims.authority_expires_at_ns
            > cert.expires_at_ns.min(context.root_key.accept_until_ns)
    {
        return Err(PortableCertifiedError::BindingMismatch);
    }
    budget.consume(context.claims.now_ns)?;
    verify_delegation_proof(
        delegation,
        &DelegationProofVerificationContext {
            expected_issuer: context.claims.issuer,
            root_key: context.root_key,
            now_ns: context.claims.now_ns,
            limits: limits.into(),
        },
    )?;
    let IssuerProofBinding::IcCanisterSignatureV1 { seed_hash } = cert.issuer_proof_binding;
    let message = domain_separated_message(CERTIFIED_SIGNATURE_DOMAIN, fingerprint)?;
    verify_canister_signature(
        &message,
        issuer_signature,
        &CanisterSignaturePolicy {
            signing_canister: context.claims.issuer,
            seed_hash,
            ic_root_public_key_raw: context.ic_root_public_key_raw,
            now_ns: context.claims.now_ns,
            max_certificate_age_ns: limits.max_certificate_age_ns,
            max_future_skew_ns: 0,
            max_signature_bytes: limits.max_issuer_signature_bytes,
            max_message_bytes: message.len(),
        },
    )?;
    Ok(VerifiedPortableClaims::new(
        claims,
        fingerprint,
        &context.claims,
    ))
}
