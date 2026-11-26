//! Fixed proof layouts. Every field is untrusted until full verification.

use ic_auth_protocol_types::Principal;
use ic_wire::{Bytes, List, Text};
use serde::{Deserialize, Serialize};

///
/// AudienceId
///
/// Claimed network and application identities, in fixed transport order.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct AudienceId {
    /// Claimed canonical network identifier.
    pub canonical_network_id: [u8; 32],
    /// Claimed application identifier; the historical field name is retained.
    pub fleet_id: [u8; 32],
}

///
/// DelegatedRoleGrant
///
/// Bounded grant text, before label grammar and authorization checks.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct DelegatedRoleGrant {
    /// Claimed role, not a protected role selection.
    pub target: Text<64>,
    /// Claimed scopes in received order; transport does not normalize them.
    pub scopes: List<Text<64>, 32>,
}

///
/// DelegationAudience
///
/// Delegation audience tag and payload.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub enum DelegationAudience {
    /// Network/application audience, retaining the existing tag name.
    Fleet(AudienceId),
}

///
/// ChainKeyAlgorithm
///
/// Root chain-key algorithm tag; this does not select trusted enrollment.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub enum ChainKeyAlgorithm {
    /// ECDSA on secp256k1.
    EcdsaSecp256k1,
}

///
/// ChainKeyKeyId
///
/// Untrusted root key lookup hint.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct ChainKeyKeyId {
    /// Bounded key name.
    pub name: Text<128>,
}

///
/// IssuerProofAlgorithm
///
/// Issuer proof algorithm tag, checked against protected enrollment later.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub enum IssuerProofAlgorithm {
    /// Version-one IC canister signature.
    IcCanisterSignatureV1,
}

///
/// IssuerProofBinding
///
/// Claimed issuer key binding; not an independent trust anchor.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub enum IssuerProofBinding {
    /// SHA-256 seed hash of the issuer canister-signature public key.
    IcCanisterSignatureV1([u8; 32]),
}

///
/// IcCanisterSignatureProofV1
///
/// Bounded opaque canister-signature material.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct IcCanisterSignatureProofV1 {
    /// CBOR signature/certificate/witness bytes requiring complete verification.
    pub signature_cbor: Bytes<131072>,
    /// DER key bytes requiring issuer and seed binding checks.
    pub public_key_der: Bytes<256>,
}

///
/// ChainKeyBatchHeaderV1
///
/// Claimed signed batch-header fields, preserving their existing order.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct ChainKeyBatchHeaderV1 {
    /// Claimed header schema, not validated by transport.
    pub schema_version: u16,
    /// Claimed root canister.
    pub root_canister_id: Principal,
    /// Batch identity.
    pub batch_id: [u8; 32],
    /// Claimed proof epoch.
    pub proof_epoch: u64,
    /// Claimed registry epoch.
    pub registry_epoch: u64,
    /// Registry commitment.
    pub registry_hash: [u8; 32],
    /// Merkle tree root.
    pub tree_root: [u8; 32],
    /// Inclusive validity start, in nanoseconds.
    pub not_before_ns: u64,
    /// Exclusive expiry, in nanoseconds.
    pub expires_at_ns: u64,
    /// Claimed root-signature algorithm.
    pub algorithm: ChainKeyAlgorithm,
    /// Claimed key name.
    pub key_id: ChainKeyKeyId,
    /// Derivation-path commitment.
    pub derivation_path_hash: [u8; 32],
    /// Claimed key version.
    pub key_version: u64,
}

///
/// ChainKeyDelegationCertV1
///
/// Delegation leaf carried with a chain-key batch proof.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct ChainKeyDelegationCertV1 {
    /// Claimed root canister.
    pub root_canister_id: Principal,
    /// Claimed delegated issuer.
    pub issuer_canister_id: Principal,
    /// Claimed proof epoch.
    pub proof_epoch: u64,
    /// Claimed issuer proof mechanism.
    pub issuer_proof_algorithm: IssuerProofAlgorithm,
    /// Issuer-binding commitment.
    pub issuer_proof_binding_hash: [u8; 32],
    /// Claimed issuer key binding.
    pub issuer_proof_binding: IssuerProofBinding,
    /// Claimed maximum credential lifetime, in nanoseconds.
    pub max_token_ttl_ns: u64,
    /// Claimed audience restriction.
    pub audience: DelegationAudience,
    /// Unnormalized delegated role/scope claims.
    pub grants: List<DelegatedRoleGrant, 16>,
    /// Inclusive validity start, in nanoseconds.
    pub not_before_ns: u64,
    /// Exclusive expiry, in nanoseconds.
    pub expires_at_ns: u64,
    /// Claimed registry epoch.
    pub registry_epoch: u64,
    /// Registry commitment.
    pub registry_hash: [u8; 32],
}

