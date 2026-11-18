//! Root authorization of an issuer delegation under protected host inputs.
//!
//! This component verifies a bounded root batch and certificate correspondence.
//! It is not portable-credential verification, enrollment installation or session
//! admission. The full credential verifier must meter the whole proof before
//! calling this component; this function itself changes no state or budget.

use crate::{
    claims::validate_grants,
    delegation_encoding::{
        DelegationEncodingError, batch_header_hash, certificate_hash, delegation_leaf_hash,
        derivation_path_hash, issuer_binding_hash,
    },
};
use ic_auth_protocol_types::{
    DelegatedRoleGrant, DelegationCertificate, DelegationLeaf, MerkleStep, Principal,
    RootBatchHeader,
};
use ic_platform::ecdsa::{PrehashVerifier, Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use std::{error::Error, fmt};

/// Untrusted root-signature fields in the current ECDSA/secp256k1 profile.
#[derive(Clone, Copy, Debug)]
pub struct RootSignature<'a> {
    /// Submitted key name; it must match both the header and protected policy.
    pub key_name: &'a str,
    /// Ordered submitted derivation components.
    pub derivation_path: &'a [Vec<u8>],
    /// Submitted SEC1 key; byte identity must match the independently enrolled key.
    pub public_key: &'a [u8],
    /// Compact 64-byte signature; high-S values are never normalized into validity.
    pub signature: &'a [u8],
}

/// Borrowed root proof; outer transport decoding must already be bounded.
#[derive(Clone, Copy, Debug)]
pub struct RootDelegationProof<'a> {
    /// Certificate whose identity is committed by a portable credential.
    pub certificate: &'a DelegationCertificate,
    /// Root-signed batch header.
    pub header: &'a RootBatchHeader,
    /// Issuer authorization leaf committed by the batch tree.
    pub leaf: &'a DelegationLeaf,
    /// Ordered sibling path, capped independently of byte material.
    pub witness: &'a [MerkleStep],
    /// Submitted root signature material.
    pub signature: RootSignature<'a>,
}

impl<'a> From<&'a ic_auth_protocol_types::RootDelegation> for RootDelegationProof<'a> {
    fn from(value: &'a ic_auth_protocol_types::RootDelegation) -> Self {
        Self {
            certificate: &value.certificate,
            header: &value.header,
            leaf: &value.leaf,
            witness: &value.witness,
            signature: RootSignature {
                key_name: &value.signature.key_name,
                derivation_path: &value.signature.derivation_path,
                public_key: &value.signature.public_key,
                signature: &value.signature.signature,
            },
        }
    }
}

/// Current independently enrolled root authority; never populate from a proof.
#[derive(Clone, Copy, Debug)]
pub struct RootKeyPolicy<'a> {
    /// Approved root canister.
    pub root: Principal,
    /// Network-approved ECDSA/secp256k1 key name.
    pub key_name: &'a str,
    /// Enrolled ordered derivation-path commitment.
    pub derivation_path_hash: [u8; 32],
    /// Enrolled SEC1 key, with exact byte identity retained.
    pub public_key: &'a [u8],
    /// Exact current enrolled version.
    pub key_version: u64,
    /// Inclusive key-version revocation floor.
    pub min_key_version: u64,
    /// Inclusive proof-epoch revocation floor.
    pub min_proof_epoch: u64,
    /// Inclusive registry-epoch floor.
    pub min_registry_epoch: u64,
    /// Inclusive enrollment start.
    pub valid_from_ns: u64,
    /// Exclusive enrollment expiry.
    pub accept_until_ns: u64,
    /// Maximum batch lifetime, bounding stale root-grant authority.
    pub max_revocation_latency_ns: u64,
}

/// Protected limits for the current zero-future-grace delegation profile.
#[derive(Clone, Copy, Debug)]
pub struct DelegationLimits {
    /// Maximum certificate validity duration.
    pub max_certificate_ttl_ns: u64,
    /// Maximum lifetime the certificate may delegate to a credential.
    pub max_token_ttl_ns: u64,
    /// Variable byte/entry allowance, also capped at 131072 bytes.
    pub max_variable_bytes: usize,
    /// Maximum sibling steps, also capped at the transport profile's 64.
    pub max_witness_steps: usize,
}

