//! Independent OpenSSL-signed root proof; no Auth encoder constructs the fixture.
use ic_auth::root_delegation::{
    DelegationError as E, DelegationLimits, RootKeyPolicy, ValidityTarget as W,
    verify_root_delegation,
};
use ic_auth_protocol_types::{CanonicalId, MerkleStep, Principal};
mod support;
use root_fixture::{Fixture, issuer, limits, policy, root};
use support::root as root_fixture;
const RECORDS: &str = include_str!("fixtures/root-delegation.tsv");
fn bytes(name: &str) -> Vec<u8> {
    root_fixture::bytes(RECORDS, name)
}

fn hash(name: &str) -> [u8; 32] {
    bytes(name).try_into().unwrap()
}

fn result(
    f: &Fixture,
    policy: &RootKeyPolicy<'_>,
    now: u64,
    limits: DelegationLimits,
) -> Result<[u8; 32], E> {
    verify_root_delegation(f.proof(), issuer(), policy, now, limits).map(|v| v.certificate_hash())
}

#[test]
fn independent_root_signature_authenticates_certificate_and_caps_enrollment_expiry() {
    let f = Fixture::new(RECORDS, [8; 32]);
    let mut p = policy(&f.key);
    for now in [100, 150, 199] {
        let v = verify_root_delegation(f.proof(), issuer(), &p, now, limits()).unwrap();
        assert_eq!(v.certificate(), &f.cert);
        assert_eq!(v.certificate_hash(), hash("certificate_hash"));
        assert_eq!(v.expires_at_ns(), 200);
    }
    p.accept_until_ns = 180;
    assert_eq!(
        verify_root_delegation(f.proof(), issuer(), &p, 150, limits())
            .unwrap()
            .expires_at_ns(),
        180
    );
    assert_eq!(result(&f, &p, 180, limits()), Err(E::Expired(W::RootKey)));
}

#[test]
fn protected_selection_and_revocation_floors_are_checked_each_time() {
    let f = Fixture::new(RECORDS, [8; 32]);
    let key = bytes("public_key");
    assert!(verify_root_delegation(f.proof(), root(), &policy(&key), 150, limits()).is_err());
    type PolicyChange = for<'a> fn(&mut RootKeyPolicy<'a>);
    let changes: [(PolicyChange, E); 8] = [
        (|p| p.root = issuer(), E::IdentityMismatch),
        (|p| p.key_name = "other", E::RootKeyMismatch),
        (|p| p.derivation_path_hash[0] ^= 1, E::RootKeyMismatch),
        (|p| p.key_version = 5, E::RootKeyMismatch),
        (|p| p.min_key_version = 5, E::StaleAuthority),
        (|p| p.min_proof_epoch = 3, E::StaleAuthority),
        (|p| p.min_registry_epoch = 4, E::StaleAuthority),
        (|p| p.max_revocation_latency_ns = 99, E::BatchLifetime),
    ];
    for (change, error) in changes {
        let mut p = policy(&key);
        change(&mut p);
        assert_eq!(result(&f, &p, 150, limits()), Err(error));
    }
    let mut p = policy(&key);
    p.public_key = &[0; 33];
    assert_eq!(result(&f, &p, 150, limits()), Err(E::RootKeyMismatch));
}

