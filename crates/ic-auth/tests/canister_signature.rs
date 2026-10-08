#![cfg(feature = "canister-signature-verification")]

use ic_auth::canister_signature::{
    CanisterSignatureError as Error, CanisterSignaturePolicy, domain_separated_message,
    verify_canister_signature,
};
use ic_auth_protocol_types::{IcCanisterSignatureProofV1, Principal};
use ic_canister_sig_creation::{CanisterSigPublicKey, IC_ROOT_PK_DER_PREFIX};
use ic_certification::{Certificate, Delegation, HashTree, fork, labeled, leaf};
use ic_verify_bls_signature::PrivateKey;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};

const SEED: &[u8] = b"canic-issuer-delegated-token";
const NOW: u64 = 1_000_000;

fn signer() -> Principal {
    Principal::from_slice(&[1; 10])
}

fn key(value: u8) -> PrivateKey {
    let mut bytes = [0; 32];
    bytes[31] = value;
    PrivateKey::deserialize(&bytes).unwrap()
}

fn hash(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn time_bytes(time: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    leb128::write::unsigned(&mut bytes, time).unwrap();
    bytes
}

fn cbor<T: Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut serializer = serde_cbor::Serializer::new(&mut bytes);
    serializer.self_describe().unwrap();
    value.serialize(&mut serializer).unwrap();
    bytes
}

fn certificate(tree: HashTree, key: &PrivateKey, delegation: Option<Delegation>) -> Certificate {
    let mut message = b"\x0Dic-state-root".to_vec();
    message.extend_from_slice(&tree.digest());
    Certificate {
        tree,
        signature: key.sign(&message).serialize().to_vec(),
        delegation,
    }
}

#[derive(Serialize, Deserialize)]
struct Envelope {
    certificate: ByteBuf,
    tree: HashTree,
}

fn proof(
    message: &[u8],
    time: Option<Vec<u8>>,
    certificate_key: &PrivateKey,
    delegation: Option<Delegation>,
) -> IcCanisterSignatureProofV1 {
    let tree = labeled(
        b"sig".to_vec(),
        labeled(hash(SEED), labeled(hash(message), leaf(Vec::new()))),
    );
    let certified_data = labeled(
        b"canister".to_vec(),
        labeled(
            signer().as_slice().to_vec(),
            labeled(b"certified_data".to_vec(), leaf(tree.digest())),
        ),
    );
    let cert_tree = match time {
        Some(bytes) => fork(certified_data, labeled(b"time".to_vec(), leaf(bytes))),
        None => certified_data,
    };
    IcCanisterSignatureProofV1 {
        signature_cbor: cbor(&Envelope {
            certificate: ByteBuf::from(cbor(&certificate(cert_tree, certificate_key, delegation))),
            tree,
        }),
        public_key_der: CanisterSigPublicKey::new(signer(), SEED.to_vec()).to_der(),
    }
}

fn policy(root: &[u8]) -> CanisterSignaturePolicy<'_> {
    CanisterSignaturePolicy {
        signing_canister: signer(),
        seed_hash: hash(SEED),
        ic_root_public_key_raw: root,
        now_ns: NOW,
        max_certificate_age_ns: 100,
        max_future_skew_ns: 10,
        max_signature_bytes: 16 * 1024,
        max_message_bytes: 1024,
    }
}

fn message() -> Vec<u8> {
    domain_separated_message(SEED, [7; 32]).unwrap()
}

#[test]
fn canic_domains_keep_the_exact_existing_signed_bytes() {
    let mut expected = b"\x1ccanic-issuer-delegated-token".to_vec();
    expected.extend_from_slice(&[7; 32]);
    assert_eq!(message(), expected);
    let mut expected = b"\x1bcanic-root-role-attestation".to_vec();
    expected.extend_from_slice(&[8; 32]);
    assert_eq!(
        domain_separated_message(b"canic-root-role-attestation", [8; 32]).unwrap(),
        expected
    );
    assert_eq!(
        domain_separated_message(&[1; 255], [0; 32]).unwrap().len(),
        288
    );
    assert_eq!(
        domain_separated_message(&[1; 256], [0; 32]),
        Err(Error::DomainTooLong)
    );
}

