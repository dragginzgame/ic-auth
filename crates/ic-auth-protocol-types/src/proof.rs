//! Passive application authentication proof contracts.
//! Adapted from Canic; see docs/design/canic-source-review.md.

use super::{DelegatedRoleGrant, DelegationAudience};
use crate::Principal;
use serde::{Deserialize, Serialize};

//
// RootProof
//

/// Unverified root-authority proof; trusted root keys come from host enrollment.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RootProof {
    /// Chain-key signature over a batch containing the issuer's delegation leaf.
    IcChainKeyBatchSignatureV1(IcChainKeyBatchSignatureProofV1),
}

//
// IcCanisterSignatureProofV1
//

/// IC canister-signature material, checked against the delegated issuer binding.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IcCanisterSignatureProofV1 {
    /// CBOR signature envelope containing the certificate and signature witness.
    pub signature_cbor: Vec<u8>,
    /// DER-encoded canister-signature public key; submission does not enroll trust.
    pub public_key_der: Vec<u8>,
}

//
// ChainKeyAlgorithm
//

/// Supported root chain-key signature algorithm.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ChainKeyAlgorithm {
    /// ECDSA over secp256k1, subject to Auth's canonical signature policy.
    EcdsaSecp256k1,
}

//
// ChainKeyKeyId
//

/// Chain-key identifier within the separately selected algorithm and network.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChainKeyKeyId {
    /// Network key name, which must match independently enrolled root policy.
    pub name: String,
}

//
// IcChainKeyBatchSignatureProofV1
//

/// Signed batch and Merkle inclusion proof for one issuer's delegation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IcChainKeyBatchSignatureProofV1 {
    /// Signed batch metadata binding the tree root and authority epochs.
    pub header: ChainKeyBatchHeaderV1,
    /// Issuer delegation leaf whose canonical hash is proven in the batch.
    pub delegation_cert: ChainKeyDelegationCertV1,
    /// Sibling path from the issuer leaf to the signed tree root.
    pub issuer_witness: ChainKeyBatchWitnessV1,
    /// Root signature and submitted key metadata, checked against enrollment.
    pub signature: ChainKeyRootSignatureV1,
}

//
// ChainKeyBatchHeaderV1
//

/// Canonically signed batch header; all identities and epochs remain untrusted
/// until matched against the host's protected root policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChainKeyBatchHeaderV1 {
    /// Schema discriminator for the batch's canonical encoding.
    pub schema_version: u16,
    /// Claimed root authority canister, compared with protected enrollment.
    pub root_canister_id: Principal,
    /// Root-assigned identifier distinguishing this batch.
    pub batch_id: [u8; 32],
    /// Proof generation, subject to the host's minimum accepted epoch.
    pub proof_epoch: u64,
    /// Registry generation, subject to the host's minimum accepted epoch.
    pub registry_epoch: u64,
    /// Digest binding this batch to the root's registry snapshot.
    pub registry_hash: [u8; 32],
    /// Merkle root against which issuer inclusion is verified.
    pub tree_root: [u8; 32],
    /// Validity start in nanoseconds, checked with explicit host skew policy.
    pub not_before_ns: u64,
    /// Exclusive batch expiry in nanoseconds.
    pub expires_at_ns: u64,
    /// Signature algorithm that must match the enrolled root key.
    pub algorithm: ChainKeyAlgorithm,
    /// Network key identifier that must match enrollment and signature metadata.
    pub key_id: ChainKeyKeyId,
    /// Canonical digest of the root signature's derivation path.
    pub derivation_path_hash: [u8; 32],
    /// Root key generation, checked against the enrolled version and lower bound.
    pub key_version: u64,
}

//
// ChainKeyDelegationCertV1
//