#[test]
fn windows_and_lifetime_limits_are_exclusive_and_do_not_wrap() {
    let f = Fixture::new(RECORDS, [8; 32]);
    let key = bytes("public_key");
    let p = policy(&key);
    assert_eq!(
        result(&f, &p, 99, limits()),
        Err(E::NotYetValid(W::Certificate))
    );
    assert_eq!(
        result(&f, &p, 200, limits()),
        Err(E::Expired(W::Certificate))
    );
    assert_eq!(
        result(&f, &p, u64::MAX, limits()),
        Err(E::Expired(W::Certificate))
    );
    for change in [
        |l: &mut DelegationLimits| l.max_certificate_ttl_ns = 99,
        |l: &mut DelegationLimits| l.max_token_ttl_ns = 59,
    ] {
        let mut l = limits();
        change(&mut l);
        assert_eq!(result(&f, &p, 150, l), Err(E::CertificateLifetime));
    }
    for (start, end, error) in [
        (151, 210, E::NotYetValid(W::RootKey)),
        (90, 150, E::Expired(W::RootKey)),
        (210, 90, E::InvalidWindow(W::RootKey)),
    ] {
        let mut p = p;
        p.valid_from_ns = start;
        p.accept_until_ns = end;
        assert_eq!(result(&f, &p, 150, limits()), Err(error));
    }
    for (start, end, error) in [
        (151, 200, E::NotYetValid(W::Batch)),
        (90, 150, E::Expired(W::Batch)),
        (200, 100, E::InvalidWindow(W::Batch)),
        (101, 200, E::ProofBindingMismatch),
    ] {
        let mut f = Fixture::new(RECORDS, [8; 32]);
        f.header.not_before_ns = start;
        f.header.expires_at_ns = end;
        assert_eq!(result(&f, &p, 150, limits()), Err(error));
    }
    let mut f = Fixture::new(RECORDS, [8; 32]);
    f.cert.issued_at_ns = 101;
    assert_eq!(result(&f, &p, 150, limits()), Err(E::IssuedAfterStart));
    f.cert.issued_at_ns = 0;
    f.cert.delegation.expires_at_ns = 0;
    assert_eq!(
        result(&f, &p, 150, limits()),
        Err(E::InvalidWindow(W::Certificate))
    );
    for ttl in [0, 101, u64::MAX] {
        let mut f = Fixture::new(RECORDS, [8; 32]);
        f.cert.delegation.max_token_ttl_ns = ttl;
        assert_eq!(result(&f, &p, 150, limits()), Err(E::CertificateLifetime));
    }
}

#[test]
fn leaf_certificate_and_batch_must_agree_before_crypto() {
    let key = bytes("public_key");
    let p = policy(&key);
    let changes: [fn(&mut Fixture); 11] = [
        |f| {
            f.leaf.delegation.grants[0]
                .scopes
                .pop()
                .map(|_| ())
                .unwrap()
        },
        |f| f.leaf.delegation.issuer = root(),
        |f| f.leaf.delegation.audience.fleet_id = CanonicalId::from_bytes([5; 32]),
        |f| f.leaf.delegation.seed_hash[0] ^= 1,
        |f| f.leaf.delegation.binding_hash[0] ^= 1,
        |f| f.leaf.delegation.max_token_ttl_ns -= 1,
        |f| f.leaf.delegation.not_before_ns += 1,
        |f| f.leaf.proof_epoch += 1,
        |f| f.leaf.registry_epoch += 1,
        |f| f.leaf.registry_hash[0] ^= 1,
        |f| f.header.schema_version = 2,
    ];
    for change in changes {
        let mut f = Fixture::new(RECORDS, [8; 32]);
        change(&mut f);
        assert_eq!(result(&f, &p, 150, limits()), Err(E::ProofBindingMismatch));
    }
    let mut f = Fixture::new(RECORDS, [8; 32]);
    f.cert.delegation.binding_hash[0] ^= 1;
    assert_eq!(result(&f, &p, 150, limits()), Err(E::IssuerBindingMismatch));
    let mut f = Fixture::new(RECORDS, [8; 32]);
    f.cert.delegation.root = Principal::anonymous();
    assert_eq!(result(&f, &p, 150, limits()), Err(E::IdentityMismatch));
    let mut f = Fixture::new(RECORDS, [8; 32]);
    f.cert.delegation.grants[0].scopes.reverse();
    assert!(matches!(result(&f, &p, 150, limits()), Err(E::Encoding(_))));
}

#[test]
fn submitted_key_metadata_cannot_override_enrollment() {
    let key = bytes("public_key");
    let p = policy(&key);
    let mut f = Fixture::new(RECORDS, [8; 32]);
    f.name = "other".into();
    assert_eq!(
        result(&f, &p, 150, limits()),
        Err(E::SignatureBindingMismatch)
    );
    f.header.key_name = "other".into();
    assert_eq!(result(&f, &p, 150, limits()), Err(E::RootKeyMismatch));
    let mut f = Fixture::new(RECORDS, [8; 32]);
    f.path.reverse();
    assert_eq!(
        result(&f, &p, 150, limits()),
        Err(E::SignatureBindingMismatch)
    );
    let mut f = Fixture::new(RECORDS, [8; 32]);
    f.key[0] ^= 1;
    assert_eq!(result(&f, &p, 150, limits()), Err(E::RootKeyMismatch));
}

