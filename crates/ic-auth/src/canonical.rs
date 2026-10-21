//! Canonical signed bytes for Root delegations, proofs and role identities.
//! Domains and tags are protocol identities, not package branding.
//! This module does not verify signatures, authority, validity or caller binding.

use ic_auth_protocol_types::*;
use sha2::{Digest, Sha256};
use thiserror::Error;

const DOMAIN_SEPARATOR: &[u8] = b"CANIC-AUTH\0";
const ISSUER_PROOF_BINDING_HASH_DOMAIN: &[u8] = b"canic-issuer-proof-binding-v1";
const CHAIN_KEY_BATCH_HEADER_DOMAIN: &[u8] = b"CANIC_ROOT_DELEGATION_CHAIN_KEY_BATCH_V1";
const CHAIN_KEY_DELEGATION_CERT_DOMAIN: &[u8] = b"CANIC_ROOT_DELEGATION_CHAIN_KEY_ISSUER_LEAF_V1";
const CHAIN_KEY_DERIVATION_PATH_DOMAIN: &[u8] =
    b"CANIC_ROOT_DELEGATION_CHAIN_KEY_DERIVATION_PATH_V1";

// Domain byte assigned to one delegated-auth canonical payload family.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CanonicalDomain {
    DelegationCert = 1,
    DelegationProof = 3,
    RoleHash = 4,
}

///
/// CanonicalAuthError
///
/// Typed failure surface for delegated auth canonicalization.
///

#[derive(Debug, Eq, Error, PartialEq)]
pub enum CanonicalAuthError {
    #[error("canonical vector length exceeds u32")]
    LengthOverflow,
    #[error("delegated auth scope is empty")]
    EmptyScope,
    #[error("delegated auth scope contains invalid characters: {scope}")]
    InvalidScope { scope: String },
    #[error("delegated auth scopes must be strictly sorted and unique")]
    NonCanonicalScopes,
    #[error("delegated auth role grants must be strictly sorted and unique")]
    NonCanonicalRoles,
}

/// Hash the canonical delegation certificate; this does not authenticate it.
pub fn cert_hash(cert: &DelegationCert) -> Result<[u8; 32], CanonicalAuthError> {
    Ok(hash_bytes(&cert_bytes(cert)?))
}

/// Hash a certificate together with its complete root proof.
pub fn proof_hash(proof: &DelegationProof) -> Result<[u8; 32], CanonicalAuthError> {
    Ok(hash_bytes(&proof_bytes(proof)?))
}

/// Hash a chain-key batch header under its existing signed domain.
pub fn chain_key_batch_header_hash(
    header: &ChainKeyBatchHeaderV1,
) -> Result<[u8; 32], CanonicalAuthError> {
    let mut payload = Vec::with_capacity(256);
    encode_chain_key_batch_header(&mut payload, header)?;
    hash_chain_key_header_payload(&payload)
}

/// Hash an issuer leaf under the existing chain-key Merkle leaf domain.
pub fn chain_key_delegation_cert_hash(
    cert: &ChainKeyDelegationCertV1,
) -> Result<[u8; 32], CanonicalAuthError> {
    let mut payload = Vec::with_capacity(256);
    encode_chain_key_delegation_cert(&mut payload, cert)?;
    hash_chain_key_leaf_payload(&payload)
}

/// Hash the exact ordered derivation path under its existing Canic domain.
/// This binds path bytes; it does not derive a key or establish its authority.
pub fn chain_key_derivation_path_hash(
    derivation_path: &[Vec<u8>],
) -> Result<[u8; 32], CanonicalAuthError> {
    let mut out = CHAIN_KEY_DERIVATION_PATH_DOMAIN.to_vec();
    encode_chain_key_derivation_path(&mut out, derivation_path)?;
    Ok(hash_bytes(&out))
}