/// A validity window whose boundary failed admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidityTarget {
    /// Issuer authorization certificate.
    Certificate,
    /// Protected root-key enrollment.
    RootKey,
    /// Root-signed batch.
    Batch,
    /// Issuer leaf.
    Leaf,
}

/// Root proof refusal, without any installed authority or session mutation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelegationError {
    /// Variable material, field bounds or witness work exceeds admission limits.
    InputTooLarge,
    /// Certificate root/issuer is anonymous or differs from protected selection.
    IdentityMismatch,
    /// Certificate issue time is after its validity start.
    IssuedAfterStart,
    /// Empty or reversed validity interval.
    InvalidWindow(ValidityTarget),
    /// Validity starts in the future; this profile allows no grace.
    NotYetValid(ValidityTarget),
    /// Exclusive deadline has been reached.
    Expired(ValidityTarget),
    /// Certificate or delegated-token lifetime exceeds protected limits.
    CertificateLifetime,
    /// Grant shape/order or bounded canonical encoding failed.
    Encoding(DelegationEncodingError),
    /// Submitted issuer/seed commitment is inconsistent.
    IssuerBindingMismatch,
    /// Unsupported batch schema or disagreement between certificate, leaf and header.
    ProofBindingMismatch,
    /// Signature metadata disagrees with the submitted header.
    SignatureBindingMismatch,
    /// Submitted metadata disagrees with protected root-key enrollment.
    RootKeyMismatch,
    /// Version or epoch is below a protected revocation floor.
    StaleAuthority,
    /// Batch lifetime exceeds protected revocation latency.
    BatchLifetime,
    /// Issuer leaf and sibling path do not reproduce the signed root.
    InvalidWitness,
    /// SEC1 key, compact signature or ECDSA prehash verification is invalid.
    InvalidSignature,
    /// Signature has high-S form; the verifier does not repair it.
    HighSignatureScalar,
}
impl fmt::Display for DelegationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "root delegation refused: {self:?}")
    }
}
impl Error for DelegationError {}

/// Root-verified issuer authorization at one protected snapshot.
///
/// The certificate's issue-time metadata is checked for consistency but not
/// independently signed by the root. Later use still needs current enrollment,
/// claim binding, issuer signature, work charging and receiver/session policy.
/// ```compile_fail
/// use ic_auth::root_delegation::VerifiedDelegation;
/// use ic_auth_protocol_types::DelegationCertificate;
/// fn forge(cert: &DelegationCertificate) -> VerifiedDelegation<'_> { cert.into() }
/// ```
/// ```compile_fail
/// use ic_auth::root_delegation::VerifiedDelegation;
/// fn change(proof: &mut VerifiedDelegation<'_>) { proof.certificate().issued_at_ns = 0; }
/// ```
#[derive(Clone, Debug)]
pub struct VerifiedDelegation<'a> {
    certificate: &'a DelegationCertificate,
    certificate_hash: [u8; 32],
    expires_at_ns: u64,
}
impl<'a> VerifiedDelegation<'a> {
    /// Borrow the checked certificate, not a continuing authorization grant.
    #[must_use]
    pub const fn certificate(&self) -> &'a DelegationCertificate {
        self.certificate
    }

    /// Exact certificate identity, including its root-unsigned issuance metadata.
    #[must_use]
    pub const fn certificate_hash(&self) -> [u8; 32] {
        self.certificate_hash
    }

    /// Exclusive certificate deadline capped by current root-key enrollment.
    #[must_use]
    pub const fn expires_at_ns(&self) -> u64 {
        self.expires_at_ns
    }
}

