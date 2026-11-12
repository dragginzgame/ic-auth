//! Reusable native backend scenarios for this source cohort. These assertions
//! exercise Auth's public engine with real fixture signatures, not a production
//! issuer. Passing them does not establish backing provenance, crash atomicity,
//! real clock provenance or production adoption.
#[cfg(test)]
mod entropy_tests;

#[cfg(feature = "persistent")]
pub mod receiver_tests;

use ic_auth::{
    admission_budget::SessionAdmissionBudget,
    portable::{
        LoginRevocationChange, MemoryLoginRevocations, PortableAuthority, PortableClaimsContext,
        ValidatedGrants, browser_principal, claims_hash, encode_claims,
    },
    portable_credential::PortableVerificationContext,
    portable_signature::PreparedIssuerKey,
    session::*,
    session_request::{SessionRequest, SessionTtlPolicy},
};
use ic_auth_protocol_types::*;
use ic_platform::ed25519::{Signer, SigningKey};

const SECOND: u64 = 1_000_000_000;
const NOW: u64 = 10 * SECOND;

/// Test-driver hooks, not another production storage interface. Each `fresh`
/// call must allocate isolated test state with the requested protected bounds.
/// Reopen must reconstruct from the adapter's persisted representation. Failure
/// injection applies to the next nonempty admission/logout/prune/generation commit,
/// before any mutation. Clock hooks project the host's protected high-water mark;
/// supplied timestamps are fixture inputs, not a production clock implementation.
pub trait Backend: SessionStore + Sized {
    fn fresh(generation: u64, limits: SessionLimits) -> Self;
    fn fail_next_commit(&mut self);
    fn reopen(self) -> Self;
    fn advance_generation(&mut self) -> Result<u64, SessionError>;
    /// Atomically remove at most min(budget, 128) expired session/replay records.
    /// Smaller positive host batch caps are accepted. Nonempty expired work must
    /// progress; empty work must not commit. Login records remain intact.
    fn prune(&mut self, now_ns: u64, budget: usize) -> Result<SessionPruned, SessionError>;
    /// Persist a monotonic trusted-clock observation, refusing rollback with
    /// SessionError::Revocation(LoginRevocationError::Clock).
    fn observe_clock(&mut self, now_ns: u64) -> Result<(), SessionError>;
    fn observed_at_ns(&self) -> u64;
    /// Complete physical login records, from the same serialized checkpoint.
    fn retained_logins(&self) -> Vec<LoginRevocationChange>;
}

/// Run the same admission, authorization, retry, rollback, logout, reopen and capacity scenarios
/// against a host adapter. Panics identify a violated assertion. Use only with
/// disposable qualification backing; this deliberately injects failed commits.
///
/// # Panics
/// Panics when the adapter violates a scenario's contract or fixture admission fails.
pub fn assert_backend_contract<B: Backend>() {
    assert_admission_retry::<B>();
    assert_logout_reopen::<B>();
    assert_unknown_logout_reopen::<B>();
    assert_replacement_generation::<B>();
    assert_ninety_day_renewal::<B>();
    assert_capacity::<B>();
    assert_authorization::<B>();
    assert_maximum_shape::<B>();
    assert_pruning::<B>();
    assert_metadata_reopen::<B>();
    assert_device_replacement::<B>();
}

/// Concurrent devices and fenced replacement survive failure and durable reopen.
///
/// # Panics
/// Panics if replacement loses another device, replay history or a slot fence.
pub fn assert_device_replacement<B: Backend>() {
    let first = Fixture::new(1);
    let mut peer = Fixture::new(2);
    peer.claims.account_subject = first.claims.account_subject;
    peer.claims.device_slot = 1;
    let mut next = Fixture::new(3);
    next.claims.account_subject = first.claims.account_subject;
    next.claims.device_generation = 2;
    let mut store = B::fresh(0, limits(2, 65_536));
    let mut ledger = first.ledger();
    let mut budget = SessionAdmissionBudget::new(32, 1, 0).unwrap();
    first
        .admit(&mut store, &mut ledger, &mut budget, NOW)
        .unwrap();
    peer.admit(&mut store, &mut ledger, &mut budget, NOW)
        .unwrap();
    store.fail_next_commit();
    assert_eq!(
        next.admit(&mut store, &mut ledger, &mut budget, NOW),
        Err(SessionError::StorageFailure)
    );
    store = store.reopen();
    assert!(first.authorize(&mut store, &mut ledger, NOW, 0).is_ok());
    assert!(peer.authorize(&mut store, &mut ledger, NOW, 0).is_ok());
    assert!(matches!(
        next.admit(&mut store, &mut ledger, &mut budget, NOW),
        Ok(SessionEstablished::Replaced(_))
    ));
    store = store.reopen();
    ledger = first
        .ledger_at(NOW)
        .restore_records(
            ledger.observed_at_ns(),
            ledger.occupancy(),
            store.retained_logins(),
        )
        .unwrap();
    assert!(next.authorize(&mut store, &mut ledger, NOW, 0).is_ok());
    assert!(peer.authorize(&mut store, &mut ledger, NOW, 0).is_ok());
    assert!(
        first
            .admit(&mut store, &mut ledger, &mut budget, NOW)
            .is_err()
    );
    let mut logout_budget = SessionAdmissionBudget::new(8, 1, 0).unwrap();
    assert!(matches!(
        logout_session(
            &mut store,
            &mut ledger,
            &mut logout_budget,
            first.caller(),
            NOW
        ),
        LogoutResult::Unconfirmed { .. }
    ));
    assert!(next.authorize(&mut store, &mut ledger, NOW, 0).is_ok());
    assert!(peer.authorize(&mut store, &mut ledger, NOW, 0).is_ok());
    store
        .read(|r| {
            assert!(r.replay_exists(claims_hash(&first.claims).unwrap())?);
            assert_eq!(
                r.occupancy((first.claims.issuer, first.claims.account_subject))?
                    .replays_for_account,
                3
            );
            Ok(())
        })
        .unwrap();
}

