//! Real certificates assembled without Auth's message builder or verifier.
//! See the package README for the pinned contract provenance and scope.

use ic_auth::issuer_signature::{
    IssuerSignatureError as Error, IssuerSignaturePolicy, certified_signature_message,
    verify_issuer_signature,
};
use ic_auth_protocol_types::Principal;
use ic_platform::{
    certificate::construction::{
        Certificate, Delegation, HashTree, encode_canister_signature, encode_certificate, fork,
        labeled, leaf,
    },
    fixture_signing::PrivateKey,
};
use sha2::{Digest, Sha256};

const SEED: &[u8] = b"auth-qualification-issuer";
const DIGEST: [u8; 32] = [7; 32];
const NOW: u64 = 100;

fn issuer() -> Principal {
    Principal::from_slice(&[1; 10])
}

fn hash(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn key(number: u8) -> PrivateKey {
    let mut scalar = [0; 32];
    scalar[31] = number;
    PrivateKey::deserialize(&scalar).unwrap()
}

fn message() -> Vec<u8> {
    // Independent literal framing; never obtain fixture bytes from Auth.
    [b"\x1bic-auth-portable-credential".as_slice(), &DIGEST].concat()
}

fn certificate(tree: HashTree, signer: &PrivateKey, delegation: Option<Delegation>) -> Certificate {
    let signed = [b"\x0Dic-state-root".as_slice(), &tree.digest()].concat();
    Certificate {
        tree,
        signature: signer.sign(&signed).serialize().to_vec(),
        delegation,
    }
}

struct Fixture {
    certificate: Certificate,
    witness: HashTree,
    der: Vec<u8>,
}
impl Fixture {
    fn new(
        message: &[u8],
        time: Option<Vec<u8>>,
        signer: &PrivateKey,
        delegation: Option<Delegation>,
    ) -> Self {
        Self::with_leaf(message, time, signer, delegation, vec![])
    }

    fn with_leaf(
        message: &[u8],
        time: Option<Vec<u8>>,
        signer: &PrivateKey,
        delegation: Option<Delegation>,
        value: Vec<u8>,
    ) -> Self {
        let witness = labeled(
            b"sig".to_vec(),
            labeled(hash(SEED), labeled(hash(message), leaf(value))),
        );
        let data = labeled(
            b"canister".to_vec(),
            labeled(
                issuer().as_slice().to_vec(),
                labeled(b"certified_data".to_vec(), leaf(witness.digest())),
            ),
        );
        let tree = match time {
            Some(time) => fork(data, labeled(b"time".to_vec(), leaf(time))),
            None => data,
        };
        Self {
            certificate: certificate(tree, signer, delegation),
            witness,
            der: ic_platform::certificate::construction::canister_key_der(issuer(), SEED).unwrap(),
        }
    }

    fn bytes(&self) -> Vec<u8> {
        let cert = encode_certificate(&self.certificate, 16_384).unwrap();
        encode_canister_signature(&cert, &self.witness, 16_384).unwrap()
    }

    fn verify(&self, policy: &IssuerSignaturePolicy<'_>) -> Result<u64, Error> {
        verify_issuer_signature(DIGEST, &self.bytes(), &self.der, policy)
    }
}

fn direct() -> Fixture {
    Fixture::new(&message(), Some(vec![100]), &key(1), None)
}

fn policy(root: &[u8]) -> IssuerSignaturePolicy<'_> {
    IssuerSignaturePolicy {
        signing_canister: issuer(),
        seed_hash: hash(SEED),
        ic_root_public_key_raw: root,
        now_ns: NOW,
        max_certificate_age_ns: 10,
        max_signature_bytes: 16_384,
    }
}

#[test]
fn literal_domain_and_host_size_boundary() {
    assert_eq!(certified_signature_message(DIGEST).as_slice(), message());
    let root = key(1).public_key().serialize();
    let proof = direct();
    let mut policy = policy(&root);
    policy.max_signature_bytes = proof.bytes().len();
    assert_eq!(proof.verify(&policy), Ok(NOW));
    policy.max_signature_bytes -= 1;
    assert_eq!(proof.verify(&policy), Err(Error::SignatureTooLarge));
    policy.max_signature_bytes = usize::MAX;
    // Profile cap precedes all parsing, including malformed DER.
    assert_eq!(
        verify_issuer_signature(DIGEST, &vec![0; 131_073], &[], &policy),
        Err(Error::SignatureTooLarge)
    );
    assert_eq!(
        verify_issuer_signature(DIGEST, &vec![0; 131_072], &[], &policy),
        Err(Error::InvalidPublicKey)
    );
}

