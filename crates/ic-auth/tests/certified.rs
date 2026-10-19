#![cfg(feature = "certified-proof-verification")]

use ic_auth::{
    canister_signature::domain_separated_message,
    canonical::*,
    delegation::{
        DelegationProofError as Error, DelegationProofVerificationContext, RootKeyPolicy,
        verify_delegation_proof,
    },
    portable_certified::CertifiedProofLimits,
};
use ic_auth_protocol_types::*;
use ic_platform::bls::PrivateKey;
use ic_platform::canister_sig_creation::CanisterSigPublicKey;
use ic_platform::certification::{Certificate, HashTree, fork, labeled, leaf};
use ic_platform::ecdsa::{PrehashSigner, Signature, SigningKey};
use serde::Serialize;
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};

const SEED: &[u8] = b"canic-issuer-delegated-token";
const NOW: u64 = 150;

mod delegation;
mod portable_certified;
mod root_bindings;

fn p(id: u8) -> Principal {
    Principal::from_slice(&[id; 10])
}
fn role() -> AuthRole {
    "app_service".parse().unwrap()
}
fn audience() -> AudienceId {
    AudienceId {
        canonical_network_id: CanonicalId::from_bytes([1; 32]),
        fleet_id: CanonicalId::from_bytes([2; 32]),
    }
}
fn grant(scopes: &[&str]) -> DelegatedRoleGrant {
    DelegatedRoleGrant {
        target: role(),
        scopes: scopes.iter().map(|s| s.to_string()).collect(),
    }
}
fn root_signing_key() -> SigningKey {
    SigningKey::from_slice(&[7; 32]).unwrap()
}
fn bls_key(value: u8) -> PrivateKey {
    let mut bytes = [0; 32];
    bytes[31] = value;
    PrivateKey::deserialize(&bytes).unwrap()
}
fn hash(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
fn cbor<T: Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut serializer = serde_cbor::Serializer::new(&mut bytes);
    serializer.self_describe().unwrap();
    value.serialize(&mut serializer).unwrap();
    bytes
}
#[derive(Serialize)]
struct Envelope {
    certificate: ByteBuf,
    tree: HashTree,
}

fn issuer_proof_message(
    issuer: Principal,
    message: &[u8],
    seed: &[u8],
    time: u64,
) -> IcCanisterSignatureProofV1 {
    let tree = labeled(
        b"sig".to_vec(),
        labeled(hash(seed), labeled(hash(message), leaf(vec![]))),
    );
    let mut time_bytes = Vec::new();
    leb128::write::unsigned(&mut time_bytes, time).unwrap();
    let certificate_tree = fork(
        labeled(
            b"canister".to_vec(),
            labeled(
                issuer.as_slice().to_vec(),
                labeled(b"certified_data".to_vec(), leaf(tree.digest())),
            ),
        ),
        labeled(b"time".to_vec(), leaf(time_bytes)),
    );
    let mut message = b"\x0Dic-state-root".to_vec();
    message.extend_from_slice(&certificate_tree.digest());
    let certificate = Certificate {
        tree: certificate_tree,
        signature: bls_key(1).sign(&message).serialize().to_vec(),
        delegation: None,
    };
    IcCanisterSignatureProofV1 {
        signature_cbor: cbor(&Envelope {
            certificate: ByteBuf::from(cbor(&certificate)),
            tree,
        }),
        public_key_der: CanisterSigPublicKey::new(issuer, seed.to_vec()).to_der(),
    }
}

fn merkle_node(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut bytes = vec![1];
    bytes.extend_from_slice(&left);
    bytes.extend_from_slice(&right);
    hash(&bytes)
}

fn root_proof(cert: &DelegationCert, policy: &RootKeyPolicy) -> RootProof {
    let leaf = ChainKeyDelegationCertV1 {
        root_canister_id: cert.root_pid,
        issuer_canister_id: cert.issuer_pid,
        proof_epoch: 2,
        registry_epoch: 3,
        registry_hash: [8; 32],
        issuer_proof_algorithm: cert.issuer_proof_alg,
        issuer_proof_binding_hash: cert.issuer_proof_binding_hash,
        issuer_proof_binding: cert.issuer_proof_binding,
        max_token_ttl_ns: cert.max_token_ttl_ns,
        audience: cert.aud.clone(),
        grants: cert.grants.clone(),
        not_before_ns: cert.not_before_ns,
        expires_at_ns: cert.expires_at_ns,
    };
    let leaf_hash = chain_key_delegation_cert_hash(&leaf).unwrap();
    let header = ChainKeyBatchHeaderV1 {
        schema_version: 1,
        root_canister_id: cert.root_pid,
        batch_id: [7; 32],
        proof_epoch: 2,
        registry_epoch: 3,
        registry_hash: [8; 32],
        tree_root: merkle_node(merkle_node([42; 32], leaf_hash), [43; 32]),
        not_before_ns: cert.not_before_ns,
        expires_at_ns: cert.expires_at_ns,
        algorithm: policy.algorithm,
        key_id: policy.key_id.clone(),
        derivation_path_hash: policy.derivation_path_hash,
        key_version: policy.key_version,
    };
    let sig: Signature = root_signing_key()
        .sign_prehash(&chain_key_batch_header_hash(&header).unwrap())
        .unwrap();
    RootProof::IcChainKeyBatchSignatureV1(IcChainKeyBatchSignatureProofV1 {
        header,
        delegation_cert: leaf,
        issuer_witness: ChainKeyBatchWitnessV1 {
            steps: vec![
                ChainKeyBatchWitnessStepV1::LeftSibling([42; 32]),
                ChainKeyBatchWitnessStepV1::RightSibling([43; 32]),
            ],
        },
        signature: ChainKeyRootSignatureV1 {
            algorithm: policy.algorithm,
            key_id: policy.key_id.clone(),
            derivation_path: vec![b"canic".to_vec(), b"delegation".to_vec()],
            public_key: policy.public_key.clone(),
            signature: sig.to_bytes().to_vec(),
        },
    })
}