/// Unknown logout commits temporary denial without confirming an authenticated
/// login window. Failed writes and retries preserve that distinction across
/// reopen, without spending admitted-login logout capacity.
///
/// # Panics
/// Panics when the adapter loses denial, publishes a failed write, or violates
/// the known/unknown logout distinction.
pub fn assert_unknown_logout_reopen<B: Backend>() {
    let unknown = Fixture::new(1);
    let known = Fixture::new(2);
    let mut store = B::fresh(0, limits(4, 65_536));
    let mut ledger = unknown.ledger();
    let mut admission = SessionAdmissionBudget::new(1, NOW, 0).unwrap();
    known
        .admit(&mut store, &mut ledger, &mut admission, NOW)
        .unwrap();
    let before = snapshot(&mut store, &unknown, &known);
    let mut budget = SessionAdmissionBudget::new(1, SECOND, NOW - SECOND).unwrap();
    store.fail_next_commit();
    assert_eq!(
        logout_session(&mut store, &mut ledger, &mut budget, unknown.caller(), NOW),
        LogoutResult::Unavailable(LogoutUnavailable::Storage)
    );
    assert_eq!(
        snapshot(&mut store, &unknown, &known),
        before,
        "failed unknown logout must preserve every record"
    );
    assert_eq!(
        logout_session(&mut store, &mut ledger, &mut budget, unknown.caller(), NOW),
        LogoutResult::Unavailable(LogoutUnavailable::Capacity),
        "failed unknown logout must retain its spent work credit"
    );
    assert_eq!(snapshot(&mut store, &unknown, &known), before);
    assert_eq!(ledger.occupancy(), (1, 0));

    let result = logout_session(
        &mut store,
        &mut ledger,
        &mut budget,
        unknown.caller(),
        NOW + SECOND,
    );
    assert_eq!(
        result,
        LogoutResult::Unconfirmed {
            deny_until_ns: NOW + SECOND + 86_400 * SECOND,
        }
    );
    let denied = snapshot(&mut store, &unknown, &known);
    let mut store = store.reopen();
    ledger = unknown
        .ledger_at(NOW + 2 * SECOND)
        .restore_records(
            ledger.observed_at_ns(),
            ledger.occupancy(),
            store.retained_logins(),
        )
        .expect("unknown denial must survive complete reopen");
    assert_eq!(snapshot(&mut store, &unknown, &known), denied);

    // Live retries require another successful commit, but no new unknown credit.
    let mut empty = SessionAdmissionBudget::new(1, SECOND, NOW + 2 * SECOND).unwrap();
    store.fail_next_commit();
    assert_eq!(
        logout_session(
            &mut store,
            &mut ledger,
            &mut empty,
            unknown.caller(),
            NOW + 2 * SECOND
        ),
        LogoutResult::Unavailable(LogoutUnavailable::Storage)
    );
    assert_eq!(
        snapshot(&mut store, &unknown, &known),
        denied,
        "failed unknown retry must preserve committed denial"
    );
    assert_eq!(
        logout_session(
            &mut store,
            &mut ledger,
            &mut empty,
            unknown.caller(),
            NOW + 2 * SECOND
        ),
        result,
        "unknown retry must neither confirm nor extend denial"
    );
    assert!(
        known
            .authorize(&mut store, &mut ledger, NOW + 2 * SECOND, 0)
            .is_ok(),
        "unknown denial must not revoke another caller"
    );

    let confirmed = LogoutResult::Revoked {
        deny_until_ns: known.claims.login_deadline_ns,
    };
    assert_eq!(
        logout_session(
            &mut store,
            &mut ledger,
            &mut empty,
            known.caller(),
            NOW + 2 * SECOND
        ),
        confirmed,
        "known logout must not depend on unknown work credit"
    );
    let revoked = snapshot(&mut store, &unknown, &known);
    assert!(revoked.sessions[1].is_none());
    assert!(
        revoked.replays[1],
        "known logout must retain replay evidence"
    );
    let mut store = store.reopen();
    ledger = unknown
        .ledger_at(NOW + 2 * SECOND)
        .restore_records(
            ledger.observed_at_ns(),
            ledger.occupancy(),
            store.retained_logins(),
        )
        .expect("mixed known and unknown denial must survive complete reopen");
    assert_eq!(
        logout_session(
            &mut store,
            &mut ledger,
            &mut empty,
            unknown.caller(),
            NOW + 2 * SECOND
        ),
        result
    );
    assert_eq!(
        logout_session(
            &mut store,
            &mut ledger,
            &mut empty,
            known.caller(),
            NOW + 2 * SECOND
        ),
        confirmed
    );
    assert_eq!(snapshot(&mut store, &unknown, &known), revoked);
}

/// Individual grant/scopes/label maxima within the aggregate byte budget survive
/// admission, record restoration and
/// scoped authorization. This is a shape contract, not a host cost measurement.
pub fn assert_maximum_shape<B: Backend>() {
    let mut f = Fixture::new(1);
    f.claims.grants = (0..limits::MAX_GRANTS)
        .map(|role| DelegatedRoleGrant {
            target: format!("r{role:02}{}", "r".repeat(limits::MAX_LABEL_BYTES - 3))
                .parse()
                .unwrap(),
            scopes: (0..if role == limits::MAX_GRANTS - 1 {
                ic_auth::portable::MAX_CREDENTIAL_SCOPE_ENTRIES - (limits::MAX_GRANTS - 1)
            } else {
                1
            })
                .map(|scope| {
                    format!("s{scope:02}{}", "s".repeat(limits::MAX_LABEL_BYTES - 3))
                        .parse()
                        .unwrap()
                })
                .collect(),
        })
        .collect();
    let grant = f.claims.grants.last().unwrap();
    f.role = grant.target.clone();
    let requested = grant.scopes[grant.scopes.len() - limits::MAX_SESSION_SCOPES..].to_vec();
    f.grants = ValidatedGrants::new(f.claims.grants.clone()).unwrap();
    let required =
        RequiredScopes::new(requested.iter().map(ToString::to_string).collect()).unwrap();
    let mut store = B::fresh(0, limits(4, 65_536));
    let mut ledger = f.ledger();
    let mut budget = SessionAdmissionBudget::new(1, NOW, 0).unwrap();
    let admitted = f
        .admit_request(
            &mut store,
            &mut ledger,
            &mut budget,
            NOW,
            SessionRequest::new(requested.iter().map(ToString::to_string).collect(), None).unwrap(),
        )
        .unwrap();
    let before = f.authorize(&mut store, &mut ledger, NOW, 0).unwrap();
    assert_eq!(before.scopes(), requested);
    assert_eq!(before.expires_at_ns(), admitted.receipt().expires_at_ns());
    let before_records = snapshot(&mut store, &f, &Fixture::new(2));
    let mut store = store.reopen();
    let mut ledger = f
        .ledger_at(NOW)
        .restore_records(
            ledger.observed_at_ns(),
            ledger.occupancy(),
            store.retained_logins(),
        )
        .unwrap();
    let after = authorize_session(
        &mut store,
        &mut ledger,
        &SessionContext {
            authority: f.authority(NOW),
            max_ttl: SessionTtlPolicy::new(5, 30).unwrap().maximum(),
            role: &f.role,
            required_scopes: &required,
        },
        policy(0),
        Session::clone,
    )
    .unwrap();
    assert_eq!(after, before, "maximum-size session must survive reopen");
    assert_eq!(snapshot(&mut store, &f, &Fixture::new(2)), before_records);
}