#[test]
fn verifies_a_real_bls_signed_signature_at_exact_host_size_limits() {
    let root = key(1).public_key().serialize();
    let proof = proof(&message(), Some(time_bytes(NOW)), &key(1), None);
    let mut policy = policy(&root);
    policy.max_signature_bytes = proof.signature_cbor.len();
    policy.max_message_bytes = message().len();
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy),
        Ok(())
    );
    policy.max_signature_bytes -= 1;
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy),
        Err(Error::SignatureTooLarge)
    );
    policy.max_signature_bytes += 1;
    policy.max_message_bytes -= 1;
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy),
        Err(Error::MessageTooLarge)
    );
}

#[test]
fn protected_signer_seed_and_network_changes_invalidate_the_same_proof() {
    let root = key(1).public_key().serialize();
    let proof = proof(&message(), Some(time_bytes(NOW)), &key(1), None);
    let mut policy = policy(&root);
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy),
        Ok(())
    );
    policy.signing_canister = Principal::from_slice(&[2; 10]);
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy),
        Err(Error::CanisterMismatch)
    );
    policy.signing_canister = signer();
    policy.seed_hash = hash(b"another approved seed");
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy),
        Err(Error::SeedMismatch)
    );
    policy.seed_hash = hash(SEED);
    let other_root = key(2).public_key().serialize();
    policy.ic_root_public_key_raw = &other_root;
    assert!(matches!(
        verify_canister_signature(&message(), &proof, &policy),
        Err(Error::InvalidSignature(_))
    ));
    policy.ic_root_public_key_raw = &root[..95];
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy),
        Err(Error::InvalidRootKeyLength)
    );
}

#[test]
fn submitted_keys_cannot_select_their_own_signer_or_seed() {
    let root = key(1).public_key().serialize();
    let mut proof = proof(&message(), Some(time_bytes(NOW)), &key(1), None);
    proof.public_key_der =
        CanisterSigPublicKey::new(Principal::from_slice(&[2; 10]), SEED.to_vec()).to_der();
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy(&root)),
        Err(Error::CanisterMismatch)
    );
    proof.public_key_der =
        CanisterSigPublicKey::new(signer(), b"unapproved seed".to_vec()).to_der();
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy(&root)),
        Err(Error::SeedMismatch)
    );
}

#[test]
fn checks_exact_freshness_boundaries_on_every_call() {
    let root = key(1).public_key().serialize();
    for (time, expected) in [
        (NOW - 100, Ok(())),
        (NOW - 101, Err(Error::CertificateTooOld)),
        (NOW + 10, Ok(())),
        (NOW + 11, Err(Error::CertificateInFuture)),
    ] {
        let proof = proof(&message(), Some(time_bytes(time)), &key(1), None);
        assert_eq!(
            verify_canister_signature(&message(), &proof, &policy(&root)),
            expected
        );
    }
    let proof = proof(&message(), Some(time_bytes(NOW)), &key(1), None);
    let mut policy = policy(&root);
    policy.now_ns += 101;
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy),
        Err(Error::CertificateTooOld)
    );
    policy.now_ns = u64::MAX;
    policy.max_certificate_age_ns = u64::MAX - NOW;
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy),
        Ok(())
    );
    policy.now_ns = 0;
    policy.max_future_skew_ns = NOW;
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy),
        Ok(())
    );
}