/// Verify root authorization for the protected issuer, without installing it.
///
/// Bounds precede canonical encoding and cryptography. Current enrollment,
/// versions, epochs and windows are rechecked on every invocation. Platform owns
/// key/scalar parsing and ECDSA; SHA-256 only assembles the fixed protocol's
/// header and ordered Merkle commitments. Full credential admission must first
/// charge its shared receiver-wide budget; this standalone component does not.
pub fn verify_root_delegation<'a>(
    proof: RootDelegationProof<'a>,
    expected_issuer: Principal,
    policy: &RootKeyPolicy<'_>,
    now_ns: u64,
    limits: DelegationLimits,
) -> Result<VerifiedDelegation<'a>, DelegationError> {
    let mut remaining = limits.max_variable_bytes.min(131_072);
    consume_proof_bytes(&proof, limits.max_witness_steps, &mut remaining)?;
    verify_authorization(proof, expected_issuer, policy, now_ns, limits)?;
    Ok(VerifiedDelegation {
        certificate: proof.certificate,
        certificate_hash: certificate_hash(proof.certificate).map_err(DelegationError::Encoding)?,
        expires_at_ns: proof
            .certificate
            .delegation
            .expires_at_ns
            .min(policy.accept_until_ns),
    })
}

// Callers admit aggregate material before entering this shared root verifier.
// The full credential path has already hashed the immutable certificate;
// authorization itself needs no second certificate commitment or copied result.
pub(crate) fn verify_authorization(
    proof: RootDelegationProof<'_>,
    expected_issuer: Principal,
    policy: &RootKeyPolicy<'_>,
    now_ns: u64,
    limits: DelegationLimits,
) -> Result<(), DelegationError> {
    use DelegationError as E;
    let cert = proof.certificate;
    let d = &cert.delegation;
    let h = proof.header;
    let leaf = proof.leaf;
    let signature = proof.signature;
    if d.root == Principal::anonymous()
        || d.issuer == Principal::anonymous()
        || d.root != policy.root
        || d.issuer != expected_issuer
    {
        return Err(E::IdentityMismatch);
    }
    if cert.issued_at_ns > d.not_before_ns {
        return Err(E::IssuedAfterStart);
    }
    if d.not_before_ns >= d.expires_at_ns {
        return Err(E::InvalidWindow(ValidityTarget::Certificate));
    }
    let ttl = d.expires_at_ns - d.not_before_ns;
    if ttl > limits.max_certificate_ttl_ns
        || d.max_token_ttl_ns == 0
        || d.max_token_ttl_ns > ttl.min(limits.max_token_ttl_ns)
    {
        return Err(E::CertificateLifetime);
    }
    window(
        ValidityTarget::Certificate,
        d.not_before_ns,
        d.expires_at_ns,
        now_ns,
    )?;
    validate_grants(&d.grants).map_err(|e| E::Encoding(DelegationEncodingError::Grants(e)))?;
    if issuer_binding_hash(d.issuer, d.seed_hash) != d.binding_hash {
        return Err(E::IssuerBindingMismatch);
    }
    if h.schema_version != 1 || h.root != policy.root || leaf.delegation.root != h.root {
        return Err(E::ProofBindingMismatch);
    }
    window(
        ValidityTarget::RootKey,
        policy.valid_from_ns,
        policy.accept_until_ns,
        now_ns,
    )?;
    window(
        ValidityTarget::Batch,
        h.not_before_ns,
        h.expires_at_ns,
        now_ns,
    )?;
    window(
        ValidityTarget::Leaf,
        leaf.delegation.not_before_ns,
        leaf.delegation.expires_at_ns,
        now_ns,
    )?;
    if leaf.delegation.not_before_ns < h.not_before_ns
        || leaf.delegation.expires_at_ns > h.expires_at_ns
        || leaf.proof_epoch != h.proof_epoch
        || leaf.registry_epoch != h.registry_epoch
        || leaf.registry_hash != h.registry_hash
        || leaf.delegation != *d
    {
        return Err(E::ProofBindingMismatch);
    }
    if signature.key_name != h.key_name {
        return Err(E::SignatureBindingMismatch);
    }
    if h.key_name != policy.key_name || signature.public_key != policy.public_key {
        return Err(E::RootKeyMismatch);
    }
    if derivation_path_hash(signature.derivation_path).map_err(E::Encoding)?
        != h.derivation_path_hash
    {
        return Err(E::SignatureBindingMismatch);
    }
    if h.derivation_path_hash != policy.derivation_path_hash || h.key_version != policy.key_version
    {
        return Err(E::RootKeyMismatch);
    }
    if h.key_version < policy.min_key_version
        || h.proof_epoch < policy.min_proof_epoch
        || h.registry_epoch < policy.min_registry_epoch
    {
        return Err(E::StaleAuthority);
    }
    if h.expires_at_ns - h.not_before_ns > policy.max_revocation_latency_ns {
        return Err(E::BatchLifetime);
    }
    let mut root = delegation_leaf_hash(leaf).map_err(E::Encoding)?;
    for step in proof.witness {
        let (left, right) = match step {
            MerkleStep::Left(hash) => (hash, &root),
            MerkleStep::Right(hash) => (&root, hash),
        };
        let mut digest = Sha256::new();
        digest.update([1]);
        digest.update(left);
        digest.update(right);
        root = digest.finalize().into();
    }
    if root != h.tree_root {
        return Err(E::InvalidWitness);
    }
    let key = VerifyingKey::from_sec1_bytes(policy.public_key).map_err(|_| E::InvalidSignature)?;
    let signature = Signature::from_slice(signature.signature).map_err(|_| E::InvalidSignature)?;
    if signature.normalize_s() != signature {
        return Err(E::HighSignatureScalar);
    }
    key.verify_prehash(&batch_header_hash(h).map_err(E::Encoding)?, &signature)
        .map_err(|_| E::InvalidSignature)?;
    Ok(())
}

