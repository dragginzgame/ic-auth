//! Public receiver operations driven by independent Node/OpenSSL signed bytes.
#![cfg(all(feature = "receiver-stable", feature = "ed25519-verification"))]
mod memory_fixture;
mod memory_model;

use ic_auth::{
    receiver_memory::{
        AdmissionHost, EnrollmentBinding, EnrollmentError as EE, EnrollmentMetadata, LoginLimits,
        LogoutError, LogoutReceipt, MemoryConfig, MemoryError as E, MemoryReceiver,
        SessionAuthority, SessionCapacity, SessionPolicyError as P, UnknownLogoutLimits,
    },
    session_request::{SessionRequest, SessionTtlPolicy},
    verification::{
        AuthorityGrants, CredentialError, Ed25519Key, VerificationContext, VerificationCosts,
        VerificationError, VerificationProfile,
    },
};
use ic_auth_protocol_types::{
    AudienceId, AuthRole, CanonicalId, Credential, CredentialProof, DelegatedRoleGrant,
    PortableMechanism, Principal,
};

use memory_fixture::*;

type FixtureReceiver = ic_auth::receiver_memory::StableReceiver;
fn fresh_receiver(
    config: MemoryConfig,
    now: u64,
) -> Result<FixtureReceiver, ic_auth::stable_upgrade::UpgradeError> {
    use ic_storage::{StableKey, VectorMemory, testing::fixture_runtime};
    let name = "auth.test.receiver.v1";
    let runtime = fixture_runtime(
        VectorMemory::default(),
        "auth",
        "auth.",
        &[name],
        Default::default(),
    )
    .unwrap();
    FixtureReceiver::new_fresh(
        runtime
            .open_memory(&StableKey::parse(name).unwrap())
            .unwrap(),
        config,
        now,
    )
}

#[test]
fn starting_profile_bounds_unknown_logout_and_preserves_known_admission_policy() {
    let h = Host::new();
    let config = MemoryConfig::starting_profile(h.audience, h.config().enrollment);
    let mut r = fresh_receiver(config, 0).unwrap();
    assert_eq!(r.logout(unknown_caller(0), 0), Err(LogoutError::Closed));
    r.request_enrollment(0, [1; 32], 10).unwrap();
    let mut snapshot = h.snapshot(7);
    snapshot.expires_at_ns = 100_000_000_000;
    r.install_enrollment(0, config.enrollment.authority, snapshot)
        .unwrap();
    assert_eq!(
        r.logout(unknown_caller(0), 999_999_999),
        Err(LogoutError::Capacity)
    );
    let now = 64_000_000_000;
    for index in 0..64 {
        assert_eq!(
            r.logout(unknown_caller(index), now),
            Ok(LogoutReceipt::Unconfirmed {
                deny_until_ns: now + 3_600_000_000_000
            })
        );
    }
    assert_eq!(
        r.logout(unknown_caller(64), now + 1_000_000_000),
        Err(LogoutError::Capacity)
    );

    // Use the signed fixtures' nanosecond timescale and one available session.
    let mut small = config;
    small.refill_interval_ns = 1;
    small.capacity.max_sessions = 1;
    let mut r = fresh_receiver(small, 24).unwrap();
    h.enroll(&mut r, 7).unwrap();
    let admit = |r: &mut MemoryReceiver, name, now| {
        r.admit(
            &credential(name),
            request(),
            h.admission(&h.context(name, now)),
        )
    };
    assert!(matches!(
        admit(&mut r, "initial", 25),
        Err(E::Proof(CredentialError::Ed25519(
            VerificationError::Budget(ic_auth::verification::BudgetError::Exhausted)
        )))
    ));
    admit(&mut r, "initial", 26).unwrap();
    assert_eq!(
        admit(&mut r, "other-device", 28),
        Err(E::Policy(P::Capacity))
    );
    assert!(matches!(
        r.logout(h.context("initial", 28).caller, 28),
        Ok(LogoutReceipt::Revoked { .. })
    ));
}

#[test]
fn authorize_invokes_projection_once_only_after_current_policy_succeeds() {
    use std::cell::Cell;
    let h = Host::new();
    let mut r = h.receiver();
    let context = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&context))
        .unwrap();
    let calls = Cell::new(0);
    let credit = r.verification_budget().available();
    let value = String::from("owned application input");
    let result = r.authorize(
        &h.read("initial", 26),
        &["read".parse().unwrap()],
        |receipt| {
            calls.set(calls.get() + 1);
            (value, (receipt.issuer, receipt.subject))
        },
    );
    assert_eq!(
        result,
        Ok((String::from("owned application input"), account()))
    );
    assert_eq!(calls.get(), 1);
    assert_eq!(r.observed_at_ns(), 26);
    assert_eq!(r.verification_budget().available(), credit);
    assert_eq!(
        r.authorize(&h.read("initial", 26), &["write".parse().unwrap()], |_| {
            calls.set(calls.get() + 1);
        }),
        Err(E::Policy(P::ScopeRejected))
    );
    assert_eq!(calls.get(), 1);
    // Application refusal is inside a successful authentication result; no
    // implicit rollback, Auth error remapping or callback retry is performed.
    assert_eq!(
        r.authorize(&h.read("initial", 26), &[], |_| {
            calls.set(calls.get() + 1);
            Err::<(), _>("application refused")
        }),
        Ok(Err("application refused"))
    );
    assert_eq!(calls.get(), 2);
}