#[test]
fn protected_issuer_seed_and_network_cannot_be_selected_by_proof() {
    let root = key(1).public_key().serialize();
    let proof = direct();
    let mut policy = policy(&root);
    policy.signing_canister = Principal::from_slice(&[2; 10]);
    assert_eq!(proof.verify(&policy), Err(Error::CanisterMismatch));
    policy.signing_canister = issuer();
    policy.seed_hash = [0; 32];
    assert_eq!(proof.verify(&policy), Err(Error::SeedMismatch));
    policy.seed_hash = hash(SEED);
    let wrong_root = key(2).public_key().serialize();
    policy.ic_root_public_key_raw = &wrong_root;
    assert_eq!(proof.verify(&policy), Err(Error::InvalidSignature));
    for length in [0, 95, 97] {
        let malformed = vec![0; length];
        let invalid = IssuerSignaturePolicy {
            ic_root_public_key_raw: &malformed,
            ..policy
        };
        assert_eq!(proof.verify(&invalid), Err(Error::InvalidRootKeyLength));
    }
}

#[test]
fn signing_time_has_inclusive_age_and_no_future_grace() {
    let root = key(1).public_key().serialize();
    let proof = direct();
    let mut policy = policy(&root);
    for (now, expected) in [
        (99, Err(Error::CertificateInFuture)),
        (100, Ok(100)),
        (110, Ok(100)),
        (111, Err(Error::CertificateTooOld)),
        (u64::MAX, Err(Error::CertificateTooOld)),
    ] {
        policy.now_ns = now;
        assert_eq!(proof.verify(&policy), expected);
    }
    // Fixed complete LEB128 of u64::MAX, without a replacement time encoder.
    let maximum = Fixture::new(
        &message(),
        Some(vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 1]),
        &key(1),
        None,
    );
    assert_eq!(maximum.verify(&policy), Ok(u64::MAX));
}

#[test]
fn malformed_authenticated_time_refuses_but_complete_nonminimal_time_survives() {
    let root = key(1).public_key().serialize();
    for time in [
        None,
        Some(vec![]),
        Some(vec![128]),
        Some(vec![255; 10]),
        Some(vec![128; 11]),
        Some(vec![100, 0]),
    ] {
        let mut proof = Fixture::new(&message(), time, &key(1), None);
        assert_eq!(
            proof.verify(&policy(&root)),
            Err(Error::InvalidCertificateTime)
        );
        proof.certificate.signature[0] ^= 1;
        // Time is never interpreted as authenticated before BLS succeeds.
        assert_eq!(proof.verify(&policy(&root)), Err(Error::InvalidSignature));
    }
    let nonminimal = Fixture::new(&message(), Some(vec![228, 0]), &key(1), None);
    assert_eq!(nonminimal.verify(&policy(&root)), Ok(NOW));
}

#[test]
fn wrong_digest_domain_witness_and_nonempty_leaf_refuse() {
    let root = key(1).public_key().serialize();
    let mut proof = direct();
    assert_eq!(
        verify_issuer_signature([8; 32], &proof.bytes(), &proof.der, &policy(&root)),
        Err(Error::InvalidSignature)
    );
    let wrong_domain = Fixture::new(
        &[b"\x19ic-request-auth-delegation".as_slice(), &DIGEST].concat(),
        Some(vec![100]),
        &key(1),
        None,
    );
    assert_eq!(
        wrong_domain.verify(&policy(&root)),
        Err(Error::InvalidSignature)
    );
    proof.witness = leaf(Vec::new());
    assert_eq!(proof.verify(&policy(&root)), Err(Error::InvalidSignature));
    let nonempty = Fixture::with_leaf(&message(), Some(vec![100]), &key(1), None, vec![1]);
    assert_eq!(
        nonempty.verify(&policy(&root)),
        Err(Error::InvalidSignature)
    );
}

