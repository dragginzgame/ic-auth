//! Pure certificate issuance rules, without signing, time or authority admission.
//!
//! The host supplies the expected Root and bounds from protected configuration.
//! Success checks shape and binding bytes only; it does not authenticate a
//! certificate, authorize issuance or install a delegation. No field is normalized.

use crate::canonical::{CanonicalAuthError, issuer_proof_binding_hash, validate_scopes};
use ic_auth_protocol_types::{AuthRole, DelegatedRoleGrant, DelegationCert, Principal};
use thiserror::Error;

/// Explicit host-selected issuance bounds; no Fleet or grant defaults are inferred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CertificateIssuanceLimits {
    pub max_cert_ttl_ns: u64,
    pub max_token_ttl_ns: u64,
    pub max_grants: usize,
    pub max_scopes_per_grant: usize,
}

/// Ordered shape rejection, separate from cryptographic and current-clock checks.
/// Role grammar is already enforced by the passive `AuthRole` type at decoding
/// or construction; scope syntax and exact grant ordering are checked here.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum CertificateIssuanceError {
    #[error("certificate Root mismatch (expected {expected}, found {found})")]
    RootPidMismatch {
        expected: Principal,
        found: Principal,
    },
    #[error("certificate validity must start at or after issuance and have positive duration")]
    InvalidCertWindow,
    #[error("certificate TTL {ttl_ns} exceeds host maximum {max_ttl_ns}")]
    CertTtlExceeded { ttl_ns: u64, max_ttl_ns: u64 },
    #[error("delegated token TTL must be greater than zero")]
    TokenTtlZero,
    #[error("delegated token TTL {ttl_ns} exceeds host maximum {max_ttl_ns}")]
    TokenTtlExceeded { ttl_ns: u64, max_ttl_ns: u64 },
    #[error("delegated token TTL {token_ttl_ns} exceeds certificate duration {cert_ttl_ns}")]
    TokenTtlOutlivesCert { token_ttl_ns: u64, cert_ttl_ns: u64 },
    #[error("certificate grants must not be empty")]
    GrantsEmpty,
    #[error("certificate grants {found} exceed host maximum {max}")]
    TooManyGrants { found: usize, max: usize },
    #[error("grant for {role:?} has no scopes")]
    EmptyGrantScopes { role: AuthRole },
    #[error("grant for {role:?} has {found} scopes exceeding host maximum {max}")]
    TooManyGrantScopes {
        role: AuthRole,
        found: usize,
        max: usize,
    },
    #[error("issuer proof binding hash mismatch")]
    IssuerProofBindingHashMismatch,
    #[error(transparent)]
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

// Internal causes let existing callers reuse the rules without changing their
// public rejection surface or the placement of independent clock/crypto checks.
pub(crate) enum CertificateLifetimeError {
    IssuedAfterStart,
    InvalidWindow,
    CertTtlExceeded { ttl_ns: u64, max_ttl_ns: u64 },
    TokenTtlZero,
    TokenTtlExceeded { ttl_ns: u64, max_ttl_ns: u64 },
    TokenTtlOutlivesCert { token_ttl_ns: u64, cert_ttl_ns: u64 },
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

impl From<CertificateLifetimeError> for CertificateIssuanceError {
    fn from(error: CertificateLifetimeError) -> Self {
        use CertificateLifetimeError as E;
        match error {
            E::IssuedAfterStart | E::InvalidWindow => Self::InvalidCertWindow,
            E::CertTtlExceeded { ttl_ns, max_ttl_ns } => {
                Self::CertTtlExceeded { ttl_ns, max_ttl_ns }
            }
            E::TokenTtlZero => Self::TokenTtlZero,
            E::TokenTtlExceeded { ttl_ns, max_ttl_ns } => {
                Self::TokenTtlExceeded { ttl_ns, max_ttl_ns }
            }
            E::TokenTtlOutlivesCert {
                token_ttl_ns,
                cert_ttl_ns,
            } => Self::TokenTtlOutlivesCert {
                token_ttl_ns,
                cert_ttl_ns,
            },
        }
    }
}

pub(crate) enum GrantValidationError {
    Empty,
    TooMany {
        found: usize,
        max: usize,
    },
    EmptyScopes {
        role: AuthRole,
    },
    TooManyScopes {
        role: AuthRole,
        found: usize,
        max: usize,
    },
    Canonical(CanonicalAuthError),
}

pub(crate) fn validate_grants(
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

impl From<GrantValidationError> for CertificateIssuanceError {
    fn from(error: GrantValidationError) -> Self {
        use GrantValidationError as E;
        match error {
            E::Empty => Self::GrantsEmpty,
            E::TooMany { found, max } => Self::TooManyGrants { found, max },
            E::EmptyScopes { role } => Self::EmptyGrantScopes { role },
            E::TooManyScopes { role, found, max } => Self::TooManyGrantScopes { role, found, max },
            E::Canonical(error) => Self::Canonical(error),
        }
    }
}

pub(crate) fn issuer_binding_matches(cert: &DelegationCert) -> Result<bool, CanonicalAuthError> {
    Ok(issuer_proof_binding_hash(
        cert.issuer_pid,
        cert.issuer_proof_alg,
        cert.issuer_proof_binding,
    )? == cert.issuer_proof_binding_hash)
}
