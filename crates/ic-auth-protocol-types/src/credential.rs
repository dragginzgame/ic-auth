//! Owned, untrusted credential material shared by transport and verification.
//!
//! These records carry no codec, signature, enrollment or session behavior.
//! Producers may construct invalid values; the selected verifier owns admission.

use crate::{DelegationCertificate, DelegationLeaf, MerkleStep, RootBatchHeader};

///
/// RootSignatureMaterial
///
/// Submitted ECDSA/secp256k1 signature metadata, never a trusted key selection.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootSignatureMaterial {
    /// Submitted key lookup hint.
    pub key_name: String,
    /// Ordered derivation components, without normalization.
    pub derivation_path: Vec<Vec<u8>>,
    /// Submitted SEC1 key bytes.
    pub public_key: Vec<u8>,
    /// Submitted compact signature, with no implied length or scalar validity.
    pub signature: Vec<u8>,
}

///
/// RootDelegation
///
/// Owned root authorization material; duplicated certificate/leaf fields remain distinct.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootDelegation {
    /// Certificate committed by the claims' key-binding identity.
    pub certificate: DelegationCertificate,
    /// Submitted root batch header.
    pub header: RootBatchHeader,
    /// Submitted issuer authorization leaf, requiring certificate agreement.
    pub leaf: DelegationLeaf,
    /// Ordered sibling steps.
    pub witness: Vec<MerkleStep>,
    /// Submitted root signature material.
    pub signature: RootSignatureMaterial,
}

///
/// CertifiedProofMaterial
///
/// Owned certified proof; successful construction authenticates nothing.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertifiedProofMaterial {
    /// Submitted root authorization.
    pub delegation: RootDelegation,
    /// Opaque IC signature/certificate/witness bytes.
    pub issuer_signature_cbor: Vec<u8>,
    /// Submitted issuer canister-signature DER key.
    pub issuer_public_key_der: Vec<u8>,
}

///
/// CredentialProof
///
/// Claimed proof variant, checked against independently protected enrollment.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CredentialProof {
    /// Root-delegated IC canister signature.
    Certified(Box<CertifiedProofMaterial>),
    /// Opaque Ed25519 signature bytes requiring strict verification.
    Ed25519(Vec<u8>),
}

///
/// Credential
///
/// Untrusted credential retaining the exact received signed claims bytes.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Credential {
    /// Canonical claims candidate; never decode/re-encode during projection.
    pub claims: Vec<u8>,
    /// Submitted proof; the host selects the verifier from protected enrollment.
    pub proof: CredentialProof,
}