/// Bind an issuer principal, proof algorithm and seed hash.
pub fn issuer_proof_binding_hash(
    issuer_pid: Principal,
    issuer_proof_alg: IssuerProofAlgorithm,
    issuer_proof_binding: IssuerProofBinding,
) -> Result<[u8; 32], CanonicalAuthError> {
    let mut out = Vec::with_capacity(128);
    out.extend_from_slice(ISSUER_PROOF_BINDING_HASH_DOMAIN);
    encode_principal(&mut out, issuer_pid)?;
    encode_issuer_proof_algorithm(&mut out, issuer_proof_alg);
    encode_issuer_proof_binding(&mut out, issuer_proof_binding);
    Ok(hash_bytes(&out))
}

/// Hash the exact validated role label under the existing role domain.
pub fn role_hash(role: &AuthRole) -> Result<[u8; 32], CanonicalAuthError> {
    let mut out = domain_bytes(CanonicalDomain::RoleHash);
    encode_string(&mut out, role.as_str())?;
    Ok(hash_bytes(&out))
}

/// Encode a certificate without changing grant order or normalizing labels.
pub fn cert_bytes(cert: &DelegationCert) -> Result<Vec<u8>, CanonicalAuthError> {
    let mut out = Vec::with_capacity(128);
    encode_cert(&mut out, cert)?;
    Ok(out)
}

fn encode_cert(out: &mut Vec<u8>, cert: &DelegationCert) -> Result<(), CanonicalAuthError> {
    out.extend_from_slice(DOMAIN_SEPARATOR);
    out.push(CanonicalDomain::DelegationCert as u8);

    encode_principal(out, cert.root_pid)?;
    encode_principal(out, cert.issuer_pid)?;
    encode_issuer_proof_algorithm(out, cert.issuer_proof_alg);
    encode_fixed_32(out, cert.issuer_proof_binding_hash);
    encode_issuer_proof_binding(out, cert.issuer_proof_binding);
    encode_u64(out, cert.issued_at_ns);
    encode_u64(out, cert.not_before_ns);
    encode_u64(out, cert.expires_at_ns);
    encode_u64(out, cert.max_token_ttl_ns);
    encode_audience(out, &cert.aud);
    encode_role_grants(out, &cert.grants)?;

    Ok(())
}

fn proof_bytes(proof: &DelegationProof) -> Result<Vec<u8>, CanonicalAuthError> {
    let mut out = domain_bytes(CanonicalDomain::DelegationProof);

    encode_cert(&mut out, &proof.cert)?;
    encode_root_proof(&mut out, &proof.root_proof)?;

    Ok(out)
}

fn domain_bytes(domain: CanonicalDomain) -> Vec<u8> {
    let mut out = Vec::with_capacity(128);
    out.extend_from_slice(DOMAIN_SEPARATOR);
    out.push(domain as u8);
    out
}