/// Only the admitted caller with current issuer authority and an admitted scope
/// may invoke the operation. Changed key/mechanism, withdrawn grants and expired
/// or shortened authority refuse without invoking it. Refresh and exact expiry
/// preserve records, including after reopening the host's persisted records.
pub fn assert_authorization<B: Backend>() {
    let f = Fixture::new(1);
    let other = Fixture::new(2);
    let read = RequiredScopes::new(vec!["read".into()]).unwrap();
    // The issuer grants write, but this session requested only read.
    let write = RequiredScopes::new(vec!["write".into()]).unwrap();
    for reopen in [false, true] {
        let mut store = B::fresh(0, limits(4, 65_536));
        let mut ledger = f.ledger();
        let mut budget = SessionAdmissionBudget::new(1, NOW, 0).unwrap();
        let expires = f
            .admit(&mut store, &mut ledger, &mut budget, NOW)
            .unwrap()
            .receipt()
            .expires_at_ns();
        let before = snapshot(&mut store, &f, &other);
        if reopen {
            store = store.reopen();
            ledger = f
                .ledger_at(NOW)
                .restore_records(
                    ledger.observed_at_ns(),
                    ledger.occupancy(),
                    store.retained_logins(),
                )
                .unwrap();
            assert_eq!(snapshot(&mut store, &f, &other), before);
        }
        let counts = ledger.occupancy();
        let mut wrong_issuer = f.authority(NOW);
        wrong_issuer.issuer = Principal::from_slice(&[99]);
        let mut rotated_key = f.authority(NOW);
        rotated_key.key_binding_id[0] ^= 1;
        let mut changed_mechanism = f.authority(NOW);
        changed_mechanism.mechanism = PortableMechanism::Certified;
        let withdrawn_grants = ValidatedGrants::new(vec![DelegatedRoleGrant {
            target: f.role.clone(),
            scopes: vec!["write".parse().unwrap()],
        }])
        .unwrap();
        let mut withdrawn = f.authority(NOW);
        withdrawn.grants = &withdrawn_grants;
        let mut expired_authority = f.authority(NOW);
        expired_authority.authority_expires_at_ns = NOW;
        let mut expired_enrollment = f.authority(NOW);
        expired_enrollment.enrollment_expires_at_ns = NOW;
        let mut shortened_enrollment = f.authority(NOW);
        shortened_enrollment.enrollment_expires_at_ns = expires - 1;
        let mut refreshed = f.authority(NOW);
        refreshed.authority_not_before_ns = NOW;
        refreshed.authority_expires_at_ns += SECOND;
        refreshed.enrollment_expires_at_ns += SECOND;
        for (case, authority, required, expected) in [
            ("admitted read", f.authority(NOW), &read, Ok(f.caller())),
            ("refreshed authority", refreshed, &read, Ok(f.caller())),
            (
                "unadmitted scope",
                f.authority(NOW),
                &write,
                Err(SessionError::ScopeRejected),
            ),
            (
                "unadmitted caller",
                other.authority(NOW),
                &read,
                Err(SessionError::MissingOrExpired),
            ),
            (
                "different issuer, same subject",
                wrong_issuer,
                &read,
                Err(SessionError::StaleAuthority),
            ),
            (
                "rotated issuer key",
                rotated_key,
                &read,
                Err(SessionError::StaleAuthority),
            ),
            (
                "changed proof mechanism",
                changed_mechanism,
                &read,
                Err(SessionError::StaleAuthority),
            ),
            (
                "withdrawn admitted scope",
                withdrawn,
                &read,
                Err(SessionError::ScopeRejected),
            ),
            (
                "expired issuer authority",
                expired_authority,
                &read,
                Err(SessionError::StaleAuthority),
            ),
            (
                "expired receiver enrollment",
                expired_enrollment,
                &read,
                Err(SessionError::StaleAuthority),
            ),
            (
                "session outlives shortened enrollment",
                shortened_enrollment,
                &read,
                Err(SessionError::StaleAuthority),
            ),
            (
                "before expiry",
                f.authority(expires - 1),
                &read,
                Ok(f.caller()),
            ),
            (
                "exact expiry",
                f.authority(expires),
                &read,
                Err(SessionError::MissingOrExpired),
            ),
            (
                "expiry before scope refusal",
                f.authority(expires),
                &write,
                Err(SessionError::MissingOrExpired),
            ),
            (
                "after expiry",
                f.authority(expires + 1),
                &read,
                Err(SessionError::MissingOrExpired),
            ),
        ] {
            let mut invoked = false;
            let result = authorize_session(
                &mut store,
                &mut ledger,
                &SessionContext {
                    authority,
                    max_ttl: SessionTtlPolicy::new(5, 30).unwrap().maximum(),
                    role: &f.role,
                    required_scopes: required,
                },
                policy(0),
                |session| {
                    invoked = true;
                    assert_eq!(session.issuer(), f.claims.issuer);
                    assert_eq!(session.subject(), f.claims.account_subject);
                    session.caller()
                },
            );
            assert_eq!(invoked, expected.is_ok(), "{case}; reopened={reopen}");
            assert_eq!(result, expected, "{case}; reopened={reopen}");
            assert_eq!(snapshot(&mut store, &f, &other), before, "{case}");
            assert_eq!(ledger.occupancy(), counts, "{case}");
        }
    }
}