/// Issuer authority committed as a leaf in a root-signed batch. This delegates
/// application credential authority, not IC ingress identity or resource ownership.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChainKeyDelegationCertV1 {
    /// Root authority that delegates the issuer's permissions.
    pub root_canister_id: Principal,
    /// Canister authorized to produce the delegated issuer proofs.
    pub issuer_canister_id: Principal,
    /// Proof generation bound to the containing batch.
    pub proof_epoch: u64,
    /// Permitted issuer proof algorithm.
    pub issuer_proof_algorithm: IssuerProofAlgorithm,
    /// Canonical digest of the issuer proof binding.
    pub issuer_proof_binding_hash: [u8; 32],
    /// Key material binding that issuer proofs must satisfy.
    pub issuer_proof_binding: IssuerProofBinding,
    /// Maximum credential lifetime delegated to this issuer, in nanoseconds.
    pub max_token_ttl_ns: u64,
    /// Audience for which the delegated grants may be used.
    pub audience: DelegationAudience,
    /// Upper bound on role/scope authority the issuer may grant.
    pub grants: Vec<DelegatedRoleGrant>,
    /// Delegation validity start in nanoseconds.
    pub not_before_ns: u64,
    /// Exclusive delegation expiry in nanoseconds.
    pub expires_at_ns: u64,
    /// Registry generation bound to the containing batch.
    pub registry_epoch: u64,
    /// Registry snapshot digest bound to the containing batch.
    pub registry_hash: [u8; 32],
}

//
// ChainKeyRootSignatureV1
//

/// Root signature with submitted key metadata. The public key is never a trust
/// anchor by itself; verification compares it with independently enrolled policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChainKeyRootSignatureV1 {
    /// Signature algorithm, also bound by the signed batch header.
    pub algorithm: ChainKeyAlgorithm,
    /// Network key identifier, also bound by the signed batch header.
    pub key_id: ChainKeyKeyId,
    /// Ordered derivation components whose canonical hash must match enrollment.
    pub derivation_path: Vec<Vec<u8>>,
    /// SEC1-encoded secp256k1 public key, compared with the enrolled key.
    pub public_key: Vec<u8>,
    /// Root signature bytes over the canonical batch header digest.
    pub signature: Vec<u8>,
}

//
// ChainKeyBatchWitnessV1
//

/// Ordered Merkle inclusion path; verification bounds its length before hashing.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChainKeyBatchWitnessV1 {
    /// Sibling hashes in leaf-to-root order.
    pub steps: Vec<ChainKeyBatchWitnessStepV1>,
}

//
// ChainKeyBatchWitnessStepV1
//

/// Position of a sibling relative to the running Merkle hash.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ChainKeyBatchWitnessStepV1 {
    /// Hash the sibling on the left of the current subtree.
    LeftSibling([u8; 32]),
    /// Hash the sibling on the right of the current subtree.
    RightSibling([u8; 32]),
}

//
// IssuerProofAlgorithm
//

/// Proof mechanism a root delegation permits the issuer to use.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IssuerProofAlgorithm {
    /// IC canister signature checked with its certified signature witness.
    IcCanisterSignatureV1,
}

//
// IssuerProofBinding
//

/// Delegated key binding; the corresponding issuer identity is carried separately.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IssuerProofBinding {
    /// Bind the issuer's canister-signature key to a particular seed digest.
    IcCanisterSignatureV1 {
        /// Expected hash of the seed extracted from the canister-signature key.
        seed_hash: [u8; 32],
    },
}

//
// DelegationCert
//

/// Application-facing delegation certificate checked against the root proof.
/// Its issue time is consistency metadata, not independently root-signed authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DelegationCert {
    /// Root authority expected to authorize this delegation.
    pub root_pid: Principal,
    /// Issuer permitted to sign application credentials.
    pub issuer_pid: Principal,
    /// Proof algorithm delegated to the issuer.
    pub issuer_proof_alg: IssuerProofAlgorithm,
    /// Canonical digest binding the issuer's proof key.
    pub issuer_proof_binding_hash: [u8; 32],
    /// Issuer key binding checked against the root's delegation leaf.
    pub issuer_proof_binding: IssuerProofBinding,
    /// Issue-time metadata in nanoseconds; excluded from the root-signed leaf.
    pub issued_at_ns: u64,
    /// Delegation validity start in nanoseconds.
    pub not_before_ns: u64,
    /// Exclusive delegation deadline in nanoseconds.
    pub expires_at_ns: u64,
    /// Maximum credential lifetime delegated to the issuer, in nanoseconds.
    pub max_token_ttl_ns: u64,
    /// Audience admitted by this delegation.
    pub aud: DelegationAudience,
    /// Maximum role/scope grants the issuer may assign under this delegation.
    pub grants: Vec<DelegatedRoleGrant>,
}

//
// DelegationProof
//

/// Unverified delegation certificate paired with its root authority proof.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DelegationProof {
    /// Certificate to compare with the root-signed issuer leaf.
    pub cert: DelegationCert,
    /// Root signature and inclusion evidence authorizing that leaf.
    pub root_proof: RootProof,
}
