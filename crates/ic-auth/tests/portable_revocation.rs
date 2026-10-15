#![cfg(feature = "portable-signature-verification")]

use ed25519_dalek::{Signer, SigningKey};
use ic_auth::portable::{
    LoginRevocationError as Error, MemoryLoginRevocations, PortableClaimsContext,
    VerifiedPortableClaims, browser_principal, claims_hash, encode_claims,
};
use ic_auth::portable_signature::verify_ed25519_claims;
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};
use sha2::{Digest, Sha256};

fn audience() -> AudienceId {
    AudienceId {
        canonical_network_id: CanonicalId::from_bytes([1; 32]),
        fleet_id: CanonicalId::from_bytes([2; 32]),
    }
}
fn browser_key(n: u64) -> [u8; 32] {
    SigningKey::from_bytes(&Sha256::digest(n.to_be_bytes()).into())
        .verifying_key()
        .to_bytes()
}
fn claims(n: u64, deadline: u64, now: u64) -> PortableClaims {
    PortableClaims {
        audience: audience(),
        mechanism: PortableMechanism::Ed25519,
        issuer: Principal::from_slice(&[1]),
        account_subject: Principal::from_slice(&[2]),
        key_binding_id: Sha256::digest(
            SigningKey::from_bytes(&[42; 32]).verifying_key().to_bytes(),
        )
        .into(),
        browser_public_key: browser_key(n),
        login_started_at_ns: deadline.saturating_sub(1000),
        login_deadline_ns: deadline,
        issued_at_ns: now,
        admission_expires_at_ns: (now + 1).min(deadline),
        nonce: [0; 16],
        grants: vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".into()],
        }],
    }
}
fn verify(c: &PortableClaims, now: u64) -> VerifiedPortableClaims {
    verify_with_budget(c, now, &mut budget()).unwrap()
}
fn verify_with_budget(
    c: &PortableClaims,
    now: u64,
    budget: &mut ic_auth::admission_budget::SessionAdmissionBudget,
) -> Result<VerifiedPortableClaims, ic_auth::portable_signature::PortableSignatureError> {
    let key = SigningKey::from_bytes(&[42; 32]);
    let signature = key.sign(&claims_hash(c).unwrap()).to_bytes();
    let context = PortableClaimsContext {
        audience: &c.audience,
        issuer: c.issuer,
        mechanism: c.mechanism,
        key_binding_id: c.key_binding_id,
        grants: &c.grants,
        caller: browser_principal(&c.browser_public_key),
        now_ns: now,
        authority_not_before_ns: 0,
        authority_expires_at_ns: u64::MAX,
        enrollment_expires_at_ns: u64::MAX,
        min_login_started_at_ns: 0,
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: DAY,
        max_proof_ttl_ns: 60,
    };
    verify_ed25519_claims(
        budget,
        &encode_claims(c).unwrap(),
        &signature,
        &context,
        &key.verifying_key().to_bytes(),
    )
}
fn verified(n: u64, deadline: u64, now: u64) -> VerifiedPortableClaims {
    verify(&claims(n, deadline, now), now)
}

const DAY: u64 = 86_400_000_000_000;
fn caller(n: u64) -> Principal {
    browser_principal(&browser_key(n))
}
fn ledger(admitted: usize, unknown: usize) -> MemoryLoginRevocations {
    MemoryLoginRevocations::new(audience(), admitted, unknown, 10, 0).unwrap()
}

#[test]
fn unknown_flood_cannot_steal_admitted_logout_capacity() {
    let mut store = ledger(1, 1);
    store
        .with_reservation::<()>(caller(1), &verified(1, 100, 1), 1, || Ok(()))
        .unwrap();
    store
        .with_revocation(&mut budget(), caller(2), 2, || Ok(()))
        .unwrap();
    assert_eq!(store.occupancy(), (1, 1));
    assert_eq!(
        store.with_revocation(&mut budget(), caller(3), 3, || panic!("must not commit")),
        Err(Error::Capacity)
    );
    assert_eq!(
        store.with_reservation::<()>(caller(4), &verified(4, 100, 3), 3, || panic!(
            "must not commit"
        )),
        Err(Error::Capacity)
    );
    // Even after the short session lease (say 5), the login keeps its reservation.
    assert_eq!(store.prune(6, 128), Ok(0));
    let deadline = store
        .with_revocation(&mut budget(), caller(1), 6, || Ok(()))
        .unwrap();
    assert_eq!(deadline, 6 + DAY + 20);
    assert_eq!(store.occupancy(), (1, 1));
    assert!(store.is_revoked(caller(1), 6).unwrap());
    assert_eq!(
        store.with_reservation::<()>(caller(1), &verified(1, 100, 7), 7, || panic!(
            "renewal after logout"
        )),
        Err(Error::Revoked)
    );
}

