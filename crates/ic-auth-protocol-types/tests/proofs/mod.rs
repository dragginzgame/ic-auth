use super::{
    SourceAudience, SourceClaims, SourceGrant, source_audience, source_claims, source_grants,
};
use candid::{CandidType, decode_one, encode_one};
use ic_auth_protocol_types::*;
use serde::Deserialize;
use std::fmt::Debug;

// Independent transport declarations from Canic c4c046f947b2b28f4342cbf6efe9221ba1ed5f70,
// crates/canic-core/src/dto/auth/{proof,token}.rs. Sample keys/signatures are
// intentionally synthetic: Candid compatibility does not authenticate them.
#[derive(CandidType, Debug, Deserialize, PartialEq)]
enum SourceRootProof {
    IcChainKeyBatchSignatureV1(SourceBatchProof),
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
enum SourceIssuerProof {
    IcCanisterSignatureV1(SourceCanisterSignature),
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceCanisterSignature {
    signature_cbor: Vec<u8>,
    public_key_der: Vec<u8>,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
enum SourceAlgorithm {
    EcdsaSecp256k1,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceKeyId {
    name: String,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceBatchProof {
    header: SourceHeader,
    delegation_cert: SourceLeaf,
    issuer_witness: SourceWitness,
    signature: SourceRootSignature,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceHeader {
    schema_version: u16,
    root_canister_id: Principal,
    batch_id: [u8; 32],
    proof_epoch: u64,
    registry_epoch: u64,
    registry_hash: [u8; 32],
    tree_root: [u8; 32],
    not_before_ns: u64,
    expires_at_ns: u64,
    algorithm: SourceAlgorithm,
    key_id: SourceKeyId,
    derivation_path_hash: [u8; 32],
    key_version: u64,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceLeaf {
    root_canister_id: Principal,
    issuer_canister_id: Principal,
    proof_epoch: u64,
    issuer_proof_algorithm: SourceIssuerAlgorithm,
    issuer_proof_binding_hash: [u8; 32],
    issuer_proof_binding: SourceBinding,
    max_token_ttl_ns: u64,
    audience: SourceAudience,
    grants: Vec<SourceGrant>,
    not_before_ns: u64,
    expires_at_ns: u64,
    registry_epoch: u64,
    registry_hash: [u8; 32],
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceRootSignature {
    algorithm: SourceAlgorithm,
    key_id: SourceKeyId,
    derivation_path: Vec<Vec<u8>>,
    public_key: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceWitness {
    steps: Vec<SourceWitnessStep>,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
enum SourceWitnessStep {
    LeftSibling([u8; 32]),
    RightSibling([u8; 32]),
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
enum SourceIssuerAlgorithm {
    IcCanisterSignatureV1,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
enum SourceBinding {
    IcCanisterSignatureV1 { seed_hash: [u8; 32] },
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceCert {
    root_pid: Principal,
    issuer_pid: Principal,
    issuer_proof_alg: SourceIssuerAlgorithm,
    issuer_proof_binding_hash: [u8; 32],
    issuer_proof_binding: SourceBinding,
    issued_at_ns: u64,
    not_before_ns: u64,
    expires_at_ns: u64,
    max_token_ttl_ns: u64,
    aud: SourceAudience,
    grants: Vec<SourceGrant>,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceDelegationProof {
    cert: SourceCert,
    root_proof: SourceRootProof,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceToken {
    claims: SourceClaims,
    proof: SourceDelegationProof,
    issuer_proof: SourceIssuerProof,
}

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
    T: CandidType + for<'de> Deserialize<'de>,
    S: CandidType + for<'de> Deserialize<'de> + Debug + PartialEq,
{
    let source_bytes = encode_one(source).unwrap();
    let checked: T = decode_one(&source_bytes).unwrap();
    let checked_bytes = encode_one(checked).unwrap();
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
    assert!(decode_one::<DelegationProof>(&encode_one(&proof).unwrap()).is_err());
    proof.cert.grants = source_grants();
    let SourceAudience::Fleet(key) = &mut proof.cert.aud;
    key.fleet_id = "cd".into();
    assert!(decode_one::<DelegationProof>(&encode_one(&proof).unwrap()).is_err());
    proof.cert.aud = source_audience();
    let SourceRootProof::IcChainKeyBatchSignatureV1(batch) = &mut proof.root_proof;
    batch.delegation_cert.grants[0].target = "app/read".into();
    assert!(decode_one::<DelegationProof>(&encode_one(&proof).unwrap()).is_err());
    let SourceRootProof::IcChainKeyBatchSignatureV1(batch) = &mut proof.root_proof;
    batch.delegation_cert.grants = source_grants();
    let SourceAudience::Fleet(key) = &mut batch.delegation_cert.audience;
    key.canonical_network_id = "AB".repeat(32);
    assert!(decode_one::<DelegationProof>(&encode_one(&proof).unwrap()).is_err());

    let token = SourceToken {
        claims: source_claims(None),
        proof,
        issuer_proof: SourceIssuerProof::IcCanisterSignatureV1(SourceCanisterSignature {
            signature_cbor: vec![],
            public_key_der: vec![],
        }),
    };
    assert!(decode_one::<DelegatedToken>(&encode_one(token).unwrap()).is_err());
}
