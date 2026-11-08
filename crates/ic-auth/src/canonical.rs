//! Canonical signed bytes for Root delegations, proofs and role identities.
//! Domains and tags are protocol identities, not package branding.
//! This module does not verify signatures, authority, validity or caller binding.

use ic_auth_protocol_types::*;
use sha2::{Digest, Sha256};
use thiserror::Error;

// One field encoder feeds either exact byte output or the maintained SHA-256.
pub(crate) type Sink<'a> = dyn FnMut(&[u8]) + 'a;

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
    #[error(transparent)]
    Scope(#[from] ScopeLabelError),
    #[error("delegated auth scopes must be strictly sorted and unique")]
    NonCanonicalScopes,
    #[error("delegated auth role grants must be strictly sorted and unique")]
    NonCanonicalRoles,
}

/// Hash the canonical delegation certificate; this does not authenticate it.
pub fn cert_hash(cert: &DelegationCert) -> Result<[u8; 32], CanonicalAuthError> {
    hash_encoded(&[], false, &|out| encode_cert(out, cert))
}

/// Hash a certificate together with its complete root proof.
pub fn proof_hash(proof: &DelegationProof) -> Result<[u8; 32], CanonicalAuthError> {
    hash_encoded(&[], false, &|out| {
        out(DOMAIN_SEPARATOR);
        out(&[CanonicalDomain::DelegationProof as u8]);
        encode_cert(out, &proof.cert)?;
        encode_root_proof(out, &proof.root_proof)
    })
}

/// Hash a chain-key batch header under its existing signed domain.
pub fn chain_key_batch_header_hash(
    header: &ChainKeyBatchHeaderV1,
) -> Result<[u8; 32], CanonicalAuthError> {
    hash_encoded(&[CHAIN_KEY_BATCH_HEADER_DOMAIN], true, &|out| {
        encode_chain_key_batch_header(out, header)
    })
}

/// Hash an issuer leaf under the existing chain-key Merkle leaf domain.
pub fn chain_key_delegation_cert_hash(
    cert: &ChainKeyDelegationCertV1,
) -> Result<[u8; 32], CanonicalAuthError> {
    hash_encoded(&[&[0], CHAIN_KEY_DELEGATION_CERT_DOMAIN], true, &|out| {
        encode_chain_key_delegation_cert(out, cert)
    })
}

/// Hash the exact ordered derivation path under its existing Canic domain.
/// This binds path bytes; it does not derive a key or establish its authority.
pub fn chain_key_derivation_path_hash(
    derivation_path: &[Vec<u8>],
) -> Result<[u8; 32], CanonicalAuthError> {
    hash_encoded(&[CHAIN_KEY_DERIVATION_PATH_DOMAIN], false, &|out| {
        encode_chain_key_derivation_path(out, derivation_path)
    })
}

/// Bind an issuer principal, proof algorithm and seed hash.
pub fn issuer_proof_binding_hash(
    issuer_pid: Principal,
    issuer_proof_alg: IssuerProofAlgorithm,
    issuer_proof_binding: IssuerProofBinding,
) -> Result<[u8; 32], CanonicalAuthError> {
    hash_encoded(&[ISSUER_PROOF_BINDING_HASH_DOMAIN], false, &|out| {
        encode_principal(out, issuer_pid)?;
        encode_issuer_proof_algorithm(out, issuer_proof_alg);
        encode_issuer_proof_binding(out, issuer_proof_binding);
        Ok(())
    })
}

/// Hash the exact validated role label under the existing role domain.
pub fn role_hash(role: &AuthRole) -> Result<[u8; 32], CanonicalAuthError> {
    hash_encoded(&[], false, &|out| {
        out(DOMAIN_SEPARATOR);
        out(&[CanonicalDomain::RoleHash as u8]);
        encode_string(out, role.as_str())
    })
}

/// Encode a certificate without changing grant order or normalizing labels.
pub fn cert_bytes(cert: &DelegationCert) -> Result<Vec<u8>, CanonicalAuthError> {
    let mut out = Vec::with_capacity(128);
    encode_cert(&mut |bytes| out.extend_from_slice(bytes), cert)?;
    Ok(out)
}

