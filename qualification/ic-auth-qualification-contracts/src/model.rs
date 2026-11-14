//! Controller input is a bounded transport projection, never trusted enrollment.
use ic_auth_wire::model::{AudienceId, DelegatedRoleGrant};
use ic_wire::{Bytes, List, Text};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
/// Controller-submitted receiver enrollment for the disposable qualification host.
pub struct ReceiverConfig {
    /// Intended network/fleet audience installed by the controller.
    pub audience: AudienceId,
    /// Issuer Principal selected by controller enrollment.
    pub issuer: ic_auth_protocol_types::Principal,
    /// Signature mechanism and key material to install as protected fixture authority.
    pub authority: ReceiverAuthority,
    /// Receiver role used to narrow credential grants.
    pub role: Text<64>,
    /// Bounded grant ceiling installed by the controller.
    pub grants: List<DelegatedRoleGrant, 16>,
    /// Inclusive authority validity start in nanoseconds.
    pub valid_from_ns: u64,
    /// Exclusive issuer authority deadline in nanoseconds.
    pub valid_until_ns: u64,
    /// Exclusive fixture enrollment deadline in nanoseconds.
    pub enrollment_until_ns: u64,
    /// Maximum accumulated verification work units.
    pub verification_burst: u32,
    /// Trusted nanoseconds required to replenish one verification unit.
    pub verification_refill_ns: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
/// Controller-selected proof mechanism; credential submissions cannot select enrollment.
pub enum ReceiverAuthority {
    /// Raw issuer Ed25519 key, admitted before installation.
    Ed25519([u8; 32]),
    /// Root and network enrollment for certified proof verification.
    Certified(CertifiedAuthority),
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
/// Independent issuer, root and IC network bindings for certified fixture admission.
pub struct CertifiedAuthority {
    /// Expected issuer key-binding identity included in canonical claims.
    pub binding: [u8; 32],
    /// Root chain-key authority to verify the issuer delegation.
    pub root: RootEnrollment,
    /// Raw IC network BLS trust anchor; exact length is checked on admission.
    pub network_key: Bytes<96>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
/// Bounded controller projection of Auth's protected root-key policy.
pub struct RootEnrollment {
    /// Expected root authority canister.
    pub root_canister_id: ic_auth_protocol_types::Principal,
    /// Enrolled root signature algorithm.
    pub algorithm: ic_auth_wire::model::ChainKeyAlgorithm,
    /// Network chain-key name, independently supplied by the controller.
    pub key_name: Text<128>,
    /// Canonical hash of the enrolled chain-key derivation path.
    pub derivation_path_hash: [u8; 32],
    /// SEC1 root public key, validated before use.
    pub public_key: Bytes<65>,
    /// Exact enrolled key generation.
    pub key_version: u64,
    /// Inclusive lower bound for accepted root key versions.
    pub min_accepted_key_version: u64,
    /// Inclusive lower bound for accepted proof generations.
    pub min_accepted_proof_epoch: u64,
    /// Inclusive lower bound for accepted registry generations.
    pub min_accepted_registry_epoch: u64,
    /// Inclusive authority validity start in nanoseconds.
    pub valid_from_ns: u64,
    /// Exclusive root-key acceptance deadline in nanoseconds.
    pub accept_until_ns: u64,
    /// Maximum root batch lifetime accepted by this fixture.
    pub max_revocation_latency_ns: u64,
}
#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
/// Installed samples separating record decoding from live authorization.
pub struct SessionProfile {
    /// Encoded session record length used by the probe.
    pub record_bytes: u64,
    /// Instructions spent decoding the selected record or certificate input.
    pub decode_instructions: u64,
    /// Instructions spent on the sampled authorization call.
    pub authorization_instructions: u64,
}
#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
/// Installed admission response paired with its sampled instruction cost.
pub struct AdmissionProfile {
    /// Original bounded Wire admission reply, preserving success or refusal.
    pub reply: Bytes<4096>,
    /// Measured admission instructions, not a worst-case cost guarantee.
    pub instructions: u64,
}

/// Independent component probes after complete verification of the same proof.
/// Component values overlap and are not an additive trace of the full verifier.
#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CertifiedProfile {
    /// Instructions for complete credential verification.
    pub credential_instructions: u64,
    /// Independent root-delegation verification sample.
    pub root_delegation_instructions: u64,
    /// Independent issuer canister-signature verification sample.
    pub issuer_instructions: u64,
    /// Instructions spent decoding the selected record or certificate input.
    pub decode_instructions: u64,
    /// Independent certified-tree hashing sample.
    pub tree_hash_instructions: u64,
    /// BLS verification sample for the signing certificate.
    pub signing_bls_instructions: u64,
    /// BLS verification sample for the subnet delegation certificate.
    pub subnet_bls_instructions: u64,
    /// Encoded issuer signature envelope size.
    pub signature_bytes: u64,
    /// Signing certificate input size.
    pub certificate_bytes: u64,
    /// Subnet certificate input size, when delegation is present.
    pub subnet_certificate_bytes: u64,
}

/// Stage of a refused component probe. Complete verification remains mandatory;
/// these codes are diagnostics, never partial authentication results.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CertifiedProfileFailure {
    /// Credential transport could not be decoded.
    Credential,
    /// No installed receiver is available for the probe.
    ReceiverUnavailable,
    /// The receiver enrollment does not select certified verification.
    CertifiedReceiverRequired,
    /// The credential does not carry a certified proof.
    CertifiedCredentialRequired,
    /// Complete credential verification refused before component profiling.
    Verification(ic_auth_wire::model::AdmissionRefusal),
    /// Root-delegation component verification failed.
    RootDelegation,
    /// The signature domain/message could not be constructed.
    MessageFraming,
    /// Issuer signature component verification failed.
    IssuerSignature,
    /// The certificate envelope could not be decoded.
    CertificateDecode,
    /// Subnet delegation BLS verification failed.
    SubnetBls,
    /// The verified subnet certificate did not expose the required key.
    SubnetKeyMissing,
    /// The subnet key could not be decoded into the required BLS key.
    SubnetKeyDecode,
    /// Signing-certificate BLS verification failed.
    SigningBls,
}
