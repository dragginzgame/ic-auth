//! Complete-owner scenarios: no private ledger export or reconstruction.
use super::{
    Fixture, LogoutResult, NOW, PortableAuthority, PortableClaimsContext,
    PortableVerificationContext, RequiredScopes, SECOND, Session, SessionAdmissionContext,
    SessionContext, SessionError, SessionEstablished, SessionLimits, SessionRequest,
    SessionTtlPolicy, policy,
};
use ic_auth::persistent::receiver::{BudgetPolicy, LoginLimits, ReceiverConfig, ReceiverStore};
use ic_storage::Memory;

/// Host transaction rejection, distinct from Auth's operation result.
#[derive(Debug, PartialEq, Eq)]
pub struct CommitRejected;

/// Disposable host backing for the complete persistent owner. `fresh` creates
/// a ReceiverStore on checked, exclusive host leases. `reopen` drops the runtime
/// and opens the same durable backing, never reconstructing private records.
/// `transaction` executes the closure exactly once. A requested failure occurs
/// after execution but before host commit and rolls back *all* writes, including
/// clock/budget metadata; discard and reopen the runtime after rollback. Normal
/// Auth errors are values, not a request to roll back their consumed budgets.
/// This test interface is not a production transaction abstraction.
pub trait ReceiverHost: Sized {
    /// Memory implementation backing the disposable persistent receiver.
    type Memory: Memory;
    /// Create isolated checked leases and initialize a fresh receiver with trusted inputs.
    fn fresh(config: ReceiverConfig, generation: u64, now_ns: u64) -> Self;
    /// Access the live complete Auth owner without extracting its private ledger.
    fn store(&mut self) -> &mut ReceiverStore<Self::Memory>;
    /// Drop runtime state and reopen the same backing at the supplied test clock.
    fn reopen(self, now_ns: u64) -> Self;
    /// Arrange rollback of every write made by the next host transaction.
    fn fail_next_commit(&mut self);
    /// Run once under the test host transaction, rolling back all writes on injected failure.
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut ReceiverStore<Self::Memory>) -> T,
    ) -> Result<T, CommitRejected>;
}

const DAYS_90: u64 = 90 * 86_400 * SECOND;
fn fresh<H: ReceiverHost>(f: &Fixture) -> H {
    H::fresh(
        ReceiverConfig::new(
            f.claims.audience,
            SessionLimits::new(4, 32, 16, 262_144).unwrap(),
            LoginLimits {
                admitted: 4,
                unknown: 4,
                maximum_ttl_ns: DAYS_90,
                devices_per_account: 4,
            },
            BudgetPolicy {
                burst: 32,
                refill_interval_ns: 1,
            },
            BudgetPolicy {
                burst: 32,
                refill_interval_ns: 1,
            },
            ic_auth::admission_budget::VerificationCosts {
                ed25519: 1,
                certified: 1,
            },
        )
        .unwrap(),
        0,
        NOW - 100,
    )
}
fn authority(f: &Fixture, now: u64) -> PortableAuthority<'_> {
    PortableAuthority {
        authority_expires_at_ns: u64::MAX,
        enrollment_expires_at_ns: u64::MAX,
        ..f.authority(now)
    }
}
fn admit<M: Memory>(
    f: &Fixture,
    s: &mut ReceiverStore<M>,
    now: u64,
) -> Result<SessionEstablished, SessionError> {
    let claims = PortableClaimsContext {
        authority: authority(f, now),
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: DAYS_90,
        max_proof_ttl_ns: 60 * SECOND,
    };
    s.admit(
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
        policy(s.generation()),
    )
}
fn authorize<M: Memory>(
    f: &Fixture,
    s: &mut ReceiverStore<M>,
    now: u64,
) -> Result<Session, SessionError> {
    let generation = s.generation();
    s.authenticate(
        &SessionContext {
            authority: authority(f, now),
            max_ttl: SessionTtlPolicy::new(5, 30).unwrap().maximum(),
            role: &f.role,
            required_scopes: RequiredScopes::NONE,
        },
        policy(generation),
        Session::clone,
    )
}

