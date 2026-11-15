//! Independent receiver scenarios from the pinned session/revocation contract.
//! These are trusted-state fact drivers, not proof admission or storage adapters.
use ic_auth::{
    receiver_policy::{
        DeviceFacts, LoginBinding, LoginFacts, ReceiverSnapshot, RetryDecision, SessionAuthority,
        SessionFacts, SessionPolicyError as E, assess_retry, assess_session,
    },
    session_request::SessionRequest,
};
use ic_auth_protocol_types::{
    AudienceId, AuthRole, CanonicalId, DelegatedRoleGrant, PortableMechanism, Principal, ScopeLabel,
};
const SECOND: u64 = 1_000_000_000;
fn principal(n: u8) -> Principal {
    Principal::from_slice(&[n])
}
fn caller(n: u8) -> Principal {
    let mut bytes = [n; 29];
    bytes[28] = 2;
    Principal::from_slice(&bytes)
}
fn request(scopes: &[&str], ttl: Option<u64>) -> SessionRequest {
    SessionRequest::new(scopes.iter().map(|s| (*s).into()).collect(), ttl).unwrap()
}
fn labels(scopes: &[&str]) -> Vec<ScopeLabel> {
    scopes.iter().map(|s| s.parse().unwrap()).collect()
}
fn grant(role: &str, scopes: &[&str]) -> DelegatedRoleGrant {
    DelegatedRoleGrant {
        target: role.parse().unwrap(),
        scopes: labels(scopes),
    }
}
fn hash(hex: &str) -> [u8; 32] {
    std::array::from_fn(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap())
}
struct Scenario {
    role: AuthRole,
    scopes: Vec<ScopeLabel>,
    grants: Vec<DelegatedRoleGrant>,
}
impl Scenario {
    fn new() -> Self {
        Self {
            role: "app".parse().unwrap(),
            scopes: labels(&["read", "write"]),
            grants: vec![grant("app", &["read", "write"])],
        }
    }
    fn audience() -> AudienceId {
        AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        }
    }
    fn binding() -> LoginBinding {
        LoginBinding {
            issuer: principal(1),
            subject: principal(3),
            device_slot: 2,
            device_generation: 7,
            started_at_ns: 100 * SECOND,
            deadline_ns: 300 * SECOND,
        }
    }
    fn session(&self) -> SessionFacts<'_> {
        SessionFacts {
            caller: caller(10),
            audience: Self::audience(),
            login: Self::binding(),
            mechanism: PortableMechanism::Ed25519,
            key_binding_id: [4; 32],
            role: &self.role,
            scopes: &self.scopes,
            authority_generation: 9,
            established_at_ns: 120 * SECOND,
            expires_at_ns: 150 * SECOND,
            fingerprint: [5; 32],
            // Retained independent request digest, not calculated by the tested helper.
            request_hash: hash("d99fc919563d8dba6db966bacbb256f2ba2437e64d12353106409934cc678551"),
        }
    }
    fn login() -> LoginFacts {
        LoginFacts {
            caller: caller(10),
            audience: Self::audience(),
            binding: Some(Self::binding()),
            until_ns: 300 * SECOND,
            denied: false,
        }
    }
    fn device() -> DeviceFacts {
        DeviceFacts {
            issuer: principal(1),
            subject: principal(3),
            slot: 2,
            generation: 7,
            caller: caller(10),
        }
    }
    fn authority(&self) -> SessionAuthority<'_> {
        SessionAuthority {
            caller: caller(10),
            now_ns: 140 * SECOND,
            audience: Self::audience(),
            issuer: principal(1),
            mechanism: PortableMechanism::Ed25519,
            key_binding_id: [4; 32],
            role: &self.role,
            grants: &self.grants,
            generation: 9,
            not_before_ns: 100 * SECOND,
            expires_at_ns: 400 * SECOND,
            enrollment_expires_at_ns: 200 * SECOND,
            min_login_started_at_ns: 100 * SECOND,
            max_session_ttl_secs: 30,
            enabled: true,
            subject_admissible: true,
        }
    }
}
fn snapshot<'a>(
    session: &'a SessionFacts<'a>,
    login: &'a LoginFacts,
    device: &'a DeviceFacts,
) -> ReceiverSnapshot<'a> {
    ReceiverSnapshot {
        audience: Scenario::audience(),
        authority_generation: 9,
        observed_at_ns: 130 * SECOND,
        session: Some(session),
        login: Some(login),
        device: Some(device),
    }
}

