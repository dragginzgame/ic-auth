//! Proof acceptance uses the fixed reference OpenSSL vector, not a Rust signer.
#![cfg(feature = "ed25519-verification")]

use ic_auth::{
    claims::{ClaimsError, decode_claims, encode_claims},
    verification::{
        AuthorityGrants, BudgetError, ContextError as C, Ed25519Key, VerificationBudget,
        VerificationContext, VerificationCosts, VerificationError as E, browser_principal,
        verify_ed25519,
    },
};
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};

fn fixture(name: &str) -> Vec<u8> {
    let (_, hex) = include_str!("fixtures/ed25519.tsv")
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

fn grant(role: &str, scopes: &[&str]) -> DelegatedRoleGrant {
    DelegatedRoleGrant {
        target: role.parse().unwrap(),
        scopes: scopes.iter().map(|s| s.parse().unwrap()).collect(),
    }
}

fn grants() -> AuthorityGrants {
    AuthorityGrants::new(vec![grant("user", &["read"])]).unwrap()
}

fn audience() -> AudienceId {
    AudienceId {
        canonical_network_id: CanonicalId::from_bytes([0x11; 32]),
        fleet_id: CanonicalId::from_bytes([0x22; 32]),
    }
}

fn key() -> Ed25519Key {
    Ed25519Key::new(fixture("public-key").try_into().unwrap()).unwrap()
}

fn budget() -> VerificationBudget {
    VerificationBudget::new(
        10,
        1,
        VerificationCosts {
            ed25519: 2,
            certified: 3,
        },
        0,
    )
    .unwrap()
}

fn host<'a>(audience: &'a AudienceId, grants: &'a AuthorityGrants) -> VerificationContext<'a> {
    VerificationContext {
        audience,
        grants,
        issuer: Principal::from_slice(&[1]),
        mechanism: PortableMechanism::Ed25519,
        key_binding_id: fixture("key-binding").try_into().unwrap(),
        caller: Principal::from_slice(&fixture("caller")),
        now_ns: 25,
        authority_not_before_ns: 0,
        authority_expires_at_ns: 200,
        enrollment_expires_at_ns: 150,
        min_login_started_at_ns: 0,
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: 100,
        max_proof_ttl_ns: 60,
    }
}

fn check_context(change: impl FnOnce(&mut VerificationContext<'_>), expected: C) {
    let audience = audience();
    let grants = grants();
    let mut context = host(&audience, &grants);
    change(&mut context);
    let mut allowance = budget();
    assert_eq!(
        verify_ed25519(
            &fixture("claims"),
            &fixture("signature"),
            &context,
            &key(),
            &mut allowance
        ),
        Err(E::Context(expected))
    );
    assert_eq!(
        allowance,
        budget(),
        "preflight refusal must not observe time or charge"
    );
}

#[test]
fn frozen_reference_signature_authenticates_exact_claims_and_caller() {
    let audience = audience();
    let grants = grants();
    let context = host(&audience, &grants);
    let key = key();
    let mut allowance = budget();
    let result = verify_ed25519(
        &fixture("claims"),
        &fixture("signature"),
        &context,
        &key,
        &mut allowance,
    )
    .unwrap();
    assert_eq!(result.fingerprint().as_slice(), fixture("digest"));
    assert_eq!(key.binding().as_slice(), fixture("key-binding"));
    assert_eq!(key.as_bytes().as_slice(), fixture("public-key"));
    assert_eq!(result.caller().as_slice(), fixture("caller"));
    assert_eq!(browser_principal(&[0x44; 32]), result.caller());
    assert_eq!(result.verified_at_ns(), 25);
    assert_eq!(encode_claims(result.claims()).unwrap(), fixture("claims"));
    assert_eq!(allowance.available(), 8);
    assert_eq!(allowance.observed_at_ns(), 25);
}

#[test]
fn protected_binding_selection_precedes_cryptography() {
    check_context(
        |h| h.issuer = Principal::from_slice(&[2]),
        C::IssuerMismatch,
    );
    check_context(
        |h| h.mechanism = PortableMechanism::Certified,
        C::MechanismMismatch,
    );
    check_context(|h| h.key_binding_id[0] ^= 1, C::KeyMismatch);
    for caller in [
        Principal::anonymous(),
        Principal::management_canister(),
        Principal::from_slice(&[2]),
    ] {
        check_context(|h| h.caller = caller, C::CallerMismatch);
    }
    let trusted_grants = grants();
    for wrong_audience in [
        AudienceId {
            fleet_id: CanonicalId::from_bytes([9; 32]),
            ..audience()
        },
        AudienceId {
            canonical_network_id: CanonicalId::from_bytes([9; 32]),
            ..audience()
        },
    ] {
        let context = host(&wrong_audience, &trusted_grants);
        let mut allowance = budget();
        assert_eq!(
            verify_ed25519(&fixture("claims"), &[], &context, &key(), &mut allowance),
            Err(E::Context(C::AudienceMismatch))
        );
        assert_eq!(allowance, budget());
    }
    // A second valid key cannot replace the enrolled key even if signature bytes are absent.
    let other = Ed25519Key::new(fixture("other-public-key").try_into().unwrap()).unwrap();
    let audience = audience();
    let context = host(&audience, &trusted_grants);
    let mut allowance = budget();
    assert_eq!(
        verify_ed25519(&fixture("claims"), &[], &context, &other, &mut allowance),
        Err(E::Context(C::KeyMismatch))
    );
    assert_eq!(allowance, budget());
    let mut signed = decode_claims(&fixture("claims")).unwrap();
    signed.mechanism = PortableMechanism::Certified;
    assert_eq!(
        verify_ed25519(
            &encode_claims(&signed).unwrap(),
            &[],
            &context,
            &key(),
            &mut allowance
        ),
        Err(E::Context(C::MechanismMismatch))
    );
    assert_eq!(allowance, budget());
}

#[test]
fn protected_policy_and_exclusive_time_boundaries_refuse_without_credit() {
    for change in [
        (|h: &mut VerificationContext<'_>| h.max_login_ttl_ns = 0)
            as fn(&mut VerificationContext<'_>),
        |h| h.max_login_ttl_ns = 90 * 86_400_000_000_000 + 1,
        |h| h.max_proof_ttl_ns = 0,
        |h| h.max_proof_ttl_ns = 60_000_000_000 + 1,
        |h| h.issuer = Principal::anonymous(),
        |h| h.authority_not_before_ns = h.authority_expires_at_ns,
        |h| h.enrollment_expires_at_ns = 0,
    ] {
        check_context(change, C::InvalidContext);
    }
    check_context(|h| h.max_login_ttl_ns = 89, C::InvalidWindow);
    check_context(|h| h.max_proof_ttl_ns = 9, C::InvalidWindow);
    check_context(|h| h.min_login_started_at_ns = 11, C::BeforeReset);
    check_context(|h| h.min_proof_issued_at_ns = 21, C::BeforeReset);
    check_context(|h| h.authority_not_before_ns = 26, C::StaleAuthority);
    check_context(|h| h.authority_not_before_ns = 21, C::StaleAuthority);
    check_context(|h| h.authority_expires_at_ns = 25, C::StaleAuthority);
    check_context(|h| h.enrollment_expires_at_ns = 25, C::StaleAuthority);
    check_context(|h| h.now_ns = 19, C::NotYetValid);
    check_context(|h| h.now_ns = 30, C::Expired);
    check_context(|h| h.now_ns = u64::MAX, C::StaleAuthority);
    let audience = audience();
    let grants = grants();
    for now_ns in [20, 29] {
        let mut context = host(&audience, &grants);
        context.now_ns = now_ns;
        context.max_login_ttl_ns = 90;
        context.max_proof_ttl_ns = 10;
        context.min_login_started_at_ns = 10;
        context.min_proof_issued_at_ns = 20;
        context.authority_not_before_ns = 20;
        context.authority_expires_at_ns = now_ns + 1;
        context.enrollment_expires_at_ns = now_ns + 1;
        verify_ed25519(
            &fixture("claims"),
            &fixture("signature"),
            &context,
            &key(),
            &mut budget(),
        )
        .unwrap();
    }
}

#[test]
fn invalid_signed_windows_and_shape_do_not_reach_charging() {
    let audience = audience();
    let grants = grants();
    let context = host(&audience, &grants);
    let changes: [fn(&mut PortableClaims); 6] = [
        |c| c.login_deadline_ns = c.login_started_at_ns,
        |c| c.login_started_at_ns = c.issued_at_ns + 1,
        |c| c.admission_expires_at_ns = c.issued_at_ns,
        |c| c.admission_expires_at_ns = c.login_deadline_ns + 1,
        |c| {
            c.login_started_at_ns = u64::MAX;
            c.login_deadline_ns = 0;
        },
        |c| {
            c.issued_at_ns = u64::MAX;
            c.admission_expires_at_ns = 0;
        },
    ];
    for change in changes {
        let mut value = decode_claims(&fixture("claims")).unwrap();
        change(&mut value);
        let mut allowance = budget();
        assert_eq!(
            verify_ed25519(
                &encode_claims(&value).unwrap(),
                &[],
                &context,
                &key(),
                &mut allowance
            ),
            Err(E::Context(C::InvalidWindow))
        );
        assert_eq!(allowance, budget());
    }
    for input in [vec![], vec![0; 65537], {
        let mut b = fixture("claims");
        b.push(0);
        b
    }] {
        let mut allowance = budget();
        assert!(matches!(
            verify_ed25519(&input, &[], &context, &key(), &mut allowance),
            Err(E::Encoding(_))
        ));
        assert_eq!(allowance, budget());
    }
}

#[test]
fn role_scopes_never_union_and_protected_grants_may_be_broader() {
    let audience = audience();
    for wrong in [
        vec![grant("user", &["write"])],
        vec![grant("admin", &["read"]), grant("user", &["write"])],
    ] {
        let selected = AuthorityGrants::new(wrong).unwrap();
        let context = host(&audience, &selected);
        let mut allowance = budget();
        assert_eq!(
            verify_ed25519(
                &fixture("claims"),
                &fixture("signature"),
                &context,
                &key(),
                &mut allowance
            ),
            Err(E::Context(C::GrantsExceeded))
        );
        assert_eq!(allowance, budget());
    }
    let broad = AuthorityGrants::new(vec![
        DelegatedRoleGrant {
            target: "admin".parse().unwrap(),
            scopes: (0..32)
                .map(|i| format!("scope{i:02}").parse().unwrap())
                .collect(),
        },
        grant("user", &["read", "write"]),
    ])
    .unwrap();
    verify_ed25519(
        &fixture("claims"),
        &fixture("signature"),
        &host(&audience, &broad),
        &key(),
        &mut budget(),
    )
    .unwrap();
    for (invalid, expected) in [
        (vec![], ClaimsError::InvalidGrantCount { found: 0 }),
        (
            vec![grant("user", &[])],
            ClaimsError::InvalidScopeCount { found: 0 },
        ),
        (
            vec![grant("user", &["write", "read"])],
            ClaimsError::NonCanonicalScopes,
        ),
        (
            vec![grant("user", &["read", "read"])],
            ClaimsError::NonCanonicalScopes,
        ),
        (
            vec![grant("user", &["read"]), grant("user", &["write"])],
            ClaimsError::NonCanonicalRoles,
        ),
    ] {
        assert_eq!(AuthorityGrants::new(invalid), Err(expected));
    }
    let mut credential = decode_claims(&fixture("claims")).unwrap();
    credential.grants = broad.as_slice().to_vec();
    assert!(matches!(
        encode_claims(&credential),
        Err(ClaimsError::TooManyScopeEntries { .. })
    ));
}

#[test]
fn signature_shape_and_canonical_scalar_refuse_before_budget() {
    let audience = audience();
    let grants = grants();
    let context = host(&audience, &grants);
    for size in [0, 1, 32, 63, 65, 128] {
        let mut allowance = budget();
        assert_eq!(
            verify_ed25519(
                &fixture("claims"),
                &vec![0; size],
                &context,
                &key(),
                &mut allowance
            ),
            Err(E::MalformedSignature)
        );
        assert_eq!(allowance, budget());
    }
    let mut all_ones = fixture("signature");
    all_ones[32..].fill(255);
    for signature in [fixture("noncanonical-signature"), all_ones] {
        let mut allowance = budget();
        assert_eq!(
            verify_ed25519(
                &fixture("claims"),
                &signature,
                &context,
                &key(),
                &mut allowance
            ),
            Err(E::InvalidSignature)
        );
        assert_eq!(allowance, budget());
    }
    for first in [0, 1] {
        let mut weak = [0; 32];
        weak[0] = first;
        assert_eq!(Ed25519Key::new(weak), Err(E::InvalidPublicKey));
    }
    assert_eq!(Ed25519Key::new([0x44; 32]), Err(E::InvalidPublicKey));
}

#[test]
fn invalid_crypto_and_wrong_message_domains_consume_attempt_credit() {
    let audience = audience();
    let grants = grants();
    let context = host(&audience, &grants);
    let mut small_r = fixture("signature");
    small_r[..32].fill(0);
    small_r[0] = 1;
    let mut wrong_r = fixture("signature");
    wrong_r[0] ^= 128;
    for signature in [fixture("raw-claims-signature"), small_r, wrong_r] {
        let mut allowance = budget();
        assert_eq!(
            verify_ed25519(
                &fixture("claims"),
                &signature,
                &context,
                &key(),
                &mut allowance
            ),
            Err(E::InvalidSignature)
        );
        assert_eq!(allowance.available(), 8);
        assert_eq!(allowance.observed_at_ns(), 25);
    }
    for change in [
        (|c: &mut PortableClaims| c.nonce[0] ^= 1) as fn(&mut PortableClaims),
        |c| c.account_subject = Principal::from_slice(&[3]),
        |c| c.device_generation += 1,
        |c| c.login_deadline_ns += 1,
    ] {
        let mut value = decode_claims(&fixture("claims")).unwrap();
        change(&mut value);
        let mut allowance = budget();
        assert_eq!(
            verify_ed25519(
                &encode_claims(&value).unwrap(),
                &fixture("signature"),
                &context,
                &key(),
                &mut allowance
            ),
            Err(E::InvalidSignature)
        );
        assert_eq!(allowance.available(), 8);
    }
}

#[test]
fn empty_budget_refill_exhaustion_and_rollback_are_retained_across_calls() {
    let audience = audience();
    let grants = grants();
    let mut context = host(&audience, &grants);
    let mut allowance = VerificationBudget::new(
        4,
        1,
        VerificationCosts {
            ed25519: 2,
            certified: 3,
        },
        25,
    )
    .unwrap();
    for (now, expected, units) in [
        (25, Some(BudgetError::Exhausted), 0),
        (26, Some(BudgetError::Exhausted), 1),
        (27, None, 0),
        (28, Some(BudgetError::Exhausted), 1),
        (27, Some(BudgetError::ClockRollback), 1),
        (29, None, 0),
    ] {
        context.now_ns = now;
        let result = verify_ed25519(
            &fixture("claims"),
            &fixture("signature"),
            &context,
            &key(),
            &mut allowance,
        );
        match expected {
            Some(error) => assert_eq!(result, Err(E::Budget(error))),
            None => {
                result.unwrap();
            }
        }
        assert_eq!(allowance.available(), units);
    }
    assert_eq!(allowance.observed_at_ns(), 29);
    for (capacity, interval, ed25519, certified) in [
        (0, 1, 1, 1),
        (2, 0, 1, 1),
        (2, 1, 0, 1),
        (2, 1, 1, 0),
        (2, 1, 3, 1),
        (2, 1, 1, 3),
    ] {
        assert_eq!(
            VerificationBudget::new(
                capacity,
                interval,
                VerificationCosts { ed25519, certified },
                0
            ),
            Err(BudgetError::InvalidPolicy)
        );
    }
}

#[test]
fn session_deadline_captures_login_key_and_enrollment_without_proof_expiry() {
    use ic_auth::session_request::{SessionRequest, SessionRequestError, SessionTtlPolicy};
    let a = audience();
    let g = grants();
    let k = key();
    let role = "user".parse().unwrap();
    let request = SessionRequest::new(vec!["read".into()], None).unwrap();
    let ttl = SessionTtlPolicy::new(30, 60).unwrap();
    // Literal reference claims: proof expires at 30, original login at 100.
    // Each independently chosen protected deadline becomes the limiting bound.
    for (key_end, enrollment_end, expected) in [
        (200, 150, 100),
        (60, 150, 60),
        (200, 55, 55),
        (60, 55, 55),
        (200, 26, 26),
    ] {
        let mut h = host(&a, &g);
        h.authority_expires_at_ns = key_end;
        h.enrollment_expires_at_ns = enrollment_end;
        let mut b = budget();
        let verified =
            verify_ed25519(&fixture("claims"), &fixture("signature"), &h, &k, &mut b).unwrap();
        let after_proof = (b.available(), b.observed_at_ns());
        assert_eq!(
            verified.session_deadline(&request, &role, ttl),
            Ok(expected)
        );
        h.now_ns = 29;
        h.authority_expires_at_ns = 1000;
        h.enrollment_expires_at_ns = 1000;
        assert_eq!(
            verified.session_deadline(&request, &role, ttl),
            Ok(expected)
        );
        let refreshed = verify_ed25519(
            &fixture("claims"),
            &fixture("signature"),
            &h,
            &k,
            &mut budget(),
        )
        .unwrap();
        assert_eq!(refreshed.session_deadline(&request, &role, ttl), Ok(100));
        assert_eq!(
            (b.available(), b.observed_at_ns()),
            after_proof,
            "request evaluation performs no cryptographic work"
        );
        for bad in [
            SessionRequest::new(vec!["read".into()], Some(61)).unwrap(),
            SessionRequest::new(vec!["read".into()], Some(u64::MAX)).unwrap(),
        ] {
            assert_eq!(
                verified.session_deadline(&bad, &role, ttl),
                Err(SessionRequestError::InvalidLifetime)
            );
        }
        let wrong = SessionRequest::new(vec!["write".into()], None).unwrap();
        assert_eq!(
            verified.session_deadline(&wrong, &role, ttl),
            Err(SessionRequestError::ScopeRejected)
        );
        assert_eq!(
            verified.session_deadline(&request, &"admin".parse().unwrap(), ttl),
            Err(SessionRequestError::ScopeRejected)
        );
    }
}

#[test]
fn credential_profile_preserves_ed25519_evidence_and_charged_errors() {
    use ic_auth::verification::{CredentialError, VerificationProfile, verify_credential};
    use ic_auth_protocol_types::{Credential, CredentialProof};
    let a = audience();
    let g = grants();
    let h = host(&a, &g);
    let k = key();
    let profile = VerificationProfile::Ed25519 {
        context: &h,
        key: &k,
    };
    let mut credential = Credential {
        claims: fixture("claims"),
        proof: CredentialProof::Ed25519(fixture("signature")),
    };
    let mut b = budget();
    let verified = verify_credential(&credential, profile, &mut b).unwrap();
    assert_eq!(verified.fingerprint().as_slice(), fixture("digest"));
    assert_eq!(b.available(), 8);
    let CredentialProof::Ed25519(sig) = &mut credential.proof else {
        unreachable!()
    };
    sig[0] ^= 1;
    let mut b = budget();
    assert_eq!(
        verify_credential(&credential, profile, &mut b),
        Err(CredentialError::Ed25519(E::InvalidSignature))
    );
    assert_eq!(b.available(), 8);
    let CredentialProof::Ed25519(sig) = &mut credential.proof else {
        unreachable!()
    };
    sig.truncate(63);
    let mut b = budget();
    assert_eq!(
        verify_credential(&credential, profile, &mut b),
        Err(CredentialError::Ed25519(E::MalformedSignature))
    );
    assert_eq!(b, budget());
}
