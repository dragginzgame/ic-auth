#![cfg(feature = "portable-signature-verification")]

mod support;

use ic_auth::portable::{
    LoginRevocationError as Error, MemoryLoginRevocations, PortableClaimsContext, ValidatedGrants,
    VerifiedPortableClaims, browser_principal, claims_hash, encode_claims,
};
use ic_auth::portable_signature::verify_ed25519_claims;
use ic_auth_protocol_types::{AudienceId, CanonicalId, PortableClaims, Principal};
use ic_platform::ed25519::{Signer, SigningKey};
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
    let mut claims = support::claims(browser_key(n));
    claims.login_started_at_ns = deadline.saturating_sub(1000);
    claims.login_deadline_ns = deadline;
    claims.issued_at_ns = now;
    claims.admission_expires_at_ns = (now + 1).min(deadline);
    claims
}

fn verify(c: &PortableClaims, now: u64) -> VerifiedPortableClaims {
    verify_with_budget(c, now, &mut budget()).unwrap()
}
fn verify_with_budget(
    c: &PortableClaims,
    now: u64,
    budget: &mut ic_auth::admission_budget::SessionAdmissionBudget,
) -> Result<VerifiedPortableClaims, ic_auth::portable_signature::PortableSignatureError> {
    let key = support::issuer_key();
    let signature = key.sign(&claims_hash(c).unwrap()).to_bytes();
    let grants = ValidatedGrants::new(c.grants.clone()).unwrap();
    let context = PortableClaimsContext {
        authority: ic_auth::portable::PortableAuthority {
            audience: &c.audience,
            issuer: c.issuer,
            mechanism: c.mechanism,
            key_binding_id: c.key_binding_id,
            grants: &grants,
            caller: browser_principal(&c.browser_public_key),
            now_ns: now,
            authority_not_before_ns: 0,
            authority_expires_at_ns: u64::MAX,
            enrollment_expires_at_ns: u64::MAX,
            min_login_started_at_ns: 0,
        },
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
        .with_reservation::<(), Error>(caller(1), &verified(1, 100, 1), 1, |_| Ok(()))
        .unwrap();
    store
        .with_revocation::<Error>(&mut budget(), caller(2), 2, |_| Ok(()))
        .unwrap();
    assert_eq!(store.occupancy(), (1, 1));
    assert_eq!(
        store.with_revocation::<Error>(&mut budget(), caller(3), 3, |_| panic!("must not commit")),
        Err(Error::Capacity)
    );
    assert_eq!(
        store.with_reservation::<(), Error>(caller(4), &verified(4, 100, 3), 3, |_| panic!(
            "must not commit"
        )),
        Err(Error::Capacity)
    );
    // Even after the short session lease (say 5), the login keeps its reservation.
    assert_eq!(store.prune(6, 128), Ok(0));
    let deadline = store
        .with_revocation::<Error>(&mut budget(), caller(1), 6, |_| Ok(()))
        .unwrap();
    assert_eq!(deadline, 6 + DAY + 20);
    assert_eq!(store.occupancy(), (1, 1));
    assert!(store.is_revoked(caller(1), 6).unwrap());
    assert_eq!(
        store.with_reservation::<(), Error>(caller(1), &verified(1, 100, 7), 7, |_| panic!(
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
        .with_revocation::<Error>(&mut budget(), caller(1), 1, |_| {
            session = false;
            Ok(())
        })
        .unwrap();
    assert_eq!(
        store.with_reservation::<(), Error>(caller(1), &verified(1, 100, 2), 2, |_| {
            session = true;
            Ok(())
        }),
        Err(Error::Revoked)
    );
    assert!(!session);
    store
        .with_reservation::<(), Error>(caller(2), &verified(2, 100, 2), 2, |_| {
            session = true;
            Ok(())
        })
        .unwrap();
    assert!(session);
    store
        .with_revocation::<Error>(&mut budget(), caller(2), 3, |_| {
            session = false;
            Ok(())
        })
        .unwrap();
    assert!(!session);
    assert!(store.is_revoked(caller(2), 3).unwrap());
    assert_eq!(
        store.with_reservation::<(), Error>(caller(2), &verified(2, 100, 3), 3, |_| panic!(
            "exact retry after logout"
        )),
        Err(Error::Revoked)
    );
}

#[test]
fn storage_failure_never_commits_reservation_or_revocation() {
    let mut store = ledger(1, 1);
    assert_eq!(
        store.with_reservation::<(), Error>(caller(1), &verified(1, 100, 1), 1, |_| Err::<(), _>(
            Error::Storage
        )),
        Err(Error::Storage)
    );
    assert_eq!(store.occupancy(), (0, 0));
    assert_eq!(
        store.with_revocation::<Error>(&mut budget(), caller(2), 2, |_| Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert_eq!(store.occupancy(), (0, 0));
    store
        .with_reservation::<(), Error>(caller(1), &verified(1, 100, 3), 3, |_| Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation::<Error>(&mut budget(), caller(1), 4, |_| Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert!(!store.is_revoked(caller(1), 4).unwrap());
    assert_eq!(store.occupancy(), (1, 0));
    let until = store
        .with_revocation::<Error>(&mut budget(), caller(1), 5, |_| Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation::<Error>(&mut budget(), caller(1), 6, |_| Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert_eq!(
        store.with_revocation::<Error>(&mut budget(), caller(1), 7, |_| Ok(())),
        Ok(until)
    );
    assert_eq!(store.prune(until - 1, 128), Ok(0));
    assert_eq!(store.prune(until, 128), Ok(1));
    assert_eq!(store.occupancy(), (0, 0));
}

#[cfg(feature = "session-restore")]
#[test]
fn host_commit_errors_propagate_without_installing_state_or_refunding_logout_credit() {
    use ic_auth::{admission_budget::SessionAdmissionBudget, session::SessionError};

    let mut store = ledger(1, 1);
    let checked = verified(1, 100, 1);
    assert_eq!(
        store.with_reservation::<(), SessionError>(caller(1), &checked, 1, |_| Err(
            SessionError::ByteCapacity
        )),
        Err(SessionError::ByteCapacity),
    );
    assert_eq!(store.occupancy(), (0, 0));
    store
        .with_reservation::<_, SessionError>(caller(1), &checked, 1, |_| Ok(()))
        .unwrap();

    let mut unknown_budget = SessionAdmissionBudget::new(1, 1, 0).unwrap();
    assert_eq!(
        store.with_revocation::<SessionError>(&mut unknown_budget, caller(1), 2, |_| Err(
            SessionError::ScopeRejected
        )),
        Err(SessionError::ScopeRejected),
    );
    assert!(!store.is_revoked(caller(1), 2).unwrap());
    assert_eq!(
        store.with_revocation::<SessionError>(&mut unknown_budget, caller(2), 2, |_| Err(
            SessionError::StorageFailure
        )),
        Err(SessionError::StorageFailure),
    );
    assert!(!store.is_revoked(caller(2), 2).unwrap());
    assert_eq!(store.occupancy(), (1, 0));
    assert_eq!(
        store.with_revocation::<SessionError>(&mut unknown_budget, caller(2), 2, |_| panic!(
            "spent unknown-logout credit"
        )),
        Err(SessionError::Revocation(Error::Capacity)),
    );
    assert_eq!(
        store.with_revocation::<SessionError>(&mut unknown_budget, caller(2), 1, |_| panic!(
            "clock rollback"
        )),
        Err(SessionError::Revocation(Error::Clock)),
    );
    assert_eq!(
        store.with_reservation::<(), SessionError>(caller(1), &checked, 3, |_| panic!(
            "verification time mismatch"
        )),
        Err(SessionError::Revocation(Error::InvalidPolicy)),
    );
    assert_eq!(store.occupancy(), (1, 0));
}

#[test]
fn reservation_retention_and_renewal_deadline_are_exact() {
    let mut store = ledger(1, 1);
    store
        .with_reservation::<(), Error>(caller(1), &verified(1, 100, 1), 1, |_| Ok(()))
        .unwrap();
    store
        .with_reservation::<(), Error>(caller(1), &verified(1, 100, 2), 2, |_| Ok(()))
        .unwrap();
    for altered in [99, 101] {
        assert_eq!(
            store.with_reservation::<(), Error>(
                caller(1),
                &verified(1, altered, 2),
                2,
                |_| panic!("changed login")
            ),
            Err(Error::InvalidPolicy)
        );
    }
    assert_eq!(store.prune(109, 128), Ok(0));
    assert_eq!(store.occupancy(), (1, 0));
    assert_eq!(store.prune(110, 128), Ok(1));
    // A distinct device has independent capacity and denial state.
    store
        .with_reservation::<(), Error>(caller(2), &verified(2, 200, 110), 110, |_| Ok(()))
        .unwrap();
    assert!(!store.is_revoked(caller(2), 110).unwrap());
}

#[test]
fn idempotent_retention_never_slides_and_expired_pool_transfer_is_atomic() {
    let mut store = ledger(1, 1);
    let until = store
        .with_revocation::<Error>(&mut budget(), caller(1), 1, |_| Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation::<Error>(&mut budget(), caller(1), until - 1, |_| Ok(())),
        Ok(until)
    );
    assert!(!store.is_revoked(caller(1), until).unwrap());
    // A failed callback must preserve the old entry and pool accounting.
    assert_eq!(
        store.with_reservation::<(), Error>(
            caller(1),
            &verified(1, until + 100, until),
            until,
            |_| Err::<(), _>(Error::Storage)
        ),
        Err(Error::Storage)
    );
    assert_eq!(store.occupancy(), (0, 1));
    store
        .with_reservation::<(), Error>(caller(1), &verified(1, until + 100, until), until, |_| {
            Ok(())
        })
        .unwrap();
    assert_eq!(store.occupancy(), (1, 0));
    store
        .with_revocation::<Error>(&mut budget(), caller(2), until, |_| Ok(()))
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
            store.with_revocation::<Error>(&mut budget(), invalid, 1, |_| panic!("invalid caller")),
            Err(Error::InvalidCaller)
        );
        assert_eq!(
            store.with_reservation::<(), Error>(invalid, &verified(1, 100, 1), 1, |_| panic!(
                "invalid caller"
            )),
            Err(Error::InvalidCaller)
        );
    }
    assert_eq!(
        store.with_reservation::<(), Error>(caller(1), &verified(1, 100, 1), 2, |_| panic!(
            "stale verification"
        )),
        Err(Error::InvalidPolicy)
    );
    assert_eq!(
        store.with_reservation::<(), Error>(caller(1), &verified(2, 100, 2), 2, |_| panic!(
            "wrong caller"
        )),
        Err(Error::InvalidPolicy)
    );
    assert_eq!(
        store.with_revocation::<Error>(&mut budget(), caller(1), 10, |_| Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert_eq!(
        store.with_revocation::<Error>(&mut budget(), caller(1), 9, |_| panic!("rollback")),
        Err(Error::Clock)
    );
    assert_eq!(store.is_revoked(caller(1), 9), Err(Error::Clock));
    assert_eq!(store.prune(9, 128), Err(Error::Clock));
    assert_eq!(
        store.with_revocation::<Error>(&mut budget(), caller(1), u64::MAX, |_| panic!("overflow")),
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
        boundary.with_revocation::<Error>(
            &mut budget(),
            caller(1),
            u64::MAX - DAY - 20,
            |_| Ok(())
        ),
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
            .with_revocation::<Error>(&mut budget(), caller(n), 1, |_| Ok(()))
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
        .with_reservation::<_, Error>(caller(1), &verified(1, 100, 1), 1, |_| Ok(()))
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
            store.with_reservation::<(), Error>(caller(1), &checked, 2, |_| panic!("rebind")),
            Err(Error::InvalidPolicy)
        );
        assert_eq!(store.occupancy(), (1, 0));
    }
    let renewed = verify(&original, 2);
    assert_eq!(renewed.fingerprint(), claims_hash(&original).unwrap());
    store
        .with_reservation::<_, Error>(caller(1), &renewed, 2, |_| Ok(()))
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
        store.with_reservation::<(), Error>(caller(1), &verify(&c, 1), 1, |_| panic!(
            "wrong domain"
        )),
        Err(Error::InvalidPolicy)
    );
    assert_eq!(store.occupancy(), (0, 0));
    let checked = verified(1, 100, 1);
    assert_eq!(
        store.with_reservation::<(), Error>(caller(1), &checked, 1, |_| Err(Error::Storage)),
        Err(Error::Storage)
    );
    c = claims(1, 100, 1);
    c.account_subject = Principal::from_slice(&[9]);
    store
        .with_reservation::<_, Error>(caller(1), &verify(&c, 1), 1, |_| Ok(()))
        .unwrap();
    assert_eq!(
        store.with_reservation::<(), Error>(caller(1), &checked, 1, |_| panic!(
            "first failed binding"
        )),
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
        .with_reservation::<_, Error>(caller(1), &verified(1, 100, 1), 1, |_| Ok(()))
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
        .with_revocation::<Error>(&mut budget(), caller(1), 1, |_| Ok(()))
        .unwrap();
    assert_eq!(
        store.with_reservation::<(), Error>(caller(1), &verified(1, 100, 1), 1, |_| panic!(
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
        .with_reservation::<_, Error>(caller(2), &verified(2, 100, 1), 1, |_| Ok(()))
        .unwrap();
    assert_eq!(
        store.with_reservation::<(), Error>(caller(1), &verified(1, 100, 1), 1, |_| panic!(
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
        store.with_reservation::<_, Error>(sender, &verified, 1, |_| Ok(()))
    }
    let mut store = ledger(1, 1);
    store
        .with_reservation::<_, Error>(caller(1), &verified(1, 100, 1), 1, |_| Ok(()))
        .unwrap();
    store
        .with_revocation::<Error>(&mut budget(), caller(1), 1, |_| Ok(()))
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
        .with_reservation::<_, Error>(caller(1), &verified(1, 100, 1), 1, |_| Ok(()))
        .unwrap();
    let mut unknown = SessionAdmissionBudget::new(2, 5, 0).unwrap();
    let first = store
        .with_revocation::<Error>(&mut unknown, caller(2), 10, |_| Ok(()))
        .unwrap();
    store
        .with_revocation::<Error>(&mut unknown, caller(3), 10, |_| Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation::<Error>(&mut unknown, caller(4), 10, |_| panic!(
            "exhausted budget must not call storage"
        )),
        Err(Error::Capacity)
    );
    assert_eq!(store.occupancy(), (1, 2));
    // Reserved capacity and already-installed denial survive a fresh-principal flood.
    store
        .with_revocation::<Error>(&mut unknown, caller(1), 10, |_| Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation::<Error>(&mut unknown, caller(2), 10, |_| Ok(())),
        Ok(first)
    );
    assert_eq!(
        store.with_revocation::<Error>(&mut unknown, caller(2), 11, |_| Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert_eq!(
        store.with_revocation::<Error>(&mut unknown, caller(4), 14, |_| panic!("refill boundary")),
        Err(Error::Capacity)
    );
    store
        .with_revocation::<Error>(&mut unknown, caller(4), 15, |_| Ok(()))
        .unwrap();
    assert_eq!(store.occupancy(), (1, 3));
}

#[test]
fn unknown_logout_failure_spends_credit_without_installing_denial() {
    use ic_auth::admission_budget::SessionAdmissionBudget;
    let mut store = ledger(1, 2);
    let mut unknown = SessionAdmissionBudget::new(1, 10, 0).unwrap();
    assert_eq!(
        store.with_revocation::<Error>(&mut unknown, caller(1), 10, |_| Err(Error::Storage)),
        Err(Error::Storage)
    );
    assert_eq!(store.occupancy(), (0, 0));
    assert!(!store.is_revoked(caller(1), 10).unwrap());
    for n in [1, 2] {
        assert_eq!(
            store.with_revocation::<Error>(&mut unknown, caller(n), 10, |_| panic!(
                "failed storage must not refund"
            )),
            Err(Error::Capacity)
        );
    }
    store
        .with_revocation::<Error>(&mut unknown, caller(1), 20, |_| Ok(()))
        .unwrap();
    assert_eq!(store.occupancy(), (0, 1));
    let mut reset = SessionAdmissionBudget::new(100, 1, 20).unwrap();
    assert_eq!(
        store.with_revocation::<Error>(&mut reset, caller(2), 20, |_| panic!("empty reset")),
        Err(Error::Capacity)
    );
    let mut future = SessionAdmissionBudget::new(1, 1, 30).unwrap();
    assert_eq!(
        store.with_revocation::<Error>(&mut future, caller(2), 20, |_| panic!(
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
        store.with_revocation::<Error>(&mut unknown, Principal::anonymous(), 1, |_| panic!(
            "invalid caller"
        )),
        Err(Error::InvalidCaller)
    );
    let until = store
        .with_revocation::<Error>(&mut unknown, caller(1), 1, |_| Ok(()))
        .unwrap();
    assert_eq!(
        store.with_revocation::<Error>(&mut unknown, caller(2), 2, |_| panic!("full pool")),
        Err(Error::Capacity)
    );
    // A full pool did not spend this interval's credit.
    ledger(1, 1)
        .with_revocation::<Error>(&mut unknown, caller(2), 2, |_| Ok(()))
        .unwrap();
    let mut empty = SessionAdmissionBudget::new(1, 1, until).unwrap();
    assert_eq!(
        store.with_revocation::<Error>(&mut empty, caller(1), until, |_| panic!(
            "expired tombstone is not a live retry"
        )),
        Err(Error::Capacity)
    );
    assert!(!store.is_revoked(caller(1), until).unwrap());
    assert_eq!(store.occupancy(), (0, 1));
    let renewed = store
        .with_revocation::<Error>(&mut empty, caller(1), until + 1, |_| Ok(()))
        .unwrap();
    assert!(renewed > until);
    assert_eq!(store.occupancy(), (0, 1));
}

#[test]
fn commit_receives_complete_login_change_before_any_ledger_installation() {
    let mut store = ledger(2, 2);
    let checked = verified(1, 100, 1);
    let c = checked.claims();
    let binding = ic_auth::portable::LoginBinding {
        issuer: c.issuer,
        subject: c.account_subject,
        started_at_ns: 0,
        deadline_ns: 100,
    };
    let result: Result<(), Error> =
        store.with_reservation::<_, Error>(caller(1), &checked, 1, |change| {
            assert_eq!(change.audience(), audience());
            assert_eq!(change.caller(), caller(1));
            assert_eq!(change.until_ns(), 110);
            assert_eq!(
                change.state(),
                ic_auth::portable::LoginRecordState::Reserved(binding)
            );
            Err(Error::Storage)
        });
    assert_eq!(result, Err(Error::Storage));
    assert_eq!(store.occupancy(), (0, 0));
    store
        .with_reservation::<_, Error>(caller(1), &checked, 1, |_| Ok(()))
        .unwrap();
    let until = store
        .with_revocation::<Error>(&mut budget(), caller(1), 2, |change| {
            assert_eq!(
                change.state(),
                ic_auth::portable::LoginRecordState::Revoked(Some(binding))
            );
            assert_eq!(change.until_ns(), 2 + DAY + 20);
            Ok(())
        })
        .unwrap();
    store
        .with_revocation::<Error>(&mut budget(), caller(1), 3, |change| {
            assert_eq!(change.until_ns(), until);
            Ok(())
        })
        .unwrap();
    store
        .with_revocation::<Error>(&mut budget(), caller(2), 3, |change| {
            assert_eq!(change.caller(), caller(2));
            assert_eq!(
                change.state(),
                ic_auth::portable::LoginRecordState::Revoked(None)
            );
            Ok(())
        })
        .unwrap();
}

#[cfg(feature = "session-restore")]
#[test]
fn restore_retains_reservations_tombstones_and_clock_without_partial_results() {
    use ic_auth::portable::LoginRevocationChange;
    let mut ledger = MemoryLoginRevocations::new(audience(), 2, 1, 10, 0).unwrap();
    let proof = verified(1, 100, 1);
    let caller = browser_principal(&proof.claims().browser_public_key);
    let mut reservation = Vec::new();
    ledger
        .with_reservation::<_, Error>(caller, &proof, 1, |change| {
            reservation = change.encode().unwrap();
            Ok(())
        })
        .unwrap();
    let mut tombstone = Vec::new();
    let unknown = browser_principal(&browser_key(2));
    ledger
        .with_revocation::<Error>(&mut budget(), unknown, 2, |change| {
            tombstone = change.encode().unwrap();
            Ok(())
        })
        .unwrap();
    let restore = |rows: Vec<Vec<u8>>, counts, observed, now| {
        MemoryLoginRevocations::new(audience(), 2, 1, 10, now)
            .unwrap()
            .restore_records(
                observed,
                counts,
                rows.iter()
                    .map(|b| LoginRevocationChange::decode(b).unwrap()),
            )
    };
    let mut restored = restore(vec![reservation.clone(), tombstone.clone()], (1, 1), 2, 3).unwrap();
    assert_eq!(restored.occupancy(), (1, 1));
    assert_eq!(restored.observed_at_ns(), 3);
    assert!(!restored.is_revoked(caller, 3).unwrap());
    assert!(restored.is_revoked(unknown, 3).unwrap());
    // Original login binding survives restoration and cannot be replaced.
    let mut changed = claims(1, 100, 3);
    changed.account_subject = Principal::from_slice(&[99]);
    assert_eq!(
        restored.check_reservation(caller, &changed, 3),
        Err(Error::InvalidPolicy)
    );
    assert_eq!(restored.is_revoked(caller, 2), Err(Error::Clock));
    assert!(restore(vec![reservation.clone()], (1, 1), 2, 3).is_err());
    assert!(restore(vec![reservation.clone(), reservation.clone()], (2, 0), 2, 3).is_err());
    assert!(restore(vec![reservation.clone()], (0, 0), 2, 3).is_err());
    assert!(restore(vec![reservation.clone()], (1, 0), 4, 3).is_err());
    assert!(restore(vec![tombstone.clone()], (0, 1), 0, 3).is_err());
    // Expired physical records still count; bounded pruning reclaims them.
    let mut expired = restore(vec![reservation], (1, 0), 2, 111).unwrap();
    assert_eq!(expired.occupancy(), (1, 0));
    assert_eq!(expired.prune(111, 1).unwrap(), 1);
    assert_eq!(expired.occupancy(), (0, 0));
    let mut wrong_domain = audience();
    wrong_domain.fleet_id = CanonicalId::from_bytes([99; 32]);
    assert!(
        MemoryLoginRevocations::new(wrong_domain, 2, 1, 10, 3)
            .unwrap()
            .restore_records(
                2,
                (0, 1),
                [LoginRevocationChange::decode(&tombstone).unwrap()]
            )
            .is_err()
    );
    // A retained revocation never extends on restoration/retry.
    let mut restored = restore(vec![tombstone], (0, 1), 2, 3).unwrap();
    assert_eq!(
        restored
            .with_revocation::<Error>(&mut budget(), unknown, 3, |_| Ok(()))
            .unwrap(),
        2 + DAY + 20
    );
}

// Independent stored contract, deliberately constructible for corruption tests.
#[cfg(feature = "session-restore")]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct LoginRecord {
    until_ns: u64,
    state: ic_auth::portable::LoginRecordState,
}
#[cfg(feature = "session-restore")]
const LOGIN_ID: u64 = 0x415554485303;
#[cfg(feature = "session-restore")]
const LOGIN_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 1024,
    allocation: 65536,
    nodes: 1024,
    depth: 16,
};

#[cfg(feature = "session-restore")]
#[test]
fn stored_login_state_has_exactly_three_valid_projections() {
    use ic_auth::portable::{LoginBinding, LoginRecordState, LoginRevocationChange};
    let login = LoginBinding {
        issuer: Principal::from_slice(&[1]),
        subject: Principal::from_slice(&[2]),
        started_at_ns: 1,
        deadline_ns: 100,
    };
    for state in [
        LoginRecordState::Reserved(login),
        LoginRecordState::Revoked(Some(login)),
        LoginRecordState::Revoked(None),
    ] {
        let value = (
            audience(),
            caller(1),
            LoginRecord {
                until_ns: 110,
                state,
            },
        );
        let bytes = ic_wire::native::encode_request(LOGIN_ID, &value, LOGIN_LIMITS).unwrap();
        let decoded = LoginRevocationChange::decode(&bytes).unwrap();
        assert_eq!(decoded.state(), state);
        assert_eq!(decoded.encode().unwrap(), bytes);
    }
    // Unknown enum discriminants are corruption, not a future fallback state.
    #[derive(serde::Serialize)]
    #[allow(dead_code)]
    enum UnknownState {
        Reserved(LoginBinding),
        Revoked(Option<LoginBinding>),
        Unknown,
    }
    let bytes = ic_wire::native::encode_request(
        LOGIN_ID,
        &(audience(), caller(1), (110_u64, UnknownState::Unknown)),
        LOGIN_LIMITS,
    )
    .unwrap();
    assert!(LoginRevocationChange::decode(&bytes).is_err());
}

#[cfg(feature = "session-restore")]
#[test]
fn login_record_decoder_and_restoration_reject_corruption_and_retention_changes() {
    use ic_auth::portable::{LoginBinding, LoginRecordState, LoginRevocationChange};
    let mut ledger = MemoryLoginRevocations::new(audience(), 1, 1, 10, 0).unwrap();
    let proof = verified(1, 100, 1);
    let caller = browser_principal(&proof.claims().browser_public_key);
    let mut bytes = Vec::new();
    ledger
        .with_reservation::<_, Error>(caller, &proof, 1, |change| {
            bytes = change.encode().unwrap();
            Ok(())
        })
        .unwrap();
    for cut in 0..bytes.len() {
        assert!(LoginRevocationChange::decode(&bytes[..cut]).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(LoginRevocationChange::decode(&trailing).is_err());
    assert!(LoginRevocationChange::decode(&[0; 1025]).is_err());
    let (domain, caller, original): (AudienceId, Principal, LoginRecord) =
        ic_wire::native::decode_request(LOGIN_ID, &bytes, LOGIN_LIMITS).unwrap();
    let encode = |record: LoginRecord| {
        ic_wire::native::encode_request(LOGIN_ID, &(domain, caller, record), LOGIN_LIMITS).unwrap()
    };
    for until_ns in [0, 109, 111, DAY + 30] {
        let changed = encode(LoginRecord {
            until_ns,
            ..original.clone()
        });
        let record = LoginRevocationChange::decode(&changed).unwrap();
        assert!(
            MemoryLoginRevocations::new(audience(), 1, 1, 10, 3)
                .unwrap()
                .restore_records(2, (1, 0), [record])
                .is_err()
        );
    }
    let LoginRecordState::Reserved(login) = original.state else {
        panic!("reservation");
    };
    for invalid in [
        LoginBinding {
            issuer: Principal::anonymous(),
            ..login
        },
        LoginBinding {
            subject: Principal::anonymous(),
            ..login
        },
        LoginBinding {
            started_at_ns: login.deadline_ns,
            ..login
        },
        LoginBinding {
            deadline_ns: u64::MAX,
            ..login
        },
    ] {
        for state in [
            LoginRecordState::Reserved(invalid),
            LoginRecordState::Revoked(Some(invalid)),
        ] {
            assert!(
                LoginRevocationChange::decode(&encode(LoginRecord {
                    state,
                    ..original.clone()
                }))
                .is_err()
            );
        }
    }
    let response =
        ic_wire::native::encode_response(LOGIN_ID, &(domain, caller, original), LOGIN_LIMITS)
            .unwrap();
    assert!(LoginRevocationChange::decode(&response).is_err());
    // Existing live state cannot be overwritten by a restore operation.
    assert!(
        ledger
            .restore_records(1, (1, 0), [LoginRevocationChange::decode(&bytes).unwrap()])
            .is_err()
    );
}

#[test]
fn durable_ledger_pruning_retains_denials_and_clock_on_commit_failure() {
    let mut ledger = MemoryLoginRevocations::new(audience(), 2, 2, 10, 0).unwrap();
    let proof = verified(1, 100, 1);
    let admitted = browser_principal(&proof.claims().browser_public_key);
    ledger
        .with_reservation::<_, Error>(admitted, &proof, 1, |_| Ok(()))
        .unwrap();
    let unknown = browser_principal(&browser_key(2));
    ledger
        .with_revocation::<Error>(&mut budget(), unknown, 2, |_| Ok(()))
        .unwrap();
    assert_eq!(
        ledger
            .prune_with_commit(2, 128, |_, _| panic!("live record commit"))
            .unwrap(),
        0
    );
    let now = DAY + 22;
    assert_eq!(
        ledger.prune_with_commit(now, 128, |callers, counts| {
            assert_eq!(callers, &[admitted, unknown]);
            assert_eq!(counts, (0, 0));
            Err(Error::Storage)
        }),
        Err(Error::Storage)
    );
    assert_eq!(ledger.occupancy(), (1, 1));
    assert_eq!(ledger.observed_at_ns(), now);
    assert_eq!(
        ledger.prune_with_commit(now - 1, 128, |_, _| panic!("clock rollback commit")),
        Err(Error::Clock)
    );
    assert_eq!(
        ledger
            .prune_with_commit(now, 0, |_, _| panic!("zero budget commit"))
            .unwrap(),
        0
    );
    assert_eq!(
        ledger
            .prune_with_commit(now, 1, |callers, counts| {
                assert_eq!(callers, &[admitted]);
                assert_eq!(counts, (0, 1));
                Ok(())
            })
            .unwrap(),
        1
    );
    assert_eq!(ledger.occupancy(), (0, 1));
    assert_eq!(
        ledger
            .prune_with_commit(now, 128, |callers, counts| {
                assert_eq!(callers, &[unknown]);
                assert_eq!(counts, (0, 0));
                Ok(())
            })
            .unwrap(),
        1
    );
    assert_eq!(ledger.occupancy(), (0, 0));
    assert_eq!(
        ledger
            .prune_with_commit(now, 128, |_, _| panic!("empty commit"))
            .unwrap(),
        0
    );
}