#[test]
fn authorize_rechecks_changes_between_host_turns_despite_an_earlier_receipt() {
    use std::cell::Cell;
    let h = Host::new();
    for (change, expected) in [
        (0, P::Revoked),
        (1, P::StaleAuthority),
        (2, P::Disabled),
        (3, P::MissingOrExpired),
    ] {
        let mut r = h.receiver();
        let context = h.context("initial", 25);
        r.admit(&credential("initial"), request(), h.admission(&context))
            .unwrap();
        let earlier = r
            .authorize(&h.read("initial", 25), &[], Clone::clone)
            .unwrap();
        let mut current = h.read("initial", 26);
        match change {
            0 => {
                r.logout(context.caller, 26).unwrap();
            }
            1 => {
                current.generation = r.advance_generation(26).unwrap();
            }
            2 => current.subject_admissible = false,
            3 => current.now_ns = 100,
            _ => unreachable!(),
        }
        let calls = Cell::new(0);
        assert_eq!(
            r.authorize(&current, &[], |_| {
                calls.set(calls.get() + 1);
            }),
            Err(E::Policy(expected))
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(earlier.expires_at_ns, 100); // facts survived; permission did not
        assert_eq!(r.usage(account()).replays, 1);
    }
}

#[test]
fn volatile_two_issuer_accounts_devices_and_receivers_remain_independent() {
    use std::collections::BTreeMap;
    let h = Host::new();
    let mut config = h.config();
    config.login.max_retained_device_slots = 3;
    config.capacity.max_sessions = 3;
    config.capacity.max_replays = 3;
    let new_receiver = || {
        let mut r = fresh_receiver(config, 0).unwrap();
        h.enroll(&mut r, 7).unwrap();
        r
    };
    let mut first = new_receiver();
    let mut second = new_receiver();
    let other_issuer = Principal::from_slice(&[3]);
    let other_account = (other_issuer, account().1); // equal subject bytes, distinct issuer
    let initial = h.context("initial", 25);
    let other_device = h.context("other-device", 25);
    let replacement = h.context("replacement", 25);
    let mut other = h.context("other-issuer", 25);
    other.issuer = other_issuer; // protected enrollment selection, not a request field
    for r in [&mut first, &mut second] {
        r.admit(&credential("initial"), request(), h.admission(&initial))
            .unwrap();
        r.admit(
            &credential("other-device"),
            request(),
            h.admission(&other_device),
        )
        .unwrap();
        assert_eq!(r.usage(account()).replays_for_account, 2);
        let credit = r.verification_budget().available();
        assert_eq!(
            r.admit(
                &credential("replacement"),
                request(),
                h.admission(&replacement)
            ),
            Err(E::Policy(P::Capacity))
        ); // account quota, before global capacity is full
        let wrong = h.context("other-issuer", 25);
        assert_eq!(
            r.admit(&credential("other-issuer"), request(), h.admission(&wrong)),
            Err(E::Context(
                ic_auth::verification::ContextError::IssuerMismatch
            ))
        );
        assert_eq!(r.verification_budget().available(), credit);
        r.admit(&credential("other-issuer"), request(), h.admission(&other))
            .unwrap();
        assert_eq!(r.known_logins(), 3);
        assert_eq!(r.usage(other_account).replays_for_account, 1);
    }
    let mut applications = BTreeMap::new();
    for name in ["initial", "other-device", "other-issuer"] {
        let mut host = h.read(name, 25);
        if name == "other-issuer" {
            host.issuer = other_issuer;
        }
        first
            .authorize(&host, &["read".parse().unwrap()], |receipt| {
                *applications
                    .entry((receipt.issuer, receipt.subject))
                    .or_insert(0) += 1;
            })
            .unwrap();
    }
    assert_eq!(applications.get(&account()), Some(&2));
    assert_eq!(applications.get(&other_account), Some(&1));
    assert_eq!(
        first.authorize(&h.read("other-issuer", 25), &[], |_| panic!(
            "wrong issuer authorized"
        )),
        Err(E::Policy(P::StaleAuthority))
    );

    // Physical replay rows count until explicitly pruned; sessions survive proof expiry.
    assert_eq!(first.prune(30, 128).unwrap().replays, 3);
    let context = h.context("replacement", 30);
    let mut bad = credential("replacement");
    if let CredentialProof::Ed25519(signature) = &mut bad.proof {
        signature[0] ^= 1;
    }
    assert_eq!(
        first.admit(&bad, request(), h.admission(&context)),
        Err(E::Proof(CredentialError::Ed25519(
            VerificationError::InvalidSignature
        )))
    );
    assert_eq!(first.usage(account()).replays, 0);
    assert!(first.authorize(&h.read("initial", 30), &[], |_| ()).is_ok());
    assert!(
        first
            .authorize(&h.read("other-device", 30), &[], |_| ())
            .is_ok()
    );
    first
        .admit(&credential("replacement"), request(), h.admission(&context))
        .unwrap();
    assert_eq!(first.known_logins(), 3); // reuse a full known-login pool
    assert_eq!(
        first.authorize(&h.read("initial", 30), &[], |_| ()),
        Err(E::Policy(P::MissingOrExpired))
    );
    assert!(
        first
            .authorize(&h.read("other-device", 30), &[], |_| ())
            .is_ok()
    );
    let mut other_read = h.read("other-issuer", 30);
    other_read.issuer = other_issuer;
    assert!(first.authorize(&other_read, &[], |_| ()).is_ok());

    let expired_proof = h.context("initial", 30);
    let retry = second
        .admit(
            &credential("initial"),
            request(),
            h.admission(&expired_proof),
        )
        .unwrap();
    assert!(retry.exact_retry);
    assert_eq!(retry.receipt.expires_at_ns, 100);
    assert_eq!(second.usage(account()).replays, 3);
    assert_eq!(second.prune(30, 128).unwrap().replays, 3);
    let renewal = h.context("renewal", 30);
    let renewed = second
        .admit(&credential("renewal"), request(), h.admission(&renewal))
        .unwrap();
    assert!(!renewed.exact_retry);
    assert_eq!(renewed.receipt.expires_at_ns, retry.receipt.expires_at_ns);

    let old_generation = h.context("late-old", 40);
    let credit = first.verification_budget().available();
    assert_eq!(
        first.admit(
            &credential("late-old"),
            request(),
            h.admission(&old_generation)
        ),
        Err(E::Policy(P::Revoked))
    );
    assert_eq!(first.verification_budget().available(), credit);
    assert_eq!(
        first.logout(context.caller, 40),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(first.usage(account()).replays_for_account, 1);
    assert!(
        first
            .authorize(&h.read("other-device", 40), &[], |_| ())
            .is_ok()
    );
    other_read.now_ns = 40;
    assert!(first.authorize(&other_read, &[], |_| ()).is_ok());
    assert!(
        second
            .authorize(&h.read("initial", 40), &[], |_| ())
            .is_ok()
    );
    assert_eq!(first.prune(100, 128).unwrap().sessions, 2);
    assert_eq!(first.known_logins(), 3); // neither expiry nor cleanup retires device fences
    assert_eq!(first.usage(account()).replays_for_account, 0);
    assert_eq!(first.usage(other_account).replays_for_account, 0);
    assert_eq!(
        second.authorize(&h.read("initial", 100), &[], |_| ()),
        Err(E::Policy(P::MissingOrExpired))
    );
}

#[test]
fn memory_admission_retry_read_and_exclusive_lease() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    let first = r
        .admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert!(!first.exact_retry);
    assert_eq!(first.receipt.expires_at_ns, 100);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.usage(account()).replays, 1);
    assert_eq!(r.verification_budget().available(), 8);
    let usage = r.usage(account());
    let mut altered = credential("initial");
    altered.proof = CredentialProof::Ed25519(vec![]);
    let later = h.context("initial", 50); // proof expired at 30, alternate malformed proof is irrelevant
    let retry = r.admit(&altered, request(), h.admission(&later)).unwrap();
    assert!(retry.exact_retry);
    assert_eq!(retry.receipt, first.receipt);
    assert_eq!(r.usage(account()), usage);
    assert_eq!(r.verification_budget().available(), 8);
    assert_eq!(r.verification_budget().observed_at_ns(), 25);
    assert_eq!(
        r.read(&h.read("initial", 50), &["read".parse().unwrap()]),
        Ok(first.receipt)
    );
    assert_eq!(
        r.read(&h.read("initial", 50), &["write".parse().unwrap()]),
        Err(E::Policy(P::ScopeRejected))
    );
    assert_eq!(
        r.admit(
            &altered,
            SessionRequest::new(vec!["read".into()], Some(60)).unwrap(),
            h.admission(&later)
        ),
        Err(E::Policy(P::ReplayConflict))
    );
    assert_eq!(
        r.read(&h.read("initial", 100), &[]),
        Err(E::Policy(P::MissingOrExpired))
    );
    assert_eq!(
        r.read(&h.read("initial", 99), &[]),
        Err(E::Policy(P::ClockRollback))
    );
    assert_eq!(r.usage(account()), usage);
}

