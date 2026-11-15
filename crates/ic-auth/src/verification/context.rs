//! Shared fresh-claim policy, independent of proof cryptography and storage.

use crate::claims::{ClaimsError, validate_grants};
use ic_auth_protocol_types::{
    AudienceId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};
use std::{error::Error, fmt};

/// Immutable structurally checked authority grants; this does not enroll them.
///
/// ```compile_fail
/// use ic_auth::verification::AuthorityGrants;
/// fn change(value: &mut AuthorityGrants) { value.as_slice()[0].scopes.clear(); }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct AuthorityGrants(Vec<DelegatedRoleGrant>);

impl AuthorityGrants {
    /// Admit strict role/scope order with 1–16 roles and 1–32 scopes per role.
    ///
    /// Unlike fresh credentials, protected enrollment may exceed 32 total scopes.
    pub fn new(grants: Vec<DelegatedRoleGrant>) -> Result<Self, ClaimsError> {
        validate_grants(&grants)?;
        Ok(Self(grants))
    }

    /// Borrow checked grants without granting authority to their source.
    #[must_use]
    pub fn as_slice(&self) -> &[DelegatedRoleGrant] {
        &self.0
    }
}

/// Explicit current host inputs; none may be derived solely from the request.
///
/// The host must select current, unrevoked enrollment under its transaction.
/// These plain values cannot attest their provenance or fence lost storage.
#[derive(Clone, Copy, Debug)]
pub struct VerificationContext<'a> {
    /// Protected receiver audience.
    pub audience: &'a AudienceId,
    /// Enrolled issuer identity.
    pub issuer: Principal,
    /// Mechanism selected by protected enrollment, never the supplied proof.
    pub mechanism: PortableMechanism,
    /// Enrolled issuer key binding.
    pub key_binding_id: [u8; 32],
    /// Structurally checked current authority grants.
    pub grants: &'a AuthorityGrants,
    /// Actual authenticated ingress caller.
    pub caller: Principal,
    /// Trusted host time, in nanoseconds.
    pub now_ns: u64,
    /// Inclusive enrolled authority start.
    pub authority_not_before_ns: u64,
    /// Exclusive enrolled key/authority expiry.
    pub authority_expires_at_ns: u64,
    /// Exclusive host enrollment freshness expiry.
    pub enrollment_expires_at_ns: u64,
    /// Protected login-start floor; not a replacement for revocation state.
    pub min_login_started_at_ns: u64,
    /// Protected credential-issuance floor.
    pub min_proof_issued_at_ns: u64,
    /// Positive maximum login lifetime, at most 90 days.
    pub max_login_ttl_ns: u64,
    /// Positive maximum fresh-proof lifetime, at most 60 seconds.
    pub max_proof_ttl_ns: u64,
}

/// Contextual refusal before signature verification or work charging.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextError {
    /// Invalid host policy or authority window.
    InvalidContext,
    /// Audience differs from protected enrollment.
    AudienceMismatch,
    /// Issuer differs from protected enrollment.
    IssuerMismatch,
    /// Signed or verifier mechanism differs from protected enrollment.
    MechanismMismatch,
    /// Signed or supplied key binding differs from protected enrollment.
    KeyMismatch,
    /// The signed browser key does not identify the authenticated caller.
    CallerMismatch,
    /// A signed role/scope exceeds that same role in protected grants.
    GrantsExceeded,
    /// Inconsistent, empty or excessive login/proof durations.
    InvalidWindow,
    /// Signed login or issuance predates a protected reset floor.
    BeforeReset,
    /// Authority/enrollment is not current or issuance predates authority.
    StaleAuthority,
    /// Issuance is in the future; this is no proof of authenticity.
    NotYetValid,
    /// Exclusive fresh-admission expiry has been reached.
    Expired,
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidContext => "invalid protected verification context",
            Self::AudienceMismatch => "credential audience mismatch",
            Self::IssuerMismatch => "credential issuer mismatch",
            Self::MechanismMismatch => "credential mechanism mismatch",
            Self::KeyMismatch => "credential key binding mismatch",
            Self::CallerMismatch => "browser key does not identify the host caller",
            Self::GrantsExceeded => "credential grants exceed protected authority",
            Self::InvalidWindow => "invalid credential lifetime window",
            Self::BeforeReset => "credential predates protected reset floor",
            Self::StaleAuthority => "authority or enrollment is not current",
            Self::NotYetValid => "credential issuance is in the future",
            Self::Expired => "credential has expired for fresh admission",
        })
    }
}
impl Error for ContextError {}