/// Failed admission is atomic, consumes proof credit, and exact retry preserves
/// the original lease after proof expiry. Uses fresh disposable backing.
pub fn assert_admission_retry<B: Backend>() {
    let f = Fixture::new(1);
    let other = Fixture::new(2);
    let mut store = B::fresh(0, limits(4, 65_536));
    let mut ledger = f.ledger();
    let mut budget = SessionAdmissionBudget::new(1, NOW, 0).unwrap();
    let before = snapshot(&mut store, &f, &other);
    store.fail_next_commit();
    assert_eq!(
        f.admit(&mut store, &mut ledger, &mut budget, NOW),
        Err(SessionError::StorageFailure)
    );
    assert_eq!(
        snapshot(&mut store, &f, &other),
        before,
        "failed admission must preserve every record"
    );
    assert_eq!(ledger.occupancy(), (0, 0));
    // Verification work remains charged even when the host commit fails.
    assert_eq!(
        f.admit(&mut store, &mut ledger, &mut budget, NOW),
        Err(SessionError::Credential(
            ic_auth::portable_credential::PortableCredentialError::Ed25519(
                ic_auth::portable_signature::PortableSignatureError::Budget(
                    ic_auth::admission_budget::AdmissionBudgetError::Exhausted
                )
            )
        ))
    );
    let mut budget = SessionAdmissionBudget::new(1, NOW, 0).unwrap();
    let created = f.admit(&mut store, &mut ledger, &mut budget, NOW).unwrap();
    assert!(matches!(created, SessionEstablished::Created(_)));
    let after = snapshot(&mut store, &f, &other);
    assert_eq!(
        f.admit(&mut store, &mut ledger, &mut budget, NOW + 3 * SECOND),
        Ok(SessionEstablished::ExactRetry(created.receipt()))
    );
    assert_eq!(
        snapshot(&mut store, &f, &other),
        after,
        "retry must preserve the original lease and records after proof expiry"
    );
}

/// Reopening retains authorization; failed logout preserves it; committed logout
/// survives another reopen while retaining consumed-proof evidence.
pub fn assert_logout_reopen<B: Backend>() {
    let f = Fixture::new(1);
    let other = Fixture::new(2);
    let mut store = B::fresh(0, limits(4, 65_536));
    let mut ledger = f.ledger();
    let mut budget = SessionAdmissionBudget::new(1, NOW, 0).unwrap();
    f.admit(&mut store, &mut ledger, &mut budget, NOW).unwrap();
    let after = snapshot(&mut store, &f, &other);
    let mut store = store.reopen();
    let observed = ledger.observed_at_ns();
    ledger = f
        .ledger_at(NOW + 3 * SECOND)
        .restore_records(observed, ledger.occupancy(), store.retained_logins())
        .unwrap();
    assert!(
        f.authorize(&mut store, &mut ledger, NOW + 3 * SECOND, 0)
            .is_ok()
    );
    assert_eq!(snapshot(&mut store, &f, &other), after);
    let mut logout_budget = SessionAdmissionBudget::new(2, SECOND, 0).unwrap();
    store.fail_next_commit();
    assert_eq!(
        logout_session(
            &mut store,
            &mut ledger,
            &mut logout_budget,
            f.caller(),
            NOW + 3 * SECOND
        ),
        LogoutResult::Unavailable(LogoutUnavailable::Storage)
    );
    assert_eq!(
        snapshot(&mut store, &f, &other),
        after,
        "failed logout must preserve session, replay and reservation"
    );
    assert!(
        f.authorize(&mut store, &mut ledger, NOW + 3 * SECOND, 0)
            .is_ok()
    );
    assert!(matches!(
        logout_session(
            &mut store,
            &mut ledger,
            &mut logout_budget,
            f.caller(),
            NOW + 3 * SECOND
        ),
        LogoutResult::Revoked { .. }
    ));
    let revoked = snapshot(&mut store, &f, &other);
    assert!(revoked.sessions[0].is_none());
    assert_eq!(
        revoked.occupancy[0].replays, 1,
        "logout retains replay evidence"
    );
    let mut store = store.reopen();
    ledger = f
        .ledger_at(NOW + 3 * SECOND)
        .restore_records(
            ledger.observed_at_ns(),
            ledger.occupancy(),
            store.retained_logins(),
        )
        .unwrap();
    assert_eq!(
        f.authorize(&mut store, &mut ledger, NOW + 3 * SECOND, 0),
        Err(SessionError::Revocation(
            ic_auth::portable::LoginRevocationError::Revoked
        ))
    );
    assert_eq!(snapshot(&mut store, &f, &other), revoked);
}

/// Replacement retains replay history; generation invalidation survives reopen.
pub fn assert_replacement_generation<B: Backend>() {
    let f = Fixture::new(1);
    let other = Fixture::new(2);
    // A fresh proof for the same login replaces the lease, retaining both replays.
    let mut store = B::fresh(0, limits(4, 65_536));
    let mut ledger = f.ledger();
    let mut budget = SessionAdmissionBudget::new(8, SECOND, 0).unwrap();
    f.admit(&mut store, &mut ledger, &mut budget, NOW).unwrap();
    let mut renewal = Fixture::new(1);
    renewal.claims.nonce = [2; 16];
    assert!(matches!(
        renewal.admit(&mut store, &mut ledger, &mut budget, NOW),
        Ok(SessionEstablished::Replaced(_))
    ));
    let before = snapshot(&mut store, &f, &other);
    assert_eq!(before.occupancy[0].replays, 2);
    assert_eq!(store.advance_generation(), Ok(1));
    assert_eq!(
        f.authorize(&mut store, &mut ledger, NOW, 0),
        Err(SessionError::StaleAuthority)
    );
    assert_eq!(
        f.authorize(&mut store, &mut ledger, NOW, 1),
        Err(SessionError::StaleAuthority)
    );
    let mut store = store.reopen();
    let changed = snapshot(&mut store, &f, &other);
    assert_eq!(changed.generation, 1);
    assert_eq!(changed.logins, before.logins);
    assert_eq!(changed.occupancy[0].replays, 2);
}