#[test]
fn boot_requires_bound_current_authenticated_enrollment() {
    let h = Host::new();
    type Mutation = fn(&mut EnrollmentMetadata, &mut Principal);
    let cases: &[(Mutation, EE)] = &[
        (
            |_, source| *source = Principal::anonymous(),
            EE::WrongAuthority,
        ),
        (
            |s, _| s.receiver = Principal::from_slice(&[12]),
            EE::WrongReceiver,
        ),
        (
            |s, _| s.audience.fleet_id = CanonicalId::from_bytes([0; 32]),
            EE::WrongReceiver,
        ),
        (|s, _| s.challenge = [9; 32], EE::ChallengeMismatch),
        (|s, _| s.issued_at_ns = 2, EE::InvalidFreshness),
        (|s, _| s.expires_at_ns = 1, EE::InvalidFreshness),
        (
            |s, _| s.expires_at_ns = 3_600_000_000_001,
            EE::InvalidFreshness,
        ),
    ];
    for (mutate, expected) in cases {
        let mut r = fresh_receiver(h.config(), 0).unwrap();
        r.request_enrollment(0, [1; 32], 10).unwrap();
        let mut snapshot = h.snapshot(7);
        let mut source = h.config().enrollment.authority;
        mutate(&mut snapshot, &mut source);
        assert_eq!(
            r.install_enrollment(1, source, snapshot),
            Err(E::Enrollment(*expected))
        );
        assert_eq!(r.enrollment(), None);
        assert_eq!(r.observed_at_ns(), 1);
        assert_eq!(r.verification_budget().available(), 0);
        let context = h.context("initial", 25);
        assert_eq!(
            r.admit(&credential("initial"), request(), h.admission(&context)),
            Err(E::Closed)
        );
        assert_eq!(r.logout(context.caller, 25), Err(LogoutError::Closed));
        // A refusal leaves the real pending request intact for a valid response.
        r.install_enrollment(1, h.config().enrollment.authority, h.snapshot(7))
            .unwrap();
        assert_eq!(r.enrollment(), Some(h.snapshot(7)));
    }
}

#[test]
fn enrollment_request_replacement_fences_late_responses_and_receipt_deadlines() {
    let h = Host::new();
    let source = h.config().enrollment.authority;
    let mut r = fresh_receiver(h.config(), 0).unwrap();
    assert_eq!(
        r.install_enrollment(0, source, h.snapshot(7)),
        Err(E::Enrollment(EE::ChallengeMismatch))
    );
    for (challenge, deadline) in [([0; 32], 10), ([1; 32], 0)] {
        assert_eq!(
            r.request_enrollment(0, challenge, deadline),
            Err(E::Enrollment(EE::InvalidRequest))
        );
    }
    r.request_enrollment(0, [1; 32], 10).unwrap();
    assert_eq!(
        r.request_enrollment(1, [1; 32], 11),
        Err(E::Enrollment(EE::InvalidRequest))
    );
    r.request_enrollment(1, [2; 32], 10).unwrap();
    assert_eq!(
        r.install_enrollment(1, source, h.snapshot(7)),
        Err(E::Enrollment(EE::ChallengeMismatch))
    );
    let snapshot = EnrollmentMetadata {
        challenge: [2; 32],
        ..h.snapshot(7)
    };
    assert_eq!(
        r.install_enrollment(0, source, snapshot),
        Err(E::Policy(P::ClockRollback))
    );
    r.install_enrollment(2, source, snapshot).unwrap();
    r.install_enrollment(3, source, snapshot).unwrap(); // exact idempotent response
    assert_eq!(r.verification_budget().available(), 0);
    assert_eq!(r.unknown_logout_credit(), 0);
    assert_eq!(
        r.request_enrollment(3, [2; 32], 12),
        Err(E::Enrollment(EE::InvalidRequest))
    );
    assert_eq!(
        r.install_enrollment(10, source, snapshot),
        Err(E::Enrollment(EE::ReceiptExpired))
    );
    // Missing a response deadline cannot erase previously accepted enrollment.
    assert_eq!(r.enrollment(), Some(snapshot));
    r.request_enrollment(10, [3; 32], 11).unwrap();
    let refresh = EnrollmentMetadata {
        challenge: [3; 32],
        revision: 2,
        snapshot_digest: [4; 32],
        ..snapshot
    };
    assert_eq!(
        r.install_enrollment(11, source, refresh),
        Err(E::Enrollment(EE::ReceiptExpired))
    );
    assert_eq!(r.enrollment(), Some(snapshot));
    let context = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&context))
        .unwrap();
}

#[test]
fn enrollment_rejects_rollback_and_conflicting_content_without_discarding_sessions() {
    let h = Host::new();
    type Mutation = fn(&mut EnrollmentMetadata);
    let cases: &[(Mutation, EE)] = &[
        (|s| s.revision = 0, EE::Regression),
        (|s| s.generation = 6, EE::Regression),
        (|s| s.revision = 1, EE::ConflictingSnapshot),
        (|s| s.authority_digest = [9; 32], EE::ConflictingSnapshot),
    ];
    for (mutate, expected) in cases {
        let mut r = h.receiver();
        let context = h.context("initial", 25);
        let original = r
            .admit(&credential("initial"), request(), h.admission(&context))
            .unwrap();
        let usage = r.usage(account());
        let credit = r.verification_budget().available();
        r.request_enrollment(25, [4; 32], 40).unwrap();
        let mut snapshot = EnrollmentMetadata {
            challenge: [4; 32],
            revision: 2,
            snapshot_digest: [5; 32],
            ..h.snapshot(7)
        };
        mutate(&mut snapshot);
        assert_eq!(
            r.install_enrollment(26, h.config().enrollment.authority, snapshot),
            Err(E::Enrollment(*expected))
        );
        assert_eq!(r.enrollment(), Some(h.snapshot(7)));
        assert_eq!(r.read(&h.read("initial", 26), &[]), Ok(original.receipt));
        assert_eq!(r.usage(account()), usage);
        assert_eq!(r.verification_budget().available(), credit);
    }
}

#[test]
fn same_generation_refresh_restores_access_without_extending_the_lease() {
    let h = Host::new();
    let source = h.config().enrollment.authority;
    let mut r = fresh_receiver(h.config(), 0).unwrap();
    r.request_enrollment(0, [1; 32], 10).unwrap();
    r.install_enrollment(
        0,
        source,
        EnrollmentMetadata {
            expires_at_ns: 60,
            ..h.snapshot(7)
        },
    )
    .unwrap();
    let context = h.context("registered", 25);
    let original = r
        .admit(&credential("registered"), request(), h.admission(&context))
        .unwrap();
    assert_eq!(original.receipt.expires_at_ns, 100);
    assert_eq!(original.receipt.ext.as_deref(), Some([0xab; 16].as_slice()));
    assert_eq!(
        r.read(&h.read("registered", 60), &[]),
        Err(E::Policy(P::StaleAuthority))
    );
    let credit = r.verification_budget().available();
    r.request_enrollment(61, [2; 32], 70).unwrap();
    r.install_enrollment(
        61,
        source,
        EnrollmentMetadata {
            challenge: [2; 32],
            revision: 2,
            snapshot_digest: [4; 32],
            issued_at_ns: 61,
            ..h.snapshot(7)
        },
    )
    .unwrap();
    let later = h.context("registered", 62);
    let retry = r
        .admit(&credential("registered"), request(), h.admission(&later))
        .unwrap();
    assert!(retry.exact_retry);
    assert_eq!(retry.receipt, original.receipt);
    assert_eq!(r.verification_budget().available(), credit);
    assert_eq!(r.read(&h.read("registered", 99), &[]), Ok(original.receipt));
    assert_eq!(
        r.read(&h.read("registered", 100), &[]),
        Err(E::Policy(P::MissingOrExpired))
    );
}