fn window(target: ValidityTarget, start: u64, end: u64, now: u64) -> Result<(), DelegationError> {
    if start >= end {
        return Err(DelegationError::InvalidWindow(target));
    }
    if start > now {
        return Err(DelegationError::NotYetValid(target));
    }
    if now >= end {
        return Err(DelegationError::Expired(target));
    }
    Ok(())
}

fn take(remaining: &mut usize, count: usize) -> Result<(), DelegationError> {
    *remaining = remaining
        .checked_sub(count)
        .ok_or(DelegationError::InputTooLarge)?;
    Ok(())
}

fn grants_size(
    grants: &[DelegatedRoleGrant],
    remaining: &mut usize,
) -> Result<(), DelegationError> {
    take(remaining, grants.len())?;
    for grant in grants {
        take(remaining, grant.target.as_str().len())?;
        take(remaining, grant.scopes.len())?;
        for scope in &grant.scopes {
            take(remaining, scope.as_str().len())?;
        }
    }
    Ok(())
}

pub(crate) fn consume_proof_bytes(
    proof: &RootDelegationProof<'_>,
    max_witness_steps: usize,
    remaining: &mut usize,
) -> Result<(), DelegationError> {
    let s = proof.signature;
    if proof.witness.len() > max_witness_steps.min(64)
        || proof.header.key_name.len() > 128
        || s.key_name.len() > 128
        || s.public_key.len() > 65
        || s.signature.len() > 64
        || s.derivation_path.len() > 32
        || s.derivation_path.iter().any(|p| p.len() > 256)
    {
        return Err(DelegationError::InputTooLarge);
    }
    grants_size(&proof.certificate.delegation.grants, remaining)?;
    grants_size(&proof.leaf.delegation.grants, remaining)?;
    take(remaining, proof.witness.len() * 33)?;
    for count in [
        proof.header.key_name.len(),
        s.key_name.len(),
        s.public_key.len(),
        s.signature.len(),
        s.derivation_path.len(),
    ] {
        take(remaining, count)?;
    }
    for part in s.derivation_path {
        take(remaining, part.len())?;
    }
    Ok(())
}
