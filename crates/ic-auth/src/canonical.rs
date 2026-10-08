//! Canonical signed bytes for the existing Canic application-token protocol.
//! Domains and tags are protocol identities, not package branding.
//! This module does not verify signatures, authority, validity or caller binding.

use ic_auth_types::*;
use sha2::{Digest, Sha256};
use thiserror::Error;

const DOMAIN_SEPARATOR: &[u8] = b"CANIC-AUTH\0";
const ISSUER_PROOF_BINDING_HASH_DOMAIN: &[u8] = b"canic-issuer-proof-binding-v1";
const CHAIN_KEY_BATCH_HEADER_DOMAIN: &[u8] = b"CANIC_ROOT_DELEGATION_CHAIN_KEY_BATCH_V1";
const CHAIN_KEY_DELEGATION_CERT_DOMAIN: &[u8] = b"CANIC_ROOT_DELEGATION_CHAIN_KEY_ISSUER_LEAF_V1";
/// Largest extension admitted by the existing application-token protocol.
pub const MAX_TOKEN_EXT_BYTES: usize = 4096;

// Domain byte assigned to one delegated-auth canonical payload family.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CanonicalDomain {
    DelegationCert = 1,
    DelegatedTokenClaims = 2,
    DelegationProof = 3,
    RoleHash = 4,
    IssuerProof = 6,
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
    #[error("delegated auth token ext is {len} bytes and exceeds max {max} bytes")]
    TokenExtTooLarge { len: usize, max: usize },
}

/// Hash the canonical delegation certificate; this does not authenticate it.
pub fn cert_hash(cert: &DelegationCert) -> Result<[u8; 32], CanonicalAuthError> {
    Ok(hash_bytes(&cert_bytes(cert)?))
}

/// Hash canonical claims, rejecting noncanonical grants and oversized extensions.
pub fn claims_hash(claims: &DelegatedTokenClaims) -> Result<[u8; 32], CanonicalAuthError> {
    Ok(hash_bytes(&claims_bytes(claims)?))
}

/// Hash a certificate together with its complete root proof.
pub fn proof_hash(proof: &DelegationProof) -> Result<[u8; 32], CanonicalAuthError> {
    Ok(hash_bytes(&proof_bytes(proof)?))
}

/// Hash issuer signature bytes together with their public key.
pub fn issuer_proof_hash(proof: &IssuerProof) -> Result<[u8; 32], CanonicalAuthError> {
    Ok(hash_bytes(&issuer_proof_bytes(proof)?))
}

/// Hash a chain-key batch header under its existing signed domain.
pub fn chain_key_batch_header_hash(
    header: &ChainKeyBatchHeaderV1,
) -> Result<[u8; 32], CanonicalAuthError> {
    hash_chain_key_header_payload(&chain_key_batch_header_bytes(header)?)
}

/// Hash an issuer leaf under the existing chain-key Merkle leaf domain.
pub fn chain_key_delegation_cert_hash(
    cert: &ChainKeyDelegationCertV1,
) -> Result<[u8; 32], CanonicalAuthError> {
    hash_chain_key_leaf_payload(&chain_key_delegation_cert_bytes(cert)?)
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
    let mut out = domain_bytes(CanonicalDomain::DelegationCert);

    encode_principal(&mut out, cert.root_pid)?;
    encode_principal(&mut out, cert.issuer_pid)?;
    encode_issuer_proof_algorithm(&mut out, cert.issuer_proof_alg);
    encode_fixed_32(&mut out, cert.issuer_proof_binding_hash);
    encode_issuer_proof_binding(&mut out, cert.issuer_proof_binding);
    encode_u64(&mut out, cert.issued_at_ns);
    encode_u64(&mut out, cert.not_before_ns);
    encode_u64(&mut out, cert.expires_at_ns);
    encode_u64(&mut out, cert.max_token_ttl_ns);
    encode_audience(&mut out, &cert.aud);
    encode_role_grants(&mut out, &cert.grants)?;

    Ok(out)
}

/// Encode claims exactly, including presenter, audience, nonce and extension presence.
pub fn claims_bytes(claims: &DelegatedTokenClaims) -> Result<Vec<u8>, CanonicalAuthError> {
    let mut out = domain_bytes(CanonicalDomain::DelegatedTokenClaims);

    encode_principal(&mut out, claims.presenter)?;
    encode_principal(&mut out, claims.subject)?;
    encode_principal(&mut out, claims.issuer_pid)?;
    encode_fixed_32(&mut out, claims.cert_hash);
    encode_u64(&mut out, claims.issued_at_ns);
    encode_u64(&mut out, claims.expires_at_ns);
    encode_audience(&mut out, &claims.aud);
    encode_role_grants(&mut out, &claims.grants)?;
    out.extend_from_slice(&claims.nonce);
    encode_token_ext(&mut out, claims.ext.as_deref())?;

    Ok(out)
}

fn proof_bytes(proof: &DelegationProof) -> Result<Vec<u8>, CanonicalAuthError> {
    let mut out = domain_bytes(CanonicalDomain::DelegationProof);

    out.extend_from_slice(&cert_bytes(&proof.cert)?);
    encode_root_proof(&mut out, &proof.root_proof)?;

    Ok(out)
}