#[test]
fn expired_enrollment_blocks_auth_but_preserves_safe_logout_and_denials() {
    let h = Host::new();
    let source = h.config().enrollment.authority;
    let mut r = fresh_receiver(h.config(), 0).unwrap();
    r.request_enrollment(0, [1; 32], 10).unwrap();
    r.install_enrollment(
        0,
        source,
        EnrollmentMetadata {
            expires_at_ns: 26,
            ..h.snapshot(7)
        },
    )
    .unwrap();
    let context = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&context))
        .unwrap();
    let credit = r.verification_budget().available();
    assert_eq!(
        r.read(&h.read("initial", 26), &[]),
        Err(E::Policy(P::StaleAuthority))
    );
    let later = h.context("initial", 26);
    assert_eq!(
        r.admit(&credential("initial"), request(), h.admission(&later)),
        Err(E::Policy(P::StaleAuthority))
    );
    assert_eq!(
        r.logout(context.caller, 26),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(
        r.logout(unknown_caller(9), 26),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 126 })
    );
    assert_eq!(r.verification_budget().available(), credit);
    assert_eq!(r.usage(account()).replays, 1);
    r.request_enrollment(27, [2; 32], 40).unwrap();
    r.install_enrollment(
        27,
        source,
        EnrollmentMetadata {
            challenge: [2; 32],
            revision: 2,
            snapshot_digest: [4; 32],
            issued_at_ns: 27,
            ..h.snapshot(7)
        },
    )
    .unwrap();
    let later = h.context("initial", 27);
    assert_eq!(
        r.admit(&credential("initial"), request(), h.admission(&later)),
        Err(E::Policy(P::Revoked))
    );
    assert_eq!(r.usage(account()).sessions, 0);
    assert_eq!(r.unknown_logins(), 1);
    assert_eq!(r.verification_budget().available(), credit);
}

#[test]
fn authority_generation_change_invalidates_sessions_without_resetting_safety_state() {
    let h = Host::new();
    let mut r = h.receiver();
    let context = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&context))
        .unwrap();
    let usage = r.usage(account());
    r.request_enrollment(26, [2; 32], 40).unwrap();
    let snapshot = EnrollmentMetadata {
        challenge: [2; 32],
        revision: 2,
        generation: 8,
        snapshot_digest: [4; 32],
        authority_digest: [5; 32],
        ..h.snapshot(7)
    };
    r.install_enrollment(26, h.config().enrollment.authority, snapshot)
        .unwrap();
    assert_eq!(
        r.read(&h.read("initial", 26), &[]),
        Err(E::Policy(P::StaleAuthority))
    );
    let mut current = h.read("initial", 26);
    current.generation = 8;
    assert_eq!(r.read(&current, &[]), Err(E::Policy(P::StaleAuthority)));
    assert_eq!(r.usage(account()), usage);
    assert_eq!(r.verification_budget().available(), 8);
    r.advance_generation(26).unwrap();
    assert_eq!(
        r.install_enrollment(26, h.config().enrollment.authority, snapshot),
        Err(E::Enrollment(EE::Regression))
    );
    r.close(); // loss/fatal failure is terminal; neither challenge nor fresh enrollment repairs it
    assert_eq!(r.request_enrollment(27, [6; 32], 40), Err(E::Closed));
    assert_eq!(
        r.install_enrollment(27, h.config().enrollment.authority, snapshot),
        Err(E::Closed)
    );
    assert_eq!(r.logout(context.caller, 27), Err(LogoutError::Closed));
    assert_eq!(r.usage(account()), usage);
}

#[test]
fn enrollment_freshness_and_clock_arithmetic_are_bounded_without_wrapping() {
    let h = Host::new();
    for (now, issued, expires) in [
        (1, 0, 3_600_000_000_000),
        (u64::MAX - 98, u64::MAX - 99, u64::MAX),
    ] {
        let start = now.saturating_sub(2);
        let mut r = fresh_receiver(h.config(), start).unwrap();
        r.request_enrollment(now, [1; 32], expires).unwrap();
        let snapshot = EnrollmentMetadata {
            issued_at_ns: issued,
            expires_at_ns: expires,
            ..h.snapshot(7)
        };
        r.install_enrollment(now, h.config().enrollment.authority, snapshot)
            .unwrap();
        assert_eq!(r.enrollment(), Some(snapshot));
        assert_eq!(
            r.request_enrollment(now - 1, [2; 32], expires),
            Err(E::Policy(P::ClockRollback))
        );
        assert_eq!(r.verification_budget().available(), 0);
    }
    for principal in [Principal::anonymous(), Principal::management_canister()] {
        let mut config = h.config();
        config.enrollment.authority = principal;
        assert!(matches!(
            fresh_receiver(config, 0),
            Err(ic_auth::stable_upgrade::UpgradeError::Configuration)
        ));
        config = h.config();
        config.enrollment.receiver = principal;
        assert!(matches!(
            fresh_receiver(config, 0),
            Err(ic_auth::stable_upgrade::UpgradeError::Configuration)
        ));
    }
}

#[test]
fn inspection_advances_empty_pages_over_a_large_denial_pool_without_mutation() {
    let h = Host::new();
    let mut config = h.config();
    config.unknown_logout.max_denials = 512;
    config.unknown_logout.capacity = 512;
    let mut r = fresh_receiver(config, 0).unwrap();
    h.enroll(&mut r, 7).unwrap();
    let mut callers: Vec<_> = (0u16..512)
        .map(|i| Principal::self_authenticating(i.to_be_bytes()))
        .collect();
    for caller in &callers {
        assert_eq!(
            r.logout(*caller, 512),
            Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 612 })
        );
    }
    callers.sort();
    let before = (
        r.usage(account()),
        r.enrollment(),
        r.unknown_logout_credit(),
    );
    let mut after = None;
    let mut visited = 0;
    loop {
        let page = r.inspect_sessions(after, 2, 4, 513).unwrap();
        assert!(page.sessions.is_empty());
        assert_eq!(page.scanned, 4);
        assert_eq!(page.observed_at_ns, 513);
        assert_eq!(page.generation, 7);
        visited += usize::from(page.scanned);
        let expected = (visited < callers.len()).then(|| callers[visited - 1]);
        assert_eq!(page.next_after, expected);
        after = page.next_after;
        if after.is_none() {
            break;
        }
        assert!(visited < 512);
    }
    assert_eq!(visited, 512);
    assert_eq!(r.unknown_logins(), 512);
    assert_eq!(r.observed_at_ns(), 512);
    assert_eq!(r.verification_budget().observed_at_ns(), 0);
    assert_eq!(r.verification_budget().available(), 0);
    assert_eq!(
        (
            r.usage(account()),
            r.enrollment(),
            r.unknown_logout_credit()
        ),
        before
    );
    let end = r
        .inspect_sessions(callers.last().copied(), 32, 128, 513)
        .unwrap();
    assert_eq!((end.scanned, end.next_after), (0, None));
    assert!(end.sessions.is_empty());
}

#[test]
fn inspection_reports_expired_stale_sessions_among_fences_without_authorizing() {
    let h = Host::new();
    let mut config = h.config();
    config.unknown_logout.max_denials = 4;
    config.unknown_logout.capacity = 4;
    let mut r = fresh_receiver(config, 0).unwrap();
    h.enroll(&mut r, 7).unwrap();
    for i in 0..4 {
        r.logout(unknown_caller(i), 25).unwrap();
    }
    let context = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&context))
        .unwrap();
    r.advance_generation(26).unwrap();
    let usage = r.usage(account());
    let mut after = None;
    let mut visited = 0;
    let mut found = Vec::new();
    loop {
        // Diagnostic time is deliberately beyond lease and enrollment expiry.
        let page = r.inspect_sessions(after, 1, 2, u64::MAX).unwrap();
        assert!(page.scanned <= 2);
        assert!(page.sessions.len() <= 1);
        visited += page.scanned;
        assert_eq!(page.generation, 8);
        for session in page.sessions {
            assert_eq!(session.caller, context.caller);
            assert_eq!((session.login.issuer, session.login.subject), account());
            assert_eq!(
                (session.login.device_slot, session.login.device_generation),
                (0, 1)
            );
            assert_eq!(
                (session.login.started_at_ns, session.login.deadline_ns),
                (10, 100)
            );
            assert_eq!(session.authority_generation, 7);
            assert_eq!(session.established_at_ns, 25);
            assert_eq!(session.expires_at_ns, 100);
            assert_eq!(session.role.as_str(), "user");
            assert_eq!(session.scopes, &["read".parse().unwrap()]);
            found.push(session.caller);
        }
        if let Some(next) = page.next_after {
            assert!(after.is_none_or(|previous| next > previous));
            after = Some(next);
            assert!(visited < 5);
        } else {
            break;
        }
    }
    assert_eq!(visited, 5);
    assert_eq!(found, [context.caller]);
    assert_eq!(r.observed_at_ns(), 26);
    assert_eq!(r.usage(account()), usage);
    assert_eq!(r.verification_budget().available(), 8);
    assert_eq!(
        r.read(&h.read("initial", 26), &[]),
        Err(E::Policy(P::StaleAuthority))
    );
    // The huge diagnostic timestamp neither advances the real clock nor prevents logout.
    assert_eq!(
        r.logout(context.caller, 26),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.usage(account()).replays, 1);
    assert!(
        r.inspect_sessions(None, 32, 128, 26)
            .unwrap()
            .sessions
            .is_empty()
    );
}