/// A restored 90-day login can renew under fresh authority just before its
/// original deadline. Reopening preserves the capped lease; authorization stops
/// exactly at the deadline without cleanup or a write on refusal.
///
/// # Panics
/// Panics when the adapter loses long-lived records or violates the fixed deadline.
pub fn assert_ninety_day_renewal<B: Backend>() {
    let mut f = Fixture::new(1);
    let other = Fixture::new(2);
    f.max_login_ttl_ns = 90 * 86_400 * SECOND;
    f.claims.login_deadline_ns = f.claims.login_started_at_ns + f.max_login_ttl_ns;
    let deadline = f.claims.login_deadline_ns;
    let mut store = B::fresh(0, limits(4, 65_536));
    let mut ledger = f.ledger();
    let mut budget = SessionAdmissionBudget::new(2, SECOND, 0).unwrap();
    store.observe_clock(NOW).unwrap();
    let initial = f.admit(&mut store, &mut ledger, &mut budget, NOW).unwrap();
    assert_eq!(initial.receipt().expires_at_ns(), NOW + 5 * SECOND);
    let before = snapshot(&mut store, &f, &other);
    let mut store = store.reopen();
    assert_eq!(
        snapshot(&mut store, &f, &other),
        before,
        "90-day login records must survive complete reopen"
    );

    let late = deadline - SECOND;
    let mut ledger = f
        .ledger_at(late)
        .restore_records(
            store.observed_at_ns(),
            ledger.occupancy(),
            store.retained_logins(),
        )
        .expect("90-day login ledger must restore completely");
    store.observe_clock(late).unwrap();
    // The issuer supplies a fresh proof of the original account/browser/window;
    // the host independently supplies a fresh protected authority snapshot.
    f.claims.issued_at_ns = late;
    f.claims.admission_expires_at_ns = deadline;
    f.claims.nonce[0] ^= 1;
    let claims = PortableClaimsContext {
        authority: PortableAuthority {
            authority_not_before_ns: late - SECOND,
            authority_expires_at_ns: late + 120 * SECOND,
            enrollment_expires_at_ns: late + 90 * SECOND,
            ..f.authority(late)
        },
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: f.max_login_ttl_ns,
        max_proof_ttl_ns: 60 * SECOND,
    };
    let renewed = establish_session(
        &mut store,
        &mut ledger,
        &mut budget,
        &f.credential(),
        SessionRequest::new(vec!["read".into()], None).unwrap(),
        &SessionAdmissionContext {
            verification: PortableVerificationContext::Ed25519 {
                claims: &claims,
                public_key: &f.prepared,
            },
            role: &f.role,
            ttl: SessionTtlPolicy::new(5, 30).unwrap(),
        },
        policy(0),
    )
    .unwrap();
    assert!(matches!(renewed, SessionEstablished::Replaced(_)));
    let receipt = renewed.receipt();
    assert_eq!(receipt.expires_at_ns(), deadline);
    let renewed_records = snapshot(&mut store, &f, &other);
    assert_eq!(
        renewed_records.logins, before.logins,
        "renewal must retain the original login window"
    );
    let mut store = store.reopen();
    let mut ledger = f
        .ledger_at(late)
        .restore_records(
            store.observed_at_ns(),
            ledger.occupancy(),
            store.retained_logins(),
        )
        .expect("renewed 90-day login ledger must restore completely");
    let mut context = SessionContext {
        authority: claims.authority,
        max_ttl: SessionTtlPolicy::new(5, 30).unwrap().maximum(),
        role: &f.role,
        required_scopes: RequiredScopes::NONE,
    };
    for now in [late, deadline - 1, deadline, deadline + 1] {
        context.authority.now_ns = now;
        let expected = if now < deadline {
            Ok(receipt.expires_at_ns())
        } else {
            Err(SessionError::MissingOrExpired)
        };
        assert_eq!(
            authorize_session(
                &mut store,
                &mut ledger,
                &context,
                policy(0),
                Session::expires_at_ns
            ),
            expected,
            "restored renewal must expire at the original login deadline"
        );
        assert_eq!(
            snapshot(&mut store, &f, &other),
            renewed_records,
            "authorization and expiry refusal must preserve records without pruning"
        );
    }
}

