//! Pure certificate issuance rules, without signing, time or authority admission.
//!
//! The host supplies the expected Root and bounds from protected configuration.
//! Success checks shape and binding bytes only; it does not authenticate a
//! certificate, authorize issuance or install a delegation. No field is normalized.

use crate::canonical::{CanonicalAuthError, issuer_proof_binding_hash, validate_scopes};
use ic_auth_protocol_types::{AuthRole, DelegatedRoleGrant, DelegationCert, Principal, ScopeLabel};
use thiserror::Error;

/// Explicit host-selected issuance bounds; no Fleet or grant defaults are inferred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CertificateIssuanceLimits {
    /// Maximum positive certificate validity duration in nanoseconds.
    pub max_cert_ttl_ns: u64,
    /// Maximum credential lifetime the certificate may delegate, in nanoseconds.
    pub max_token_ttl_ns: u64,
    /// Maximum nonempty role grants accepted by this host.
    pub max_grants: usize,
    /// Maximum nonempty scopes per role grant accepted by this host.
    pub max_scopes_per_grant: usize,
}

/// Ordered shape rejection, separate from cryptographic and current-clock checks.
/// Role grammar is already enforced by the passive `AuthRole` type at decoding
/// or construction; scope syntax and exact grant ordering are checked here.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum CertificateIssuanceError {
    #[error("certificate Root mismatch (expected {expected}, found {found})")]
    /// Certificate names a different root than protected host configuration.
    RootPidMismatch {
        /// Root identity supplied by trusted host configuration.
        expected: Principal,
        /// Root identity carried in the submitted certificate.
        found: Principal,
    },
    #[error(transparent)]
    /// Certificate validity or delegated token lifetime violates host bounds.
    Lifetime(#[from] CertificateLifetimeError),
    #[error(transparent)]
    /// Grants violate the canonical shape or host cardinality bounds.
    Grants(#[from] GrantValidationError),
    #[error("issuer proof binding hash mismatch")]
    /// Submitted binding hash does not match the canonical issuer binding.
    IssuerProofBindingHashMismatch,
    #[error(transparent)]
    /// Certificate material cannot be encoded under the canonical contract.
    Canonical(#[from] CanonicalAuthError),
}

/// Validate Root binding, lifetime, canonical bounded grants, then issuer binding.
/// Exact bound equality is accepted. Anonymous principals are not implicitly
/// refused: identity eligibility, live policy, proof verification and clock
/// acceptance remain separate host/verification obligations.
pub fn validate_certificate_issuance(
    cert: &DelegationCert,
    expected_root: Principal,
    limits: CertificateIssuanceLimits,
) -> Result<(), CertificateIssuanceError> {
    if cert.root_pid != expected_root {
        return Err(CertificateIssuanceError::RootPidMismatch {
            expected: expected_root,
            found: cert.root_pid,
        });
    }
    validate_certificate_lifetime(cert, limits.max_cert_ttl_ns, limits.max_token_ttl_ns)?;
    validate_grants(&cert.grants, limits.max_grants, limits.max_scopes_per_grant)?;
    if !issuer_binding_matches(cert)? {
        return Err(CertificateIssuanceError::IssuerProofBindingHashMismatch);
    }
    Ok(())
}

/// Ordered lifetime causes shared by issuance and delegation verification.
/// Neither this error nor the shape validator establishes current authority.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum CertificateLifetimeError {
    #[error("certificate was issued after its validity start")]
    /// Issue-time metadata is later than the certificate validity start.
    IssuedAfterStart,
    #[error("certificate validity must have positive duration")]
    /// Expiry is not strictly later than validity start.
    InvalidWindow,
    #[error("certificate TTL {ttl_ns} exceeds host maximum {max_ttl_ns}")]
    /// Certificate validity exceeds the configured duration ceiling.
    CertTtlExceeded {
        /// Submitted certificate duration in nanoseconds.
        ttl_ns: u64,
        /// Host-admitted duration ceiling in nanoseconds.
        max_ttl_ns: u64,
    },
    #[error("delegated token TTL must be greater than zero")]
    /// Delegation permits a zero token lifetime, which is not a valid policy.
    TokenTtlZero,
    #[error("delegated token TTL {ttl_ns} exceeds host maximum {max_ttl_ns}")]
    /// Delegated token lifetime exceeds the host's maximum.
    TokenTtlExceeded {
        /// Delegated token lifetime in nanoseconds.
        ttl_ns: u64,
        /// Host-admitted token lifetime ceiling in nanoseconds.
        max_ttl_ns: u64,
    },
    #[error("delegated token TTL {token_ttl_ns} exceeds certificate duration {cert_ttl_ns}")]
    /// Delegated token lifetime is longer than the certificate's entire window.
    TokenTtlOutlivesCert {
        /// Delegated token lifetime in nanoseconds.
        token_ttl_ns: u64,
        /// Certificate validity duration in nanoseconds.
        cert_ttl_ns: u64,
    },
}

pub(crate) fn positive_duration(start: u64, end: u64) -> Option<u64> {
    end.checked_sub(start).filter(|duration| *duration != 0)
}

pub(crate) fn validate_certificate_lifetime(
    cert: &DelegationCert,
    max_cert_ttl_ns: u64,
    max_token_ttl_ns: u64,
) -> Result<(), CertificateLifetimeError> {
    use CertificateLifetimeError as E;
    if cert.issued_at_ns > cert.not_before_ns {
        return Err(E::IssuedAfterStart);
    }
    let cert_ttl_ns =
        positive_duration(cert.not_before_ns, cert.expires_at_ns).ok_or(E::InvalidWindow)?;
    if cert_ttl_ns > max_cert_ttl_ns {
        return Err(E::CertTtlExceeded {
            ttl_ns: cert_ttl_ns,
            max_ttl_ns: max_cert_ttl_ns,
        });
    }
    if cert.max_token_ttl_ns == 0 {
        return Err(E::TokenTtlZero);
    }
    if cert.max_token_ttl_ns > max_token_ttl_ns {
        return Err(E::TokenTtlExceeded {
            ttl_ns: cert.max_token_ttl_ns,
            max_ttl_ns: max_token_ttl_ns,
        });
    }
    if cert.max_token_ttl_ns > cert_ttl_ns {
        return Err(E::TokenTtlOutlivesCert {
            token_ttl_ns: cert.max_token_ttl_ns,
            cert_ttl_ns,
        });
    }
    Ok(())
}

/// A standalone grant-shape failure, before any certificate or token exists.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum GrantValidationError {
    #[error("grants are empty")]
    /// No role grants were supplied.
    Empty,
    #[error("{found} grants exceed maximum {max}")]
    /// The grant list exceeds the configured role count.
    TooMany {
        /// Number of submitted grants.
        found: usize,
        /// Inclusive host grant-count ceiling.
        max: usize,
    },
    #[error("grant for {role:?} has no scopes")]
    /// A role grant contains no scopes.
    EmptyScopes {
        /// Role with the empty scope list.
        role: AuthRole,
    },
    #[error("grant for {role:?} has {found} scopes; maximum is {max}")]
    /// One role's scope list exceeds the configured count.
    TooManyScopes {
        /// Role whose scope list exceeds the bound.
        role: AuthRole,
        /// Submitted scope count for that role.
        found: usize,
        /// Inclusive per-role scope ceiling.
        max: usize,
    },
    #[error(transparent)]
    /// Role or scope ordering/encoding is noncanonical; inputs are never sorted here.
    Canonical(CanonicalAuthError),
}

