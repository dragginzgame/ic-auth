#![cfg(feature = "canister-signature-preparation")]

use ic_auth::signature_store::{
    MAX_COMPOSITION_DEPTH, MAX_PRUNE_BATCH, MAX_RETENTION_NS, MAX_SIGNATURES, SignatureInputs,
    SignatureLimits, SignatureSibling, SignatureStore, SignatureStoreError as Error,
};
use ic_auth_protocol_types::Principal;
use ic_canister_sig_creation::{CanisterSigPublicKey, hash_bytes, hash_with_domain};
use ic_certification::{Certificate, Hash, HashTree, fork, fork_hash, labeled, leaf};
use ic_verify_bls_signature::PrivateKey;
use serde::Serialize;

const NOW: u64 = 10;
const SEED: &[u8] = b"canic-issuer-delegated-token";
const MESSAGE: [u8; 32] = [7; 32];

fn signer() -> Principal {
    Principal::from_slice(&[1; 10])
}

fn limits() -> SignatureLimits {
    SignatureLimits {
        max_signatures: MAX_SIGNATURES,
        max_message_bytes: 1024,
        max_certificate_bytes: 64 * 1024,
        max_signature_bytes: 128 * 1024,
    }
}

fn store() -> SignatureStore {
    SignatureStore::new(signer(), limits()).unwrap()
}

fn inputs() -> SignatureInputs<'static> {
    SignatureInputs {
        seed: SEED,
        domain: SEED,
        message: &MESSAGE,
    }
}

fn key() -> PrivateKey {
    let mut bytes = [0; 32];
    bytes[31] = 1;
    PrivateKey::deserialize(&bytes).unwrap()
}

fn cbor<T: Serialize>(value: &T) -> Vec<u8> {
    let mut serializer = serde_cbor::Serializer::new(Vec::new());
    serializer.self_describe().unwrap();
    value.serialize(&mut serializer).unwrap();
    serializer.into_inner()
}

fn certificate(canister: Principal, root: Hash) -> Vec<u8> {
    let tree = fork(
        labeled(
            b"canister",
            labeled(canister.as_slice(), labeled(b"certified_data", leaf(root))),
        ),
        labeled(b"time", leaf(vec![NOW as u8])),
    );
    let mut message = b"\x0Dic-state-root".to_vec();
    message.extend_from_slice(&tree.digest());
    cbor(&Certificate {
        tree,
        signature: key().sign(&message).serialize().to_vec(),
        delegation: None,
    })
}

fn expected_leaf(input: SignatureInputs<'_>) -> HashTree {
    labeled(
        b"sig",
        labeled(
            hash_bytes(input.seed),
            labeled(
                hash_with_domain(input.domain, input.message),
                leaf(Vec::new()),
            ),
        ),
    )
}

#[test]
fn both_canic_domains_keep_the_exact_certified_path_and_der_key() {
    for domain in [SEED, b"canic-root-role-attestation".as_slice()] {
        let mut store = store();
        let input = SignatureInputs {
            seed: domain,
            domain,
            message: &MESSAGE,
        };
        let prepared = store.prepare(input, NOW, MAX_RETENTION_NS).unwrap();
        assert_eq!(prepared.retrieval_expires_at_ns, NOW + MAX_RETENTION_NS);
        assert_eq!(store.root_hash(), expected_leaf(input).digest());
        let proof = store
            .retrieve(input, NOW, &certificate(signer(), store.root_hash()), &[])
            .unwrap();
        assert_eq!(
            proof.public_key_der,
            CanisterSigPublicKey::new(signer(), domain.to_vec()).to_der()
        );
        assert_eq!(&proof.signature_cbor[..3], &[0xd9, 0xd9, 0xf7]);
    }
}

#[test]
fn live_retries_do_not_extend_the_deadline_and_expired_keys_can_be_reprepared() {
    let mut store = store();
    let first = store.prepare(inputs(), NOW, 20).unwrap();
    assert_eq!(store.prepare(inputs(), NOW + 1, 60).unwrap(), first);
    let cert = certificate(signer(), first.signature_root);
    assert!(store.retrieve(inputs(), NOW + 19, &cert, &[]).is_ok());
    assert_eq!(
        store.retrieve(inputs(), NOW + 20, &cert, &[]),
        Err(Error::Expired)
    );
    let next = store.prepare(inputs(), NOW + 20, 50).unwrap();
    assert_eq!(next.retrieval_expires_at_ns, NOW + 70);
    assert_eq!(next.signature_root, first.signature_root);
    assert_eq!(store.len(), 1);
    assert_eq!(store.prune(NOW + 20, 128), Ok(0));
    assert_eq!(store.prune(NOW + 70, 128), Ok(1));
    assert_eq!(
        store.retrieve(inputs(), NOW + 70, &cert, &[]),
        Err(Error::NotPrepared)
    );
}