#[test]
fn inspection_cursor_remains_exclusive_after_pruning_and_interleaved_insertion() {
    let h = Host::new();
    let mut config = h.config();
    config.unknown_logout.max_denials = 5;
    config.unknown_logout.capacity = 5;
    let mut r = fresh_receiver(config, 0).unwrap();
    h.enroll(&mut r, 7).unwrap();
    let mut callers: Vec<_> = (0..5).map(unknown_caller).collect();
    for caller in &callers {
        r.logout(*caller, 25).unwrap();
    }
    callers.sort();
    let first = r.inspect_sessions(None, 1, 2, 25).unwrap();
    assert_eq!((first.scanned, first.next_after), (2, Some(callers[1])));
    assert!(first.sessions.is_empty());
    let cursor = first.next_after;
    assert_eq!(r.prune(125, 2).unwrap().unknown_denials, 2);
    // Reinsert a caller before the deleted cursor. Pages are not a frozen snapshot.
    r.logout(callers[0], 125).unwrap();
    let next = r.inspect_sessions(cursor, 1, 2, 125).unwrap();
    assert_eq!((next.scanned, next.next_after), (2, Some(callers[3])));
    let last = r.inspect_sessions(next.next_after, 1, 2, 125).unwrap();
    assert_eq!((last.scanned, last.next_after), (1, None));
    assert_eq!(r.unknown_logins(), 4);
}

#[test]
fn inspection_refuses_bad_bounds_rollback_and_poison_but_can_observe_pending_boot() {
    let h = Host::new();
    let mut r = fresh_receiver(h.config(), 25).unwrap();
    r.request_enrollment(25, [1; 32], 30).unwrap();
    let empty = r.inspect_sessions(None, 32, 128, 26).unwrap();
    assert_eq!(
        (empty.scanned, empty.next_after, empty.generation),
        (0, None, 0)
    );
    assert!(empty.sessions.is_empty());
    for (limit, scan) in [
        (0, 4),
        (33, 128),
        (1, 0),
        (2, 1),
        (1, 129),
        (u16::MAX, u16::MAX),
    ] {
        assert!(matches!(
            r.inspect_sessions(None, limit, scan, 26),
            Err(E::Policy(P::InvalidContext))
        ));
    }
    assert!(matches!(
        r.inspect_sessions(None, 1, 1, 24),
        Err(E::Policy(P::ClockRollback))
    ));
    assert_eq!(r.observed_at_ns(), 25);
    assert_eq!(r.enrollment(), None);
    r.close();
    assert!(matches!(r.inspect_sessions(None, 1, 1, 26), Err(E::Closed)));
}

#[test]
fn memory_invalid_proof_retains_charge_without_reservation() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    let mut bad = credential("initial");
    if let CredentialProof::Ed25519(sig) = &mut bad.proof {
        sig[0] ^= 1;
    }
    let empty = r.usage(account());
    assert_eq!(
        r.admit(&bad, request(), h.admission(&c)),
        Err(E::Proof(CredentialError::Ed25519(
            VerificationError::InvalidSignature
        )))
    );
    assert_eq!(r.usage(account()), empty);
    assert_eq!(r.known_logins(), 0);
    assert_eq!(r.verification_budget().available(), 8);
    assert_eq!(r.observed_at_ns(), 25);
    r.admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(r.verification_budget().available(), 6);
    let rebound = h.context("rebound", 25);
    assert_eq!(
        r.admit(&credential("rebound"), request(), h.admission(&rebound)),
        Err(E::Policy(P::BindingConflict))
    );
    assert_eq!(r.verification_budget().available(), 6);
}

#[test]
fn memory_full_pool_replacement_is_verified_before_displacement() {
    let h = Host::new();
    let mut r = h.receiver();
    let first = h.context("initial", 25);
    let original = r
        .admit(&credential("initial"), request(), h.admission(&first))
        .unwrap();
    let replacement = h.context("replacement", 25);
    let mut bad = credential("replacement");
    if let CredentialProof::Ed25519(sig) = &mut bad.proof {
        sig[0] ^= 1;
    }
    let before = r.usage(account());
    assert_eq!(
        r.admit(&bad, request(), h.admission(&replacement)),
        Err(E::Proof(CredentialError::Ed25519(
            VerificationError::InvalidSignature
        )))
    );
    assert_eq!(r.usage(account()), before);
    assert_eq!(r.read(&h.read("initial", 25), &[]), Ok(original.receipt));
    let next = r
        .admit(
            &credential("replacement"),
            request(),
            h.admission(&replacement),
        )
        .unwrap();
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.usage(account()).sessions, 1);
    assert_eq!(r.usage(account()).replays_for_account, 2);
    assert_eq!(r.read(&h.read("replacement", 25), &[]), Ok(next.receipt));
    assert_eq!(
        r.read(&h.read("initial", 25), &[]),
        Err(E::Policy(P::MissingOrExpired))
    );
    assert_eq!(
        r.admit(&credential("initial"), request(), h.admission(&first)),
        Err(E::Policy(P::ReplayConflict))
    );
    let third = h.context("third", 25);
    let credit = r.verification_budget().available();
    assert_eq!(
        r.admit(&credential("third"), request(), h.admission(&third)),
        Err(E::Policy(P::Capacity))
    );
    assert_eq!(r.verification_budget().available(), credit);
    // Replay expiry cannot erase the slot generation fence. This old browser
    // presents a separately signed, still-fresh credential after both replays go.
    assert_eq!(r.prune(40, 2).unwrap().replays, 2);
    let late_old = h.context("late-old", 40);
    assert_eq!(
        r.admit(&credential("late-old"), request(), h.admission(&late_old)),
        Err(E::Policy(P::Revoked))
    );
    assert_eq!(r.verification_budget().available(), credit);
    assert_eq!(r.prune(109, 128).unwrap().device_slots, 0);
    assert_eq!(r.prune(110, 1).unwrap().device_slots, 1);
    assert_eq!(r.known_logins(), 0);
    assert!(
        r.admit(
            &credential("late-old"),
            request(),
            h.admission(&h.context("late-old", 125))
        )
        .is_err()
    );
    r.admit(
        &credential("future"),
        request(),
        h.admission(&h.context("future", 125)),
    )
    .unwrap();
    assert_eq!(r.known_logins(), 1);
}

