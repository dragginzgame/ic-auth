use super::{
    FixtureModel, decode_domain, encode_domain, source_audience, source_claims, source_grants,
};
use candid::CandidType;
use candid::{decode_one, encode_one};
use ic_auth_protocol_types::*;
use ic_auth_qualification_contracts::model::{
    ChainKeyAlgorithm as SourceAlgorithm, ChainKeyBatchHeaderV1 as SourceHeader,
    ChainKeyBatchWitnessStepV1 as SourceWitnessStep, ChainKeyBatchWitnessV1 as SourceWitness,
    ChainKeyDelegationCertV1 as SourceLeaf, ChainKeyKeyId as SourceKeyId,
    ChainKeyRootSignatureV1 as SourceRootSignature, DelegatedToken as SourceToken,
    DelegationAudience as SourceAudience, DelegationCert as SourceCert,
    DelegationProof as SourceDelegationProof,
    IcCanisterSignatureProofV1 as SourceCanisterSignature,
    IcChainKeyBatchSignatureProofV1 as SourceBatchProof, IssuerProof as SourceIssuerProof,
    IssuerProofAlgorithm as SourceIssuerAlgorithm, IssuerProofBinding as SourceBinding,
    RootProof as SourceRootProof,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// Independent transport declarations from Canic c4c046f947b2b28f4342cbf6efe9221ba1ed5f70,
// crates/canic-core/src/dto/auth/{proof,token}.rs. Sample keys/signatures are
// intentionally synthetic: Candid compatibility does not authenticate them.
fn source_proof(steps: Vec<SourceWitnessStep>) -> SourceDelegationProof {
    let root = Principal::from_slice(&[3; 29]);
    let issuer = Principal::from_slice(&[2; 29]);
    let binding = || SourceBinding::IcCanisterSignatureV1 {
        seed_hash: [0x80; 32],
    };
    let key_id = || SourceKeyId {
        name: "test_key_1".into(),
    };
    SourceDelegationProof {
        cert: SourceCert {
            root_pid: root,
            issuer_pid: issuer,
            issuer_proof_alg: SourceIssuerAlgorithm::IcCanisterSignatureV1,
            issuer_proof_binding_hash: [0xff; 32],
            issuer_proof_binding: binding(),
            issued_at_ns: u64::MAX - 100,
            not_before_ns: u64::MAX - 100,
            expires_at_ns: u64::MAX,
            max_token_ttl_ns: 100,
            aud: source_audience(),
            grants: source_grants(),
        },
        root_proof: SourceRootProof::IcChainKeyBatchSignatureV1(SourceBatchProof {
            header: SourceHeader {
                schema_version: 1,
                root_canister_id: root,
                batch_id: [0x80; 32],
                proof_epoch: u64::MAX,
                registry_epoch: u64::MAX,
                registry_hash: [0xff; 32],
                tree_root: [0x80; 32],
                not_before_ns: u64::MAX - 100,
                expires_at_ns: u64::MAX,
                algorithm: SourceAlgorithm::EcdsaSecp256k1,
                key_id: key_id(),
                derivation_path_hash: [0xff; 32],
                key_version: u64::MAX,
            },
            delegation_cert: SourceLeaf {
                root_canister_id: root,
                issuer_canister_id: issuer,
                proof_epoch: u64::MAX,
                issuer_proof_algorithm: SourceIssuerAlgorithm::IcCanisterSignatureV1,
                issuer_proof_binding_hash: [0xff; 32],
                issuer_proof_binding: binding(),
                max_token_ttl_ns: 100,
                audience: source_audience(),
                grants: source_grants(),
                not_before_ns: u64::MAX - 100,
                expires_at_ns: u64::MAX,
                registry_epoch: u64::MAX,
                registry_hash: [0xff; 32],
            },
            issuer_witness: SourceWitness { steps },
            signature: SourceRootSignature {
                algorithm: SourceAlgorithm::EcdsaSecp256k1,
                key_id: key_id(),
                derivation_path: vec![vec![], b"canic".to_vec(), vec![0, 0x80, 0xff]],
                public_key: vec![2, 0, 0x80, 0xff],
                signature: vec![0, 0x80, 0xff],
            },
        }),
    }
}

fn assert_wire_round_trip<T, S>(source: &S)
where
    T: FixtureModel + Serialize + for<'de> Deserialize<'de>,
    S: CandidType + for<'de> Deserialize<'de> + Debug + PartialEq,
{
    let source_bytes = encode_one(source).unwrap();
    let checked: T = decode_domain(&source_bytes).unwrap();
    let checked_bytes = encode_domain(checked).unwrap();
    assert_eq!(checked_bytes, source_bytes);
    assert_eq!(&decode_one::<S>(&checked_bytes).unwrap(), source);
}

#[test]
fn delegation_proof_preserves_canic_cert_batch_witness_and_signature() {
    for steps in [
        vec![],
        vec![SourceWitnessStep::LeftSibling([0x80; 32])],
        vec![
            SourceWitnessStep::RightSibling([0xff; 32]),
            SourceWitnessStep::LeftSibling([0; 32]),
        ],
    ] {
        let proof = source_proof(steps);
        assert_wire_round_trip::<DelegationCert, _>(&proof.cert);
        assert_wire_round_trip::<RootProof, _>(&proof.root_proof);
        assert_wire_round_trip::<DelegationProof, _>(&proof);
    }
}

#[test]
fn retrieved_token_preserves_complete_proofs_and_optional_extensions() {
    for ext in [None, Some(vec![]), Some(vec![0, 0x80, 0xff])] {
        for bytes in [vec![], vec![0, 0x80, 0xff]] {
            let token = SourceToken {
                claims: source_claims(ext.clone()),
                proof: source_proof(vec![SourceWitnessStep::RightSibling([0x80; 32])]),
                issuer_proof: SourceIssuerProof::IcCanisterSignatureV1(SourceCanisterSignature {
                    signature_cbor: bytes.clone(),
                    public_key_der: bytes,
                }),
            };
            assert_wire_round_trip::<IssuerProof, _>(&token.issuer_proof);
            assert_wire_round_trip::<DelegatedToken, _>(&token);
        }
    }
}

#[test]
fn nested_certificate_and_leaf_cannot_bypass_role_or_audience_validation() {
    let mut proof = source_proof(vec![]);
    proof.cert.grants[0].target = "Root".into();
    assert!(decode_domain::<DelegationProof>(&encode_one(&proof).unwrap()).is_err());
    proof.cert.grants = source_grants();
    let SourceAudience::Fleet(key) = &mut proof.cert.aud;
    key.fleet_id = "cd".into();
    assert!(decode_domain::<DelegationProof>(&encode_one(&proof).unwrap()).is_err());
    proof.cert.aud = source_audience();
    let SourceRootProof::IcChainKeyBatchSignatureV1(batch) = &mut proof.root_proof;
    batch.delegation_cert.grants[0].target = "app/read".into();
    assert!(decode_domain::<DelegationProof>(&encode_one(&proof).unwrap()).is_err());
    let SourceRootProof::IcChainKeyBatchSignatureV1(batch) = &mut proof.root_proof;
    batch.delegation_cert.grants = source_grants();
    let SourceAudience::Fleet(key) = &mut batch.delegation_cert.audience;
    key.canonical_network_id = "AB".repeat(32);
    assert!(decode_domain::<DelegationProof>(&encode_one(&proof).unwrap()).is_err());

    let token = SourceToken {
        claims: source_claims(None),
        proof,
        issuer_proof: SourceIssuerProof::IcCanisterSignatureV1(SourceCanisterSignature {
            signature_cbor: vec![],
            public_key_der: vec![],
        }),
    };
    assert!(decode_domain::<DelegatedToken>(&encode_one(token).unwrap()).is_err());
}

#[test]
fn portable_envelope_preserves_both_proofs_and_exact_claim_blobs() {
    let delegation =
        decode_domain::<DelegationProof>(&encode_one(source_proof(vec![])).unwrap()).unwrap();
    for proof in [
        PortableProof::Ed25519 {
            signature: vec![7; 64],
        },
        PortableProof::Certified {
            delegation: Box::new(delegation),
            issuer_signature: IcCanisterSignatureProofV1 {
                signature_cbor: vec![0, 255],
                public_key_der: vec![1, 2, 3],
            },
        },
    ] {
        let credential = PortableCredential {
            claims: vec![0, 255, 128],
            proof,
        };
        let bytes = encode_domain(credential.clone()).unwrap();
        assert_eq!(
            decode_domain::<PortableCredential>(&bytes).unwrap(),
            credential
        );
        assert!(decode_domain::<DelegatedToken>(&bytes).is_err());
        assert!(decode_domain::<PortableCredential>(&bytes[..bytes.len() - 1]).is_err());
    }
}
