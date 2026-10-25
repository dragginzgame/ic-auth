//! Common portable claims checks; neither proof verification nor session admission.

use super::{PortableEncodingError, browser_principal, validate_shape};
use crate::certificate_issuance::{grants_are_subset, positive_duration, validate_grants};
use ic_auth_protocol_types::{
    AudienceId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};
use thiserror::Error;

/// Protected runtime and active enrollment inputs, never inferred from claims.
/// The host must select current, unrevoked authority under its transaction lock.
/// Generation, login revocation, replay and atomic session writes remain host/
/// session-engine obligations; this context cannot attest its own provenance.
#[derive(Clone, Copy, Debug)]
pub struct PortableClaimsContext<'a> {
    pub audience: &'a AudienceId,
    pub issuer: Principal,
    pub mechanism: PortableMechanism,
    pub key_binding_id: [u8; 32],
    pub grants: &'a [DelegatedRoleGrant],
    /// Actual authenticated ingress sender, not an account subject or request field.
    pub caller: Principal,
    pub now_ns: u64,
    pub authority_not_before_ns: u64,
    pub authority_expires_at_ns: u64,
    pub enrollment_expires_at_ns: u64,
    /// Immutable login start floor, including the host's qualified reset margin.
    pub min_login_started_at_ns: u64,
    pub min_proof_issued_at_ns: u64,
    /// Positive host limits, capped at 86,400 and 60 seconds respectively.
    pub max_login_ttl_ns: u64,
    pub max_proof_ttl_ns: u64,
}

impl PortableClaimsContext<'_> {
    // Shared structural authority policy, independent of clock, proof lifetime,
    // caller and session liveness. Each path preserves its own refusal type.
    pub(crate) fn authority_shape_valid(&self) -> bool {
        self.issuer != Principal::anonymous()
            && self.authority_not_before_ns < self.authority_expires_at_ns
            && validate_grants(
                self.grants,
                crate::limits::MAX_GRANTS,
                crate::limits::MAX_SCOPES_PER_GRANT,
            )
            .is_ok()
    }

    /// Advance both admission cutoffs after losing session/replay/revocation state.
    /// Uses this context's trusted boot time and deployment-qualified skew bound;
    /// the host must reject unavailable/rolled-back clocks before calling this.
    /// Floors never decrease, and overflow leaves both unchanged. This only
    /// calculates policy: the host must retain these values atomically with state
    /// invalidation, authenticate fresh enrollment and fail closed until ready.
    /// Do not call per request or discard the resulting floors after startup.
    pub fn advance_reset_floors(&mut self, clock_skew_ns: u64) -> Result<(), PortableClaimsError> {
        let proof_floor = self
            .now_ns
            .checked_add(1)
            .ok_or(PortableClaimsError::InvalidContext)?;
        let login_floor = proof_floor
            .checked_add(clock_skew_ns)
            .ok_or(PortableClaimsError::InvalidContext)?;
        self.min_login_started_at_ns = self.min_login_started_at_ns.max(login_floor);
        self.min_proof_issued_at_ns = self.min_proof_issued_at_ns.max(proof_floor);
        Ok(())
    }
}

/// Pure rejection of claims against protected context; success is not admission.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum PortableClaimsError {
    #[error("invalid protected portable claims context")]
    InvalidContext,
    #[error("portable audience does not match protected authority")]
    AudienceMismatch,
    #[error("portable issuer does not match protected authority")]
    IssuerMismatch,
    #[error("portable mechanism does not match protected authority")]
    MechanismMismatch,
    #[error("portable key binding does not match protected authority")]
    KeyMismatch,
    #[error("browser key does not identify the authenticated caller")]
    CallerMismatch,
    #[error("portable grants exceed protected authority")]
    GrantsExceeded,
    #[error("portable login or proof window is invalid")]
    InvalidWindow,
    #[error("portable proof is not yet valid")]
    NotYetValid,
    #[error("portable proof has expired")]
    Expired,
    #[error("portable login or issuance predates the protected reset floor")]
    BeforeReset,
    #[error("portable authority or enrollment is not current")]
    StaleAuthority,
    #[error(transparent)]
    Encoding(#[from] PortableEncodingError),
}

/// Check both mechanisms' caller, protected bindings, grants and exact time bounds.
///
/// Success authenticates nothing by itself. Refusal may precede cryptography;
/// notably `NotYetValid` is a bounded retry hint, never evidence of a valid proof.
/// No skew grace is added to admission. The host incorporates its qualified skew
/// bound into reset floors and tombstone retention, not the clock supplied here.
pub fn validate_claims_context(
    claims: &PortableClaims,
    context: &PortableClaimsContext<'_>,
) -> Result<(), PortableClaimsError> {
    use PortableClaimsError as Error;
    if context.max_login_ttl_ns == 0
        || context.max_login_ttl_ns > crate::limits::MAX_LOGIN_NS
        || context.max_proof_ttl_ns == 0
        || context.max_proof_ttl_ns > crate::limits::MAX_PROOF_NS
        || !context.authority_shape_valid()
        || context.enrollment_expires_at_ns == 0
    {
        return Err(Error::InvalidContext);
    }
    validate_shape(claims)?;
    if claims.audience != *context.audience {
        return Err(Error::AudienceMismatch);
    }
    if claims.issuer != context.issuer {
        return Err(Error::IssuerMismatch);
    }
    if claims.mechanism != context.mechanism {
        return Err(Error::MechanismMismatch);
    }
    if claims.key_binding_id != context.key_binding_id {
        return Err(Error::KeyMismatch);
    }
    if browser_principal(&claims.browser_public_key) != context.caller {
        return Err(Error::CallerMismatch);
    }
    if !grants_are_subset(&claims.grants, context.grants) {
        return Err(Error::GrantsExceeded);
    }
    if positive_duration(claims.login_started_at_ns, claims.login_deadline_ns)
        .is_none_or(|ttl| ttl > context.max_login_ttl_ns)
        || positive_duration(claims.issued_at_ns, claims.admission_expires_at_ns)
            .is_none_or(|ttl| ttl > context.max_proof_ttl_ns)
        || claims.login_started_at_ns > claims.issued_at_ns
        || claims.admission_expires_at_ns > claims.login_deadline_ns
    {
        return Err(Error::InvalidWindow);
    }
    if claims.login_started_at_ns < context.min_login_started_at_ns
        || claims.issued_at_ns < context.min_proof_issued_at_ns
    {
        return Err(Error::BeforeReset);
    }
    if context.now_ns < context.authority_not_before_ns
        || context.now_ns >= context.authority_expires_at_ns
        || context.now_ns >= context.enrollment_expires_at_ns
        || claims.issued_at_ns < context.authority_not_before_ns
    {
        return Err(Error::StaleAuthority);
    }
    if context.now_ns < claims.issued_at_ns {
        return Err(Error::NotYetValid);
    }
    if context.now_ns >= claims.admission_expires_at_ns {
        return Err(Error::Expired);
    }
    Ok(())
}