fn encode_cert(out: &mut Sink<'_>, cert: &DelegationCert) -> Result<(), CanonicalAuthError> {
    out(DOMAIN_SEPARATOR);
    out(&[CanonicalDomain::DelegationCert as u8]);

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
    encode_role_grants::<false>(out, &cert.grants)?;

    Ok(())
}

fn encode_issuer_proof_algorithm(out: &mut Sink<'_>, alg: IssuerProofAlgorithm) {
    let tag = match alg {
        IssuerProofAlgorithm::IcCanisterSignatureV1 => 1,
    };
    out(&[tag]);
}

fn encode_audience(out: &mut Sink<'_>, audience: &DelegationAudience) {
    match audience {
        DelegationAudience::Fleet(fleet) => {
            out(&[1]);
            encode_fleet_key(out, *fleet);
        }
    }
}

fn encode_fleet_key(out: &mut Sink<'_>, fleet: AudienceId) {
    encode_fixed_32(out, *fleet.canonical_network_id.as_bytes());
    encode_fixed_32(out, *fleet.fleet_id.as_bytes());
}

// ADMITTED is private same-call shape evidence: portable encoding validates the
// immutable grants first. Standalone canonical encoders retain their own checks.
pub(crate) fn encode_role_grants<const ADMITTED: bool>(
    out: &mut Sink<'_>,
    grants: &[DelegatedRoleGrant],
) -> Result<(), CanonicalAuthError> {
    encode_len(out, grants.len())?;
    let mut previous = None;
    for grant in grants {
        let current = grant.target.as_str().as_bytes();
        if !ADMITTED && previous.is_some_and(|previous| previous >= current) {
            return Err(CanonicalAuthError::NonCanonicalRoles);
        }
        previous = Some(current);
        encode_role(out, &grant.target)?;
        if !ADMITTED {
            validate_scopes(&grant.scopes)?;
        }
        encode_len(out, grant.scopes.len())?;
        for scope in &grant.scopes {
            encode_bytes(out, scope.as_str().as_bytes())?;
        }
    }
    Ok(())
}