#[test]
fn capacity_failure_does_not_prune_or_change_the_root() {
    let mut limit = limits();
    limit.max_signatures = 1;
    let mut store = SignatureStore::new(signer(), limit).unwrap();
    store.prepare(inputs(), NOW, 1).unwrap();
    let root = store.root_hash();
    let other = SignatureInputs {
        message: b"other",
        ..inputs()
    };
    assert_eq!(store.prepare(other, NOW + 1, 1), Err(Error::Capacity));
    assert_eq!(store.len(), 1);
    assert_eq!(store.root_hash(), root);
    assert_eq!(store.prune(NOW + 1, 0), Ok(0));
    assert_eq!(store.prune(NOW + 1, 1), Ok(1));
    assert!(store.prepare(other, NOW + 1, 1).is_ok());
}

#[test]
fn cleanup_is_bounded_and_does_not_delete_refreshed_or_live_keys() {
    let mut store = store();
    for index in 0..140_u32 {
        store
            .prepare(
                SignatureInputs {
                    seed: &index.to_be_bytes(),
                    ..inputs()
                },
                NOW,
                1,
            )
            .unwrap();
    }
    store.prepare(inputs(), NOW + 1, 100).unwrap();
    assert_eq!(
        store.prune(NOW + 1, MAX_PRUNE_BATCH + 1),
        Err(Error::InvalidPruneBudget)
    );
    assert_eq!(store.len(), 141);
    assert_eq!(store.prune(NOW + 1, MAX_PRUNE_BATCH), Ok(128));
    assert_eq!(store.len(), 13);
    assert_eq!(store.prune(NOW + 1, MAX_PRUNE_BATCH), Ok(12));
    let cert = certificate(signer(), store.root_hash());
    assert!(store.retrieve(inputs(), NOW + 1, &cert, &[]).is_ok());
    assert_eq!(store.prune(NOW + 101, 128), Ok(1));
    assert_eq!(store.root_hash(), crate::store().root_hash());
}

#[test]
fn removing_one_leaf_preserves_other_messages_and_seeds() {
    let mut store = store();
    let same_seed = SignatureInputs {
        message: b"another payload",
        ..inputs()
    };
    let other_seed = SignatureInputs {
        seed: b"another seed",
        ..inputs()
    };
    for input in [inputs(), same_seed, other_seed] {
        store.prepare(input, NOW, 100).unwrap();
    }
    assert_eq!(store.remove(inputs(), NOW), Ok(true));
    assert_eq!(store.remove(inputs(), NOW), Ok(false));
    let cert = certificate(signer(), store.root_hash());
    assert_eq!(
        store.retrieve(inputs(), NOW, &cert, &[]),
        Err(Error::NotPrepared)
    );
    for input in [same_seed, other_seed] {
        let cert = certificate(signer(), store.root_hash());
        assert!(store.retrieve(input, NOW, &cert, &[]).is_ok());
        assert_eq!(store.remove(input, NOW), Ok(true));
    }
    assert!(store.is_empty());
    assert_eq!(store.prune(NOW + 100, 128), Ok(0));
}