#[test]
fn witness_direction_order_and_signature_are_authenticated() {
    let key = bytes("public_key");
    let p = policy(&key);
    for steps in [
        vec![],
        vec![MerkleStep::Right([11; 32]), MerkleStep::Right([12; 32])],
        vec![MerkleStep::Right([12; 32]), MerkleStep::Left([11; 32])],
    ] {
        let mut f = Fixture::new(RECORDS, [8; 32]);
        f.steps = steps;
        assert_eq!(result(&f, &p, 150, limits()), Err(E::InvalidWitness));
    }
    let mut f = Fixture::new(RECORDS, [8; 32]);
    f.header.batch_id[0] ^= 1;
    assert_eq!(result(&f, &p, 150, limits()), Err(E::InvalidSignature));
    let mut f = Fixture::new(RECORDS, [8; 32]);
    f.signature = bytes("high_signature");
    assert_eq!(result(&f, &p, 150, limits()), Err(E::HighSignatureScalar));
    for length in [0, 63, 64] {
        let mut f = Fixture::new(RECORDS, [8; 32]);
        f.signature = vec![0; length];
        assert_eq!(result(&f, &p, 150, limits()), Err(E::InvalidSignature));
    }
    let mut f = Fixture::new(RECORDS, [8; 32]);
    f.key = vec![0; 33];
    let p = policy(&f.key);
    assert_eq!(result(&f, &p, 150, limits()), Err(E::InvalidSignature));
}

#[test]
fn variable_material_and_witness_budgets_refuse_before_hashing_or_signature() {
    let f = Fixture::new(RECORDS, [8; 32]);
    let key = bytes("public_key");
    let p = policy(&key);
    // Independent accounting: both grant copies (1 + 3 + 2 + 4 + 5), two
    // 33-byte sibling entries, two 10-byte key names, key33, signature64,
    // two path entries and 5 + 10 path bytes = 230.
    let mut l = limits();
    l.max_variable_bytes = 230;
    l.max_witness_steps = 2;
    assert_eq!(result(&f, &p, 150, l), Ok(hash("certificate_hash")));
    l.max_variable_bytes = 229;
    assert_eq!(result(&f, &p, 150, l), Err(E::InputTooLarge));
    l.max_variable_bytes = 231;
    assert_eq!(result(&f, &p, 150, l), Ok(hash("certificate_hash")));
    l.max_witness_steps = 1;
    assert_eq!(result(&f, &p, 150, l), Err(E::InputTooLarge));
    let mut f = Fixture::new(RECORDS, [8; 32]);
    f.signature.clear();
    f.steps = vec![MerkleStep::Left([0; 32]); 65];
    let mut l = limits();
    l.max_variable_bytes = usize::MAX;
    l.max_witness_steps = usize::MAX;
    assert_eq!(result(&f, &p, 150, l), Err(E::InputTooLarge));
    for change in [
        |f: &mut Fixture| f.path = vec![vec![]; 33],
        |f: &mut Fixture| f.path = vec![vec![0; 257]],
        |f: &mut Fixture| f.name = "x".repeat(129),
        |f: &mut Fixture| f.signature = vec![0; 65],
        |f: &mut Fixture| f.key = vec![0; 66],
    ] {
        let mut f = Fixture::new(RECORDS, [8; 32]);
        change(&mut f);
        assert_eq!(result(&f, &p, 150, limits()), Err(E::InputTooLarge));
    }
}

#[test]
fn sec1_point_equivalence_does_not_replace_exact_enrolled_key_identity() {
    let mut f = Fixture::new(RECORDS, [8; 32]);
    let compressed = bytes("public_key");
    f.key = bytes("public_key_uncompressed");
    assert_eq!(
        result(&f, &policy(&compressed), 150, limits()),
        Err(E::RootKeyMismatch)
    );
    assert_eq!(
        result(&f, &policy(&f.key), 150, limits()),
        Ok(hash("certificate_hash"))
    );
}