#[test]
fn logout_before_admission_and_admission_before_logout_both_deny() {
    let mut store = ledger(2, 2);
    let mut session = false;
    store
        .with_revocation(&mut budget(), caller(1), 1, || {
            session = false;
            Ok(())
        })
        .unwrap();
    assert_eq!(
        store.with_reservation::<()>(caller(1), &verified(1, 100, 2), 2, || {
            session = true;
            Ok(())
        }),
        Err(Error::Revoked)
    );
    assert!(!session);
    store
        .with_reservation::<()>(caller(2), &verified(2, 100, 2), 2, || {
            session = true;
            Ok(())
        })
        .unwrap();
    assert!(session);
    store
        .with_revocation(&mut budget(), caller(2), 3, || {
            session = false;
            Ok(())
        })
        .unwrap();
    assert!(!session);
    assert!(store.is_revoked(caller(2), 3).unwrap());
    assert_eq!(
        store.with_reservation::<()>(caller(2), &verified(2, 100, 3), 3, || panic!(
            "exact retry after logout"
        )),
        Err(Error::Revoked)
    );
}

#[test]
fn storage_failure_never_commits_reservation_or_revocation() {
    let mut store = ledger(1, 1);
    assert_eq!(
        store.with_reservation::<()>(caller(1), &verified(1, 100, 1), 1, || Err::<(), _>(
            Error::Storage
        )),
        Err(Error::Storage)
    );
    assert_eq!(store.occupancy(), (0, 0));
    assert_eq!(
        store.with_revocation(&mut budget(), caller(2), 2, || Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert_eq!(store.occupancy(), (0, 0));
    store
        .with_reservation::<()>(caller(1), &verified(1, 100, 3), 3, || Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation(&mut budget(), caller(1), 4, || Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert!(!store.is_revoked(caller(1), 4).unwrap());
    assert_eq!(store.occupancy(), (1, 0));
    let until = store
        .with_revocation(&mut budget(), caller(1), 5, || Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation(&mut budget(), caller(1), 6, || Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert_eq!(
        store.with_revocation(&mut budget(), caller(1), 7, || Ok(())),
        Ok(until)
    );
    assert_eq!(store.prune(until - 1, 128), Ok(0));
    assert_eq!(store.prune(until, 128), Ok(1));
    assert_eq!(store.occupancy(), (0, 0));
}

#[test]
fn reservation_retention_and_renewal_deadline_are_exact() {
    let mut store = ledger(1, 1);
    store
        .with_reservation::<()>(caller(1), &verified(1, 100, 1), 1, || Ok(()))
        .unwrap();
    store
        .with_reservation::<()>(caller(1), &verified(1, 100, 2), 2, || Ok(()))
        .unwrap();
    for altered in [99, 101] {
        assert_eq!(
            store.with_reservation::<()>(caller(1), &verified(1, altered, 2), 2, || panic!(
                "changed login"
            )),
            Err(Error::InvalidPolicy)
        );
    }
    assert_eq!(store.prune(109, 128), Ok(0));
    assert_eq!(store.occupancy(), (1, 0));
    assert_eq!(store.prune(110, 128), Ok(1));
    // A distinct device has independent capacity and denial state.
    store
        .with_reservation::<()>(caller(2), &verified(2, 200, 110), 110, || Ok(()))
        .unwrap();
    assert!(!store.is_revoked(caller(2), 110).unwrap());
}

#[test]
fn idempotent_retention_never_slides_and_expired_pool_transfer_is_atomic() {
    let mut store = ledger(1, 1);
    let until = store
        .with_revocation(&mut budget(), caller(1), 1, || Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation(&mut budget(), caller(1), until - 1, || Ok(())),
        Ok(until)
    );
    assert!(!store.is_revoked(caller(1), until).unwrap());
    // A failed callback must preserve the old entry and pool accounting.
    assert_eq!(
        store.with_reservation::<()>(
            caller(1),
            &verified(1, until + 100, until),
            until,
            || Err::<(), _>(Error::Storage)
        ),
        Err(Error::Storage)
    );
    assert_eq!(store.occupancy(), (0, 1));
    store
        .with_reservation::<()>(
            caller(1),
            &verified(1, until + 100, until),
            until,
            || Ok(()),
        )
        .unwrap();
    assert_eq!(store.occupancy(), (1, 0));
    store
        .with_revocation(&mut budget(), caller(2), until, || Ok(()))
        .unwrap();
    assert_eq!(store.occupancy(), (1, 1));
    // Same caller may belong to another independently installed trust domain.
    let mut other_domain = ledger(1, 1);
    assert!(!other_domain.is_revoked(caller(2), until).unwrap());
}

#[test]
fn invalid_callers_clock_rollback_and_overflow_cannot_acknowledge_logout() {
    let mut store = ledger(1, 1);
    for invalid in [
        Principal::anonymous(),
        Principal::management_canister(),
        Principal::from_slice(&[2]),
    ] {
        assert_eq!(
            store.with_revocation(&mut budget(), invalid, 1, || panic!("invalid caller")),
            Err(Error::InvalidCaller)
        );
        assert_eq!(
            store.with_reservation::<()>(invalid, &verified(1, 100, 1), 1, || panic!(
                "invalid caller"
            )),
            Err(Error::InvalidCaller)
        );
    }
    assert_eq!(
        store.with_reservation::<()>(caller(1), &verified(1, 100, 1), 2, || panic!(
            "stale verification"
        )),
        Err(Error::InvalidPolicy)
    );
    assert_eq!(
        store.with_reservation::<()>(caller(1), &verified(2, 100, 2), 2, || panic!(
            "wrong caller"
        )),
        Err(Error::InvalidPolicy)
    );
    assert_eq!(
        store.with_revocation(&mut budget(), caller(1), 10, || Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert_eq!(
        store.with_revocation(&mut budget(), caller(1), 9, || panic!("rollback")),
        Err(Error::Clock)
    );
    assert_eq!(store.is_revoked(caller(1), 9), Err(Error::Clock));
    assert_eq!(store.prune(9, 128), Err(Error::Clock));
    assert_eq!(
        store.with_revocation(&mut budget(), caller(1), u64::MAX, || panic!("overflow")),
        Err(Error::Clock)
    );
    assert_eq!(store.occupancy(), (0, 0));
    assert!(matches!(
        MemoryLoginRevocations::new(audience(), 1, 1, u64::MAX, 0),
        Err(Error::Clock)
    ));
    assert!(matches!(
        MemoryLoginRevocations::new(audience(), 0, 1, 0, 0),
        Err(Error::InvalidPolicy)
    ));
    let mut boundary =
        MemoryLoginRevocations::new(audience(), 1, 1, 10, u64::MAX - DAY - 20).unwrap();
    assert_eq!(
        boundary.with_revocation(&mut budget(), caller(1), u64::MAX - DAY - 20, || Ok(())),
        Ok(u64::MAX)
    );
    assert!(boundary.is_revoked(caller(1), u64::MAX - 1).unwrap());
    assert_eq!(boundary.prune(u64::MAX, 1), Ok(1));
}

#[test]
fn maintenance_is_bounded_and_does_not_scan_or_evict_live_entries() {
    let mut store = ledger(1, 130);
    for n in 0..130 {
        store
            .with_revocation(&mut budget(), caller(n), 1, || Ok(()))
            .unwrap();
    }
    assert_eq!(store.prune(1 + DAY + 19, usize::MAX), Ok(0));
    assert_eq!(store.prune(1 + DAY + 20, 0), Ok(0));
    assert_eq!(store.prune(1 + DAY + 20, usize::MAX), Ok(128));
    assert_eq!(store.occupancy(), (0, 2));
    assert_eq!(store.prune(1 + DAY + 20, 1), Ok(1));
    assert_eq!(store.prune(1 + DAY + 20, 128), Ok(1));
    assert_eq!(store.occupancy(), (0, 0));
}

#[test]
fn signed_renewal_cannot_replace_account_issuer_or_original_login_start() {
    let mut store = ledger(1, 1);
    store
        .with_reservation(caller(1), &verified(1, 100, 1), 1, || Ok(()))
        .unwrap();
    let original = claims(1, 100, 2);
    let mutations: [fn(&mut PortableClaims); 4] = [
        |c| c.issuer = Principal::from_slice(&[3]),
        |c| c.account_subject = Principal::from_slice(&[3]),
        |c| c.login_started_at_ns = 1,
        |c| c.login_deadline_ns = 101,
    ];
    for change in mutations {
        let mut renewal = original.clone();
        change(&mut renewal);
        // Even a newly valid signature under approved authority cannot rebind a login.
        let checked = verify(&renewal, 2);
        assert_eq!(
            store.with_reservation::<()>(caller(1), &checked, 2, || panic!("rebind")),
            Err(Error::InvalidPolicy)
        );
        assert_eq!(store.occupancy(), (1, 0));
    }
    let renewed = verify(&original, 2);
    assert_eq!(renewed.fingerprint(), claims_hash(&original).unwrap());
    store
        .with_reservation(caller(1), &renewed, 2, || Ok(()))
        .unwrap();
    // Editing a copy does not edit authenticated evidence.
    let mut copy = renewed.claims().clone();
    copy.account_subject = Principal::from_slice(&[9]);
    assert_ne!(renewed.claims().account_subject, copy.account_subject);
}

#[test]
fn domain_mismatch_and_failed_commit_do_not_pin_a_login() {
    let mut store = ledger(1, 1);
    let mut c = claims(1, 100, 1);
    c.audience.fleet_id = CanonicalId::from_bytes([9; 32]);
    assert_eq!(
        store.with_reservation::<()>(caller(1), &verify(&c, 1), 1, || panic!("wrong domain")),
        Err(Error::InvalidPolicy)
    );
    assert_eq!(store.occupancy(), (0, 0));
    let checked = verified(1, 100, 1);
    assert_eq!(
        store.with_reservation::<()>(caller(1), &checked, 1, || Err(Error::Storage)),
        Err(Error::Storage)
    );
    c = claims(1, 100, 1);
    c.account_subject = Principal::from_slice(&[9]);
    store
        .with_reservation(caller(1), &verify(&c, 1), 1, || Ok(()))
        .unwrap();
    assert_eq!(
        store.with_reservation::<()>(caller(1), &checked, 1, || panic!("first failed binding")),
        Err(Error::InvalidPolicy)
    );
}

// Crypto fixtures prepare credit; budget behavior tests retain a single instance.
fn budget() -> ic_auth::admission_budget::SessionAdmissionBudget {
    ic_auth::admission_budget::SessionAdmissionBudget::new(4096, 1, 0).unwrap()
}

#[test]
fn preflight_never_reserves_or_authenticates_and_commit_rechecks_state() {
    let mut store = ledger(1, 1);
    let mut untrusted = claims(1, 100, 1);
    untrusted.account_subject = Principal::from_slice(&[99]);
    store.check_reservation(caller(1), &untrusted, 1).unwrap();
    assert_eq!(store.occupancy(), (0, 0));
    // Unauthenticated preflight must not pin the claimed account identity.
    store
        .with_reservation(caller(1), &verified(1, 100, 1), 1, || Ok(()))
        .unwrap();
    assert_eq!(
        store.check_reservation(caller(1), &untrusted, 1),
        Err(Error::InvalidPolicy)
    );
    assert_eq!(
        store.check_reservation(caller(2), &claims(2, 100, 1), 1),
        Err(Error::Capacity)
    );
    store
        .check_reservation(caller(1), &claims(1, 100, 1), 1)
        .unwrap();
    // Preflight is not a permit that can survive a logout at the same clock tick.
    store
        .with_revocation(&mut budget(), caller(1), 1, || Ok(()))
        .unwrap();
    assert_eq!(
        store.with_reservation::<()>(caller(1), &verified(1, 100, 1), 1, || panic!(
            "logout must prevent commit"
        )),
        Err(Error::Revoked)
    );
    assert_eq!(store.occupancy(), (1, 0));

    let mut store = ledger(1, 1);
    store
        .check_reservation(caller(1), &claims(1, 100, 1), 1)
        .unwrap();
    store
        .with_reservation(caller(2), &verified(2, 100, 1), 1, || Ok(()))
        .unwrap();
    assert_eq!(
        store.with_reservation::<()>(caller(1), &verified(1, 100, 1), 1, || panic!(
            "capacity must be rechecked"
        )),
        Err(Error::Capacity)
    );
}

#[test]
fn host_ordering_refuses_revoked_and_full_before_spending_crypto_credit() {
    use ic_auth::admission_budget::SessionAdmissionBudget;
    fn attempt(
        store: &mut MemoryLoginRevocations,
        budget: &mut SessionAdmissionBudget,
        c: &PortableClaims,
    ) -> Result<(), Error> {
        let sender = browser_principal(&c.browser_public_key);
        store.check_reservation(sender, c, 1)?;
        let verified = verify_with_budget(c, 1, budget).map_err(|_| Error::Storage)?;
        store.with_reservation(sender, &verified, 1, || Ok(()))
    }
    let mut store = ledger(1, 1);
    store
        .with_reservation(caller(1), &verified(1, 100, 1), 1, || Ok(()))
        .unwrap();
    store
        .with_revocation(&mut budget(), caller(1), 1, || Ok(()))
        .unwrap();
    let mut budget = SessionAdmissionBudget::new(1, 1, 0).unwrap();
    assert_eq!(
        attempt(&mut store, &mut budget, &claims(1, 100, 1)),
        Err(Error::Revoked)
    );
    assert_eq!(
        attempt(&mut store, &mut budget, &claims(2, 100, 1)),
        Err(Error::Capacity)
    );
    // Both refusals preserve the sole credit for another admissible store/domain.
    attempt(&mut ledger(1, 1), &mut budget, &claims(3, 100, 1)).unwrap();
    assert!(matches!(
        verify_with_budget(&claims(4, 100, 1), 1, &mut budget),
        Err(ic_auth::portable_signature::PortableSignatureError::Budget(
            ic_auth::admission_budget::AdmissionBudgetError::Exhausted
        ))
    ));
}

#[test]
fn preflight_bounds_untrusted_input_and_retains_clock_on_failure() {
    let mut store = ledger(1, 1);
    let mut c = claims(1, 100, 10);
    c.grants.clear();
    assert_eq!(
        store.check_reservation(caller(1), &c, 10),
        Err(Error::InvalidPolicy)
    );
    assert_eq!(
        store.check_reservation(caller(1), &claims(1, 100, 9), 9),
        Err(Error::Clock)
    );
    let c = claims(1, 100, 10);
    assert_eq!(
        store.check_reservation(caller(2), &c, 10),
        Err(Error::InvalidPolicy)
    );
    assert_eq!(
        store.check_reservation(Principal::anonymous(), &c, 10),
        Err(Error::InvalidCaller)
    );
    let mut c = c;
    c.audience.fleet_id = CanonicalId::from_bytes([99; 32]);
    assert_eq!(
        store.check_reservation(caller(1), &c, 10),
        Err(Error::InvalidPolicy)
    );
    c = claims(1, 100, 10);
    c.login_deadline_ns = u64::MAX;
    assert_eq!(
        store.check_reservation(caller(1), &c, 10),
        Err(Error::Clock)
    );
    assert_eq!(store.occupancy(), (0, 0));
}

#[test]
fn unknown_logout_flood_cannot_throttle_admitted_users_or_live_retries() {
    use ic_auth::admission_budget::SessionAdmissionBudget;
    let mut store = ledger(1, 4);
    store
        .with_reservation(caller(1), &verified(1, 100, 1), 1, || Ok(()))
        .unwrap();
    let mut unknown = SessionAdmissionBudget::new(2, 5, 0).unwrap();
    let first = store
        .with_revocation(&mut unknown, caller(2), 10, || Ok(()))
        .unwrap();
    store
        .with_revocation(&mut unknown, caller(3), 10, || Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation(&mut unknown, caller(4), 10, || panic!(
            "exhausted budget must not call storage"
        )),
        Err(Error::Capacity)
    );
    assert_eq!(store.occupancy(), (1, 2));
    // Reserved capacity and already-installed denial survive a fresh-principal flood.
    store
        .with_revocation(&mut unknown, caller(1), 10, || Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation(&mut unknown, caller(2), 10, || Ok(())),
        Ok(first)
    );
    assert_eq!(
        store.with_revocation(&mut unknown, caller(2), 11, || Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert_eq!(
        store.with_revocation(&mut unknown, caller(4), 14, || panic!("refill boundary")),
        Err(Error::Capacity)
    );
    store
        .with_revocation(&mut unknown, caller(4), 15, || Ok(()))
        .unwrap();
    assert_eq!(store.occupancy(), (1, 3));
}

#[test]
fn unknown_logout_failure_spends_credit_without_installing_denial() {
    use ic_auth::admission_budget::SessionAdmissionBudget;
    let mut store = ledger(1, 2);
    let mut unknown = SessionAdmissionBudget::new(1, 10, 0).unwrap();
    assert_eq!(
        store.with_revocation(&mut unknown, caller(1), 10, || Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert_eq!(store.occupancy(), (0, 0));
    assert!(!store.is_revoked(caller(1), 10).unwrap());
    for n in [1, 2] {
        assert_eq!(
            store.with_revocation(&mut unknown, caller(n), 10, || panic!(
                "failed storage must not refund"
            )),
            Err(Error::Capacity)
        );
    }
    store
        .with_revocation(&mut unknown, caller(1), 20, || Ok(()))
        .unwrap();
    assert_eq!(store.occupancy(), (0, 1));
    let mut reset = SessionAdmissionBudget::new(100, 1, 20).unwrap();
    assert_eq!(
        store.with_revocation(&mut reset, caller(2), 20, || panic!("empty reset")),
        Err(Error::Capacity)
    );
    let mut future = SessionAdmissionBudget::new(1, 1, 30).unwrap();
    assert_eq!(
        store.with_revocation(&mut future, caller(2), 20, || panic!(
            "budget clock rollback"
        )),
        Err(Error::Clock)
    );
}

#[test]
fn cheap_logout_refusals_preserve_credit_and_expired_retries_need_new_credit() {
    use ic_auth::admission_budget::SessionAdmissionBudget;
    let mut store = ledger(1, 1);
    let mut unknown = SessionAdmissionBudget::new(1, 1, 0).unwrap();
    assert_eq!(
        store.with_revocation(&mut unknown, Principal::anonymous(), 1, || panic!(
            "invalid caller"
        )),
        Err(Error::InvalidCaller)
    );
    let until = store
        .with_revocation(&mut unknown, caller(1), 1, || Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation(&mut unknown, caller(2), 2, || panic!("full pool")),
        Err(Error::Capacity)
    );
    // A full pool did not spend this interval's credit.
    ledger(1, 1)
        .with_revocation(&mut unknown, caller(2), 2, || Ok(()))
        .unwrap();
    let mut empty = SessionAdmissionBudget::new(1, 1, until).unwrap();
    assert_eq!(
        store.with_revocation(&mut empty, caller(1), until, || panic!(
            "expired tombstone is not a live retry"
        )),
        Err(Error::Capacity)
    );
    assert!(!store.is_revoked(caller(1), until).unwrap());
    assert_eq!(store.occupancy(), (0, 1));
    let renewed = store
        .with_revocation(&mut empty, caller(1), until + 1, || Ok(()))
        .unwrap();
    assert!(renewed > until);
    assert_eq!(store.occupancy(), (0, 1));
}
