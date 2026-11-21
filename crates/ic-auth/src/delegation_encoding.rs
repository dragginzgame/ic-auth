//! Bounded signing commitments for the current root-delegation profile.
//!
//! These operations establish canonical bytes only. They verify no signature,
//! issuer binding, grant authority, schema support, validity window or enrollment.
//! Certificate issue time is hashed but is absent from the root-signed leaf.

use crate::claims::{ClaimsError, validate_grants, write_grants};
use ic_auth_protocol_types::{
    DelegationCertificate, DelegationLeaf, IssuerDelegation, Principal, RootBatchHeader,
};
use sha2::{Digest, Sha256};
use std::{error::Error, fmt};

///
/// DelegationEncodingError
///
/// Bounded structural refusal; never an authentication or authorization result.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelegationEncodingError {
    /// Delegated grants violate shared role/scope shape or ordering.
    Grants(ClaimsError),
    /// Root key name exceeds the transport profile's 128-byte cap.
    KeyNameTooLong,
    /// More than 32 derivation components, or a component exceeds 256 bytes.
    DerivationPathTooLarge,
}

impl fmt::Display for DelegationEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Grants(error) => write!(f, "invalid delegation grants: {error}"),
            Self::KeyNameTooLong => f.write_str("root key name exceeds 128 bytes"),
            Self::DerivationPathTooLarge => f.write_str("root derivation path exceeds its bounds"),
        }
    }
}

impl Error for DelegationEncodingError {}

/// Encode the current certificate without normalization or implicit binding repair.
pub fn certificate_bytes(cert: &DelegationCertificate) -> Result<Vec<u8>, DelegationEncodingError> {
    validate_grants(&cert.delegation.grants).map_err(DelegationEncodingError::Grants)?;
    let mut bytes = Vec::new();
    write_certificate(cert, &mut |part| bytes.extend_from_slice(part));
    Ok(bytes)
}

/// Hash the exact certificate, including its root-unsigned issuance metadata.
pub fn certificate_hash(cert: &DelegationCertificate) -> Result<[u8; 32], DelegationEncodingError> {
    validate_grants(&cert.delegation.grants).map_err(DelegationEncodingError::Grants)?;
    let mut hash = Sha256::new();
    write_certificate(cert, &mut |part| hash.update(part));
    Ok(hash.finalize().into())
}

/// Hash the IC canister-signature issuer/algorithm/seed commitment.
#[must_use]
pub fn issuer_binding_hash(issuer: Principal, seed_hash: [u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"canic-issuer-proof-binding-v1");
    write_principal(issuer, &mut |part| hash.update(part));
    hash.update([1, 1]); // Algorithm, then binding variant: both are IC V1.
    hash.update(seed_hash);
    hash.finalize().into()
}

/// Hash one root issuer leaf with its zero prefix and length-framed domain.
pub fn delegation_leaf_hash(leaf: &DelegationLeaf) -> Result<[u8; 32], DelegationEncodingError> {
    validate_grants(&leaf.delegation.grants).map_err(DelegationEncodingError::Grants)?;
    Ok(framed_hash(
        b"\0CANIC_ROOT_DELEGATION_CHAIN_KEY_ISSUER_LEAF_V1",
        |emit| {
            let d = &leaf.delegation;
            write_principal(d.root, emit);
            write_principal(d.issuer, emit);
            emit(&leaf.proof_epoch.to_be_bytes());
            write_binding(d, emit);
            emit(&d.max_token_ttl_ns.to_be_bytes());
            write_audience_grants(d, emit);
            emit(&d.not_before_ns.to_be_bytes());
            emit(&d.expires_at_ns.to_be_bytes());
            emit(&leaf.registry_epoch.to_be_bytes());
            emit(&leaf.registry_hash);
        },
    ))
}

/// Hash the root batch's fixed ECDSA/secp256k1 header signing preimage.
pub fn batch_header_hash(header: &RootBatchHeader) -> Result<[u8; 32], DelegationEncodingError> {
    if header.key_name.len() > 128 {
        return Err(DelegationEncodingError::KeyNameTooLong);
    }
    Ok(framed_hash(
        b"CANIC_ROOT_DELEGATION_CHAIN_KEY_BATCH_V1",
        |emit| {
            emit(&header.schema_version.to_be_bytes());
            write_principal(header.root, emit);
            emit(&header.batch_id);
            emit(&header.proof_epoch.to_be_bytes());
            emit(&header.registry_epoch.to_be_bytes());
            emit(&header.registry_hash);
            emit(&header.tree_root);
            emit(&header.not_before_ns.to_be_bytes());
            emit(&header.expires_at_ns.to_be_bytes());
            emit(&[1]); // ECDSA/secp256k1.
            write_bytes(header.key_name.as_bytes(), emit);
            emit(&header.derivation_path_hash);
            emit(&header.key_version.to_be_bytes());
        },
    ))
}

/// Hash an ordered bounded path; empty components and an empty path are retained.
pub fn derivation_path_hash(path: &[Vec<u8>]) -> Result<[u8; 32], DelegationEncodingError> {
    if path.len() > 32 || path.iter().any(|part| part.len() > 256) {
        return Err(DelegationEncodingError::DerivationPathTooLarge);
    }
    let mut hash = Sha256::new();
    hash.update(b"CANIC_ROOT_DELEGATION_CHAIN_KEY_DERIVATION_PATH_V1");
    hash.update((path.len() as u32).to_be_bytes());
    for part in path {
        write_bytes(part, &mut |part| hash.update(part));
    }
    Ok(hash.finalize().into())
}

fn write_certificate(cert: &DelegationCertificate, emit: &mut dyn FnMut(&[u8])) {
    let d = &cert.delegation;
    emit(b"CANIC-AUTH\0\x01");
    write_principal(d.root, emit);
    write_principal(d.issuer, emit);
    write_binding(d, emit);
    emit(&cert.issued_at_ns.to_be_bytes());
    emit(&d.not_before_ns.to_be_bytes());
    emit(&d.expires_at_ns.to_be_bytes());
    emit(&d.max_token_ttl_ns.to_be_bytes());
    write_audience_grants(d, emit);
}

fn write_binding(d: &IssuerDelegation, emit: &mut dyn FnMut(&[u8])) {
    emit(&[1]);
    emit(&d.binding_hash);
    emit(&[1]);
    emit(&d.seed_hash);
}

fn write_audience_grants(d: &IssuerDelegation, emit: &mut dyn FnMut(&[u8])) {
    emit(&[1]); // Fleet audience.
    emit(d.audience.canonical_network_id.as_bytes());
    emit(d.audience.fleet_id.as_bytes());
    write_grants(&d.grants, emit);
}

fn write_principal(value: Principal, emit: &mut dyn FnMut(&[u8])) {
    write_bytes(value.as_slice(), emit);
}

fn write_bytes(value: &[u8], emit: &mut dyn FnMut(&[u8])) {
    // Every caller admits field bounds before this private traversal.
    emit(&(value.len() as u32).to_be_bytes());
    emit(value);
}

fn framed_hash(domain: &[u8], write: impl Fn(&mut dyn FnMut(&[u8]))) -> [u8; 32] {
    // At most 16 roles x 32 scopes x 64-byte labels, or a 128-byte key name;
    // both admitted layouts fit comfortably within u32 on every supported target.
    let mut length = 0u32;
    write(&mut |part| length += part.len() as u32);
    let mut hash = Sha256::new();
    hash.update(domain);
    hash.update(length.to_be_bytes());
    write(&mut |part| hash.update(part));
    hash.finalize().into()
}