fn hash_encoded(
    prefix: &[&[u8]],
    length_prefixed: bool,
    encode: &dyn Fn(&mut Sink<'_>) -> Result<(), CanonicalAuthError>,
) -> Result<[u8; 32], CanonicalAuthError> {
    let mut hash = Sha256::new();
    for bytes in prefix {
        hash.update(bytes);
    }
    if length_prefixed {
        let mut length = 0u64;
        encode(&mut |bytes| length = length.saturating_add(bytes.len() as u64))?;
        let length = u32::try_from(length).map_err(|_| CanonicalAuthError::LengthOverflow)?;
        hash.update(length.to_be_bytes());
    }
    encode(&mut |bytes| hash.update(bytes))?;
    Ok(hash.finalize().into())
}

fn encode_chain_key_batch_header(
    out: &mut Sink<'_>,
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
    out: &mut Sink<'_>,
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
    encode_role_grants::<false>(out, &cert.grants)?;
    encode_u64(out, cert.not_before_ns);
    encode_u64(out, cert.expires_at_ns);
    encode_u64(out, cert.registry_epoch);
    encode_fixed_32(out, cert.registry_hash);
    Ok(())
}

fn encode_root_proof(out: &mut Sink<'_>, proof: &RootProof) -> Result<(), CanonicalAuthError> {
    match proof {
        RootProof::IcChainKeyBatchSignatureV1(proof) => {
            out(&[2]);
            encode_chain_key_proof(out, proof)?;
        }
    }
    Ok(())
}

fn encode_chain_key_proof(
    out: &mut Sink<'_>,
    proof: &IcChainKeyBatchSignatureProofV1,
) -> Result<(), CanonicalAuthError> {
    encode_chain_key_batch_header(out, &proof.header)?;
    encode_chain_key_delegation_cert(out, &proof.delegation_cert)?;
    encode_chain_key_witness(out, &proof.issuer_witness)?;
    encode_chain_key_signature(out, &proof.signature)?;
    Ok(())
}

fn encode_chain_key_witness(
    out: &mut Sink<'_>,
    witness: &ChainKeyBatchWitnessV1,
) -> Result<(), CanonicalAuthError> {
    encode_len(out, witness.steps.len())?;
    for step in &witness.steps {
        match step {
            ChainKeyBatchWitnessStepV1::LeftSibling(hash) => {
                out(&[1]);
                encode_fixed_32(out, *hash);
            }
            ChainKeyBatchWitnessStepV1::RightSibling(hash) => {
                out(&[2]);
                encode_fixed_32(out, *hash);
            }
        }
    }
    Ok(())
}

fn encode_chain_key_signature(
    out: &mut Sink<'_>,
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
    out: &mut Sink<'_>,
    derivation_path: &[Vec<u8>],
) -> Result<(), CanonicalAuthError> {
    encode_len(out, derivation_path.len())?;
    for path_component in derivation_path {
        encode_bytes(out, path_component)?;
    }
    Ok(())
}

fn encode_chain_key_algorithm(out: &mut Sink<'_>, algorithm: ChainKeyAlgorithm) {
    let tag = match algorithm {
        ChainKeyAlgorithm::EcdsaSecp256k1 => 1,
    };
    out(&[tag]);
}

fn encode_chain_key_key_id(
    out: &mut Sink<'_>,
    key_id: &ChainKeyKeyId,
) -> Result<(), CanonicalAuthError> {
    encode_string(out, &key_id.name)?;
    Ok(())
}

fn encode_issuer_proof_binding(out: &mut Sink<'_>, binding: IssuerProofBinding) {
    match binding {
        IssuerProofBinding::IcCanisterSignatureV1 { seed_hash } => {
            out(&[1]);
            encode_fixed_32(out, seed_hash);
        }
    }
}

fn encode_role(out: &mut Sink<'_>, role: &AuthRole) -> Result<(), CanonicalAuthError> {
    encode_bytes(out, role.as_str().as_bytes())?;
    Ok(())
}

/// Validate strictly increasing, unique UTF-8 byte order of checked labels.
///
/// An empty list is valid. Cardinality, normalization and application authority
/// remain caller-owned; use the bounded grant validator for nonempty grants.
pub fn validate_scopes(scopes: &[ScopeLabel]) -> Result<(), CanonicalAuthError> {
    let mut previous = None;
    for scope in scopes {
        let current = scope.as_str().as_bytes();
        if previous.is_some_and(|previous| previous >= current) {
            return Err(CanonicalAuthError::NonCanonicalScopes);
        }
        previous = Some(current);
    }

    Ok(())
}

/// Check the existing lowercase, colon-separated application-scope grammar.
pub fn validate_scope_label(scope: &str) -> Result<(), CanonicalAuthError> {
    ScopeLabel::validate(scope).map_err(Into::into)
}

fn encode_string(out: &mut Sink<'_>, value: &str) -> Result<(), CanonicalAuthError> {
    encode_bytes(out, value.as_bytes())?;
    Ok(())
}

pub(crate) fn encode_principal(
    out: &mut Sink<'_>,
    principal: Principal,
) -> Result<(), CanonicalAuthError> {
    encode_bytes(out, principal.as_slice())?;
    Ok(())
}

fn encode_bytes(out: &mut Sink<'_>, bytes: &[u8]) -> Result<(), CanonicalAuthError> {
    encode_len(out, bytes.len())?;
    out(bytes);
    Ok(())
}

fn encode_fixed_32(out: &mut Sink<'_>, bytes: [u8; 32]) {
    out(&bytes);
}

fn encode_u64(out: &mut Sink<'_>, value: u64) {
    out(&value.to_be_bytes());
}

fn encode_u16(out: &mut Sink<'_>, value: u16) {
    out(&value.to_be_bytes());
}

fn encode_len(out: &mut Sink<'_>, len: usize) -> Result<(), CanonicalAuthError> {
    let len = u32::try_from(len).map_err(|_| CanonicalAuthError::LengthOverflow)?;
    out(&len.to_be_bytes());
    Ok(())
}

#[cfg(test)]
mod tests;