/// Session, global/per-account replay and encoded-byte exhaustion refuse without
/// changing records or ledger. Replacement must still respect replay capacity.
pub fn assert_capacity<B: Backend>() {
    let f = Fixture::new(1);
    let other = Fixture::new(2);
    let mut duplicate = Fixture::new(3);
    duplicate.claims.account_subject = f.claims.account_subject;
    {
        let mut store = B::fresh(0, SessionLimits::new(4, 8, 1, 65_536).unwrap());
        let mut ledger = f.ledger();
        let mut budget = SessionAdmissionBudget::new(8, SECOND, 0).unwrap();
        f.admit(&mut store, &mut ledger, &mut budget, NOW).unwrap();
        let mut logout_budget = SessionAdmissionBudget::new(8, SECOND, 0).unwrap();
        assert!(matches!(
            logout_session(&mut store, &mut ledger, &mut logout_budget, f.caller(), NOW),
            LogoutResult::Revoked { .. }
        ));
        store = store.reopen();
        // No session remains: the durable replay itself must retain ownership.
        store
            .read(|reader| {
                assert!(reader.session(f.caller())?.is_none());
                assert_eq!(
                    reader
                        .occupancy((f.claims.issuer, f.claims.account_subject))?
                        .replays_for_account,
                    1
                );
                Ok(())
            })
            .unwrap();
        let before = snapshot(&mut store, &f, &duplicate);
        assert_eq!(
            duplicate.admit(&mut store, &mut ledger, &mut budget, NOW),
            Err(SessionError::Capacity)
        );
        assert_eq!(snapshot(&mut store, &f, &duplicate), before);
        other
            .admit(&mut store, &mut ledger, &mut budget, NOW)
            .unwrap();
    }
    let mut store = B::fresh(0, limits(4, 65_536));
    let mut ledger = f.ledger();
    let mut budget = SessionAdmissionBudget::new(8, SECOND, 0).unwrap();
    f.admit(&mut store, &mut ledger, &mut budget, NOW).unwrap();
    let mut logout_budget = SessionAdmissionBudget::new(8, SECOND, 0).unwrap();
    for revoked in [false, true] {
        if revoked {
            assert!(matches!(
                logout_session(&mut store, &mut ledger, &mut logout_budget, f.caller(), NOW),
                LogoutResult::Revoked { .. }
            ));
        }
        store = store.reopen();
        ledger = f
            .ledger_at(NOW)
            .restore_records(
                ledger.observed_at_ns(),
                ledger.occupancy(),
                store.retained_logins(),
            )
            .unwrap();
        let before = snapshot(&mut store, &f, &duplicate);
        assert_eq!(
            duplicate.admit(&mut store, &mut ledger, &mut budget, NOW),
            Err(SessionError::Revocation(
                ic_auth::portable::LoginRevocationError::Revoked
            ))
        );
        assert_eq!(snapshot(&mut store, &f, &duplicate), before);
        assert_eq!(ledger.occupancy(), (1, 0));
    }
    other
        .admit(&mut store, &mut ledger, &mut budget, NOW)
        .unwrap();
    for (bounds, expected) in [
        (limits(1, 65_536), SessionError::Capacity),
        (limits(4, 1), SessionError::ByteCapacity),
    ] {
        let mut store = B::fresh(0, bounds);
        let mut ledger = f.ledger();
        let mut budget = SessionAdmissionBudget::new(8, SECOND, 0).unwrap();
        if bounds.max_encoded_bytes() > 1 {
            f.admit(&mut store, &mut ledger, &mut budget, NOW).unwrap();
        }
        let before = snapshot(&mut store, &f, &other);
        let counts = ledger.occupancy();
        assert_eq!(
            other.admit(&mut store, &mut ledger, &mut budget, NOW),
            Err(expected)
        );
        assert_eq!(snapshot(&mut store, &f, &other), before);
        assert_eq!(ledger.occupancy(), counts);
    }
    for (scope, bounds, device) in [
        (
            "global replay",
            SessionLimits::new(4, 1, 4, 65_536).unwrap(),
            2,
        ),
        (
            "account replay",
            SessionLimits::new(4, 8, 1, 65_536).unwrap(),
            1,
        ),
    ] {
        let mut store = B::fresh(0, bounds);
        let mut ledger = f.ledger();
        let mut budget = SessionAdmissionBudget::new(8, SECOND, 0).unwrap();
        let created = f.admit(&mut store, &mut ledger, &mut budget, NOW).unwrap();
        let mut candidate = Fixture::new(device);
        candidate.claims.nonce = [2; 16];
        let before = snapshot(&mut store, &f, &candidate);
        let counts = ledger.occupancy();
        assert_eq!(
            candidate.admit(&mut store, &mut ledger, &mut budget, NOW),
            Err(SessionError::Capacity),
            "{scope} exhaustion must refuse before writing"
        );
        assert_eq!(snapshot(&mut store, &f, &candidate), before);
        assert_eq!(ledger.occupancy(), counts);
        let mut store = store.reopen();
        assert_eq!(snapshot(&mut store, &f, &candidate), before);
        assert_eq!(
            f.admit(&mut store, &mut ledger, &mut budget, NOW),
            Ok(SessionEstablished::ExactRetry(created.receipt())),
            "a full replay pool must still allow the original exact retry"
        );
        if device == 1 {
            // Per-account saturation does not exhaust the receiver-wide pool.
            assert!(matches!(
                other.admit(&mut store, &mut ledger, &mut budget, NOW),
                Ok(SessionEstablished::Created(_))
            ));
        }
    }
}

/// Cleanup errors preserve records across reopen, zero/empty work never commits,
/// and a large caller budget still removes at most 128 expired records. Live
/// sessions/replays and login reservations survive session/replay maintenance.
///
/// # Panics
/// Panics on a violated cleanup contract, unexpected adapter error or fixture refusal.
pub fn assert_pruning<B: Backend>() {
    let mut f = Fixture::new(1);
    let mut other = Fixture::new(2);
    let mut store = B::fresh(0, SessionLimits::new(4, 132, 132, 65_536).unwrap());
    let mut ledger = f.ledger();
    let mut budget = SessionAdmissionBudget::new(256, 1, 0).unwrap();
    for nonce in 0..129 {
        f.claims.nonce = [nonce; 16];
        f.admit(&mut store, &mut ledger, &mut budget, NOW).unwrap();
    }
    other.claims.issued_at_ns += SECOND;
    other.claims.admission_expires_at_ns += SECOND;
    other
        .admit(&mut store, &mut ledger, &mut budget, NOW + SECOND)
        .unwrap();
    let before = snapshot(&mut store, &f, &other);
    store.fail_next_commit();
    assert_eq!(store.prune(NOW + SECOND, 128), Ok(SessionPruned::default()));
    assert_eq!(
        store.prune(NOW + 2 * SECOND, 0),
        Ok(SessionPruned::default())
    );
    assert_eq!(
        store.prune(NOW + 2 * SECOND, 128),
        Err(SessionError::StorageFailure)
    );
    assert_eq!(
        snapshot(&mut store, &f, &other),
        before,
        "failed pruning must preserve every record"
    );
    let mut store = store.reopen();
    assert_eq!(
        snapshot(&mut store, &f, &other),
        before,
        "failed pruning must preserve every record after reopen"
    );
    let pruned = store.prune(NOW + 2 * SECOND, usize::MAX).unwrap();
    assert!(
        pruned.sessions == 0 && (1..=128).contains(&pruned.replays),
        "cleanup must obey its 128-record work bound"
    );
    let after = snapshot(&mut store, &f, &other);
    assert_eq!(after.sessions, before.sessions);
    assert_eq!(after.logins, before.logins);
    assert_eq!(after.occupancy[0].replays, 130 - pruned.replays);
    assert!(after.replays[1], "live replay must survive cleanup");
    let mut store = store.reopen();
    assert_eq!(snapshot(&mut store, &f, &other), after);
    for _ in pruned.replays..129 {
        assert_eq!(
            store.prune(NOW + 2 * SECOND, 1),
            Ok(SessionPruned {
                sessions: 0,
                replays: 1
            })
        );
    }
    assert!(matches!(
        f.admit(&mut store, &mut ledger, &mut budget, NOW + 2 * SECOND),
        Ok(SessionEstablished::ExactRetry(_))
    ));
    // Exactly one session and one replay are now expired. Hosts may choose
    // either record kind first; each single-record budget must still hold.
    let mut total = SessionPruned::default();
    for _ in 0..2 {
        let pruned = store.prune(NOW + 5 * SECOND, 1).unwrap();
        assert_eq!(pruned.sessions + pruned.replays, 1);
        total.sessions += pruned.sessions;
        total.replays += pruned.replays;
    }
    assert_eq!(
        total,
        SessionPruned {
            sessions: 1,
            replays: 1
        }
    );
    let after = snapshot(&mut store, &f, &other);
    assert!(after.sessions[0].is_none());
    assert_eq!(after.sessions[1], before.sessions[1]);
    assert_eq!(after.logins, before.logins);
    let mut store = store.reopen();
    assert_eq!(snapshot(&mut store, &f, &other), after);
    other
        .authorize(&mut store, &mut ledger, NOW + 5 * SECOND, 0)
        .unwrap();
}