#[test]
fn input_retention_overflow_and_clock_rejections_leave_state_unchanged() {
    let mut store = store();
    store.prepare(inputs(), NOW, 1).unwrap();
    let root = store.root_hash();
    for (input, expected) in [
        (
            SignatureInputs {
                seed: &[0; 33],
                ..inputs()
            },
            Error::SeedTooLong,
        ),
        (
            SignatureInputs {
                domain: &[0; 256],
                ..inputs()
            },
            Error::DomainTooLong,
        ),
        (
            SignatureInputs {
                message: &[0; 1025],
                ..inputs()
            },
            Error::MessageTooLarge,
        ),
    ] {
        assert_eq!(store.prepare(input, NOW, 1), Err(expected));
    }
    for retention in [0, MAX_RETENTION_NS + 1] {
        assert_eq!(
            store.prepare(inputs(), NOW, retention),
            Err(Error::InvalidRetention)
        );
    }
    assert_eq!(
        store.prepare(inputs(), u64::MAX, 1),
        Err(Error::DeadlineOverflow)
    );
    assert_eq!(
        store.prepare(inputs(), NOW - 1, 1),
        Err(Error::ClockRegressed)
    );
    assert_eq!(store.prune(NOW - 1, 1), Err(Error::ClockRegressed));
    assert_eq!(store.remove(inputs(), NOW - 1), Err(Error::ClockRegressed));
    assert_eq!(
        store.retrieve(inputs(), NOW - 1, &[], &[]),
        Err(Error::ClockRegressed)
    );
    assert_eq!(store.len(), 1);
    assert_eq!(store.root_hash(), root);
    assert_eq!(store.prune(NOW + 1, 1), Ok(1));
    assert_eq!(store.prepare(inputs(), NOW, 1), Err(Error::ClockRegressed));
}

#[test]
fn constructor_and_exact_input_size_boundaries_are_checked() {
    assert!(matches!(
        SignatureStore::new(Principal::anonymous(), limits()),
        Err(Error::AnonymousSigner)
    ));
    for value in [0, MAX_SIGNATURES + 1] {
        assert!(matches!(
            SignatureStore::new(
                signer(),
                SignatureLimits {
                    max_signatures: value,
                    ..limits()
                }
            ),
            Err(Error::InvalidLimits)
        ));
    }
    for field in 0..3 {
        for value in [0, 128 * 1024 + 1] {
            let mut limit = limits();
            match field {
                0 => limit.max_message_bytes = value,
                1 => limit.max_certificate_bytes = value,
                _ => limit.max_signature_bytes = value,
            }
            assert!(matches!(
                SignatureStore::new(signer(), limit),
                Err(Error::InvalidLimits)
            ));
        }
    }
    let mut store = store();
    let input = SignatureInputs {
        seed: &[0; 32],
        domain: &[0; 255],
        message: &[0; 1024],
    };
    assert!(store.prepare(input, u64::MAX - 1, 1).is_ok());
    let cert = certificate(signer(), store.root_hash());
    assert!(store.retrieve(input, u64::MAX - 1, &cert, &[]).is_ok());
    assert_eq!(
        store.retrieve(input, u64::MAX, &cert, &[]),
        Err(Error::Expired)
    );
}

#[test]
fn retrieval_rejects_wrong_payload_missing_bad_or_stale_certification() {
    let mut store = store();
    store.prepare(inputs(), NOW, 100).unwrap();
    let original = certificate(signer(), store.root_hash());
    for input in [
        SignatureInputs {
            seed: b"other",
            ..inputs()
        },
        SignatureInputs {
            domain: b"other",
            ..inputs()
        },
        SignatureInputs {
            message: b"other",
            ..inputs()
        },
    ] {
        assert_eq!(
            store.retrieve(input, NOW, &original, &[]),
            Err(Error::NotPrepared)
        );
    }
    assert_eq!(
        store.retrieve(inputs(), NOW, &[], &[]),
        Err(Error::NoCertificate)
    );
    for bad in [&[0xff][..], &[0xa0], &[0x80], &[0xd9, 0xd9, 0xf7, 0xff]] {
        assert_eq!(
            store.retrieve(inputs(), NOW, bad, &[]),
            Err(Error::InvalidCertificate)
        );
    }
    let wrong_signer = certificate(Principal::from_slice(&[2; 10]), store.root_hash());
    assert_eq!(
        store.retrieve(inputs(), NOW, &wrong_signer, &[]),
        Err(Error::CertificateRootMismatch)
    );
    store
        .prepare(
            SignatureInputs {
                message: b"new message",
                ..inputs()
            },
            NOW,
            100,
        )
        .unwrap();
    assert_eq!(
        store.retrieve(inputs(), NOW, &original, &[]),
        Err(Error::CertificateRootMismatch)
    );
    assert!(
        store
            .retrieve(
                inputs(),
                NOW,
                &certificate(signer(), store.root_hash()),
                &[]
            )
            .is_ok()
    );
}

