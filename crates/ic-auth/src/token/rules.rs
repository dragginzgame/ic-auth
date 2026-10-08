use super::{
    TokenVerificationContext, TokenVerificationError, TokenVerificationLimits, binding, window,
};
use crate::canonical::{
    CanonicalAuthError, cert_hash, claims_hash, issuer_proof_binding_hash, validate_scope_label,
};
use ic_auth_protocol_types::{
    DelegatedRoleGrant, DelegatedToken, DelegationAudience, IssuerProof, Principal, RootProof,
};

pub(super) fn check_size(
    token: &DelegatedToken,
    limits: TokenVerificationLimits,
) -> Result<(), TokenVerificationError> {
    let mut remaining = limits.max_variable_bytes;
    let mut take = |length: usize| {
        remaining = remaining
            .checked_sub(length)
            .ok_or(TokenVerificationError::InputTooLarge)?;
        Ok::<_, TokenVerificationError>(())
    };
    let RootProof::IcChainKeyBatchSignatureV1(root) = &token.proof.root_proof;
    for grants in [
        &token.proof.cert.grants,
        &token.claims.grants,
        &root.delegation_cert.grants,
    ] {
        take(grants.len())?;
        for grant in grants {
            take(grant.target.as_str().len())?;
            take(grant.scopes.len())?;
            for scope in &grant.scopes {
                take(scope.len())?;
            }
        }
    }
    if root.issuer_witness.steps.len() > limits.max_witness_steps {
        return Err(TokenVerificationError::InputTooLarge);
    }
    take(
        root.issuer_witness
            .steps
            .len()
            .checked_mul(33)
            .ok_or(TokenVerificationError::InputTooLarge)?,
    )?;
    take(root.header.key_id.name.len())?;
    take(root.signature.key_id.name.len())?;
    take(root.signature.public_key.len())?;
    take(root.signature.signature.len())?;
    take(root.signature.derivation_path.len())?;
    for component in &root.signature.derivation_path {
        take(component.len())?;
    }
    take(token.claims.ext.as_ref().map_or(0, Vec::len))?;
    let IssuerProof::IcCanisterSignatureV1(issuer) = &token.issuer_proof;
    if issuer.signature_cbor.len() > limits.max_issuer_signature_bytes {
        return Err(TokenVerificationError::InputTooLarge);
    }
    take(issuer.signature_cbor.len())?;
    take(issuer.public_key_der.len())?;
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
    let ttl_ns = end
        .checked_sub(start)
        .filter(|ttl| *ttl != 0)
        .ok_or(TokenVerificationError::InvalidWindow { target })?;
    if ttl_ns > max {
        return Err(TokenVerificationError::TtlExceeded {
            target,
            ttl_ns,
            max_ttl_ns: max,
        });
    }
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
    binding(cert.issued_at_ns <= cert.not_before_ns, "cert_issued_at")?;
    ttl(
        "certificate",
        cert.not_before_ns,
        cert.expires_at_ns,
        ctx.limits.max_cert_ttl_ns,
    )?;
    if cert.max_token_ttl_ns == 0 {
        return Err(TokenVerificationError::InvalidWindow {
            target: "max_token_ttl",
        });
    }
    let cert_ttl = cert.expires_at_ns - cert.not_before_ns;
    if cert.max_token_ttl_ns > ctx.limits.max_token_ttl_ns.min(cert_ttl) {
        return Err(TokenVerificationError::TtlExceeded {
            target: "max_token_ttl",
            ttl_ns: cert.max_token_ttl_ns,
            max_ttl_ns: ctx.limits.max_token_ttl_ns.min(cert_ttl),
        });
    }
    window(
        "certificate",
        cert.not_before_ns,
        cert.expires_at_ns,
        ctx.now_ns,
        ctx.limits.max_future_skew_ns,
    )?;
    grants_valid(&cert.grants)?;
    grants_valid(&claims.grants)?;
    binding(
        issuer_proof_binding_hash(
            cert.issuer_pid,
            cert.issuer_proof_alg,
            cert.issuer_proof_binding,
        )? == cert.issuer_proof_binding_hash,
        "issuer_proof_binding_hash",
    )?;
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
    let mut previous: Option<&str> = None;
    for scope in ctx.allowed_scopes {
        validate_scope_label(scope)?;
        if previous.is_some_and(|previous| previous >= scope.as_str()) {
            return Err(CanonicalAuthError::NonCanonicalScopes.into());
        }
        previous = Some(scope);
    }
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