fn hash_bytes(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn encode_issuer_proof_algorithm(out: &mut Vec<u8>, alg: IssuerProofAlgorithm) {
    let tag = match alg {
        IssuerProofAlgorithm::IcCanisterSignatureV1 => 1,
    };
    out.push(tag);
}

fn encode_audience(out: &mut Vec<u8>, audience: &DelegationAudience) {
    match audience {
        DelegationAudience::Fleet(fleet) => {
            out.push(1);
            encode_fleet_key(out, *fleet);
        }
    }
}

fn encode_fleet_key(out: &mut Vec<u8>, fleet: AudienceId) {
    encode_fixed_32(out, *fleet.canonical_network_id.as_bytes());
    encode_fixed_32(out, *fleet.fleet_id.as_bytes());
}

pub(crate) fn encode_role_grants(
    out: &mut Vec<u8>,
    grants: &[DelegatedRoleGrant],
) -> Result<(), CanonicalAuthError> {
    encode_len(out, grants.len())?;
    let mut previous = None;
    for grant in grants {
        let current = grant.target.as_str().as_bytes();
        if previous.is_some_and(|previous| previous >= current) {
            return Err(CanonicalAuthError::NonCanonicalRoles);
        }
        previous = Some(current);
        encode_role(out, &grant.target)?;
        encode_scopes(out, &grant.scopes)?;
    }
    Ok(())
}

fn hash_chain_key_header_payload(payload: &[u8]) -> Result<[u8; 32], CanonicalAuthError> {
    let len = u32::try_from(payload.len()).map_err(|_| CanonicalAuthError::LengthOverflow)?;
    let mut hash = Sha256::new();
    hash.update(CHAIN_KEY_BATCH_HEADER_DOMAIN);
    hash.update(len.to_be_bytes());
    hash.update(payload);
    Ok(hash.finalize().into())
}

fn hash_chain_key_leaf_payload(payload: &[u8]) -> Result<[u8; 32], CanonicalAuthError> {
    let len = u32::try_from(payload.len()).map_err(|_| CanonicalAuthError::LengthOverflow)?;
    let mut hash = Sha256::new();
    hash.update([0]);
    hash.update(CHAIN_KEY_DELEGATION_CERT_DOMAIN);
    hash.update(len.to_be_bytes());
    hash.update(payload);
    Ok(hash.finalize().into())
}

fn encode_chain_key_batch_header(
    out: &mut Vec<u8>,
    header: &ChainKeyBatchHeaderV1,
) -> Result<(), CanonicalAuthError> {
    encode_u16(out, header.schema_version);
    encode_principal(out, header.root_canister_id)?;
    encode_fixed_32(out, header.batch_id);
    encode_u64(out, header.proof_epoch);
    encode_u64(out, header.registry_epoch);
    encode_fixed_32(out, header.registry_hash);
    encode_fixed_32(out, header.tree_root);
    encode_u64(out, header.not_before_ns);
    encode_u64(out, header.expires_at_ns);
    encode_chain_key_algorithm(out, header.algorithm);
    encode_chain_key_key_id(out, &header.key_id)?;
    encode_fixed_32(out, header.derivation_path_hash);
    encode_u64(out, header.key_version);
    Ok(())
}

fn encode_chain_key_delegation_cert(
    out: &mut Vec<u8>,
    cert: &ChainKeyDelegationCertV1,
) -> Result<(), CanonicalAuthError> {
    encode_principal(out, cert.root_canister_id)?;
    encode_principal(out, cert.issuer_canister_id)?;
    encode_u64(out, cert.proof_epoch);
    encode_issuer_proof_algorithm(out, cert.issuer_proof_algorithm);
    encode_fixed_32(out, cert.issuer_proof_binding_hash);
    encode_issuer_proof_binding(out, cert.issuer_proof_binding);
    encode_u64(out, cert.max_token_ttl_ns);
    encode_audience(out, &cert.audience);
    encode_role_grants(out, &cert.grants)?;
    encode_u64(out, cert.not_before_ns);
    encode_u64(out, cert.expires_at_ns);
    encode_u64(out, cert.registry_epoch);
    encode_fixed_32(out, cert.registry_hash);
    Ok(())
}

fn encode_root_proof(out: &mut Vec<u8>, proof: &RootProof) -> Result<(), CanonicalAuthError> {
    match proof {
        RootProof::IcChainKeyBatchSignatureV1(proof) => {
            out.push(2);
            encode_chain_key_proof(out, proof)?;
        }
    }
    Ok(())
}

fn encode_chain_key_proof(
    out: &mut Vec<u8>,
    proof: &IcChainKeyBatchSignatureProofV1,
) -> Result<(), CanonicalAuthError> {
    encode_chain_key_batch_header(out, &proof.header)?;
    encode_chain_key_delegation_cert(out, &proof.delegation_cert)?;
    encode_chain_key_witness(out, &proof.issuer_witness)?;
    encode_chain_key_signature(out, &proof.signature)?;
    Ok(())
}

fn encode_chain_key_witness(
    out: &mut Vec<u8>,
    witness: &ChainKeyBatchWitnessV1,
) -> Result<(), CanonicalAuthError> {
    encode_len(out, witness.steps.len())?;
    for step in &witness.steps {
        match step {
            ChainKeyBatchWitnessStepV1::LeftSibling(hash) => {
                out.push(1);
                encode_fixed_32(out, *hash);
            }
            ChainKeyBatchWitnessStepV1::RightSibling(hash) => {
                out.push(2);
                encode_fixed_32(out, *hash);
            }
        }
    }
    Ok(())
}

fn encode_chain_key_signature(
    out: &mut Vec<u8>,
    signature: &ChainKeyRootSignatureV1,
) -> Result<(), CanonicalAuthError> {
    encode_chain_key_algorithm(out, signature.algorithm);
    encode_chain_key_key_id(out, &signature.key_id)?;
    encode_chain_key_derivation_path(out, &signature.derivation_path)?;
    encode_bytes(out, &signature.public_key)?;
    encode_bytes(out, &signature.signature)?;
    Ok(())
}

fn encode_chain_key_derivation_path(
    out: &mut Vec<u8>,
    derivation_path: &[Vec<u8>],
) -> Result<(), CanonicalAuthError> {
    encode_len(out, derivation_path.len())?;
    for path_component in derivation_path {
        encode_bytes(out, path_component)?;
    }
    Ok(())
}

fn encode_chain_key_algorithm(out: &mut Vec<u8>, algorithm: ChainKeyAlgorithm) {
    let tag = match algorithm {
        ChainKeyAlgorithm::EcdsaSecp256k1 => 1,
    };
    out.push(tag);
}

fn encode_chain_key_key_id(
    out: &mut Vec<u8>,
    key_id: &ChainKeyKeyId,
) -> Result<(), CanonicalAuthError> {
    encode_string(out, &key_id.name)?;
    Ok(())
}

fn encode_issuer_proof_binding(out: &mut Vec<u8>, binding: IssuerProofBinding) {
    match binding {
        IssuerProofBinding::IcCanisterSignatureV1 { seed_hash } => {
            out.push(1);
            encode_fixed_32(out, seed_hash);
        }
    }
}

fn encode_role(out: &mut Vec<u8>, role: &AuthRole) -> Result<(), CanonicalAuthError> {
    encode_bytes(out, role.as_str().as_bytes())?;
    Ok(())
}

fn encode_scopes(out: &mut Vec<u8>, scopes: &[String]) -> Result<(), CanonicalAuthError> {
    validate_scopes(scopes)?;
    encode_len(out, scopes.len())?;
    for scope in scopes {
        encode_bytes(out, scope.as_bytes())?;
    }
    Ok(())
}

/// Validate label grammar and strictly increasing, unique UTF-8 byte order.
///
/// An empty list is valid. Cardinality, normalization and application authority
/// remain caller-owned; use the bounded grant validator for nonempty grants.
pub fn validate_scopes(scopes: &[String]) -> Result<(), CanonicalAuthError> {
    let mut previous = None;
    for scope in scopes {
        validate_scope_label(scope)?;
        let current = scope.as_bytes();
        if previous.is_some_and(|previous| previous >= current) {
            return Err(CanonicalAuthError::NonCanonicalScopes);
        }
        previous = Some(current);
    }

    Ok(())
}

/// A scope label's grammar failure. Offsets and lengths count UTF-8 bytes.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ScopeLabelError {
    #[error("scope is empty")]
    Empty,
    #[error("scope has {bytes} bytes; maximum is {max}")]
    TooLong { bytes: usize, max: usize },
    #[error("scope contains an empty segment")]
    EmptySegment,
    #[error("invalid scope segment start at byte {offset}")]
    InvalidSegmentStart { offset: usize },
    #[error("invalid scope byte at offset {offset}")]
    InvalidByte { offset: usize },
}

