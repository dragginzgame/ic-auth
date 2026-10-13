use super::{
    DelegationProofVerificationLimits, TokenVerificationContext, TokenVerificationError,
    TokenVerificationLimits, binding, window,
};
use crate::canonical::{cert_hash, claims_hash, validate_scope_label, validate_scopes};
use crate::certificate_issuance::{
    CertificateLifetimeError, issuer_binding_matches, positive_duration,
    validate_certificate_lifetime,
};
use ic_auth_protocol_types::{
    DelegatedRoleGrant, DelegatedToken, DelegationAudience, DelegationCert, DelegationProof,
    IssuerProof, Principal, RootProof,
};

pub(crate) fn check_size(
    token: &DelegatedToken,
    limits: TokenVerificationLimits,
) -> Result<(), TokenVerificationError> {
    let mut remaining = limits.max_variable_bytes;
    check_proof_size(&token.proof, limits.max_witness_steps, &mut remaining)?;
    check_grants_size(&token.claims.grants, &mut remaining)?;
    take(
        &mut remaining,
        token.claims.ext.as_ref().map_or(0, Vec::len),
    )?;
    let IssuerProof::IcCanisterSignatureV1(issuer) = &token.issuer_proof;
    if issuer.signature_cbor.len() > limits.max_issuer_signature_bytes {
        return Err(TokenVerificationError::InputTooLarge);
    }
    take(&mut remaining, issuer.signature_cbor.len())?;
    take(&mut remaining, issuer.public_key_der.len())?;
    Ok(())
}

fn take(remaining: &mut usize, length: usize) -> Result<(), TokenVerificationError> {
    *remaining = remaining
        .checked_sub(length)
        .ok_or(TokenVerificationError::InputTooLarge)?;
    Ok(())
}

fn check_grants_size(
    grants: &[DelegatedRoleGrant],
    remaining: &mut usize,
) -> Result<(), TokenVerificationError> {
    take(remaining, grants.len())?;
    for grant in grants {
        take(remaining, grant.target.as_str().len())?;
        take(remaining, grant.scopes.len())?;
        for scope in &grant.scopes {
            take(remaining, scope.len())?;
        }
    }
    Ok(())
}

pub(super) fn check_proof_size(
    proof: &DelegationProof,
    max_witness_steps: usize,
    remaining: &mut usize,
) -> Result<(), TokenVerificationError> {
    let RootProof::IcChainKeyBatchSignatureV1(root) = &proof.root_proof;
    for grants in [&proof.cert.grants, &root.delegation_cert.grants] {
        check_grants_size(grants, remaining)?;
    }
    if root.issuer_witness.steps.len() > max_witness_steps {
        return Err(TokenVerificationError::InputTooLarge);
    }
    take(
        remaining,
        root.issuer_witness
            .steps
            .len()
            .checked_mul(33)
            .ok_or(TokenVerificationError::InputTooLarge)?,
    )?;
    take(remaining, root.header.key_id.name.len())?;
    take(remaining, root.signature.key_id.name.len())?;
    take(remaining, root.signature.public_key.len())?;
    take(remaining, root.signature.signature.len())?;
    take(remaining, root.signature.derivation_path.len())?;
    for component in &root.signature.derivation_path {
        take(remaining, component.len())?;
    }
    Ok(())
}

fn grants_valid(grants: &[DelegatedRoleGrant]) -> Result<(), TokenVerificationError> {
    if grants.is_empty()
        || grants.len() > 16
        || grants
            .iter()
            .any(|grant| grant.scopes.is_empty() || grant.scopes.len() > 32)
    {
        return Err(TokenVerificationError::InvalidGrants);
    }
    // Existing canonical encoders enforce role/scope grammar and exact order.
    // No normalization is allowed at verification time.
    Ok(())
}

fn ttl(target: &'static str, start: u64, end: u64, max: u64) -> Result<(), TokenVerificationError> {
    let ttl_ns =
        positive_duration(start, end).ok_or(TokenVerificationError::InvalidWindow { target })?;
    if ttl_ns > max {
        return Err(TokenVerificationError::TtlExceeded {
            target,
            ttl_ns,
            max_ttl_ns: max,
        });
    }
    Ok(())
}

