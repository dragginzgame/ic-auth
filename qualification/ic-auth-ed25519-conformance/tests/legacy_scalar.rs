//! Deliberately weaken the dependency feature graph; Auth must stay strict.

use ic_auth::{
    claims::decode_claims,
    verification::{
        AuthorityGrants, Ed25519Key, VerificationBudget, VerificationContext, VerificationCosts,
        VerificationError, browser_principal, verify_ed25519,
    },
};
use ic_platform::ed25519::{Signature, VerifyingKey};

fn fixture(name: &str) -> Vec<u8> {
    let (_, hex) = include_str!("../../../crates/ic-auth/tests/fixtures/ed25519.tsv")
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .find(|(key, _)| *key == name)
        .unwrap();
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn legacy_scalar_acceptance_is_a_working_negative_control() {
    let public_key = fixture("public-key").try_into().unwrap();
    // Assignment proves the feature activator and Platform expose the same type.
    let legacy_key: ed25519_dalek::VerifyingKey = VerifyingKey::from_bytes(&public_key).unwrap();
    let signature = Signature::from_slice(&fixture("noncanonical-signature")).unwrap();
    legacy_key
        .verify_strict(&fixture("digest"), &signature)
        .expect("negative control must actually accept S+L under legacy compatibility");

    let bytes = fixture("claims");
    let trusted_fixture = decode_claims(&bytes).unwrap();
    let grants = AuthorityGrants::new(trusted_fixture.grants.clone()).unwrap();
    let host = VerificationContext {
        audience: &trusted_fixture.audience,
        issuer: trusted_fixture.issuer,
        mechanism: trusted_fixture.mechanism,
        key_binding_id: trusted_fixture.key_binding_id,
        grants: &grants,
        caller: browser_principal(&[0x44; 32]),
        now_ns: 25,
        authority_not_before_ns: 0,
        authority_expires_at_ns: 200,
        enrollment_expires_at_ns: 150,
        min_login_started_at_ns: 0,
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: 100,
        max_proof_ttl_ns: 60,
    };
    let key = Ed25519Key::new(public_key).unwrap();
    let mut budget = VerificationBudget::new(
        10,
        1,
        VerificationCosts {
            ed25519: 2,
            certified: 3,
        },
        0,
    )
    .unwrap();
    assert_eq!(
        verify_ed25519(
            &bytes,
            &fixture("noncanonical-signature"),
            &host,
            &key,
            &mut budget
        ),
        Err(VerificationError::InvalidSignature)
    );
    assert_eq!(budget.available(), 0);
    assert_eq!(budget.observed_at_ns(), 0);
    verify_ed25519(&bytes, &fixture("signature"), &host, &key, &mut budget).unwrap();
    assert_eq!(budget.available(), 8);
}