#[test]
fn memory_logout_pruning_and_device_fences_survive_expiry() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(
        r.logout(c.caller, 25),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(r.usage(account()).sessions, 0);
    assert_eq!(r.usage(account()).replays, 1);
    assert_eq!(
        r.admit(&credential("renewal"), request(), h.admission(&c)),
        Err(E::Policy(P::Revoked))
    );
    let replacement = h.context("replacement", 25);
    r.admit(
        &credential("replacement"),
        request(),
        h.admission(&replacement),
    )
    .unwrap();
    assert_eq!(r.unknown_logins(), 1);
    assert_eq!(
        r.admit(
            &credential("rebound"),
            request(),
            h.admission(&h.context("rebound", 26))
        ),
        Err(E::Policy(P::Revoked))
    );
    assert_eq!(r.prune(100, 1).unwrap().sessions, 1);
    assert_eq!(r.usage(account()).replays, 2);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.prune(100, 1).unwrap().replays, 1);
    assert_eq!(r.usage(account()).replays_for_account, 1);
    assert_eq!(r.prune(100, 1).unwrap().replays, 1);
    assert_eq!(r.usage(account()).encoded_bytes, 0);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(
        r.logout(replacement.caller, 100),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(
        r.logout(c.caller, 100),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 200 })
    );
}

#[test]
fn full_denial_pool_refuses_displacement_before_proof_work() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    r.logout(c.caller, 25).unwrap();
    for i in 0..2 {
        r.logout(unknown_caller(i), 25).unwrap();
    }
    let credit = r.verification_budget().available();
    let replacement = h.context("replacement", 26);
    assert_eq!(
        r.admit(
            &credential("replacement"),
            request(),
            h.admission(&replacement)
        ),
        Err(E::Policy(P::Capacity))
    );
    assert_eq!(r.verification_budget().available(), credit);
    assert_eq!(
        r.logout(c.caller, 26),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
}

#[test]
fn memory_renewal_reuses_session_but_retains_account_replay_quota() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    let first = r
        .admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    let renewed = r
        .admit(&credential("renewal"), request(), h.admission(&c))
        .unwrap();
    assert!(!renewed.exact_retry);
    assert_eq!(renewed.receipt.expires_at_ns, first.receipt.expires_at_ns);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.usage(account()).sessions, 1);
    assert_eq!(r.usage(account()).replays_for_account, 2);
    assert_eq!(
        r.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Policy(P::ReplayConflict))
    );
}

#[test]
fn memory_readiness_generation_and_byte_capacity_are_enforced() {
    let h = Host::new();
    let c = h.context("initial", 25);
    let mut fresh = fresh_receiver(h.config(), 0).unwrap();
    assert_eq!(
        fresh.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Closed)
    );
    h.enroll(&mut fresh, 7).unwrap();
    fresh
        .admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(fresh.advance_generation(25), Ok(8));
    assert_eq!(
        fresh.read(&h.read("initial", 25), &[]),
        Err(E::Policy(P::StaleAuthority))
    );
    let mut current = h.read("initial", 25);
    current.generation = 8;
    assert_eq!(fresh.read(&current, &[]), Err(E::Policy(P::StaleAuthority)));
    fresh.close();
    assert_eq!(h.enroll(&mut fresh, 9), Err(E::Closed));
    assert_eq!(fresh.prune(100, 1), Err(E::Closed));
    let mut config = h.config();
    config.capacity.max_encoded_bytes = 1;
    let mut tiny = fresh_receiver(config, 0).unwrap();
    h.enroll(&mut tiny, 7).unwrap();
    assert_eq!(
        tiny.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Policy(P::ByteCapacity))
    );
    assert_eq!(tiny.verification_budget().observed_at_ns(), 0);
    assert_eq!(tiny.known_logins(), 0);
    assert_eq!(tiny.observed_at_ns(), 25);
}

#[test]
fn memory_logical_record_bytes_match_checkpoint_field_sizes() {
    // Checkpoint payload, summed independently: absent ext and mechanism 1+1;
    // signing key 32; role 5; scopes 6; three single-byte varints; hashes 32+32.
    // Replay: fingerprint 32, two one-byte principals with their lengths, expiry 1.
    const SESSION: usize = 112;
    const REPLAY: usize = 37;
    let h = Host::new();
    let c = h.context("initial", 25);
    for (generation, expected) in [(7, SESSION + REPLAY), (u64::MAX, SESSION + REPLAY + 9)] {
        // u64::MAX takes ten bytes in the session instead of one.
        for ceiling in [expected - 1, expected] {
            let mut config = h.config();
            config.capacity.max_encoded_bytes = ceiling;
            let mut r = fresh_receiver(config, 0).unwrap();
            h.enroll(&mut r, generation).unwrap();
            let mut host = h.admission(&c);
            host.generation = generation;
            let result = r.admit(&credential("initial"), request(), host);
            if ceiling < expected {
                assert_eq!(result, Err(E::Policy(P::ByteCapacity)));
                assert_eq!(r.usage(account()).encoded_bytes, 0);
                assert_eq!(r.verification_budget().observed_at_ns(), 0);
            } else {
                result.unwrap();
                assert_eq!(r.usage(account()).encoded_bytes, expected);
                if generation == u64::MAX {
                    assert_eq!(r.advance_generation(25), Err(E::Policy(P::StaleAuthority)));
                }
            }
        }
    }
}

#[test]
fn memory_replay_pruning_does_not_extend_or_destroy_live_exact_retry() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    let first = r
        .admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(r.prune(30, 1).unwrap().replays, 1);
    assert_eq!(r.usage(account()).encoded_bytes, 112);
    let later = h.context("initial", 30);
    assert_eq!(
        r.admit(&credential("initial"), request(), h.admission(&later))
            .unwrap()
            .receipt,
        first.receipt
    );
    assert_eq!(r.verification_budget().available(), 8);
    let mut separate = h.receiver();
    assert!(matches!(
        separate.admit(&credential("initial"), request(), h.admission(&later)),
        Err(E::Context(_))
    ));
    assert_eq!(separate.known_logins(), 0);
    assert_eq!(separate.verification_budget().observed_at_ns(), 0);
}

#[test]
fn memory_uncharged_refusals_and_empty_budget_cannot_publish() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    let empty = r.usage(account());
    let mut malformed = credential("initial");
    malformed.proof = CredentialProof::Ed25519(vec![0; 63]);
    assert_eq!(
        r.admit(&malformed, request(), h.admission(&c)),
        Err(E::Proof(CredentialError::Ed25519(
            VerificationError::MalformedSignature
        )))
    );
    assert_eq!(r.verification_budget().observed_at_ns(), 0);
    assert_eq!(r.observed_at_ns(), 25);
    assert_eq!(r.usage(account()), empty);
    let mut disabled = h.admission(&c);
    disabled.enabled = false;
    assert_eq!(
        r.admit(&credential("initial"), request(), disabled),
        Err(E::Policy(P::Disabled))
    );
    // A receiver created at this same trusted time starts with no work credit.
    let mut fresh = fresh_receiver(h.config(), 25).unwrap();
    h.enroll(&mut fresh, 7).unwrap();
    assert!(matches!(
        fresh.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Proof(CredentialError::Ed25519(
            VerificationError::Budget(_)
        )))
    ));
    assert_eq!(fresh.known_logins(), 0);
    assert_eq!(fresh.usage(account()), empty);
    // A wrong authenticated caller never establishes the submitted login.
    let wrong = h.context("replacement", 26);
    assert!(matches!(
        r.admit(&credential("initial"), request(), h.admission(&wrong)),
        Err(E::Context(_))
    ));
    assert_eq!(r.observed_at_ns(), 25);
    assert_eq!(r.known_logins(), 0);
}