/// Derive the standard IC self-authenticating caller for a raw browser key.
/// This checks neither curve membership nor key possession/ingress provenance.
#[must_use]
pub fn browser_principal(public_key: &[u8; 32]) -> Principal {
    let mut der = [0; 44];
    der[..12].copy_from_slice(&[
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0,
    ]);
    der[12..].copy_from_slice(public_key);
    Principal::self_authenticating(der)
}

// Claims have already passed the canonical decoder. No mutable prepared value
// or public "context checked" token is needed between decode and verification.
pub(crate) fn check(
    claims: &PortableClaims,
    host: &VerificationContext<'_>,
) -> Result<Principal, ContextError> {
    use ContextError as E;
    if host.max_login_ttl_ns == 0
        || host.max_login_ttl_ns > 90 * 86_400_000_000_000
        || host.max_proof_ttl_ns == 0
        || host.max_proof_ttl_ns > 60_000_000_000
        || host.issuer == Principal::anonymous()
        || host.authority_not_before_ns >= host.authority_expires_at_ns
        || host.enrollment_expires_at_ns == 0
    {
        return Err(E::InvalidContext);
    }
    if claims.audience != *host.audience {
        return Err(E::AudienceMismatch);
    }
    if claims.issuer != host.issuer {
        return Err(E::IssuerMismatch);
    }
    if claims.mechanism != host.mechanism {
        return Err(E::MechanismMismatch);
    }
    if claims.key_binding_id != host.key_binding_id {
        return Err(E::KeyMismatch);
    }
    let caller = browser_principal(&claims.browser_public_key);
    if caller != host.caller {
        return Err(E::CallerMismatch);
    }
    if !grants_subset(&claims.grants, host.grants.as_slice()) {
        return Err(E::GrantsExceeded);
    }
    let valid_duration = |start: u64, end: u64, maximum| {
        end.checked_sub(start)
            .is_some_and(|n| n > 0 && n <= maximum)
    };
    if !valid_duration(
        claims.login_started_at_ns,
        claims.login_deadline_ns,
        host.max_login_ttl_ns,
    ) || !valid_duration(
        claims.issued_at_ns,
        claims.admission_expires_at_ns,
        host.max_proof_ttl_ns,
    ) || claims.login_started_at_ns > claims.issued_at_ns
        || claims.admission_expires_at_ns > claims.login_deadline_ns
    {
        return Err(E::InvalidWindow);
    }
    if claims.login_started_at_ns < host.min_login_started_at_ns
        || claims.issued_at_ns < host.min_proof_issued_at_ns
    {
        return Err(E::BeforeReset);
    }
    if host.now_ns < host.authority_not_before_ns
        || host.now_ns >= host.authority_expires_at_ns
        || host.now_ns >= host.enrollment_expires_at_ns
        || claims.issued_at_ns < host.authority_not_before_ns
    {
        return Err(E::StaleAuthority);
    }
    if host.now_ns < claims.issued_at_ns {
        return Err(E::NotYetValid);
    }
    if host.now_ns >= claims.admission_expires_at_ns {
        return Err(E::Expired);
    }
    Ok(caller)
}

// Both lists have passed strict order/shape admission at their owning boundary.
pub(super) fn grants_subset(
    requested: &[DelegatedRoleGrant],
    allowed: &[DelegatedRoleGrant],
) -> bool {
    requested.iter().all(|grant| {
        crate::grant_policy::for_role(allowed, &grant.target).is_some_and(|allowed| {
            crate::grant_policy::scopes_subset(&grant.scopes, &allowed.scopes)
        })
    })
}