struct Fixture {
    proof: DelegationProof,
    policy: RootKeyPolicy,
    ic_root: [u8; 96],
}
impl Fixture {
    fn new() -> Self {
        let policy = RootKeyPolicy {
            root_canister_id: p(1),
            algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
            key_id: ChainKeyKeyId {
                name: "test_key_1".into(),
            },
            derivation_path_hash: chain_key_derivation_path_hash(&[
                b"canic".to_vec(),
                b"delegation".to_vec(),
            ])
            .unwrap(),
            public_key: root_signing_key()
                .verifying_key()
                .to_encoded_point(true)
                .as_bytes()
                .to_vec(),
            key_version: 4,
            min_accepted_key_version: 4,
            min_accepted_proof_epoch: 2,
            min_accepted_registry_epoch: 3,
            valid_from_ns: 1,
            accept_until_ns: 1000,
            max_revocation_latency_ns: 500,
        };
        let binding = IssuerProofBinding::IcCanisterSignatureV1 {
            seed_hash: hash(SEED),
        };
        let cert = DelegationCert {
            root_pid: p(1),
            issuer_pid: p(2),
            issuer_proof_alg: IssuerProofAlgorithm::IcCanisterSignatureV1,
            issuer_proof_binding_hash: issuer_proof_binding_hash(
                p(2),
                IssuerProofAlgorithm::IcCanisterSignatureV1,
                binding,
            )
            .unwrap(),
            issuer_proof_binding: binding,
            issued_at_ns: 100,
            not_before_ns: 100,
            expires_at_ns: 500,
            max_token_ttl_ns: 100,
            aud: DelegationAudience::Fleet(audience()),
            grants: vec![grant(&["read", "write"])],
        };
        Self {
            proof: DelegationProof {
                root_proof: root_proof(&cert, &policy),
                cert,
            },
            policy,
            ic_root: bls_key(1).public_key().serialize(),
        }
    }
    fn context(&self) -> DelegationProofVerificationContext<'_> {
        DelegationProofVerificationContext {
            expected_issuer: p(2),
            root_key: &self.policy,
            now_ns: NOW,
            limits: limits().into(),
        }
    }
    fn assert_error(&self, expected: Error) {
        assert_eq!(
            verify_delegation_proof(&self.proof, &self.context()).unwrap_err(),
            expected
        );
    }
}
fn limits() -> CertifiedProofLimits {
    CertifiedProofLimits {
        max_cert_ttl_ns: 600,
        max_token_ttl_ns: 120,
        max_future_skew_ns: 10,
        max_certificate_age_ns: 100,
        max_variable_bytes: 64 * 1024,
        max_issuer_signature_bytes: 16 * 1024,
        max_witness_steps: 32,
    }
}

#[test]
fn constructed_batch_witness_authenticates_existing_signed_bytes() {
    let mut f = Fixture::new();
    let before = proof_hash(&f.proof).unwrap();
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.proof.root_proof;
    let leaves = [
        [42; 32],
        chain_key_delegation_cert_hash(&root.delegation_cert).unwrap(),
        [43; 32],
    ];
    let (tree_root, witnesses) =
        ic_auth::chain_key_batch::merkle_root_and_witnesses(&leaves, 3).unwrap();
    assert_eq!(tree_root, root.header.tree_root);
    root.issuer_witness = witnesses[1].clone();
    assert_eq!(proof_hash(&f.proof).unwrap(), before);
    assert!(verify_delegation_proof(&f.proof, &f.context()).is_ok());
    let RootProof::IcChainKeyBatchSignatureV1(root) = &mut f.proof.root_proof;
    root.issuer_witness = witnesses[0].clone();
    f.assert_error(Error::InvalidMerkleWitness);
}