fn issuer_proof_bytes(proof: &IssuerProof) -> Result<Vec<u8>, CanonicalAuthError> {
    let mut out = domain_bytes(CanonicalDomain::IssuerProof);
    encode_issuer_proof(&mut out, proof)?;
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

fn encode_role_grants(
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
    let mut out = Vec::with_capacity(CHAIN_KEY_BATCH_HEADER_DOMAIN.len() + 4 + payload.len());
    out.extend_from_slice(CHAIN_KEY_BATCH_HEADER_DOMAIN);
    encode_bytes(&mut out, payload)?;
    Ok(hash_bytes(&out))
}

fn hash_chain_key_leaf_payload(payload: &[u8]) -> Result<[u8; 32], CanonicalAuthError> {
    let mut out =
        Vec::with_capacity(1 + CHAIN_KEY_DELEGATION_CERT_DOMAIN.len() + 4 + payload.len());
    out.push(0);
    out.extend_from_slice(CHAIN_KEY_DELEGATION_CERT_DOMAIN);
    encode_bytes(&mut out, payload)?;
    Ok(hash_bytes(&out))
}

fn chain_key_batch_header_bytes(
    header: &ChainKeyBatchHeaderV1,
) -> Result<Vec<u8>, CanonicalAuthError> {
    let mut out = Vec::with_capacity(256);
    encode_u16(&mut out, header.schema_version);
    encode_principal(&mut out, header.root_canister_id)?;
    encode_fixed_32(&mut out, header.batch_id);
    encode_u64(&mut out, header.proof_epoch);
    encode_u64(&mut out, header.registry_epoch);
    encode_fixed_32(&mut out, header.registry_hash);
    encode_fixed_32(&mut out, header.tree_root);
    encode_u64(&mut out, header.not_before_ns);
    encode_u64(&mut out, header.expires_at_ns);
    encode_chain_key_algorithm(&mut out, header.algorithm);
    encode_chain_key_key_id(&mut out, &header.key_id)?;
    encode_fixed_32(&mut out, header.derivation_path_hash);
    encode_u64(&mut out, header.key_version);
    Ok(out)
}

fn chain_key_delegation_cert_bytes(
    cert: &ChainKeyDelegationCertV1,
) -> Result<Vec<u8>, CanonicalAuthError> {
    let mut out = Vec::with_capacity(256);
    encode_principal(&mut out, cert.root_canister_id)?;
    encode_principal(&mut out, cert.issuer_canister_id)?;
    encode_u64(&mut out, cert.proof_epoch);
    encode_issuer_proof_algorithm(&mut out, cert.issuer_proof_algorithm);
    encode_fixed_32(&mut out, cert.issuer_proof_binding_hash);
    encode_issuer_proof_binding(&mut out, cert.issuer_proof_binding);
    encode_u64(&mut out, cert.max_token_ttl_ns);
    encode_audience(&mut out, &cert.audience);
    encode_role_grants(&mut out, &cert.grants)?;
    encode_u64(&mut out, cert.not_before_ns);
    encode_u64(&mut out, cert.expires_at_ns);
    encode_u64(&mut out, cert.registry_epoch);
    encode_fixed_32(&mut out, cert.registry_hash);
    Ok(out)
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
    out.extend_from_slice(&chain_key_batch_header_bytes(&proof.header)?);
    out.extend_from_slice(&chain_key_delegation_cert_bytes(&proof.delegation_cert)?);
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

fn encode_issuer_proof(out: &mut Vec<u8>, proof: &IssuerProof) -> Result<(), CanonicalAuthError> {
    match proof {
        IssuerProof::IcCanisterSignatureV1(proof) => {
            out.push(1);
            encode_bytes(out, &proof.signature_cbor)?;
            encode_bytes(out, &proof.public_key_der)?;
        }
    }
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

fn encode_token_ext(out: &mut Vec<u8>, ext: Option<&[u8]>) -> Result<(), CanonicalAuthError> {
    match ext {
        Some(ext) => {
            if ext.len() > MAX_TOKEN_EXT_BYTES {
                return Err(CanonicalAuthError::TokenExtTooLarge {
                    len: ext.len(),
                    max: MAX_TOKEN_EXT_BYTES,
                });
            }
            out.push(1);
            encode_bytes(out, ext)?;
        }
        None => out.push(0),
    }
    Ok(())
}

fn encode_role(out: &mut Vec<u8>, role: &AuthRole) -> Result<(), CanonicalAuthError> {
    encode_bytes(out, role.as_str().as_bytes())?;
    Ok(())
}

fn encode_scopes(out: &mut Vec<u8>, scopes: &[String]) -> Result<(), CanonicalAuthError> {
    let mut previous = None;
    for scope in scopes {
        validate_scope_label(scope)?;
        let current = scope.as_bytes();
        if previous.is_some_and(|previous| previous >= current) {
            return Err(CanonicalAuthError::NonCanonicalScopes);
        }
        previous = Some(current);
    }

    encode_len(out, scopes.len())?;
    for scope in scopes {
        encode_bytes(out, scope.as_bytes())?;
    }

    Ok(())
}

/// Check the existing lowercase, colon-separated application-scope grammar.
pub fn validate_scope_label(scope: &str) -> Result<(), CanonicalAuthError> {
    if scope.is_empty() {
        return Err(CanonicalAuthError::EmptyScope);
    }
    if !is_valid_scope(scope) {
        return Err(CanonicalAuthError::InvalidScope {
            scope: scope.to_string(),
        });
    }
    Ok(())
}

fn encode_string(out: &mut Vec<u8>, value: &str) -> Result<(), CanonicalAuthError> {
    encode_bytes(out, value.as_bytes())?;
    Ok(())
}

fn encode_principal(out: &mut Vec<u8>, principal: Principal) -> Result<(), CanonicalAuthError> {
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

// Existing application-scope grammar, including its 64-byte bound.
fn is_valid_scope(scope: &str) -> bool {
    if scope.is_empty() || scope.len() > 64 {
        return false;
    }
    scope.split(':').all(|segment| {
        let mut bytes = segment.bytes();
        bytes
            .next()
            .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            && bytes
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