#[test]
fn rejects_absent_truncated_overflow_and_trailing_certificate_time() {
    let root = key(1).public_key().serialize();
    for time in [
        None,
        Some(vec![]),
        Some(vec![0x80]),
        Some(vec![0xff; 10]),
        Some(vec![0x80; 11]),
        Some(vec![0, 0]),
    ] {
        let proof = proof(&message(), time, &key(1), None);
        assert_eq!(
            verify_canister_signature(&message(), &proof, &policy(&root)),
            Err(Error::InvalidCertificateTime)
        );
    }
    let proof = proof(&message(), Some(time_bytes(u64::MAX)), &key(1), None);
    let mut policy = policy(&root);
    policy.now_ns = u64::MAX;
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy),
        Ok(())
    );
}

#[test]
fn rejects_modified_message_domain_and_signature_tree() {
    let root = key(1).public_key().serialize();
    let mut proof = proof(&message(), Some(time_bytes(NOW)), &key(1), None);
    for message in [
        domain_separated_message(SEED, [8; 32]).unwrap(),
        domain_separated_message(b"ic-request-auth-delegation", [7; 32]).unwrap(),
    ] {
        assert!(matches!(
            verify_canister_signature(&message, &proof, &policy(&root)),
            Err(Error::InvalidSignature(_))
        ));
    }
    let mut envelope: Envelope = serde_cbor::from_slice(&proof.signature_cbor).unwrap();
    envelope.tree = leaf(Vec::new());
    proof.signature_cbor = cbor(&envelope);
    assert!(matches!(
        verify_canister_signature(&message(), &proof, &policy(&root)),
        Err(Error::InvalidSignature(_))
    ));
}

#[test]
fn rejects_tampered_certificate_and_nonempty_signature_leaf() {
    let root = key(1).public_key().serialize();
    let original = proof(&message(), Some(time_bytes(NOW)), &key(1), None);
    let mut proof = original.clone();
    let mut envelope: Envelope = serde_cbor::from_slice(&proof.signature_cbor).unwrap();
    let mut certificate: Certificate = serde_cbor::from_slice(&envelope.certificate).unwrap();
    certificate.signature[0] ^= 1;
    envelope.certificate = ByteBuf::from(cbor(&certificate));
    proof.signature_cbor = cbor(&envelope);
    assert!(matches!(
        verify_canister_signature(&message(), &proof, &policy(&root)),
        Err(Error::InvalidSignature(_))
    ));

    // Even a correctly BLS-signed certificate cannot authenticate a nonempty leaf.
    let tree = labeled(
        b"sig".to_vec(),
        labeled(hash(SEED), labeled(hash(&message()), leaf(vec![1]))),
    );
    let cert_tree = fork(
        labeled(
            b"canister".to_vec(),
            labeled(
                signer().as_slice().to_vec(),
                labeled(b"certified_data".to_vec(), leaf(tree.digest())),
            ),
        ),
        labeled(b"time".to_vec(), leaf(time_bytes(NOW))),
    );
    proof.signature_cbor = cbor(&Envelope {
        certificate: ByteBuf::from(cbor(&crate::certificate(cert_tree, &key(1), None))),
        tree,
    });
    assert!(matches!(
        verify_canister_signature(&message(), &proof, &policy(&root)),
        Err(Error::InvalidSignature(_))
    ));
}

#[test]
fn rejects_every_truncated_key_and_inconsistent_der_without_panicking() {
    let root = key(1).public_key().serialize();
    let original = proof(&message(), Some(time_bytes(NOW)), &key(1), None);
    for length in 0..original.public_key_der.len() {
        let mut proof = original.clone();
        proof.public_key_der.truncate(length);
        assert_eq!(
            verify_canister_signature(&message(), &proof, &policy(&root)),
            Err(Error::InvalidPublicKey)
        );
    }
    for offset in [0, 1, 2, 4, 6, 16, 17, 18] {
        let mut proof = original.clone();
        proof.public_key_der[offset] ^= 1;
        assert_eq!(
            verify_canister_signature(&message(), &proof, &policy(&root)),
            Err(Error::InvalidPublicKey)
        );
    }
    let mut proof = original.clone();
    proof.public_key_der[19] = 255;
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy(&root)),
        Err(Error::InvalidPublicKey)
    );
    let mut proof = original.clone();
    proof.public_key_der.push(0);
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy(&root)),
        Err(Error::InvalidPublicKey)
    );
    proof.public_key_der = CanisterSigPublicKey::new(signer(), vec![0; 128]).to_der();
    assert_eq!(
        verify_canister_signature(&message(), &proof, &policy(&root)),
        Err(Error::InvalidPublicKey)
    );
}