/// Admission, expired-proof exact retry, durable logout and generation fencing.
/// # Panics
/// Panics on a violated complete-owner or host transaction contract.
pub fn assert_receiver_lifecycle<H: ReceiverHost>() {
    let f = Fixture::new(1);
    let peer = Fixture::new(2);
    let mut h = fresh::<H>(&f);
    h.fail_next_commit();
    assert_eq!(h.transaction(|s| admit(&f, s, NOW)), Err(CommitRejected));
    h = h.reopen(NOW);
    assert_eq!(
        h.store().occupancy(),
        Default::default(),
        "failed admission leaked records"
    );
    assert!(authorize(&f, h.store(), NOW).is_err());
    let receipt = h.transaction(|s| admit(&f, s, NOW)).unwrap().unwrap();
    h.transaction(|s| admit(&peer, s, NOW)).unwrap().unwrap();
    h = h.reopen(NOW + 3 * SECOND);
    let counts = h.store().occupancy();
    let retry = h
        .transaction(|s| admit(&f, s, NOW + 3 * SECOND))
        .unwrap()
        .unwrap();
    assert!(matches!(retry, SessionEstablished::ExactRetry(_)));
    assert_eq!(retry.receipt(), receipt.receipt());
    assert_eq!(h.store().occupancy(), counts);
    h.fail_next_commit();
    assert_eq!(
        h.transaction(|s| s.logout(f.caller(), NOW + 3 * SECOND)),
        Err(CommitRejected)
    );
    h = h.reopen(NOW + 3 * SECOND);
    assert_eq!(
        h.store().occupancy(),
        counts,
        "failed logout leaked records"
    );
    assert!(authorize(&f, h.store(), NOW + 3 * SECOND).is_ok());
    assert!(matches!(
        h.transaction(|s| s.logout(f.caller(), NOW + 3 * SECOND))
            .unwrap(),
        LogoutResult::Revoked { .. }
    ));
    h = h.reopen(NOW + 3 * SECOND);
    assert!(authorize(&f, h.store(), NOW + 3 * SECOND).is_err());
    assert!(admit(&f, h.store(), NOW + 3 * SECOND).is_err());
    assert!(authorize(&peer, h.store(), NOW + 3 * SECOND).is_ok());
    h.fail_next_commit();
    assert_eq!(
        h.transaction(ReceiverStore::advance_generation),
        Err(CommitRejected)
    );
    h = h.reopen(NOW + 3 * SECOND);
    assert_eq!(
        h.store().generation(),
        0,
        "failed generation leaked authority"
    );
    assert!(authorize(&peer, h.store(), NOW + 3 * SECOND).is_ok());
    assert_eq!(h.transaction(ReceiverStore::advance_generation), Ok(Ok(1)));
    h = h.reopen(NOW + 3 * SECOND);
    assert_eq!(
        authorize(&peer, h.store(), NOW + 3 * SECOND),
        Err(SessionError::StaleAuthority)
    );
}

/// A renewed short lease retains the original ninety-day deadline across reopen.
/// # Panics
/// Panics if renewal, rollback, expiry or durable identity differs.
pub fn assert_receiver_ninety_day_renewal<H: ReceiverHost>() {
    let mut f = Fixture::new(1);
    f.claims.login_deadline_ns = f.claims.login_started_at_ns + DAYS_90;
    let mut h = fresh::<H>(&f);
    h.transaction(|s| admit(&f, s, NOW)).unwrap().unwrap();
    let deadline = f.claims.login_deadline_ns;
    h = h.reopen(deadline - SECOND);
    assert!(authorize(&f, h.store(), deadline - SECOND).is_err());
    f.claims.issued_at_ns = deadline - SECOND;
    f.claims.admission_expires_at_ns = deadline;
    f.claims.nonce[0] += 1;
    let counts = h.store().occupancy();
    h.fail_next_commit();
    assert_eq!(
        h.transaction(|s| admit(&f, s, deadline - SECOND)),
        Err(CommitRejected)
    );
    h = h.reopen(deadline - SECOND);
    assert_eq!(
        h.store().occupancy(),
        counts,
        "failed renewal leaked records"
    );
    assert!(authorize(&f, h.store(), deadline - SECOND).is_err());
    let renewed = h
        .transaction(|s| admit(&f, s, deadline - SECOND))
        .unwrap()
        .unwrap();
    assert_eq!(renewed.receipt().expires_at_ns(), deadline);
    h = h.reopen(deadline - SECOND);
    assert_eq!(
        authorize(&f, h.store(), deadline - SECOND)
            .unwrap()
            .expires_at_ns(),
        deadline
    );
    assert!(authorize(&f, h.store(), deadline).is_err());
    assert!(authorize(&f, h.store(), deadline + 1).is_err());
    let counts = h.store().occupancy();
    h.fail_next_commit();
    assert_eq!(
        h.transaction(|s| s.prune(deadline + SECOND, 128)),
        Err(CommitRejected)
    );
    h = h.reopen(deadline + SECOND);
    assert_eq!(h.store().occupancy(), counts, "failed prune leaked records");
    h.transaction(|s| s.prune(deadline + SECOND, 128))
        .unwrap()
        .unwrap();
    h = h.reopen(deadline + SECOND);
    assert_eq!(h.store().occupancy().sessions, 0);
    assert_eq!(
        h.store().occupancy().admitted_logins,
        1,
        "slot fence must survive expiry"
    );
}