#[test]
fn exact_retry_returns_original_lease_without_any_proof_or_capacity_input() {
    let f = Scenario::new();
    let session = f.session();
    let login = Scenario::login();
    let device = Scenario::device();
    let state = snapshot(&session, &login, &device);
    let mut host = f.authority();
    // Admission proof expired at 130 seconds. Lease is still live at 140.
    // Retained replay is consumed and capacity could be full: neither is new work.
    for scopes in [["read", "write"], ["write", "read"]] {
        assert_eq!(
            assess_retry(&state, &host, [5; 32], &request(&scopes, Some(30)), true),
            Ok(RetryDecision::ExactRetry {
                expires_at_ns: 150 * SECOND
            })
        );
    }
    host.expires_at_ns = 1000 * SECOND;
    host.enrollment_expires_at_ns = 1000 * SECOND;
    host.now_ns = 149 * SECOND;
    assert_eq!(
        assess_retry(
            &state,
            &host,
            [5; 32],
            &request(&["read", "write"], Some(30)),
            true
        ),
        Ok(RetryDecision::ExactRetry {
            expires_at_ns: 150 * SECOND
        })
    );
    host.now_ns = 150 * SECOND;
    assert_eq!(
        assess_retry(
            &state,
            &host,
            [5; 32],
            &request(&["read", "write"], Some(30)),
            true
        ),
        Err(E::MissingOrExpired)
    );
}

#[test]
fn changed_request_conflicts_before_liveness_and_consumed_replay_never_becomes_fresh() {
    let f = Scenario::new();
    let session = f.session();
    let login = Scenario::login();
    let device = Scenario::device();
    let mut state = snapshot(&session, &login, &device);
    let mut host = f.authority();
    host.now_ns = 160 * SECOND;
    for req in [
        request(&["read"], Some(30)),
        request(&["read", "write"], None),
        request(&["read", "write"], Some(29)),
    ] {
        assert_eq!(
            assess_retry(&state, &host, [5; 32], &req, true),
            Err(E::ReplayConflict)
        );
    }
    state.session = None;
    assert_eq!(
        assess_retry(&state, &host, [5; 32], &request(&["read"], None), true),
        Err(E::ReplayConflict)
    );
    assert_eq!(
        assess_retry(&state, &host, [6; 32], &request(&["read"], None), false),
        Ok(RetryDecision::FreshProofRequired)
    );
    assert_eq!(assess_session(&state, &host, &[]), Err(E::MissingOrExpired));
}

#[test]
fn logout_denial_precedes_expiry_and_preserves_consumed_replay_after_retention() {
    let f = Scenario::new();
    let session = f.session();
    let mut login = Scenario::login();
    let device = Scenario::device();
    login.denied = true;
    let mut state = snapshot(&session, &login, &device);
    let mut host = f.authority();
    host.now_ns = 160 * SECOND;
    assert_eq!(assess_session(&state, &host, &[]), Err(E::Revoked));
    assert_eq!(
        assess_retry(&state, &host, [5; 32], &request(&["read"], None), true),
        Err(E::Revoked)
    );
    state.session = None;
    host.now_ns = 300 * SECOND;
    assert_eq!(
        assess_retry(&state, &host, [5; 32], &request(&["read"], None), true),
        Err(E::ReplayConflict)
    );
    assert_eq!(assess_session(&state, &host, &[]), Err(E::MissingOrExpired));
}

