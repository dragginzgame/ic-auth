//! Root-authorized delegation verification, independent of application credentials.
//! Hosts supply enrolled authority and clock; success installs no policy or session.
mod diagnostic;
mod root;
pub(crate) mod rules;
use crate::canonical::{CanonicalAuthError, cert_hash};
pub use diagnostic::{DelegationField, DelegationTarget};
use ic_auth_protocol_types::{
    ChainKeyAlgorithm, ChainKeyKeyId, DelegationCert, DelegationProof, Principal,
};
use thiserror::Error;

/// Protected enrollment of one root chain-key. The host selects the network's
/// permitted key names and obtains this public key independently of the proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootKeyPolicy {
    /// Independently enrolled root authority, never inferred from the proof.
    pub root_canister_id: Principal,
    /// Enrolled chain-key algorithm and network-approved key identifier.
    pub algorithm: ChainKeyAlgorithm,
    pub key_id: ChainKeyKeyId,
    /// Canonical hash of the enrolled derivation path, including its domain.
    pub derivation_path_hash: [u8; 32],
    /// Enrolled SEC1-encoded secp256k1 public key.
    pub public_key: Vec<u8>,
    /// Exact enrolled version; the submitted batch must match it.
    pub key_version: u64,
    /// Current lower bounds for authority invalidation, applied on every call.
    pub min_accepted_key_version: u64,
    pub min_accepted_proof_epoch: u64,
    pub min_accepted_registry_epoch: u64,
    /// Enrollment start, subject to the host's explicit future-skew allowance.
    pub valid_from_ns: u64,
    /// Exclusive deadline, checked on every call even for the same token.
    pub accept_until_ns: u64,
    /// Upper bound on batch lifetime, limiting stale root-grant authority.
    pub max_revocation_latency_ns: u64,
}

/// Typed rejection of bounds, policy, canonical encoding or a proof. Field and
/// target labels provide diagnostics; callers should match the enum variants.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum DelegationProofError {
    #[error("delegation proof material exceeds host limits")]
    InputTooLarge,
    #[error("anonymous {field} is not permitted")]
    AnonymousPrincipal { field: DelegationField },
    #[error("proof binding mismatch: {field}")]
    BindingMismatch { field: DelegationField },
    /// Submitted signature metadata does not match its submitted batch header.
    #[error("root signature binding mismatch: {field}")]
    RootSignatureBindingMismatch { field: DelegationField },
    /// The batch header does not match independently enrolled root-key policy.
    #[error("root key policy binding mismatch: {field}")]
    RootKeyPolicyBindingMismatch { field: DelegationField },
    #[error("invalid {target} validity window")]
    InvalidWindow { target: DelegationTarget },
    #[error("{target} is not yet valid")]
    NotYetValid { target: DelegationTarget },
    #[error("{target} has expired")]
    Expired { target: DelegationTarget },
    #[error("{target} TTL {ttl_ns} exceeds {max_ttl_ns}")]
    TtlExceeded {
        target: DelegationTarget,
        ttl_ns: u64,
        max_ttl_ns: u64,
    },
    #[error("grants must be nonempty, with at most 16 roles and 32 scopes per role")]
    InvalidGrants,
    #[error("root proof signature or public key encoding is invalid")]
    RootSignatureInvalid,
    #[error("root proof signature has high-s encoding")]
    HighSSignature,
    #[error("root proof Merkle witness is invalid")]
    InvalidMerkleWitness,
    #[error("root proof {field} is below the protected epoch/version floor")]
    StaleAuthority { field: DelegationField },
    #[error(transparent)]
    Canonical(#[from] CanonicalAuthError),
}

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

/// Authenticate a root-issued delegation with protected issuer/key policy.
/// Certificate rules and root cryptography are independent of portable admission. Hosts bound outer decoding
/// and retain installation/issuance authorization, live policy and storage.
pub fn verify_delegation_proof<'a>(
    proof: &'a DelegationProof,
    context: &DelegationProofVerificationContext<'_>,
) -> Result<VerifiedDelegationProof<'a>, DelegationProofError> {
    let mut remaining = context.limits.max_variable_bytes;
    rules::check_proof_size(proof, context.limits.max_witness_steps, &mut remaining)?;
    check_certificate(proof, context)?;
    PreparedDelegationProof::new(proof)?.verify_root(context)
}

/// Canonical material for one immutable, already size-admitted proof. This is
/// not authentication evidence: protected policy and root verification still run.
pub(crate) struct PreparedDelegationProof<'a> {
    proof: &'a DelegationProof,
    certificate_hash: [u8; 32],
}

impl<'a> PreparedDelegationProof<'a> {
    // Callers bound variable material before canonical encoding/allocation.
    pub(crate) fn new(proof: &'a DelegationProof) -> Result<Self, CanonicalAuthError> {
        Ok(Self {
            proof,
            certificate_hash: cert_hash(&proof.cert)?,
        })
    }

    #[cfg(feature = "certified-proof-verification")]
    pub(crate) fn certificate_hash(&self) -> [u8; 32] {
        self.certificate_hash
    }

    #[cfg(feature = "certified-proof-verification")]
    pub(crate) fn verify(
        self,
        context: &DelegationProofVerificationContext<'_>,
    ) -> Result<VerifiedDelegationProof<'a>, DelegationProofError> {
        check_certificate(self.proof, context)?;
        self.verify_root(context)
    }

    fn verify_root(
        self,
        context: &DelegationProofVerificationContext<'_>,
    ) -> Result<VerifiedDelegationProof<'a>, DelegationProofError> {
        let cert = &self.proof.cert;
        root::verify(
            cert,
            &self.proof.root_proof,
            context.root_key,
            context.now_ns,
            context.limits.max_future_skew_ns,
        )?;
        Ok(VerifiedDelegationProof {
            certificate: cert,
            certificate_hash: self.certificate_hash,
            expires_at_ns: cert.expires_at_ns.min(context.root_key.accept_until_ns),
        })
    }
}

fn check_certificate(
    proof: &DelegationProof,
    context: &DelegationProofVerificationContext<'_>,
) -> Result<(), DelegationProofError> {
    let cert = &proof.cert;
    for (field, principal) in [
        (DelegationField::Issuer, cert.issuer_pid),
        (DelegationField::Root, cert.root_pid),
    ] {
        if principal == Principal::anonymous() {
            return Err(DelegationProofError::AnonymousPrincipal { field });
        }
    }
    binding(
        cert.root_pid == context.root_key.root_canister_id,
        DelegationField::RootCanisterId,
    )?;
    binding(
        cert.issuer_pid == context.expected_issuer,
        DelegationField::IssuerCanisterId,
    )?;
    rules::verify_certificate_window(cert, context.now_ns, context.limits)?;
    rules::verify_certificate_binding(cert)
}

pub(super) fn binding(condition: bool, field: DelegationField) -> Result<(), DelegationProofError> {
    if condition {
        Ok(())
    } else {
        Err(DelegationProofError::BindingMismatch { field })
    }
}

pub(super) fn window(
    target: DelegationTarget,
    start: u64,
    end: u64,
    now: u64,
    skew: u64,
) -> Result<(), DelegationProofError> {
    if start >= end {
        return Err(DelegationProofError::InvalidWindow { target });
    }
    if start > now && start - now > skew {
        return Err(DelegationProofError::NotYetValid { target });
    }
    if now >= end {
        return Err(DelegationProofError::Expired { target });
    }
    Ok(())
}