fn unknown_caller(byte: u8) -> Principal {
    let mut bytes = [byte; 29];
    bytes[28] = 2;
    Principal::from_slice(&bytes)
}

#[test]
fn logout_unknown_retries_remain_unconfirmed_without_sliding_or_charging() {
    let h = Host::new();
    let mut r = h.receiver();
    let caller = unknown_caller(1);
    assert_eq!(r.logout(caller, 0), Err(LogoutError::Capacity)); // starts empty
    assert_eq!(r.unknown_logins(), 0);
    let original = LogoutReceipt::Unconfirmed { deny_until_ns: 101 };
    assert_eq!(r.logout(caller, 1), Ok(original));
    assert_eq!(r.unknown_logout_credit(), 0);
    assert_eq!(r.logout(caller, 100), Ok(original));
    assert_eq!(r.unknown_logout_credit(), 0); // retry does not even refill
    assert_eq!(r.unknown_logins(), 1);
    assert_eq!(r.known_logins(), 0);
    assert_eq!(r.verification_budget().observed_at_ns(), 0);
    assert_eq!(
        r.logout(unknown_caller(2), 100),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 200 })
    );
    assert_eq!(r.unknown_logout_credit(), 1);
    assert_eq!(r.logout(unknown_caller(3), 100), Err(LogoutError::Capacity));
    assert_eq!(r.unknown_logout_credit(), 1); // pool refusal precedes a charge
    assert_eq!(r.logout(caller, 100), Ok(original));
    assert_eq!(r.unknown_logins(), 2);
}

#[test]
fn logout_unknown_flood_cannot_spend_known_capacity_or_proof_credit() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    for caller in [unknown_caller(10), unknown_caller(11)] {
        r.logout(caller, 26).unwrap();
    }
    assert_eq!(r.logout(unknown_caller(12), 26), Err(LogoutError::Capacity));
    assert_eq!(r.unknown_logout_credit(), 0);
    assert_eq!(r.verification_budget().available(), 8);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.unknown_logins(), 2);
    assert_eq!(
        r.logout(c.caller, 26),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(
        r.logout(c.caller, 101),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(r.unknown_logins(), 2);
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.usage(account()).sessions, 0);
    assert_eq!(r.usage(account()).replays, 1);
    assert_eq!(r.unknown_logout_credit(), 0);
    assert_eq!(r.verification_budget().available(), 8);
}

#[test]
fn logout_expired_unknown_reuses_its_slot_and_replaces_only_its_expiry() {
    let h = Host::new();
    let mut config = h.config();
    config.unknown_logout.max_denials = 1;
    let mut r = fresh_receiver(config, 0).unwrap();
    h.enroll(&mut r, 7).unwrap();
    let caller = unknown_caller(4);
    assert_eq!(
        r.logout(caller, 1),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 101 })
    );
    assert_eq!(r.logout(unknown_caller(5), 101), Err(LogoutError::Capacity)); // expired physical row still counts
    assert_eq!(
        r.logout(caller, 101),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 201 })
    );
    assert_eq!(r.unknown_logins(), 1);
    assert_eq!(r.prune(101, 128).unwrap().unknown_denials, 0); // old expiry removed
    assert_eq!(r.prune(201, 1).unwrap().unknown_denials, 1);
    assert_eq!(r.unknown_logins(), 0);
    assert_eq!(r.known_logins(), 0);
}

#[test]
fn logout_unconfirmed_cannot_exclude_unseen_future_credentials() {
    let h = Host::new();
    for prune_first in [false, true] {
        let mut r = h.receiver();
        let future = h.context("future", 125);
        assert_eq!(
            r.logout(future.caller, 1),
            Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 101 })
        );
        let early = h.context("future", 100);
        assert_eq!(
            r.admit(&credential("future"), request(), h.admission(&early)),
            Err(E::Policy(P::Revoked))
        );
        if prune_first {
            assert_eq!(r.prune(101, 1).unwrap().unknown_denials, 1);
        }
        let mut invalid = credential("future");
        if let CredentialProof::Ed25519(sig) = &mut invalid.proof {
            sig[0] ^= 1;
        }
        assert_eq!(
            r.admit(&invalid, request(), h.admission(&future)),
            Err(E::Proof(CredentialError::Ed25519(
                VerificationError::InvalidSignature
            )))
        );
        assert_eq!(r.known_logins(), 0);
        assert_eq!(r.unknown_logins(), usize::from(!prune_first));
        // Same independently signed future credential becomes admissible after
        // the temporary denial. No caller-selected time or fabricated verifier.
        let result = r
            .admit(&credential("future"), request(), h.admission(&future))
            .unwrap();
        assert_eq!(result.receipt.subject, Principal::from_slice(&[3]));
        assert_eq!(r.known_logins(), 1);
        assert_eq!(r.unknown_logins(), 0);
        assert_eq!(r.verification_budget().available(), 6);
        assert_eq!(r.prune(125, 128).unwrap().unknown_denials, 0);
        r.logout(unknown_caller(6), 125).unwrap();
        assert_eq!(r.unknown_logins(), 1);
    }
}

#[test]
fn logout_expired_unknown_transfer_refuses_without_freeing_either_pool() {
    let h = Host::new();
    let mut r = h.receiver();
    let initial = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&initial))
        .unwrap();
    let future = h.context("future", 125);
    r.logout(future.caller, 25).unwrap();
    assert_eq!(
        r.admit(&credential("future"), request(), h.admission(&future)),
        Err(E::Policy(P::Capacity))
    );
    assert_eq!(r.known_logins(), 1);
    assert_eq!(r.unknown_logins(), 1);
    assert_eq!(r.verification_budget().available(), 8);
    let pruned = r.prune(125, 128).unwrap();
    assert_eq!(pruned.unknown_denials, 1);
    assert_eq!(pruned.device_slots, 1);
    assert_eq!(r.known_logins(), 0);
    assert_eq!(r.unknown_logins(), 0);
    assert_eq!(
        r.logout(initial.caller, 125),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 225 })
    );
}

#[test]
fn logout_invalid_clock_overflow_and_closed_state_never_acknowledge() {
    let h = Host::new();
    let mut r = h.receiver();
    let c = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    for caller in [
        Principal::anonymous(),
        Principal::management_canister(),
        Principal::from_slice(&[2]),
    ] {
        assert_eq!(r.logout(caller, 26), Err(LogoutError::InvalidCaller));
    }
    assert_eq!(r.observed_at_ns(), 25);
    assert_eq!(r.logout(c.caller, 24), Err(LogoutError::Clock));
    assert_eq!(
        r.logout(unknown_caller(7), u64::MAX),
        Err(LogoutError::Clock)
    );
    assert_eq!(r.observed_at_ns(), u64::MAX);
    assert_eq!(r.unknown_logins(), 0);
    assert_eq!(r.unknown_logout_credit(), 0);
    // Known expiry is retained even when computing a new temporary lease would overflow.
    assert_eq!(
        r.logout(c.caller, u64::MAX),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    r.close();
    assert_eq!(r.logout(c.caller, u64::MAX), Err(LogoutError::Closed));
    let mut fresh = fresh_receiver(h.config(), 0).unwrap();
    assert_eq!(fresh.logout(c.caller, 1), Err(LogoutError::Closed));
}

#[test]
fn logout_operation_budget_is_separate_and_retains_refill_remainder() {
    let h = Host::new();
    let mut config = h.config();
    config.unknown_logout = UnknownLogoutLimits {
        max_denials: 4,
        capacity: 2,
        refill_interval_ns: 3,
    };
    let mut r = fresh_receiver(config, 0).unwrap();
    h.enroll(&mut r, 7).unwrap();
    assert_eq!(r.logout(unknown_caller(1), 2), Err(LogoutError::Capacity));
    assert_eq!(
        r.logout(unknown_caller(1), 5),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 105 })
    );
    assert_eq!(r.logout(unknown_caller(2), 5), Err(LogoutError::Capacity));
    assert_eq!(
        r.logout(unknown_caller(2), 6),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 106 })
    );
    assert_eq!(r.unknown_logout_credit(), 0);
    assert_eq!(r.verification_budget().observed_at_ns(), 0);
    let c = h.context("initial", 25);
    r.admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(r.verification_budget().available(), 8);
    assert_eq!(r.unknown_logout_credit(), 0);
    r.logout(c.caller, 25).unwrap();
    assert_eq!(r.unknown_logout_credit(), 0);
    assert_eq!(r.verification_budget().available(), 8);
}

