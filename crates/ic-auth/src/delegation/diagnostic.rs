//! Closed Rust diagnostic labels; these are not signed fields or Wire tags.
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Field identity for a delegation mismatch; these labels are not signed protocol tags.
pub enum DelegationField {
    /// Expected issuer identity supplied by the host.
    Issuer,
    /// Expected root authority identity.
    Root,
    /// Canonical batch schema discriminator.
    SchemaVersion,
    /// Root canister identity in the compared proof records.
    RootCanisterId,
    /// Issuer canister identity in the delegation leaf.
    IssuerCanisterId,
    /// Certificate issue-time consistency metadata.
    CertIssuedAt,
    /// Issuer leaf validity compared with the enclosing batch window.
    LeafBatchWindow,
    /// Root proof generation.
    ProofEpoch,
    /// Root registry generation.
    RegistryEpoch,
    /// Digest binding the delegation to a registry snapshot.
    RegistryHash,
    /// Delegated issuer signature mechanism.
    IssuerProofAlgorithm,
    /// Delegated issuer key binding.
    IssuerProofBinding,
    /// Canonical digest of that key binding.
    IssuerProofBindingHash,
    /// Maximum credential lifetime delegated by the root.
    MaxTokenTtl,
    /// Audience shared by the certificate and issuer leaf.
    LeafAudience,
    /// Role/scope grants shared by the certificate and issuer leaf.
    LeafGrants,
    /// Issuer leaf validity compared with the public certificate.
    LeafCertWindow,
    /// Root signature algorithm.
    Algorithm,
    /// Network chain-key identifier.
    KeyId,
    /// Root key bytes compared with protected enrollment.
    RootPublicKey,
    /// Canonical digest of the chain-key derivation path.
    DerivationPathHash,
    /// Root key generation.
    KeyVersion,
}

impl fmt::Display for DelegationField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Issuer => "issuer",
            Self::Root => "root",
            Self::SchemaVersion => "schema_version",
            Self::RootCanisterId => "root_canister_id",
            Self::IssuerCanisterId => "issuer_canister_id",
            Self::CertIssuedAt => "cert_issued_at",
            Self::LeafBatchWindow => "leaf_batch_window",
            Self::ProofEpoch => "proof_epoch",
            Self::RegistryEpoch => "registry_epoch",
            Self::RegistryHash => "registry_hash",
            Self::IssuerProofAlgorithm => "issuer_proof_algorithm",
            Self::IssuerProofBinding => "issuer_proof_binding",
            Self::IssuerProofBindingHash => "issuer_proof_binding_hash",
            Self::MaxTokenTtl => "max_token_ttl",
            Self::LeafAudience => "leaf_audience",
            Self::LeafGrants => "leaf_grants",
            Self::LeafCertWindow => "leaf_cert_window",
            Self::Algorithm => "algorithm",
            Self::KeyId => "key_id",
            Self::RootPublicKey => "root_public_key",
            Self::DerivationPathHash => "derivation_path_hash",
            Self::KeyVersion => "key_version",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Object whose validity or lifetime failed a delegation check.
pub enum DelegationTarget {
    /// Application-facing delegation certificate.
    Certificate,
    /// Protected root-key enrollment policy.
    RootKeyPolicy,
    /// Root-signed batch header.
    Batch,
    /// Issuer delegation leaf proven in the batch.
    IssuerLeaf,
    /// Credential lifetime delegated by the issuer leaf or certificate.
    MaxTokenTtl,
}

impl fmt::Display for DelegationTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Certificate => "certificate",
            Self::RootKeyPolicy => "root_key_policy",
            Self::Batch => "batch",
            Self::IssuerLeaf => "issuer_leaf",
            Self::MaxTokenTtl => "max_token_ttl",
        })
    }
}