/// Failed generation writes cannot invalidate authority. Committed generation
/// and clock floors survive reopen, and ordinary admission errors cannot erase
/// a separately committed clock observation. Clock authenticity and atomic host
/// configuration changes still need the consumer's installed qualification.
///
/// # Panics
/// Panics on a violated metadata contract, unexpected adapter error or fixture refusal.
pub fn assert_metadata_reopen<B: Backend>() {
    let f = Fixture::new(1);
    let other = Fixture::new(2);
    let mut store = B::fresh(0, limits(4, 65_536));
    let mut ledger = f.ledger();
    let mut budget = SessionAdmissionBudget::new(8, SECOND, 0).unwrap();
    f.admit(&mut store, &mut ledger, &mut budget, NOW).unwrap();
    store.observe_clock(NOW).unwrap();
    let before = snapshot(&mut store, &f, &other);
    store.fail_next_commit();
    assert_eq!(
        store.advance_generation(),
        Err(SessionError::StorageFailure)
    );
    assert_eq!(
        snapshot(&mut store, &f, &other),
        before,
        "failed generation write must preserve authority"
    );
    let mut store = store.reopen();
    assert_eq!(snapshot(&mut store, &f, &other), before);
    assert_eq!(
        store.observed_at_ns(),
        NOW,
        "clock floor must survive reopen"
    );
    assert_eq!(
        store.observe_clock(NOW - 1),
        Err(SessionError::Revocation(
            ic_auth::portable::LoginRevocationError::Clock
        ))
    );
    assert_eq!(store.observed_at_ns(), NOW);
    f.authorize(&mut store, &mut ledger, NOW, 0).unwrap();
    store.observe_clock(NOW + SECOND).unwrap();
    store.fail_next_commit();
    assert_eq!(
        other.admit(&mut store, &mut ledger, &mut budget, NOW + SECOND),
        Err(SessionError::StorageFailure)
    );
    let mut store = store.reopen();
    assert_eq!(snapshot(&mut store, &f, &other), before);
    assert_eq!(
        store.observed_at_ns(),
        NOW + SECOND,
        "clock floor must survive refused admission"
    );
    assert_eq!(
        store.observe_clock(NOW),
        Err(SessionError::Revocation(
            ic_auth::portable::LoginRevocationError::Clock
        ))
    );
    assert_eq!(store.advance_generation(), Ok(1));
    let mut store = store.reopen();
    assert_eq!(store.observed_at_ns(), NOW + SECOND);
    assert_eq!(snapshot(&mut store, &f, &other).generation, 1);
    assert_eq!(
        f.authorize(&mut store, &mut ledger, NOW + SECOND, 1),
        Err(SessionError::StaleAuthority)
    );
}

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    generation: u64,
    sessions: [Option<Vec<u8>>; 2],
    occupancy: [SessionOccupancy; 2],
    replays: [bool; 2],
    logins: Vec<Vec<u8>>,
}
fn snapshot(store: &mut impl Backend, f: &Fixture, other: &Fixture) -> Snapshot {
    let mut logins: Vec<_> = store
        .retained_logins()
        .iter()
        .map(|r| r.encode().unwrap())
        .collect();
    logins.sort();
    store
        .read(|reader| {
            Ok(Snapshot {
                generation: reader.generation(),
                sessions: [f.caller(), other.caller()]
                    .map(|caller| reader.session(caller).unwrap().map(|s| s.encode().unwrap())),
                occupancy: [
                    reader.occupancy((f.claims.issuer, f.claims.account_subject))?,
                    reader.occupancy((other.claims.issuer, other.claims.account_subject))?,
                ],
                replays: [
                    reader.replay_exists(claims_hash(&f.claims).unwrap())?,
                    reader.replay_exists(claims_hash(&other.claims).unwrap())?,
                ],
                logins,
            })
        })
        .unwrap()
}
fn limits(sessions: usize, bytes: usize) -> SessionLimits {
    SessionLimits::new(sessions, 8, 4, bytes).unwrap()
}
fn policy(generation: u64) -> SessionPolicy {
    SessionPolicy {
        generation,
        enabled: true,
        subject_admissible: true,
    }
}
struct Fixture {
    claims: PortableClaims,
    key: SigningKey,
    prepared: PreparedIssuerKey,
    role: AuthRole,
    grants: ValidatedGrants,
    max_login_ttl_ns: u64,
}
impl Fixture {
    fn new(device: u8) -> Self {
        let key = SigningKey::from_bytes(&[42; 32]);
        let prepared = PreparedIssuerKey::new(key.verifying_key().to_bytes()).unwrap();
        let role: AuthRole = "user".parse().unwrap();
        let claims = PortableClaims {
            audience: AudienceId {
                canonical_network_id: CanonicalId::from_bytes([1; 32]),
                fleet_id: CanonicalId::from_bytes([2; 32]),
            },
            mechanism: PortableMechanism::Ed25519,
            issuer: Principal::from_slice(&[1]),
            device_slot: 0,
            device_generation: 1,
            account_subject: Principal::from_slice(&[device, 0]),
            key_binding_id: prepared.binding(),
            browser_public_key: SigningKey::from_bytes(&[device; 32])
                .verifying_key()
                .to_bytes(),
            login_started_at_ns: NOW - SECOND,
            login_deadline_ns: NOW + 60 * SECOND,
            issued_at_ns: NOW,
            admission_expires_at_ns: NOW + 2 * SECOND,
            nonce: [1; 16],
            grants: vec![DelegatedRoleGrant {
                target: role.clone(),
                scopes: vec!["read".parse().unwrap(), "write".parse().unwrap()],
            }],
        };
        let grants = ValidatedGrants::new(claims.grants.clone()).unwrap();
        Self {
            claims,
            key,
            prepared,
            role,
            grants,
            max_login_ttl_ns: 86_400 * SECOND,
        }
    }
    fn caller(&self) -> Principal {
        browser_principal(&self.claims.browser_public_key)
    }
    fn ledger(&self) -> MemoryLoginRevocations {
        self.ledger_at(0)
    }
    fn ledger_at(&self, now: u64) -> MemoryLoginRevocations {
        MemoryLoginRevocations::new(self.claims.audience, 4, 4, self.max_login_ttl_ns, now).unwrap()
    }
    fn authority(&self, now: u64) -> PortableAuthority<'_> {
        PortableAuthority {
            audience: &self.claims.audience,
            issuer: self.claims.issuer,
            mechanism: self.claims.mechanism,
            key_binding_id: self.claims.key_binding_id,
            grants: &self.grants,
            caller: self.caller(),
            now_ns: now,
            authority_not_before_ns: 0,
            authority_expires_at_ns: NOW + 120 * SECOND,
            enrollment_expires_at_ns: NOW + 90 * SECOND,
            min_login_started_at_ns: 0,
        }
    }
    fn admit(
        &self,
        store: &mut impl SessionStore,
        ledger: &mut MemoryLoginRevocations,
        budget: &mut SessionAdmissionBudget,
        now: u64,
    ) -> Result<SessionEstablished, SessionError> {
        self.admit_request(
            store,
            ledger,
            budget,
            now,
            SessionRequest::new(vec!["read".into()], None).unwrap(),
        )
    }
    fn admit_request(
        &self,
        store: &mut impl SessionStore,
        ledger: &mut MemoryLoginRevocations,
        budget: &mut SessionAdmissionBudget,
        now: u64,
        request: SessionRequest,
    ) -> Result<SessionEstablished, SessionError> {
        let claims = PortableClaimsContext {
            authority: self.authority(now),
            min_proof_issued_at_ns: 0,
            max_login_ttl_ns: self.max_login_ttl_ns,
            max_proof_ttl_ns: 60 * SECOND,
        };
        establish_session(
            store,
            ledger,
            budget,
            &self.credential(),
            request,
            &SessionAdmissionContext {
                verification: PortableVerificationContext::Ed25519 {
                    claims: &claims,
                    public_key: &self.prepared,
                },
                role: &self.role,
                ttl: SessionTtlPolicy::new(5, 30).unwrap(),
            },
            policy(0),
        )
    }
    fn credential(&self) -> PortableCredential {
        PortableCredential {
            claims: encode_claims(&self.claims).unwrap(),
            proof: PortableProof::Ed25519 {
                signature: self
                    .key
                    .sign(&claims_hash(&self.claims).unwrap())
                    .to_bytes()
                    .to_vec(),
            },
        }
    }
    fn authorize(
        &self,
        store: &mut impl SessionStore,
        ledger: &mut MemoryLoginRevocations,
        now: u64,
        generation: u64,
    ) -> Result<Session, SessionError> {
        authorize_session(
            store,
            ledger,
            &SessionContext {
                authority: self.authority(now),
                max_ttl: SessionTtlPolicy::new(5, 30).unwrap().maximum(),
                role: &self.role,
                required_scopes: RequiredScopes::NONE,
            },
            policy(generation),
            Session::clone,
        )
    }
}

