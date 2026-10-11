//! Pure delegated-token issuance over a host-admitted delegation proof.
//!
//! Preparation and finishing do not authenticate any proof or authorize signing.
//! The host owns issuer approval, independently enrolled authority, authenticated
//! identity, clock, operation custody, signing and certification-root publication.

use crate::canonical::{CanonicalAuthError, cert_hash, claims_hash, validate_scopes};
use ic_auth_protocol_types::{
    AuthRole, DelegatedRoleGrant, DelegatedToken, DelegatedTokenClaims, DelegationAudience,
    DelegationProof, IssuerProof, Principal,
};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Explicit host-selected request bounds; there are no implicit fleet defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TokenPreparationLimits {
    pub max_grants: usize,
    pub max_scopes_per_grant: usize,
}

/// Inputs resolved by the host before preparing an issuer-local token.
pub struct PrepareDelegatedTokenInput<'a> {
    /// Already installed/verified proof belonging to the authorized local issuer.
    /// Never resolve this authority from client-submitted proof fields.
    pub proof: &'a DelegationProof,
    /// Exclusive protected authority deadline, capped by enrolled root policy.
    pub authority_expires_at_ns: u64,
    /// Host-retained operation identity. Retry must retain the original inputs,
    /// including time, rather than rebuilding a different token for this ID.
    pub operation_id: [u8; 32],
    /// Authenticated presenter, also used as the subject; not a request assertion.
    pub prepared_by: Principal,
    pub audience: DelegationAudience,
    pub grants: Vec<DelegatedRoleGrant>,
    pub ttl_ns: u64,
    pub ext: Option<Vec<u8>>,
    pub now_ns: u64,
    pub limits: TokenPreparationLimits,
}

/// Immutable claims, signing hash and attached Root proof. This is prepared
/// material, not a verified token, signature, resource grant or session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedDelegatedToken {
    claims: DelegatedTokenClaims,
    claims_hash: [u8; 32],
    proof: DelegationProof,
}

impl PreparedDelegatedToken {
    pub const fn claims(&self) -> &DelegatedTokenClaims {
        &self.claims
    }

    /// Canonical hash to pass to the host's authorized issuer-signature path.
    pub const fn claims_hash(&self) -> [u8; 32] {
        self.claims_hash
    }

    pub const fn proof(&self) -> &DelegationProof {
        &self.proof
    }
}

/// Ordered request rejection. Neither certificate bytes nor these shape checks
/// establish the provenance of the host's supplied authority.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum PrepareDelegatedTokenError {
    #[error("delegation certificate is not yet valid")]
    CertNotYetValid,
    #[error("delegation certificate or local authority expired")]
    CertExpired,
    #[error("token TTL must be greater than zero")]
    TokenTtlZero,
    #[error("token TTL {ttl_ns} exceeds certificate maximum {max_ttl_ns}")]
    TokenTtlExceeded { ttl_ns: u64, max_ttl_ns: u64 },
    #[error("token expiry overflows")]
    TokenExpiresAtOverflow,
    #[error("token outlives certificate or local authority")]
    TokenOutlivesCert,
    #[error("token grants must not be empty")]
    GrantsEmpty,
    #[error("token grants {found} exceed host maximum {max}")]
    TooManyGrants { found: usize, max: usize },
    #[error("grant for {role:?} has no scopes")]
    EmptyGrantScopes { role: AuthRole },
    #[error("grant for {role:?} has {found} scopes exceeding host maximum {max}")]
    TooManyGrantScopes {
        role: AuthRole,
        found: usize,
        max: usize,
    },
    #[error("token audience is not a subset of certificate audience")]
    AudienceNotSubset,
    #[error("token grants are not a subset of certificate grants")]
    GrantsNotSubset,
    #[error(transparent)]
    Canonical(#[from] CanonicalAuthError),
}

