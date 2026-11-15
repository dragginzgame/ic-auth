//! Complete fresh certified-proof verification at one protected host snapshot.

use super::{
    BudgetError, ContextError, VerificationBudget, VerificationContext, VerifiedClaims, context,
};
use crate::{
    claims::{ClaimsError, decode_claims},
    delegation_encoding::{DelegationEncodingError, certificate_hash},
    issuer_signature::{IssuerSignatureError, IssuerSignaturePolicy, verify_issuer_signature},
    root_delegation::{
        DelegationError, DelegationLimits, RootDelegationProof, RootKeyPolicy, consume_proof_bytes,
        verify_authorization,
    },
};
use ic_auth_protocol_types::PortableMechanism;
use sha2::{Digest, Sha256};
use std::{error::Error, fmt};

/// Borrowed complete certified proof; all fields remain untrusted on input.
#[derive(Clone, Copy, Debug)]
pub struct CertifiedProof<'a> {
    /// Root-signed issuer authorization and certificate.
    pub delegation: RootDelegationProof<'a>,
    /// IC canister signature over the exact canonical claims digest and domain.
    pub issuer_signature_cbor: &'a [u8],
    /// Submitted issuer DER key, checked against root-authorized issuer and seed.
    pub issuer_public_key_der: &'a [u8],
}

impl<'a> From<&'a ic_auth_protocol_types::CertifiedProofMaterial> for CertifiedProof<'a> {
    fn from(value: &'a ic_auth_protocol_types::CertifiedProofMaterial) -> Self {
        Self {
            delegation: (&value.delegation).into(),
            issuer_signature_cbor: &value.issuer_signature_cbor,
            issuer_public_key_der: &value.issuer_public_key_der,
        }
    }
}

/// Protected proof bounds, additional to the transport's resource allowances.
#[derive(Clone, Copy, Debug)]
pub struct CertifiedLimits {
    /// Certificate, delegated-token and witness bounds. Its variable allowance
    /// applies to the complete proof, including claims and issuer bytes.
    pub delegation: DelegationLimits,
    /// Separate issuer CBOR byte cap before decoding or charging.
    pub max_issuer_signature_bytes: usize,
    /// Inclusive maximum age of the authenticated issuer signing certificate.
    pub max_certificate_age_ns: u64,
}

/// Protected claims, root enrollment and network trust for one atomic host view.
#[derive(Clone, Copy, Debug)]
pub struct CertifiedContext<'a> {
    /// Current protected caller, clock, audience, enrollment, grants and floors.
    pub claims: VerificationContext<'a>,
    /// Independently enrolled root chain key and current revocation floors.
    pub root_key: RootKeyPolicy<'a>,
    /// Protected raw IC network BLS root; never taken from the submitted proof.
    pub ic_root_public_key_raw: &'a [u8],
    /// Host-selected aggregate, lifetime, age and witness limits.
    pub limits: CertifiedLimits,
}

/// Fresh certified-proof refusal; session and replay policy remain separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertifiedError {
    /// Aggregate material, field or witness bounds refused before decode/charging.
    InputTooLarge,
    /// Claims failed bounded canonical decoding before charging.
    Encoding(ClaimsError),
    /// Claims failed shared protected-context checks before charging.
    Context(ContextError),
    /// Delegation certificate failed canonical shape/order before charging.
    DelegationEncoding(DelegationEncodingError),
    /// Certificate identity, grants or window does not bind claims/enrollment.
    BindingMismatch,
    /// Shared receiver budget refused the attempt before root verification.
    Budget(BudgetError),
    /// Root authorization failed after one retained certified-work charge.
    Root(DelegationError),
    /// Issuer signature/certificate failed after the same single retained charge.
    Issuer(IssuerSignatureError),
}
impl fmt::Display for CertifiedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "certified proof refused: {self:?}")
    }
}
impl Error for CertifiedError {}

/// Verify the complete root/issuer proof and return immutable claims evidence.
///
/// Aggregate admission, canonical claims, shared context and certificate binding
/// precede charging. Charge once before root authorization; retain that charge
/// if root or issuer checks fail. Both cryptographic components share the same
/// protected clock and enrollment snapshot. Successful evidence still requires
/// current session/replay/revocation policy and host-owned atomic storage.
pub fn verify_certified(
    encoded_claims: &[u8],
    proof: CertifiedProof<'_>,
    host: &CertifiedContext<'_>,
    budget: &mut VerificationBudget,
) -> Result<VerifiedClaims, CertifiedError> {
    use CertifiedError as E;
    let context = &host.claims;
    if context.mechanism != PortableMechanism::Certified {
        return Err(E::BindingMismatch);
    }
    let mut remaining = host.limits.delegation.max_variable_bytes.min(131_072);
    for length in [
        encoded_claims.len(),
        proof.issuer_signature_cbor.len(),
        proof.issuer_public_key_der.len(),
    ] {
        remaining = remaining.checked_sub(length).ok_or(E::InputTooLarge)?;
    }
    if proof.issuer_signature_cbor.len() > host.limits.max_issuer_signature_bytes {
        return Err(E::InputTooLarge);
    }
    consume_proof_bytes(
        &proof.delegation,
        host.limits.delegation.max_witness_steps,
        &mut remaining,
    )
    .map_err(|_| E::InputTooLarge)?;
    let claims = decode_claims(encoded_claims).map_err(E::Encoding)?;
    let caller = context::check(&claims, context).map_err(E::Context)?;
    let certificate = proof.delegation.certificate;
    let delegation = &certificate.delegation;
    let binding = certificate_hash(certificate).map_err(E::DelegationEncoding)?;
    // Context already checked strict positive claim durations, so subtraction
    // below cannot wrap. No sorting, key enrollment or binding repair is allowed.
    if binding != claims.key_binding_id
        || delegation.issuer != context.issuer
        || delegation.audience != *context.audience
        || !context::grants_subset(&claims.grants, &delegation.grants)
        || claims.issued_at_ns < delegation.not_before_ns
        || claims.admission_expires_at_ns > delegation.expires_at_ns
        || claims.admission_expires_at_ns - claims.issued_at_ns > delegation.max_token_ttl_ns
        || context.authority_not_before_ns < delegation.not_before_ns
        || context.authority_expires_at_ns
            > delegation.expires_at_ns.min(host.root_key.accept_until_ns)
    {
        return Err(E::BindingMismatch);
    }
    let fingerprint: [u8; 32] = Sha256::digest(encoded_claims).into();
    budget
        .charge(context.now_ns, PortableMechanism::Certified)
        .map_err(E::Budget)?;
    verify_authorization(
        proof.delegation,
        context.issuer,
        &host.root_key,
        context.now_ns,
        host.limits.delegation,
    )
    .map_err(E::Root)?;
    verify_issuer_signature(
        fingerprint,
        proof.issuer_signature_cbor,
        proof.issuer_public_key_der,
        &IssuerSignaturePolicy {
            signing_canister: context.issuer,
            seed_hash: delegation.seed_hash,
            ic_root_public_key_raw: host.ic_root_public_key_raw,
            now_ns: context.now_ns,
            max_certificate_age_ns: host.limits.max_certificate_age_ns,
            max_signature_bytes: host.limits.max_issuer_signature_bytes,
        },
    )
    .map_err(E::Issuer)?;
    Ok(VerifiedClaims::from_verification(
        claims,
        fingerprint,
        caller,
        context,
    ))
}