///
/// ChainKeyRootSignatureV1
///
/// Bounded root-signature material; length bounds do not verify a signature.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct ChainKeyRootSignatureV1 {
    /// Claimed root-signature algorithm.
    pub algorithm: ChainKeyAlgorithm,
    /// Untrusted key lookup hint.
    pub key_id: ChainKeyKeyId,
    /// Derivation components, in their supplied order.
    pub derivation_path: List<Bytes<256>, 32>,
    /// Root public-key material, not a trusted key installation.
    pub public_key: Bytes<65>,
    /// Root signature bytes.
    pub signature: Bytes<64>,
}

///
/// ChainKeyBatchWitnessStepV1
///
/// Ordered Merkle witness step with the existing discriminants.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub enum ChainKeyBatchWitnessStepV1 {
    /// Sibling preceding the current node.
    LeftSibling([u8; 32]),
    /// Sibling following the current node.
    RightSibling([u8; 32]),
}

///
/// ChainKeyBatchWitnessV1
///
/// Bounded ordered batch witness.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct ChainKeyBatchWitnessV1 {
    /// At most 64 sibling steps.
    pub steps: List<ChainKeyBatchWitnessStepV1, 64>,
}

///
/// IcChainKeyBatchSignatureProofV1
///
/// Root batch proof with all authenticated fields still unverified.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct IcChainKeyBatchSignatureProofV1 {
    /// Signed batch header.
    pub header: ChainKeyBatchHeaderV1,
    /// Issuer delegation leaf.
    pub delegation_cert: ChainKeyDelegationCertV1,
    /// Leaf inclusion witness.
    pub issuer_witness: ChainKeyBatchWitnessV1,
    /// Root signature and untrusted key material.
    pub signature: ChainKeyRootSignatureV1,
}

///
/// RootProof
///
/// Root proof tag and its bounded payload.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub enum RootProof {
    /// Version-one chain-key batch signature proof.
    IcChainKeyBatchSignatureV1(IcChainKeyBatchSignatureProofV1),
}

///
/// DelegationCert
///
/// Claimed root delegation; duplicate commitments require verifier agreement.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct DelegationCert {
    /// Claimed root identity.
    pub root_pid: Principal,
    /// Claimed issuer identity.
    pub issuer_pid: Principal,
    /// Claimed issuer proof mechanism.
    pub issuer_proof_alg: IssuerProofAlgorithm,
    /// Issuer-binding commitment.
    pub issuer_proof_binding_hash: [u8; 32],
    /// Claimed issuer key binding.
    pub issuer_proof_binding: IssuerProofBinding,
    /// Claimed issuance time, in nanoseconds.
    pub issued_at_ns: u64,
    /// Inclusive validity start, in nanoseconds.
    pub not_before_ns: u64,
    /// Exclusive expiry, in nanoseconds.
    pub expires_at_ns: u64,
    /// Claimed maximum credential lifetime, in nanoseconds.
    pub max_token_ttl_ns: u64,
    /// Claimed audience restriction.
    pub aud: DelegationAudience,
    /// Unnormalized delegated role/scope claims.
    pub grants: List<DelegatedRoleGrant, 16>,
}

///
/// DelegationProof
///
/// Claimed delegation and its root evidence.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct DelegationProof {
    /// Claimed delegation certificate.
    pub cert: DelegationCert,
    /// Root evidence, requiring protected authority verification.
    pub root_proof: RootProof,
}

///
/// CertifiedProof
///
/// Bounded certified credential proof.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct CertifiedProof {
    /// Root delegation evidence for the issuer.
    pub delegation: DelegationProof,
    /// Issuer signature over the canonical claims digest and domain.
    pub issuer_signature: IcCanisterSignatureProofV1,
}

///
/// PortableProof
///
/// Claimed proof mechanism; the host's enrollment selects the verifier.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub enum PortableProof {
    /// Root-delegated certified issuer proof, transport tag zero.
    Certified(Box<CertifiedProof>),
    /// At most 64 signature bytes; strict verification later requires exactly 64.
    Ed25519(Bytes<64>),
}

///
/// PortableCredential
///
/// Opaque signed claims with an unverified proof envelope.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
#[cfg_attr(feature = "typescript", wire(fixed))]
pub struct PortableCredential {
    /// Exact received signed bytes; transport must not decode/re-encode them.
    pub claims: Bytes<65536>,
    /// Claimed proof material requiring verification against protected inputs.
    pub proof: PortableProof,
}