/// Prepare exact canonical claims without reading runtime state or signing.
/// Expiry is checked against both certificate and supplied authority: an
/// overlong request rejects rather than silently shortening the requested TTL.
pub fn prepare_delegated_token(
    input: PrepareDelegatedTokenInput<'_>,
) -> Result<PreparedDelegatedToken, PrepareDelegatedTokenError> {
    use PrepareDelegatedTokenError as Error;
    let cert = &input.proof.cert;
    let deadline = cert.expires_at_ns.min(input.authority_expires_at_ns);
    if input.now_ns < cert.not_before_ns {
        return Err(Error::CertNotYetValid);
    }
    if input.now_ns >= deadline {
        return Err(Error::CertExpired);
    }
    if input.ttl_ns == 0 {
        return Err(Error::TokenTtlZero);
    }
    if input.ttl_ns > cert.max_token_ttl_ns {
        return Err(Error::TokenTtlExceeded {
            ttl_ns: input.ttl_ns,
            max_ttl_ns: cert.max_token_ttl_ns,
        });
    }
    let expires_at_ns = input
        .now_ns
        .checked_add(input.ttl_ns)
        .ok_or(Error::TokenExpiresAtOverflow)?;
    if expires_at_ns > deadline {
        return Err(Error::TokenOutlivesCert);
    }
    if input.grants.is_empty() {
        return Err(Error::GrantsEmpty);
    }
    if input.grants.len() > input.limits.max_grants {
        return Err(Error::TooManyGrants {
            found: input.grants.len(),
            max: input.limits.max_grants,
        });
    }
    let mut previous = None;
    for grant in &input.grants {
        let role = grant.target.as_str().as_bytes();
        if previous.is_some_and(|previous| previous >= role) {
            return Err(CanonicalAuthError::NonCanonicalRoles.into());
        }
        previous = Some(role);
        if grant.scopes.is_empty() {
            return Err(Error::EmptyGrantScopes {
                role: grant.target.clone(),
            });
        }
        if grant.scopes.len() > input.limits.max_scopes_per_grant {
            return Err(Error::TooManyGrantScopes {
                role: grant.target.clone(),
                found: grant.scopes.len(),
                max: input.limits.max_scopes_per_grant,
            });
        }
        validate_scopes(&grant.scopes)?;
    }
    if input.audience != cert.aud {
        return Err(Error::AudienceNotSubset);
    }
    if !input.grants.iter().all(|child| {
        cert.grants
            .iter()
            .find(|parent| parent.target == child.target)
            .is_some_and(|parent| {
                child
                    .scopes
                    .iter()
                    .all(|scope| parent.scopes.contains(scope))
            })
    }) {
        return Err(Error::GrantsNotSubset);
    }
    let cert_hash = cert_hash(cert)?;
    let nonce = delegated_token_nonce(
        input.prepared_by,
        input.operation_id,
        cert.issuer_pid,
        cert_hash,
    );
    let claims = DelegatedTokenClaims {
        presenter: input.prepared_by,
        subject: input.prepared_by,
        issuer_pid: cert.issuer_pid,
        cert_hash,
        issued_at_ns: input.now_ns,
        expires_at_ns,
        aud: input.audience,
        grants: input.grants,
        nonce,
        ext: input.ext,
    };
    Ok(PreparedDelegatedToken {
        claims_hash: claims_hash(&claims)?,
        claims,
        proof: input.proof.clone(),
    })
}

fn delegated_token_nonce(
    presenter: Principal,
    operation_id: [u8; 32],
    issuer: Principal,
    cert_hash: [u8; 32],
) -> [u8; 16] {
    // Frozen Canic domain and raw principal framing; do not rebrand signed bytes.
    let mut hash = Sha256::new();
    hash.update(b"canic-token-nonce-v1");
    hash.update(presenter.as_slice());
    hash.update(operation_id);
    hash.update(issuer.as_slice());
    hash.update(cert_hash);
    let digest: [u8; 32] = hash.finalize().into();
    let mut nonce = [0; 16];
    nonce.copy_from_slice(&digest[..16]);
    nonce
}

/// Attach host-provided issuer material verbatim, without proof verification.
/// Obtain and authorize that material through the host's signature service;
/// recipients still need complete verification before admitting the token.
pub fn finish_delegated_token(
    prepared: PreparedDelegatedToken,
    issuer_proof: IssuerProof,
) -> DelegatedToken {
    DelegatedToken {
        claims: prepared.claims,
        proof: prepared.proof,
        issuer_proof,
    }
}