/// Check the 64-byte, lowercase ASCII/digit, colon-separated scope grammar.
///
/// Every segment starts with a lowercase letter or digit; subsequent bytes may
/// also be `_` or `-`. Length is checked first, then bytes from left to right.
/// This allocation-free check supports static construction and grants no authority.
pub const fn validate_scope_label_detailed(scope: &str) -> Result<(), ScopeLabelError> {
    let bytes = scope.as_bytes();
    if bytes.is_empty() {
        return Err(ScopeLabelError::Empty);
    }
    if bytes.len() > crate::limits::MAX_LABEL_BYTES {
        return Err(ScopeLabelError::TooLong {
            bytes: bytes.len(),
            max: crate::limits::MAX_LABEL_BYTES,
        });
    }
    let mut segment_start = true;
    let mut offset = 0;
    while offset < bytes.len() {
        let byte = bytes[offset];
        if byte == b':' {
            if segment_start {
                return Err(ScopeLabelError::EmptySegment);
            }
            segment_start = true;
        } else if segment_start {
            if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() {
                return Err(ScopeLabelError::InvalidSegmentStart { offset });
            }
            segment_start = false;
        } else if !byte.is_ascii_lowercase()
            && !byte.is_ascii_digit()
            && byte != b'_'
            && byte != b'-'
        {
            return Err(ScopeLabelError::InvalidByte { offset });
        }
        offset += 1;
    }
    if segment_start {
        return Err(ScopeLabelError::EmptySegment);
    }
    Ok(())
}

