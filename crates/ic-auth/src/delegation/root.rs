use super::{DelegationField, DelegationTarget};
use super::{DelegationProofError, RootKeyPolicy, binding, window};
use crate::canonical::{
    chain_key_batch_header_hash, chain_key_delegation_cert_hash, chain_key_derivation_path_hash,
};
use crate::chain_key_batch::witness_root;
use crate::chain_key_signature::{ChainKeySignatureError, VerificationMaterial};
use ic_auth_protocol_types::{ChainKeyAlgorithm, DelegationCert, RootProof};

pub(super) fn verify(
    cert: &DelegationCert,
    root: &RootProof,
    policy: &RootKeyPolicy,
    now_ns: u64,
    max_future_skew_ns: u64,
) -> Result<(), DelegationProofError> {
    let RootProof::IcChainKeyBatchSignatureV1(proof) = root;
    let header = &proof.header;
    let leaf = &proof.delegation_cert;
    let signature = &proof.signature;
    binding(header.schema_version == 1, DelegationField::SchemaVersion)?;
    binding(
        header.root_canister_id == policy.root_canister_id
            && leaf.root_canister_id == header.root_canister_id,
        DelegationField::RootCanisterId,
    )?;
    window(
        DelegationTarget::RootKeyPolicy,
        policy.valid_from_ns,
        policy.accept_until_ns,
        now_ns,
        max_future_skew_ns,
    )?;
    window(
        DelegationTarget::Batch,
        header.not_before_ns,
        header.expires_at_ns,
        now_ns,
        max_future_skew_ns,
    )?;
    window(
        DelegationTarget::IssuerLeaf,
        leaf.not_before_ns,
        leaf.expires_at_ns,
        now_ns,
        max_future_skew_ns,
    )?;
    binding(
        leaf.not_before_ns >= header.not_before_ns && leaf.expires_at_ns <= header.expires_at_ns,
        DelegationField::LeafBatchWindow,
    )?;
    binding(
        leaf.proof_epoch == header.proof_epoch,
        DelegationField::ProofEpoch,
    )?;
    binding(
        leaf.registry_epoch == header.registry_epoch,
        DelegationField::RegistryEpoch,
    )?;
    binding(
        leaf.registry_hash == header.registry_hash,
        DelegationField::RegistryHash,
    )?;
    binding(
        leaf.issuer_canister_id == cert.issuer_pid,
        DelegationField::IssuerCanisterId,
    )?;
    binding(
        leaf.issuer_proof_algorithm == cert.issuer_proof_alg,
        DelegationField::IssuerProofAlgorithm,
    )?;
    binding(
        leaf.issuer_proof_binding_hash == cert.issuer_proof_binding_hash
            && leaf.issuer_proof_binding == cert.issuer_proof_binding,
        DelegationField::IssuerProofBinding,
    )?;
    binding(
        leaf.max_token_ttl_ns == cert.max_token_ttl_ns,
        DelegationField::MaxTokenTtl,
    )?;
    binding(leaf.audience == cert.aud, DelegationField::LeafAudience)?;
    binding(leaf.grants == cert.grants, DelegationField::LeafGrants)?;
    binding(
        leaf.not_before_ns == cert.not_before_ns && leaf.expires_at_ns == cert.expires_at_ns,
        DelegationField::LeafCertWindow,
    )?;
    // For each field, inconsistent submitted material takes precedence over
    // an enrollment mismatch. Consumers project this cause without rechecking
    // submitted metadata; neither rejection establishes verified authority.
    if signature.algorithm != header.algorithm {
        return Err(DelegationProofError::RootSignatureBindingMismatch {
            field: DelegationField::Algorithm,
        });
    }
    if header.algorithm != policy.algorithm {
        return Err(DelegationProofError::RootKeyPolicyBindingMismatch {
            field: DelegationField::Algorithm,
        });
    }
    if signature.key_id != header.key_id {
        return Err(DelegationProofError::RootSignatureBindingMismatch {
            field: DelegationField::KeyId,
        });
    }
    if header.key_id != policy.key_id {
        return Err(DelegationProofError::RootKeyPolicyBindingMismatch {
            field: DelegationField::KeyId,
        });
    }
    binding(
        signature.public_key == policy.public_key,
        DelegationField::RootPublicKey,
    )?;
    if chain_key_derivation_path_hash(&signature.derivation_path)? != header.derivation_path_hash {
        return Err(DelegationProofError::RootSignatureBindingMismatch {
            field: DelegationField::DerivationPathHash,
        });
    }
    if header.derivation_path_hash != policy.derivation_path_hash {
        return Err(DelegationProofError::RootKeyPolicyBindingMismatch {
            field: DelegationField::DerivationPathHash,
        });
    }
    binding(
        header.key_version == policy.key_version,
        DelegationField::KeyVersion,
    )?;
    for (field, found, minimum) in [
        (
            DelegationField::KeyVersion,
            header.key_version,
            policy.min_accepted_key_version,
        ),
        (
            DelegationField::ProofEpoch,
            header.proof_epoch,
            policy.min_accepted_proof_epoch,
        ),
        (
            DelegationField::RegistryEpoch,
            header.registry_epoch,
            policy.min_accepted_registry_epoch,
        ),
    ] {
        if found < minimum {
            return Err(DelegationProofError::StaleAuthority { field });
        }
    }
    let ttl_ns = header.expires_at_ns - header.not_before_ns;
    if ttl_ns > policy.max_revocation_latency_ns {
        return Err(DelegationProofError::TtlExceeded {
            target: DelegationTarget::Batch,
            ttl_ns,
            max_ttl_ns: policy.max_revocation_latency_ns,
        });
    }
    let leaf_hash = chain_key_delegation_cert_hash(leaf)?;
    if witness_root(leaf_hash, &proof.issuer_witness) != header.tree_root {
        return Err(DelegationProofError::InvalidMerkleWitness);
    }
    let ChainKeyAlgorithm::EcdsaSecp256k1 = signature.algorithm;
    let material = VerificationMaterial::parse(&policy.public_key, &signature.signature)
        .map_err(signature_error)?;
    material
        .verify_prehash(&chain_key_batch_header_hash(header)?)
        .map_err(signature_error)
}

fn signature_error(error: ChainKeySignatureError) -> DelegationProofError {
    match error {
        ChainKeySignatureError::HighSSignature => DelegationProofError::HighSSignature,
        ChainKeySignatureError::MalformedPublicKey
        | ChainKeySignatureError::MalformedSignature
        | ChainKeySignatureError::InvalidSignature => DelegationProofError::RootSignatureInvalid,
    }
}