/// Run all complete-owner scenarios on fresh disposable host backing.
/// # Panics
/// Panics if any scenario fails. This proves neither crash atomicity nor endpoints.
pub fn assert_receiver_contract<H: ReceiverHost>() {
    assert_receiver_lifecycle::<H>();
    assert_receiver_ninety_day_renewal::<H>();
    assert_receiver_authority_and_devices::<H>();
}

/// Wrong authority/scope never reaches the application; unknown logout fences
/// and device replacement remain durable without exporting the login ledger.
/// # Panics
/// Panics on unauthorized projection, forgotten denial or displaced-device access.
pub fn assert_receiver_authority_and_devices<H: ReceiverHost>() {
    let f = Fixture::new(1);
    let mut peer = Fixture::new(2);
    peer.claims.account_subject = f.claims.account_subject;
    peer.claims.device_slot = 1;
    let mut replacement = Fixture::new(3);
    replacement.claims.account_subject = f.claims.account_subject;
    replacement.claims.device_generation = 2;
    let unknown = Fixture::new(4);
    let mut h = fresh::<H>(&f);
    h.transaction(|s| admit(&f, s, NOW)).unwrap().unwrap();
    h.transaction(|s| admit(&peer, s, NOW)).unwrap().unwrap();
    h = h.reopen(NOW);
    let write = RequiredScopes::new(vec!["write".into()]).unwrap();
    let mut wrong = authority(&f, NOW);
    wrong.key_binding_id[0] ^= 1;
    let mut expired = authority(&f, NOW);
    expired.authority_expires_at_ns = NOW;
    let before = h.store().occupancy();
    for (authority, required) in [
        (wrong, RequiredScopes::NONE),
        (expired, RequiredScopes::NONE),
        (authority(&f, NOW), &write),
    ] {
        let mut invoked = false;
        let result = h.store().authenticate(
            &SessionContext {
                authority,
                max_ttl: SessionTtlPolicy::new(5, 30).unwrap().maximum(),
                role: &f.role,
                required_scopes: required,
            },
            policy(0),
            |_| {
                invoked = true;
            },
        );
        assert!(result.is_err());
        assert!(!invoked, "unauthorized projection ran");
        assert_eq!(h.store().occupancy(), before);
    }
    h.fail_next_commit();
    assert_eq!(
        h.transaction(|s| admit(&replacement, s, NOW)),
        Err(CommitRejected)
    );
    h = h.reopen(NOW);
    assert!(authorize(&f, h.store(), NOW).is_ok());
    h.transaction(|s| admit(&replacement, s, NOW))
        .unwrap()
        .unwrap();
    h = h.reopen(NOW);
    assert!(authorize(&f, h.store(), NOW).is_err());
    assert!(authorize(&replacement, h.store(), NOW).is_ok());
    assert!(authorize(&peer, h.store(), NOW).is_ok());
    h.fail_next_commit();
    assert_eq!(
        h.transaction(|s| s.logout(unknown.caller(), NOW)),
        Err(CommitRejected)
    );
    h = h.reopen(NOW);
    assert_eq!(h.store().occupancy().unknown_denials, 0);
    h.transaction(|s| s.logout(unknown.caller(), NOW)).unwrap();
    h = h.reopen(NOW);
    assert_eq!(h.store().occupancy().unknown_denials, 1);
    assert!(
        admit(&unknown, h.store(), NOW).is_err(),
        "unknown logout fence forgotten"
    );
    assert!(authorize(&peer, h.store(), NOW).is_ok());
}