#[test]
fn current_authority_changes_cannot_reuse_or_shorten_a_retained_lease_into_validity() {
    let f = Scenario::new();
    let session = f.session();
    let login = Scenario::login();
    let device = Scenario::device();
    let state = snapshot(&session, &login, &device);
    let baseline = f.authority();
    type Change = fn(&mut SessionAuthority<'_>);
    for change in [
        (|h: &mut SessionAuthority<'_>| h.generation += 1) as Change,
        |h| h.key_binding_id[0] ^= 1,
        |h| h.mechanism = PortableMechanism::Certified,
        |h| h.issuer = principal(8),
        |h| h.caller = caller(11),
        |h| h.min_login_started_at_ns += 1,
        |h| h.not_before_ns = 141 * SECOND,
        |h| h.expires_at_ns = 149 * SECOND,
        |h| h.enrollment_expires_at_ns = 149 * SECOND,
        |h| h.max_session_ttl_secs = 29,
    ] {
        let mut h = baseline;
        change(&mut h);
        let expected = if h.caller != baseline.caller {
            E::InvalidFacts
        } else {
            E::StaleAuthority
        };
        assert_eq!(assess_session(&state, &h, &[]), Err(expected));
        assert_eq!(
            assess_retry(
                &state,
                &h,
                [5; 32],
                &request(&["read", "write"], Some(30)),
                true
            ),
            Err(expected)
        );
    }
    for (enabled, eligible) in [(false, true), (true, false)] {
        let mut h = baseline;
        h.enabled = enabled;
        h.subject_admissible = eligible;
        assert_eq!(assess_session(&state, &h, &[]), Err(E::Disabled));
    }
}

#[test]
fn read_requires_retained_scopes_in_one_current_role_and_operation_scopes_in_session() {
    let f = Scenario::new();
    let session = f.session();
    let login = Scenario::login();
    let device = Scenario::device();
    let state = snapshot(&session, &login, &device);
    let mut host = f.authority();
    assert_eq!(assess_session(&state, &host, &[]), Ok(()));
    assert_eq!(
        assess_session(&state, &host, &labels(&["write", "read", "write"])),
        Ok(())
    );
    assert_eq!(
        assess_session(&state, &host, &labels(&["admin"])),
        Err(E::ScopeRejected)
    );
    assert_eq!(
        assess_session(&state, &host, &labels(&["read"; 17])),
        Err(E::InvalidContext)
    );
    let narrowed = [grant("app", &["read"]), grant("other", &["write"])];
    host.grants = &narrowed;
    assert_eq!(
        assess_session(&state, &host, &labels(&["read"])),
        Err(E::ScopeRejected)
    );
    let missing = [grant("other", &["read", "write"])];
    host.grants = &missing;
    assert_eq!(assess_session(&state, &host, &[]), Err(E::ScopeRejected));
}

#[test]
fn missing_reservation_or_changed_account_slot_cannot_recreate_authority() {
    let f = Scenario::new();
    let session = f.session();
    let login = Scenario::login();
    let device = Scenario::device();
    let state = snapshot(&session, &login, &device);
    let host = f.authority();
    let mut missing = state;
    missing.login = None;
    assert_eq!(assess_session(&missing, &host, &[]), Err(E::InvalidFacts));
    missing = state;
    missing.device = None;
    assert_eq!(assess_session(&missing, &host, &[]), Err(E::InvalidFacts));
    for change in [
        (|d: &mut DeviceFacts| d.generation += 1) as fn(&mut DeviceFacts),
        |d| d.generation -= 1,
        |d| d.caller = caller(11),
        |d| d.issuer = principal(9),
        |d| d.subject = principal(9),
        |d| d.slot = 3,
    ] {
        let mut d = device;
        change(&mut d);
        let mut changed = state;
        changed.device = Some(&d);
        assert_eq!(assess_session(&changed, &host, &[]), Err(E::InvalidFacts));
    }
    for change in [
        (|l: &mut LoginFacts| l.binding = None) as fn(&mut LoginFacts),
        |l| l.binding.as_mut().unwrap().deadline_ns += 1,
        |l| l.binding.as_mut().unwrap().issuer = principal(9),
        |l| l.until_ns = 299 * SECOND,
    ] {
        let mut l = login;
        change(&mut l);
        let mut changed = state;
        changed.login = Some(&l);
        assert_eq!(assess_session(&changed, &host, &[]), Err(E::InvalidFacts));
    }
}

#[test]
fn clock_and_record_bounds_refuse_without_repairing_the_snapshot() {
    let f = Scenario::new();
    let session = f.session();
    let login = Scenario::login();
    let device = Scenario::device();
    let state = snapshot(&session, &login, &device);
    let mut host = f.authority();
    host.now_ns = 129 * SECOND;
    assert_eq!(assess_session(&state, &host, &[]), Err(E::ClockRollback));
    host.now_ns = 140 * SECOND;
    for change in [
        (|s: &mut SessionFacts<'_>| s.expires_at_ns = s.established_at_ns)
            as fn(&mut SessionFacts<'_>),
        |s| s.login.device_generation = 0,
        |s| s.login.device_slot = 8,
        |s| s.login.deadline_ns = u64::MAX,
        |s| s.established_at_ns = 99 * SECOND,
    ] {
        let mut changed_session = session;
        change(&mut changed_session);
        let changed = snapshot(&changed_session, &login, &device);
        assert_eq!(assess_session(&changed, &host, &[]), Err(E::InvalidFacts));
    }
    let unordered = labels(&["write", "read"]);
    let mut changed_session = session;
    changed_session.scopes = &unordered;
    assert_eq!(
        assess_session(&snapshot(&changed_session, &login, &device), &host, &[]),
        Err(E::InvalidFacts)
    );
    // Inputs remain borrowed and the original snapshot still passes.
    assert_eq!(assess_session(&state, &host, &[]), Ok(()));
    assert_eq!(state.observed_at_ns, 130 * SECOND);
    assert_eq!(session.expires_at_ns, 150 * SECOND);
}