#[test]
fn certificate_output_and_composition_limits_are_inclusive() {
    let mut original = store();
    original.prepare(inputs(), NOW, 100).unwrap();
    let cert = certificate(signer(), original.root_hash());
    let proof = original.retrieve(inputs(), NOW, &cert, &[]).unwrap();
    let mut exact = limits();
    exact.max_certificate_bytes = cert.len();
    exact.max_signature_bytes = proof.signature_cbor.len();
    let mut store = SignatureStore::new(signer(), exact).unwrap();
    store.prepare(inputs(), NOW, 100).unwrap();
    assert_eq!(store.retrieve(inputs(), NOW, &cert, &[]), Ok(proof));
    for (limit, error) in [
        (
            SignatureLimits {
                max_certificate_bytes: cert.len() - 1,
                ..exact
            },
            Error::CertificateTooLarge,
        ),
        (
            SignatureLimits {
                max_signature_bytes: exact.max_signature_bytes - 1,
                ..exact
            },
            Error::SignatureTooLarge,
        ),
    ] {
        let mut store = SignatureStore::new(signer(), limit).unwrap();
        store.prepare(inputs(), NOW, 100).unwrap();
        assert_eq!(store.retrieve(inputs(), NOW, &cert, &[]), Err(error));
    }
    let siblings = [SignatureSibling::Left([3; 32]); MAX_COMPOSITION_DEPTH];
    let composed = siblings
        .iter()
        .fold(original.root_hash(), |root, _| fork_hash(&[3; 32], &root));
    assert!(
        original
            .retrieve(inputs(), NOW, &certificate(signer(), composed), &siblings)
            .is_ok()
    );
    assert_eq!(
        store.retrieve(
            inputs(),
            NOW,
            &cert,
            &[SignatureSibling::Left([3; 32]); MAX_COMPOSITION_DEPTH + 1]
        ),
        Err(Error::CompositionTooDeep)
    );
}

#[cfg(feature = "canister-signature-verification")]
#[test]
fn composed_witnesses_verify_with_real_bls_and_protected_receiver_policy() {
    use ic_auth::canister_signature::{
        CanisterSignaturePolicy, domain_separated_message, verify_canister_signature,
    };
    let mut store = store();
    for domain in [SEED, b"canic-root-role-attestation".as_slice()] {
        store
            .prepare(
                SignatureInputs {
                    seed: domain,
                    domain,
                    message: &MESSAGE,
                },
                NOW,
                100,
            )
            .unwrap();
    }
    // Host-owned labels surround /sig in canonical order. The library sees only
    // their root hashes and never replaces either branch or publishes a root.
    let assets = labeled(b"http_expr", leaf(b"assets".to_vec())).digest();
    let status = labeled(b"status", leaf(b"status".to_vec())).digest();
    let siblings = [
        SignatureSibling::Left(assets),
        SignatureSibling::Right(status),
    ];
    let root = fork_hash(&fork_hash(&assets, &store.root_hash()), &status);
    let cert = certificate(signer(), root);
    let network = key().public_key().serialize();
    for domain in [SEED, b"canic-root-role-attestation".as_slice()] {
        let input = SignatureInputs {
            seed: domain,
            domain,
            message: &MESSAGE,
        };
        let proof = store.retrieve(input, NOW, &cert, &siblings).unwrap();
        let policy = CanisterSignaturePolicy {
            signing_canister: signer(),
            seed_hash: hash_bytes(domain),
            ic_root_public_key_raw: &network,
            now_ns: NOW,
            max_certificate_age_ns: 0,
            max_future_skew_ns: 0,
            max_signature_bytes: proof.signature_cbor.len(),
            max_message_bytes: 288,
        };
        assert_eq!(
            verify_canister_signature(
                &domain_separated_message(domain, MESSAGE).unwrap(),
                &proof,
                &policy
            ),
            Ok(())
        );
        assert!(
            verify_canister_signature(
                &domain_separated_message(domain, [8; 32]).unwrap(),
                &proof,
                &policy
            )
            .is_err()
        );
        let mut other = policy.clone();
        other.ic_root_public_key_raw = &network[..95];
        assert!(
            verify_canister_signature(
                &domain_separated_message(domain, MESSAGE).unwrap(),
                &proof,
                &other
            )
            .is_err()
        );
    }
    assert_eq!(
        store.retrieve(inputs(), NOW, &cert, &[]),
        Err(Error::CertificateRootMismatch)
    );
    assert_eq!(
        store.retrieve(
            inputs(),
            NOW,
            &cert,
            &[
                SignatureSibling::Right(assets),
                SignatureSibling::Left(status)
            ]
        ),
        Err(Error::CertificateRootMismatch)
    );
}
