use super::{DelegationProofError, DelegationProofVerificationLimits, binding, window};
use crate::certificate_issuance::{
    CertificateLifetimeError, issuer_binding_matches, validate_certificate_lifetime,
};
use ic_auth_protocol_types::{DelegatedRoleGrant, DelegationCert, DelegationProof, RootProof};

fn take(remaining: &mut usize, length: usize) -> Result<(), DelegationProofError> {
    *remaining = remaining
        .checked_sub(length)
        .ok_or(DelegationProofError::InputTooLarge)?;
    Ok(())
}

fn check_grants_size(
    grants: &[DelegatedRoleGrant],
    remaining: &mut usize,
) -> Result<(), DelegationProofError> {
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

pub(crate) fn check_proof_size(
    proof: &DelegationProof,
    max_witness_steps: usize,
    remaining: &mut usize,
) -> Result<(), DelegationProofError> {
    let RootProof::IcChainKeyBatchSignatureV1(root) = &proof.root_proof;
    for grants in [&proof.cert.grants, &root.delegation_cert.grants] {
        check_grants_size(grants, remaining)?;
    }
    if root.issuer_witness.steps.len() > max_witness_steps {
        return Err(DelegationProofError::InputTooLarge);
    }
    take(
        remaining,
        root.issuer_witness
            .steps
            .len()
            .checked_mul(33)
            .ok_or(DelegationProofError::InputTooLarge)?,
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

fn grants_valid(grants: &[DelegatedRoleGrant]) -> Result<(), DelegationProofError> {
    if grants.is_empty()
        || grants.len() > crate::limits::MAX_GRANTS
        || grants.iter().any(|grant| {
            grant.scopes.is_empty() || grant.scopes.len() > crate::limits::MAX_SCOPES_PER_GRANT
        })
    {
        return Err(DelegationProofError::InvalidGrants);
    }
    // Existing canonical encoders enforce role/scope grammar and exact order.
    // No normalization is allowed at verification time.
    Ok(())
}

pub(super) fn verify_certificate_window(
    cert: &DelegationCert,
    now_ns: u64,
    limits: DelegationProofVerificationLimits,
) -> Result<(), DelegationProofError> {
    validate_certificate_lifetime(cert, limits.max_cert_ttl_ns, limits.max_token_ttl_ns).map_err(
        |error| {
            use CertificateLifetimeError as E;
            match error {
                E::IssuedAfterStart => DelegationProofError::BindingMismatch {
                    field: "cert_issued_at",
                },
                E::InvalidWindow => DelegationProofError::InvalidWindow {
                    target: "certificate",
                },
                E::CertTtlExceeded { ttl_ns, max_ttl_ns } => DelegationProofError::TtlExceeded {
                    target: "certificate",
                    ttl_ns,
                    max_ttl_ns,
                },
                E::TokenTtlZero => DelegationProofError::InvalidWindow {
                    target: "max_token_ttl",
                },
                E::TokenTtlExceeded { ttl_ns, .. } => DelegationProofError::TtlExceeded {
                    target: "max_token_ttl",
                    ttl_ns,
                    max_ttl_ns: limits
                        .max_token_ttl_ns
                        .min(cert.expires_at_ns - cert.not_before_ns),
                },
                E::TokenTtlOutlivesCert {
                    token_ttl_ns,
                    cert_ttl_ns,
                } => DelegationProofError::TtlExceeded {
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
) -> Result<(), DelegationProofError> {
    binding(issuer_binding_matches(cert)?, "issuer_proof_binding_hash")?;
    Ok(())
}