/// Check the existing lowercase, colon-separated application-scope grammar.
pub fn validate_scope_label(scope: &str) -> Result<(), CanonicalAuthError> {
    validate_scope_label_detailed(scope).map_err(|error| match error {
        ScopeLabelError::Empty => CanonicalAuthError::EmptyScope,
        _ => CanonicalAuthError::InvalidScope {
            scope: scope.to_string(),
        },
    })
}

fn encode_string(out: &mut Vec<u8>, value: &str) -> Result<(), CanonicalAuthError> {
    encode_bytes(out, value.as_bytes())?;
    Ok(())
}

pub(crate) fn encode_principal(
    out: &mut Vec<u8>,
    principal: Principal,
) -> Result<(), CanonicalAuthError> {
    encode_bytes(out, principal.as_slice())?;
    Ok(())
}

fn encode_bytes(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), CanonicalAuthError> {
    encode_len(out, bytes.len())?;
    out.extend_from_slice(bytes);
    Ok(())
}

fn encode_fixed_32(out: &mut Vec<u8>, bytes: [u8; 32]) {
    out.extend_from_slice(&bytes);
}

fn encode_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn encode_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn encode_len(out: &mut Vec<u8>, len: usize) -> Result<(), CanonicalAuthError> {
    let len = u32::try_from(len).map_err(|_| CanonicalAuthError::LengthOverflow)?;
    out.extend_from_slice(&len.to_be_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_key_payload_hashes_preserve_framing() {
        for length in [0, 1, 255, 256, 4096] {
            let payload: Vec<u8> = (0..length).map(|i| (i % 256) as u8).collect();
            let len = (length as u32).to_be_bytes();
            let header = [CHAIN_KEY_BATCH_HEADER_DOMAIN, &len, &payload].concat();
            let leaf = [&[0], CHAIN_KEY_DELEGATION_CERT_DOMAIN, &len, &payload].concat();
            assert_eq!(
                hash_chain_key_header_payload(&payload).unwrap(),
                <[u8; 32]>::from(Sha256::digest(header)),
            );
            assert_eq!(
                hash_chain_key_leaf_payload(&payload).unwrap(),
                <[u8; 32]>::from(Sha256::digest(leaf)),
            );
        }
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn length_overflow_is_fallible_before_writing() {
        let mut out = vec![7];
        assert_eq!(
            encode_len(&mut out, u32::MAX as usize + 1),
            Err(CanonicalAuthError::LengthOverflow)
        );
        assert_eq!(out, [7]);
    }
}