/// Produce one current trusted-storage fixture through real Ed25519 admission.
/// Protected authority is synthetic test configuration supplied by the fixture;
/// this helper is not an authentication API for untrusted credentials.
///
/// # Panics
/// Panics if the fixture fails any normal engine verification or record bound.
pub fn admitted_records(
    claims: &PortableClaims,
    credential: &PortableCredential,
    issuer_key: [u8; 32],
    now_ns: u64,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut store = MemorySessionStore::new(0, SessionLimits::new(1, 1, 1, 65_536).unwrap());
    let mut ledger =
        MemoryLoginRevocations::new(claims.audience, 1, 1, 90 * 86_400_000_000_000, now_ns - 1)
            .unwrap();
    let mut budget = SessionAdmissionBudget::new(1, 1, now_ns - 1).unwrap();
    let grants = ValidatedGrants::new(claims.grants.clone()).unwrap();
    let context = PortableClaimsContext {
        authority: PortableAuthority {
            audience: &claims.audience,
            issuer: claims.issuer,
            mechanism: claims.mechanism,
            key_binding_id: claims.key_binding_id,
            grants: &grants,
            caller: browser_principal(&claims.browser_public_key),
            now_ns,
            authority_not_before_ns: 0,
            authority_expires_at_ns: u64::MAX,
            enrollment_expires_at_ns: u64::MAX,
            min_login_started_at_ns: 0,
        },
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: 90 * 86_400_000_000_000,
        max_proof_ttl_ns: 60_000_000_000,
    };
    establish_session(
        &mut store,
        &mut ledger,
        &mut budget,
        credential,
        SessionRequest::new(vec!["read".into()], None).unwrap(),
        &SessionAdmissionContext {
            verification: PortableVerificationContext::Ed25519 {
                claims: &context,
                public_key: &PreparedIssuerKey::new(issuer_key).unwrap(),
            },
            role: &"user".parse().unwrap(),
            ttl: SessionTtlPolicy::new(30, 60).unwrap(),
        },
        SessionPolicy {
            generation: 0,
            enabled: true,
            subject_admissible: true,
        },
    )
    .unwrap();
    (
        store.session_records().next().unwrap().encode().unwrap(),
        store.replay_records().next().unwrap().encode().unwrap(),
        ledger.records().next().unwrap().encode().unwrap(),
    )
}