#[test]
fn malformed_key_and_cbor_refuse_without_panics() {
    let root = key(1).public_key().serialize();
    let proof = direct();
    let bytes = proof.bytes();
    for length in 0..proof.der.len() {
        assert_eq!(
            verify_issuer_signature(DIGEST, &bytes, &proof.der[..length], &policy(&root)),
            Err(Error::InvalidPublicKey)
        );
    }
    for bad_der in [vec![0; 130], [proof.der.as_slice(), &[0]].concat()] {
        assert_eq!(
            verify_issuer_signature(DIGEST, &bytes, &bad_der, &policy(&root)),
            Err(Error::InvalidPublicKey)
        );
    }
    for malformed in [
        vec![],
        vec![0xd9, 0xd9, 0xf7, 0xff],
        bytes[3..].to_vec(),
        bytes[..bytes.len() - 1].to_vec(),
    ] {
        assert_eq!(
            verify_issuer_signature(DIGEST, &malformed, &proof.der, &policy(&root)),
            Err(Error::InvalidSignature)
        );
    }
}

#[test]
fn identity_bls_points_cannot_authenticate_an_arbitrary_tree() {
    let mut proof = direct();
    proof.certificate.signature = vec![0; 48];
    proof.certificate.signature[0] = 0xc0;
    let mut root = [0; 96];
    root[0] = 0xc0;
    assert_eq!(proof.verify(&policy(&root)), Err(Error::InvalidSignature));
}

fn delegation(
    sharded: bool,
    range: Principal,
    nested: bool,
    time: Option<&[u8]>,
    subnet_type: bool,
) -> Delegation {
    let subnet_id = vec![3; 10];
    let der = ic_platform::certificate::PreparedRootKey::from_raw(&key(2).public_key().serialize())
        .unwrap()
        .as_der()
        .to_vec();
    // Independent CBOR: one pair of exact principal byte strings.
    let mut ranges = vec![0x81, 0x82];
    for _ in 0..2 {
        ranges.push(0x40 + u8::try_from(range.as_slice().len()).unwrap());
        ranges.extend_from_slice(range.as_slice());
    }
    let public_key = labeled(b"public_key".to_vec(), leaf(der));
    let public_key = if subnet_type {
        fork(
            public_key,
            labeled(b"type".to_vec(), leaf(b"application".to_vec())),
        )
    } else {
        public_key
    };
    let tree = if sharded {
        fork(
            labeled(
                b"canister_ranges".to_vec(),
                labeled(
                    subnet_id.clone(),
                    labeled(range.as_slice().to_vec(), leaf(ranges)),
                ),
            ),
            labeled(b"subnet".to_vec(), labeled(subnet_id.clone(), public_key)),
        )
    } else {
        labeled(
            b"subnet".to_vec(),
            labeled(
                subnet_id.clone(),
                fork(
                    labeled(b"canister_ranges".to_vec(), leaf(ranges)),
                    public_key,
                ),
            ),
        )
    };
    // Auth permits an older delegation; the supplier still requires a valid
    // authenticated time and explicit certified subnet type.
    let tree = match time {
        Some(bytes) => fork(tree, labeled(b"time".to_vec(), leaf(bytes.to_vec()))),
        None => tree,
    };
    let cert = certificate(
        tree,
        &key(1),
        nested.then(|| Delegation {
            subnet_id: vec![],
            certificate: vec![],
        }),
    );
    Delegation {
        subnet_id,
        certificate: encode_certificate(&cert, 16_384).unwrap(),
    }
}

#[test]
fn subnet_range_formats_bind_issuer_and_refuse_nested_delegation() {
    let root = key(1).public_key().serialize();
    for sharded in [false, true] {
        for (range, nested, expected) in [
            (issuer(), false, Ok(NOW)),
            (
                Principal::from_slice(&[2; 10]),
                false,
                Err(Error::InvalidSignature),
            ),
            (issuer(), true, Err(Error::InvalidSignature)),
        ] {
            let proof = Fixture::new(
                &message(),
                Some(vec![100]),
                &key(2),
                Some(delegation(sharded, range, nested, Some(&[1]), true)),
            );
            assert_eq!(proof.verify(&policy(&root)), expected);
        }
    }
}

#[test]
fn delegation_requires_authenticated_time_and_subnet_type() {
    let root = key(1).public_key().serialize();
    for (time, subnet_type, expected) in [
        (None, true, Error::InvalidCertificateTime),
        (Some([0x80].as_slice()), true, Error::InvalidCertificateTime),
        (Some([1].as_slice()), false, Error::InvalidSignature),
    ] {
        let proof = Fixture::new(
            &message(),
            Some(vec![100]),
            &key(2),
            Some(delegation(false, issuer(), false, time, subnet_type)),
        );
        assert_eq!(proof.verify(&policy(&root)), Err(expected));
    }
}