#[test]
fn rejects_malformed_cbor_and_missing_self_describing_tag() {
    let root = key(1).public_key().serialize();
    let mut proof = proof(&message(), Some(time_bytes(NOW)), &key(1), None);
    let original = proof.signature_cbor.clone();
    for bytes in [vec![], vec![0xd9, 0xd9, 0xf7, 0xff], original[3..].to_vec()] {
        proof.signature_cbor = bytes;
        assert!(matches!(
            verify_canister_signature(&message(), &proof, &policy(&root)),
            Err(Error::InvalidSignature(_))
        ));
    }
}

fn subnet_delegation(sharded: bool, includes_signer: bool, nested: bool) -> Delegation {
    let subnet_id = vec![3; 10];
    let subnet_key = key(2).public_key().serialize();
    let mut subnet_key_der = IC_ROOT_PK_DER_PREFIX.to_vec();
    subnet_key_der.extend_from_slice(&subnet_key);
    let range = if includes_signer {
        signer()
    } else {
        Principal::from_slice(&[2; 10])
    };
    let range_bytes = serde_cbor::to_vec(&vec![(range, range)]).unwrap();
    let tree = if sharded {
        fork(
            labeled(
                b"canister_ranges".to_vec(),
                labeled(
                    subnet_id.clone(),
                    labeled(range.as_slice().to_vec(), leaf(range_bytes)),
                ),
            ),
            labeled(
                b"subnet".to_vec(),
                labeled(
                    subnet_id.clone(),
                    labeled(b"public_key".to_vec(), leaf(subnet_key_der)),
                ),
            ),
        )
    } else {
        labeled(
            b"subnet".to_vec(),
            labeled(
                subnet_id.clone(),
                fork(
                    labeled(b"canister_ranges".to_vec(), leaf(range_bytes)),
                    labeled(b"public_key".to_vec(), leaf(subnet_key_der)),
                ),
            ),
        )
    };
    // Root delegation certificates need not have a recent /time leaf. They are
    // trust-chain material; the leaf signing certificate must still be fresh.
    let nested = nested.then(|| Delegation {
        subnet_id: vec![],
        certificate: vec![],
    });
    Delegation {
        subnet_id,
        certificate: cbor(&certificate(tree, &key(1), nested)),
    }
}

#[test]
fn verifies_upstream_subnet_range_formats_and_rejects_unapproved_or_nested_delegation() {
    let root = key(1).public_key().serialize();
    for sharded in [false, true] {
        let proof = proof(
            &message(),
            Some(time_bytes(NOW)),
            &key(2),
            Some(subnet_delegation(sharded, true, false)),
        );
        assert_eq!(
            verify_canister_signature(&message(), &proof, &policy(&root)),
            Ok(())
        );
        let proof = crate::proof(
            &message(),
            Some(time_bytes(NOW)),
            &key(2),
            Some(subnet_delegation(sharded, false, false)),
        );
        assert!(matches!(
            verify_canister_signature(&message(), &proof, &policy(&root)),
            Err(Error::InvalidSignature(_))
        ));
    }
    let proof = proof(
        &message(),
        Some(time_bytes(NOW)),
        &key(2),
        Some(subnet_delegation(true, true, true)),
    );
    assert!(matches!(
        verify_canister_signature(&message(), &proof, &policy(&root)),
        Err(Error::InvalidSignature(_))
    ));
}
