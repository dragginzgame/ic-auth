//! Passive root-delegation records, separate from transport and authentication.

use crate::{AudienceId, DelegatedRoleGrant, Principal};

/// Common issuer authorization fields carried by a certificate and its root leaf.
///
/// This profile supports IC canister signatures and a fleet audience only.
/// Construction checks no authority, binding, ordering or time relationships.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerDelegation {
    /// Claimed root canister.
    pub root: Principal,
    /// Claimed delegated issuer.
    pub issuer: Principal,
    /// Submitted issuer/algorithm/seed commitment; never silently recomputed.
    pub binding_hash: [u8; 32],
    /// Claimed IC canister-signature seed commitment.
    pub seed_hash: [u8; 32],
    /// Maximum delegated credential lifetime, in nanoseconds.
    pub max_token_ttl_ns: u64,
    /// Claimed network/application restriction.
    pub audience: AudienceId,
    /// Claimed role-wise scopes in received order.
    pub grants: Vec<DelegatedRoleGrant>,
    /// Inclusive validity start.
    pub not_before_ns: u64,
    /// Exclusive expiry.
    pub expires_at_ns: u64,
}

/// Certificate commitment used as the portable credential's key-binding identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationCertificate {
    /// Claimed authorization, requiring agreement with the signed issuer leaf.
    pub delegation: IssuerDelegation,
    /// Metadata included in the certificate hash but not the root-signed leaf.
    pub issued_at_ns: u64,
}

/// Issuer leaf committed by a root-signed chain-key batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationLeaf {
    /// Claimed authorization. The verifier must compare it with the certificate.
    pub delegation: IssuerDelegation,
    /// Claimed proof epoch.
    pub proof_epoch: u64,
    /// Claimed registry epoch.
    pub registry_epoch: u64,
    /// Registry commitment.
    pub registry_hash: [u8; 32],
}

/// Root batch header for the current ECDSA/secp256k1 signing profile.
///
/// Schema, key identity, epochs, tree root and time still require verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootBatchHeader {
    /// Submitted schema, checked separately from canonical field encoding.
    pub schema_version: u16,
    /// Claimed root canister.
    pub root: Principal,
    /// Batch identity.
    pub batch_id: [u8; 32],
    /// Claimed proof epoch.
    pub proof_epoch: u64,
    /// Claimed registry epoch.
    pub registry_epoch: u64,
    /// Registry commitment.
    pub registry_hash: [u8; 32],
    /// Committed Merkle root.
    pub tree_root: [u8; 32],
    /// Inclusive validity start.
    pub not_before_ns: u64,
    /// Exclusive expiry.
    pub expires_at_ns: u64,
    /// Untrusted key name; the encoder bounds its UTF-8 length to 128 bytes.
    pub key_name: String,
    /// Ordered derivation-path commitment.
    pub derivation_path_hash: [u8; 32],
    /// Claimed key version.
    pub key_version: u64,
}

/// One ordered sibling in the current root batch Merkle proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MerkleStep {
    /// Sibling preceding the current node.
    Left([u8; 32]),
    /// Sibling following the current node.
    Right([u8; 32]),
}