/// Validate nonempty grants against caller-owned cardinality bounds.
///
/// Checks the grant count first, then each role's strict byte ordering, scope
/// count and strict scope ordering in input order. It neither sorts nor grants
/// authority. Roles and scope labels are checked by their types; hosts retain
/// raw projection and its error precedence at their own input boundary.
pub fn validate_grants(
    grants: &[DelegatedRoleGrant],
    max_grants: usize,
    max_scopes_per_grant: usize,
) -> Result<(), GrantValidationError> {
    use GrantValidationError as E;
    if grants.is_empty() {
        return Err(E::Empty);
    }
    if grants.len() > max_grants {
        return Err(E::TooMany {
            found: grants.len(),
            max: max_grants,
        });
    }
    let mut previous = None;
    for grant in grants {
        let role = grant.target.as_str().as_bytes();
        if previous.is_some_and(|previous| previous >= role) {
            return Err(E::Canonical(CanonicalAuthError::NonCanonicalRoles));
        }
        previous = Some(role);
        if grant.scopes.is_empty() {
            return Err(E::EmptyScopes {
                role: grant.target.clone(),
            });
        }
        if grant.scopes.len() > max_scopes_per_grant {
            return Err(E::TooManyScopes {
                role: grant.target.clone(),
                found: grant.scopes.len(),
                max: max_scopes_per_grant,
            });
        }
        validate_scopes(&grant.scopes).map_err(E::Canonical)?;
    }
    Ok(())
}

// Both callers admit canonical parent scopes first: protected claims authority
// through validate_grants, or the delegation certificate through cert_hash.
pub(crate) fn grants_are_subset(
    grants: &[DelegatedRoleGrant],
    ceiling: &[DelegatedRoleGrant],
) -> bool {
    grants.iter().all(|child| {
        grant_for_role(ceiling, &child.target)
            .is_some_and(|parent| scopes_are_subset(&child.scopes, &parent.scopes))
    })
}

pub(crate) fn grant_for_role<'a>(
    grants: &'a [DelegatedRoleGrant],
    role: &AuthRole,
) -> Option<&'a DelegatedRoleGrant> {
    grants.iter().find(|grant| &grant.target == role)
}

// The ceiling must already be canonical. Requested scopes need not be ordered
// (ordinary authorization accepts host-required scopes in any order).
pub(crate) fn scopes_are_subset(requested: &[ScopeLabel], ceiling: &[ScopeLabel]) -> bool {
    requested.iter().all(|scope| {
        let scope = scope.as_str();
        // The measured one/two-scope profile is cheaper with a direct scan;
        // wider canonical ceilings avoid repeated full scans through search.
        if ceiling.len() <= 2 {
            ceiling.iter().any(|allowed| allowed.as_str() == scope)
        } else {
            ceiling
                .binary_search_by(|allowed| allowed.as_str().cmp(scope))
                .is_ok()
        }
    })
}

pub(crate) fn issuer_binding_matches(cert: &DelegationCert) -> Result<bool, CanonicalAuthError> {
    Ok(issuer_proof_binding_hash(
        cert.issuer_pid,
        cert.issuer_proof_alg,
        cert.issuer_proof_binding,
    )? == cert.issuer_proof_binding_hash)
}

#[cfg(test)]
mod tests;
