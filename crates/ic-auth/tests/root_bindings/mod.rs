//! Both public verifier boundaries expose the same canonical Root rejection.

use super::*;
use ic_auth::token::{DelegationProofVerificationContext, verify_delegation_proof};
use k256::ecdsa::signature::hazmat::PrehashVerifier;

fn assert_rejected(f: &Fixture, expected: Error) {
    let unchanged = f.token.clone();
    assert_eq!(verify_token(&f.token, &f.context()).unwrap_err(), expected);
    let standalone = verify_delegation_proof(
        &f.token.proof,
        &DelegationProofVerificationContext {
            expected_issuer: p(2),
            root_key: &f.policy,
            now_ns: NOW,
            limits: f.context().limits.into(),
        },
    )
    .unwrap_err();
    assert_eq!(standalone, expected);
    assert_eq!(f.token, unchanged);
}

fn resign(root: &mut IcChainKeyBatchSignatureProofV1) {
    let hash = chain_key_batch_header_hash(&root.header).unwrap();
    let key = root_signing_key();
    let signature: Signature = key.sign_prehash(&hash).unwrap();
    key.verifying_key()
        .verify_prehash(&hash, &signature)
        .unwrap();
    root.signature.signature = signature.to_bytes().to_vec();
}

#[test]
fn root_policy_and_witness_rejections_precede_isolated_crypto_errors() {
    for rejection in ["policy", "witness"] {
        let mut f = Fixture::new();
        assert!(verify_token(&f.token, &f.context()).is_ok());
        f.policy.public_key = vec![0; 33];
        let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
        root.signature.public_key = f.policy.public_key.clone();
        root.signature.signature = vec![0; 64];
        root.header.tree_root[0] ^= 1;
        let expected = if rejection == "policy" {
            f.policy.min_accepted_proof_epoch = root.header.proof_epoch + 1;
            Error::StaleAuthority {
                field: "proof_epoch",
            }
        } else {
            Error::InvalidMerkleWitness
        };
        assert_rejected(&f, expected);
    }
}

#[test]
fn key_id_distinguishes_submitted_binding_from_enrollment_and_defines_precedence() {
    for mutation in ["signature", "header", "both", "policy"] {
        let mut f = Fixture::new();
        assert!(verify_token(&f.token, &f.context()).is_ok());
        let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
        match mutation {
            "signature" => root.signature.key_id.name = "other".into(),
            "header" => {
                root.header.key_id.name = "other".into();
                resign(root);
            }
            "both" => {
                root.header.key_id.name = "other".into();
                root.signature.key_id = root.header.key_id.clone();
                resign(root);
            }
            _ => f.policy.key_id.name = "other".into(),
        }
        let expected = if matches!(mutation, "signature" | "header") {
            Error::RootSignatureBindingMismatch { field: "key_id" }
        } else {
            Error::RootKeyPolicyBindingMismatch { field: "key_id" }
        };
        assert_rejected(&f, expected);
        if mutation == "both" {
            // The header/signature really are usable with a matching enrollment;
            // the rejection above is not an accidentally invalid signature.
            f.policy.key_id.name = "other".into();
            assert!(verify_token(&f.token, &f.context()).is_ok());
        }
    }
}

#[test]
fn derivation_path_distinguishes_submitted_binding_from_enrollment() {
    for mutation in ["signature", "header", "both", "policy", "empty"] {
        let mut f = Fixture::new();
        let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
        match mutation {
            "signature" => root.signature.derivation_path[0].push(0),
            "header" => {
                root.header.derivation_path_hash[0] ^= 1;
                resign(root);
            }
            "both" => {
                root.signature.derivation_path[0].push(0);
                root.header.derivation_path_hash =
                    chain_key_derivation_path_hash(&root.signature.derivation_path).unwrap();
                resign(root);
            }
            "empty" => root.signature.derivation_path.clear(),
            _ => f.policy.derivation_path_hash[0] ^= 1,
        }
        let expected = if matches!(mutation, "both" | "policy") {
            Error::RootKeyPolicyBindingMismatch {
                field: "derivation_path_hash",
            }
        } else {
            Error::RootSignatureBindingMismatch {
                field: "derivation_path_hash",
            }
        };
        assert_rejected(&f, expected);
        if mutation == "both" {
            let RootProof::IcChainKeyBatchSignatureV1(root) = &f.token.proof.root_proof;
            f.policy.derivation_path_hash = root.header.derivation_path_hash;
            assert!(verify_token(&f.token, &f.context()).is_ok());
        }
    }
}

#[test]
fn root_binding_errors_preserve_guard_order_and_bounded_path_admission() {
    let mut f = Fixture::new();
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.signature.key_id.name.push('x');
    f.policy.accept_until_ns = NOW;
    assert_rejected(
        &f,
        Error::Expired {
            target: "root_key_policy",
        },
    );

    let mut f = Fixture::new();
    f.policy.key_id.name.push('x');
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.signature.derivation_path.clear();
    assert_rejected(&f, Error::RootKeyPolicyBindingMismatch { field: "key_id" });

    let mut f = Fixture::new();
    let too_large = f.context().limits.max_variable_bytes + 1;
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.token.proof.root_proof;
    root.signature.derivation_path = vec![vec![0; too_large]];
    f.policy.derivation_path_hash[0] ^= 1;
    assert_rejected(&f, Error::InputTooLarge);
}
