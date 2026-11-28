//! Consuming projection from bounded Wire records to passive Auth values.
//!
//! No caller, clock, enrollment, verifier selection or endpoint is supplied here.
//! Opaque buffers and strings are moved. Only label grammar is checked; signed
//! order, duplicate commitments, counts, schema and cryptographic validity remain
//! the verifier's responsibility. Nothing is sorted or repaired into validity.

use crate::model as wire;
use ic_auth_protocol_types as domain;
use std::{error::Error, fmt};

///
/// ProjectionError
///
/// A transport label cannot be represented by the passive domain grammar.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionError {
    /// Invalid role spelling, without normalization.
    Role(domain::RoleError),
    /// Invalid scope spelling, without normalization.
    Scope(domain::ScopeLabelError),
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Role(e) => e.fmt(f),
            Self::Scope(e) => e.fmt(f),
        }
    }
}

impl Error for ProjectionError {}

impl TryFrom<wire::PortableCredential> for domain::Credential {
    type Error = ProjectionError;
    fn try_from(value: wire::PortableCredential) -> Result<Self, Self::Error> {
        Ok(Self {
            claims: value.claims.into_vec(),
            proof: match value.proof {
                wire::PortableProof::Ed25519(signature) => {
                    domain::CredentialProof::Ed25519(signature.into_vec())
                }
                wire::PortableProof::Certified(proof) => {
                    domain::CredentialProof::Certified(Box::new(certified(proof)?))
                }
            },
        })
    }
}

fn audience(value: wire::DelegationAudience) -> domain::AudienceId {
    let wire::DelegationAudience::Fleet(value) = value;
    domain::AudienceId {
        canonical_network_id: domain::CanonicalId::from_bytes(value.canonical_network_id),
        fleet_id: domain::CanonicalId::from_bytes(value.fleet_id),
    }
}

fn grants(
    value: ic_wire::List<wire::DelegatedRoleGrant, 16>,
) -> Result<Vec<domain::DelegatedRoleGrant>, ProjectionError> {
    value
        .into_vec()
        .into_iter()
        .map(|grant| {
            Ok(domain::DelegatedRoleGrant {
                target: domain::AuthRole::try_from(grant.target.into_string())
                    .map_err(ProjectionError::Role)?,
                scopes: grant
                    .scopes
                    .into_vec()
                    .into_iter()
                    .map(|scope| {
                        domain::ScopeLabel::try_from(scope.into_string())
                            .map_err(ProjectionError::Scope)
                    })
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect()
}

fn certified(
    value: wire::CertifiedProof,
) -> Result<domain::CertifiedProofMaterial, ProjectionError> {
    let cert = value.delegation.cert;
    let wire::IssuerProofAlgorithm::IcCanisterSignatureV1 = cert.issuer_proof_alg;
    let wire::IssuerProofBinding::IcCanisterSignatureV1(seed_hash) = cert.issuer_proof_binding;
    let certificate = domain::DelegationCertificate {
        issued_at_ns: cert.issued_at_ns,
        delegation: domain::IssuerDelegation {
            root: cert.root_pid,
            issuer: cert.issuer_pid,
            binding_hash: cert.issuer_proof_binding_hash,
            seed_hash,
            max_token_ttl_ns: cert.max_token_ttl_ns,
            audience: audience(cert.aud),
            grants: grants(cert.grants)?,
            not_before_ns: cert.not_before_ns,
            expires_at_ns: cert.expires_at_ns,
        },
    };
    let wire::RootProof::IcChainKeyBatchSignatureV1(root) = value.delegation.root_proof;
    let h = root.header;
    let wire::ChainKeyAlgorithm::EcdsaSecp256k1 = h.algorithm;
    let header = domain::RootBatchHeader {
        schema_version: h.schema_version,
        root: h.root_canister_id,
        batch_id: h.batch_id,
        proof_epoch: h.proof_epoch,
        registry_epoch: h.registry_epoch,
        registry_hash: h.registry_hash,
        tree_root: h.tree_root,
        not_before_ns: h.not_before_ns,
        expires_at_ns: h.expires_at_ns,
        key_name: h.key_id.name.into_string(),
        derivation_path_hash: h.derivation_path_hash,
        key_version: h.key_version,
    };
    let leaf = root.delegation_cert;
    let wire::IssuerProofAlgorithm::IcCanisterSignatureV1 = leaf.issuer_proof_algorithm;
    let wire::IssuerProofBinding::IcCanisterSignatureV1(seed_hash) = leaf.issuer_proof_binding;
    let leaf = domain::DelegationLeaf {
        proof_epoch: leaf.proof_epoch,
        registry_epoch: leaf.registry_epoch,
        registry_hash: leaf.registry_hash,
        delegation: domain::IssuerDelegation {
            root: leaf.root_canister_id,
            issuer: leaf.issuer_canister_id,
            binding_hash: leaf.issuer_proof_binding_hash,
            seed_hash,
            max_token_ttl_ns: leaf.max_token_ttl_ns,
            audience: audience(leaf.audience),
            grants: grants(leaf.grants)?,
            not_before_ns: leaf.not_before_ns,
            expires_at_ns: leaf.expires_at_ns,
        },
    };
    let signature = root.signature;
    let wire::ChainKeyAlgorithm::EcdsaSecp256k1 = signature.algorithm;
    Ok(domain::CertifiedProofMaterial {
        delegation: domain::RootDelegation {
            certificate,
            header,
            leaf,
            witness: root
                .issuer_witness
                .steps
                .into_vec()
                .into_iter()
                .map(|step| match step {
                    wire::ChainKeyBatchWitnessStepV1::LeftSibling(hash) => {
                        domain::MerkleStep::Left(hash)
                    }
                    wire::ChainKeyBatchWitnessStepV1::RightSibling(hash) => {
                        domain::MerkleStep::Right(hash)
                    }
                })
                .collect(),
            signature: domain::RootSignatureMaterial {
                key_name: signature.key_id.name.into_string(),
                derivation_path: signature
                    .derivation_path
                    .into_vec()
                    .into_iter()
                    .map(ic_wire::Bytes::into_vec)
                    .collect(),
                public_key: signature.public_key.into_vec(),
                signature: signature.signature.into_vec(),
            },
        },
        issuer_signature_cbor: value.issuer_signature.signature_cbor.into_vec(),
        issuer_public_key_der: value.issuer_signature.public_key_der.into_vec(),
    })
}
