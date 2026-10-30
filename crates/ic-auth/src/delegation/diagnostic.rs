//! Closed Rust diagnostic labels; these are not signed fields or Wire tags.
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelegationField {
    Issuer,
    Root,
    SchemaVersion,
    RootCanisterId,
    IssuerCanisterId,
    CertIssuedAt,
    LeafBatchWindow,
    ProofEpoch,
    RegistryEpoch,
    RegistryHash,
    IssuerProofAlgorithm,
    IssuerProofBinding,
    IssuerProofBindingHash,
    MaxTokenTtl,
    LeafAudience,
    LeafGrants,
    LeafCertWindow,
    Algorithm,
    KeyId,
    RootPublicKey,
    DerivationPathHash,
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
pub enum DelegationTarget {
    Certificate,
    RootKeyPolicy,
    Batch,
    IssuerLeaf,
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