#[test]
fn logout_admission_orderings_deny_without_cross_receiver_effects() {
    let h = Host::new();
    let c = h.context("initial", 25);
    let mut before = h.receiver();
    let mut after = h.receiver();
    assert_eq!(
        before.logout(c.caller, 25),
        Ok(LogoutReceipt::Unconfirmed { deny_until_ns: 125 })
    );
    assert_eq!(
        before.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Policy(P::Revoked))
    );
    assert_eq!(before.verification_budget().available(), 0);
    after
        .admit(&credential("initial"), request(), h.admission(&c))
        .unwrap();
    assert_eq!(
        after.logout(c.caller, 25),
        Ok(LogoutReceipt::Revoked { deny_until_ns: 100 })
    );
    assert_eq!(
        after.admit(&credential("initial"), request(), h.admission(&c)),
        Err(E::Policy(P::Revoked))
    );
    assert_eq!(before.unknown_logins(), 1);
    assert_eq!(after.unknown_logins(), 0);
    assert_eq!(before.known_logins(), 0);
    assert_eq!(after.known_logins(), 1);
}

#[cfg(feature = "receiver-stable")]
#[test]
fn stable_upgrade_retains_live_session_then_logout_across_two_boots() {
    for name in ["initial", "registered", "max-ext"] {
        check_stable_session(name);
    }
}

#[cfg(feature = "receiver-stable")]
fn check_stable_session(name: &str) {
    use ic_auth::receiver_memory::StableReceiver;
    let h = Host::new();
    let mut config = h.config();
    config.capacity.max_encoded_bytes = 8192;
    let backing = ic_storage::VectorMemory::default();
    let key = "auth.receiver.v1";
    let lease = || {
        let runtime = ic_storage::testing::fixture_runtime(
            backing.clone(),
            "auth",
            "auth.",
            &[key],
            Default::default(),
        )
        .unwrap();

        runtime
            .open_memory(&ic_storage::StableKey::parse(key).unwrap())
            .unwrap()
    };
    let mut owner = StableReceiver::new_fresh(lease(), config, 0).unwrap();
    h.enroll(owner.state(), 7).unwrap();
    let context = h.context(name, 25);
    let admitted = owner
        .state()
        .admit(&credential(name), request(), h.admission(&context))
        .unwrap();
    if name == "max-ext" {
        assert_eq!(admitted.receipt.ext, Some(vec![0xfc; 4096]));
    }
    let mut credit = owner.state().verification_budget().available();
    owner.seal(1).unwrap();
    drop(owner);
    for revision in [2, 3] {
        if revision == 2 {
            let mut forbidden = config;
            forbidden.login.max_login_ttl_ns += 1;
            assert!(matches!(
                StableReceiver::reopen(lease(), forbidden, 26, revision),
                Err(ic_auth::stable_upgrade::UpgradeError::Configuration)
            ));
            config.login.max_retained_device_slots += 1;
        } else {
            let mut decrease = config;
            decrease.login.max_retained_device_slots -= 1;
            assert!(matches!(
                StableReceiver::reopen(lease(), decrease, 26, revision),
                Err(ic_auth::stable_upgrade::UpgradeError::Configuration)
            ));
        }
        let mut owner = StableReceiver::reopen(lease(), config, 26, revision).unwrap();
        let r = owner.state();
        assert_eq!(r.verification_budget().available(), credit);
        assert_eq!(r.read(&h.read(name, 26), &[]), Err(E::Closed));
        assert!(
            r.install_enrollment(26, h.config().enrollment.authority, h.snapshot(7))
                .is_err()
        );
        let mut snapshot = h.snapshot(7);
        snapshot.revision = revision;
        snapshot.challenge = [revision as u8; 32];
        r.request_enrollment(26, snapshot.challenge, 30).unwrap();
        r.install_enrollment(26, h.config().enrollment.authority, snapshot)
            .unwrap();
        if revision == 2 {
            assert_eq!(r.read(&h.read(name, 26), &[]).unwrap(), admitted.receipt);
            assert!(
                r.admit(
                    &credential(name),
                    request(),
                    h.admission(&h.context(name, 26))
                )
                .unwrap()
                .exact_retry
            );
            r.logout(context.caller, 26).unwrap();
            r.admit(
                &credential("replacement"),
                request(),
                h.admission(&h.context("replacement", 26)),
            )
            .unwrap();
        } else {
            assert_eq!(r.unknown_logins(), 1);
            assert_eq!(
                r.admit(
                    &credential("rebound"),
                    request(),
                    h.admission(&h.context("rebound", 26))
                ),
                Err(E::Policy(P::Revoked))
            );
            assert!(r.read(&h.read(name, 26), &[]).is_err());
            assert!(
                r.admit(
                    &credential(name),
                    request(),
                    h.admission(&h.context(name, 26))
                )
                .is_err()
            );
        }
        credit = r.verification_budget().available();
        owner.seal(revision).unwrap();
    }
}

#[test]
fn two_receivers_authenticate_only_the_issuer_attested_application_user() {
    let host = Host::new();
    for receiver in [10, 12] {
        let mut config = host.config();
        config.enrollment.receiver = Principal::from_slice(&[receiver]);
        config.capacity.max_replays = 3;
        config.capacity.max_replays_per_account = 3;
        let mut state = fresh_receiver(config, 0).unwrap();
        let mut enrollment = host.snapshot(7);
        enrollment.receiver = config.enrollment.receiver;
        state
            .request_enrollment(0, enrollment.challenge, 10)
            .unwrap();
        state
            .install_enrollment(0, config.enrollment.authority, enrollment)
            .unwrap();
        let context = host.context("initial", 25);
        let admit = |state: &mut MemoryReceiver, credential: &Credential| {
            state.admit(credential, request(), host.admission(&context))
        };
        let read = |state: &mut MemoryReceiver| {
            state
                .authorize(&host.read("initial", 25), &[], |s| s.ext.clone())
                .unwrap()
        };
        assert_eq!(
            admit(&mut state, &credential("initial"))
                .unwrap()
                .receipt
                .ext,
            None
        );
        let mut altered = credential("registered");
        let offset = altered
            .claims
            .windows(16)
            .position(|bytes| bytes == [0xab; 16])
            .unwrap();
        altered.claims[offset] ^= 1;
        assert!(admit(&mut state, &altered).is_err());
        assert_eq!(read(&mut state), None);

        for name in ["registered", "registered-renewal"] {
            let proof = credential(name);
            let result = admit(&mut state, &proof).unwrap();
            assert_eq!(result.receipt.ext, Some(vec![0xab; 16]));
            assert_eq!(admit(&mut state, &proof).unwrap().receipt, result.receipt);
            assert_eq!(read(&mut state), Some(vec![0xab; 16]));
        }
        assert!(admit(&mut state, &credential("initial")).is_err());
        assert_eq!(read(&mut state), Some(vec![0xab; 16]));
    }
}