pub(super) fn verify_certificate_window(
    cert: &DelegationCert,
    now_ns: u64,
    limits: DelegationProofVerificationLimits,
) -> Result<(), TokenVerificationError> {
    validate_certificate_lifetime(cert, limits.max_cert_ttl_ns, limits.max_token_ttl_ns).map_err(
        |error| {
            use CertificateLifetimeError as E;
            match error {
                E::IssuedAfterStart => TokenVerificationError::BindingMismatch {
                    field: "cert_issued_at",
                },
                E::InvalidWindow => TokenVerificationError::InvalidWindow {
                    target: "certificate",
                },
                E::CertTtlExceeded { ttl_ns, max_ttl_ns } => TokenVerificationError::TtlExceeded {
                    target: "certificate",
                    ttl_ns,
                    max_ttl_ns,
                },
                E::TokenTtlZero => TokenVerificationError::InvalidWindow {
                    target: "max_token_ttl",
                },
                E::TokenTtlExceeded { ttl_ns, .. } => TokenVerificationError::TtlExceeded {
                    target: "max_token_ttl",
                    ttl_ns,
                    max_ttl_ns: limits
                        .max_token_ttl_ns
                        .min(cert.expires_at_ns - cert.not_before_ns),
                },
                E::TokenTtlOutlivesCert {
                    token_ttl_ns,
                    cert_ttl_ns,
                } => TokenVerificationError::TtlExceeded {
                    target: "max_token_ttl",
                    ttl_ns: token_ttl_ns,
                    max_ttl_ns: cert_ttl_ns,
                },
            }
        },
    )?;
    window(
        "certificate",
        cert.not_before_ns,
        cert.expires_at_ns,
        now_ns,
        limits.max_future_skew_ns,
    )?;
    grants_valid(&cert.grants)?;
    Ok(())
}

pub(super) fn verify_certificate_binding(
    cert: &DelegationCert,
) -> Result<(), TokenVerificationError> {
    binding(issuer_binding_matches(cert)?, "issuer_proof_binding_hash")?;
    Ok(())
}

pub(super) fn verify_material<'a>(
    token: &'a DelegatedToken,
    ctx: &TokenVerificationContext<'_>,
) -> Result<(&'a DelegatedRoleGrant, [u8; 32]), TokenVerificationError> {
    let cert = &token.proof.cert;
    let claims = &token.claims;
    for (field, principal) in [
        ("caller", ctx.caller),
        ("presenter", claims.presenter),
        ("subject", claims.subject),
        ("issuer", cert.issuer_pid),
        ("root", cert.root_pid),
    ] {
        if principal == Principal::anonymous() {
            return Err(TokenVerificationError::AnonymousPrincipal { field });
        }
    }
    if claims.presenter != ctx.caller {
        return Err(TokenVerificationError::PresenterMismatch);
    }
    if claims.subject != claims.presenter {
        return Err(TokenVerificationError::SubjectMismatch);
    }
    binding(
        cert.root_pid == ctx.root_key.root_canister_id,
        "root_canister_id",
    )?;
    binding(claims.issuer_pid == cert.issuer_pid, "issuer_canister_id")?;
    verify_certificate_window(cert, ctx.now_ns, ctx.limits.into())?;
    grants_valid(&claims.grants)?;
    verify_certificate_binding(cert)?;
    binding(cert_hash(cert)? == claims.cert_hash, "cert_hash")?;
    // Check claims canonicality (including extension size) even if grants narrow.
    let claims_hash = claims_hash(claims)?;
    ttl(
        "token",
        claims.issued_at_ns,
        claims.expires_at_ns,
        cert.max_token_ttl_ns,
    )?;
    binding(
        claims.issued_at_ns >= cert.not_before_ns && claims.expires_at_ns <= cert.expires_at_ns,
        "token_certificate_window",
    )?;
    window(
        "token",
        claims.issued_at_ns,
        claims.expires_at_ns,
        ctx.now_ns,
        ctx.limits.max_future_skew_ns,
    )?;
    let audience = DelegationAudience::Fleet(ctx.audience);
    if claims.aud != audience || cert.aud != audience {
        return Err(TokenVerificationError::AudienceRejected);
    }
    if !claims.grants.iter().all(|child| {
        cert.grants.iter().any(|parent| {
            parent.target == child.target
                && child
                    .scopes
                    .iter()
                    .all(|scope| parent.scopes.contains(scope))
        })
    }) {
        return Err(TokenVerificationError::GrantsNotSubset);
    }
    let grant = claims
        .grants
        .iter()
        .find(|grant| &grant.target == ctx.role)
        .ok_or(TokenVerificationError::RoleRejected)?;
    validate_scopes(ctx.allowed_scopes)?;
    for scope in &grant.scopes {
        if !ctx.allowed_scopes.contains(scope) {
            return Err(TokenVerificationError::ScopeRejected {
                scope: scope.clone(),
            });
        }
    }
    for scope in ctx.required_scopes {
        validate_scope_label(scope)?;
        if !grant.scopes.contains(scope) {
            return Err(TokenVerificationError::ScopeRejected {
                scope: scope.clone(),
            });
        }
    }
    Ok((grant, claims_hash))
}
