use super::{RootKeyPolicy, TokenVerificationError, TokenVerificationLimits, binding, root, rules};
use crate::canonical::cert_hash;
use ic_auth_protocol_types::{DelegationCert, DelegationProof, Principal};

/// Host-selected bounds for a root delegation proof, before a token exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DelegationProofVerificationLimits {
    /// Maximum root certificate validity duration.
    pub max_cert_ttl_ns: u64,
    /// Maximum token lifetime that this certificate may delegate.
    pub max_token_ttl_ns: u64,
    /// Inclusive allowance for future validity starts.
    pub max_future_skew_ns: u64,
    /// Variable material budget, using the same accounting as token verification
    /// but excluding claims and issuer signature material. Not a wire-size limit.
    pub max_variable_bytes: usize,
    /// Maximum number of Merkle sibling steps.
    pub max_witness_steps: usize,
}

impl From<TokenVerificationLimits> for DelegationProofVerificationLimits {
    fn from(limits: TokenVerificationLimits) -> Self {
        Self {
            max_cert_ttl_ns: limits.max_cert_ttl_ns,
            max_token_ttl_ns: limits.max_token_ttl_ns,
            max_future_skew_ns: limits.max_future_skew_ns,
            max_variable_bytes: limits.max_variable_bytes,
            max_witness_steps: limits.max_witness_steps,
        }
    }
}

/// Independently protected inputs for issuer-local delegation installation.
pub struct DelegationProofVerificationContext<'a> {
    /// Actual intended issuer from runtime/configuration, not the submitted proof.
    pub expected_issuer: Principal,
    /// Current independently enrolled authority and live revocation policy.
    pub root_key: &'a RootKeyPolicy,
    /// Actual host clock in nanoseconds.
    pub now_ns: u64,
    pub limits: DelegationProofVerificationLimits,
}

/// Evidence that the root delegated this certificate to the expected issuer at
/// this call's clock and policy. Neither a verified token nor issuance approval.
/// No public constructor or deserializer; later use still requires live checks.
/// The root signs the issuer leaf, which excludes certificate issue-time metadata;
/// that metadata is checked for consistency, not independently authenticated.
#[derive(Clone, Debug)]
pub struct VerifiedDelegationProof<'a> {
    certificate: &'a DelegationCert,
    certificate_hash: [u8; 32],
    expires_at_ns: u64,
}

impl<'a> VerifiedDelegationProof<'a> {
    pub fn certificate(&self) -> &'a DelegationCert {
        self.certificate
    }

    /// Canonical hash of the validated certificate, including its checked but
    /// root-unsigned issue time. This hash is not separately signed by the root.
    pub const fn certificate_hash(&self) -> [u8; 32] {
        self.certificate_hash
    }

    /// Exclusive certificate deadline capped by the current root-key policy.
    pub const fn expires_at_ns(&self) -> u64 {
        self.expires_at_ns
    }
}

/// Validate a root-issued delegation without fabricating token claims or an
/// issuer signature. Uses the complete token verifier's certificate rules and
/// root cryptography. Errors reuse its typed rejection surface; issuer-signature
/// and token-only context errors cannot arise here. Hosts bound outer decoding
/// and retain installation/issuance authorization, live policy and storage.
pub fn verify_delegation_proof<'a>(
    proof: &'a DelegationProof,
    context: &DelegationProofVerificationContext<'_>,
) -> Result<VerifiedDelegationProof<'a>, TokenVerificationError> {
    let mut remaining = context.limits.max_variable_bytes;
    rules::check_proof_size(proof, context.limits.max_witness_steps, &mut remaining)?;
    let cert = &proof.cert;
    for (field, principal) in [("issuer", cert.issuer_pid), ("root", cert.root_pid)] {
        if principal == Principal::anonymous() {
            return Err(TokenVerificationError::AnonymousPrincipal { field });
        }
    }
    binding(
        cert.root_pid == context.root_key.root_canister_id,
        "root_canister_id",
    )?;
    binding(
        cert.issuer_pid == context.expected_issuer,
        "issuer_canister_id",
    )?;
    rules::verify_certificate_window(cert, context.now_ns, context.limits)?;
    rules::verify_certificate_binding(cert)?;
    let certificate_hash = cert_hash(cert)?;
    root::verify(
        cert,
        &proof.root_proof,
        context.root_key,
        context.now_ns,
        context.limits.max_future_skew_ns,
    )?;
    Ok(VerifiedDelegationProof {
        certificate: cert,
        certificate_hash,
        expires_at_ns: cert.expires_at_ns.min(context.root_key.accept_until_ns),
    })
}
