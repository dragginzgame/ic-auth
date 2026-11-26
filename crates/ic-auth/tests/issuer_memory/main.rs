//! Issuance/renewal contracts use literal effect traces and the independent
//! Node/OpenSSL reference vector. No issuer implementation computes expectations.
#![cfg(feature = "issuer-stable")]

mod certified;
mod logout;
mod renewal;
mod retention;
#[cfg(feature = "issuer-stable")]
mod stable;

use ic_auth::issuer_memory::{
    IssueRequest, IssuerAuthority, IssuerConfig, IssuerError as E, IssuerPolicy, IssuerStatus,
    IssuerTicket, MemoryIssuer,
};
#[cfg(feature = "ed25519-verification")]
use ic_auth::{
    issuer_memory::IssuedCredential,
    verification::{
        AuthorityGrants, Ed25519Key, VerificationBudget, VerificationContext, VerificationCosts,
        verify_ed25519,
    },
};
#[cfg(feature = "ed25519-verification")]
use ic_auth_protocol_types::PortableMechanism;
use ic_auth_protocol_types::{AudienceId, CanonicalId, DelegatedRoleGrant, Principal};

fn fresh_issuer(
    config: IssuerConfig,
    now: u64,
) -> Result<ic_auth::issuer_memory::StableIssuer, ic_auth::stable_upgrade::UpgradeError> {
    use ic_storage::{StableKey, VectorMemory, testing::fixture_runtime};
    let name = "auth.test.issuer.v1";
    let runtime = fixture_runtime(
        VectorMemory::default(),
        "auth",
        "auth.",
        &[name],
        Default::default(),
    )
    .unwrap();
    ic_auth::issuer_memory::StableIssuer::new_fresh(
        runtime
            .open_memory(&StableKey::parse(name).unwrap())
            .unwrap(),
        config,
        now,
    )
}

fn fixture(name: &str) -> Vec<u8> {
    fixture_bytes(include_str!("../fixtures/ed25519.tsv"), name)
}

fn fixture_bytes(vectors: &str, name: &str) -> Vec<u8> {
    let line = vectors
        .lines()
        .find(|line| line.split_once('\t').unwrap().0 == name)
        .unwrap();
    let hex = line.split_once('\t').unwrap().1;
    assert_eq!(hex.len() % 2, 0);
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
        .collect()
}

fn account() -> Principal {
    Principal::from_slice(&[2])
}

fn config() -> IssuerConfig {
    IssuerConfig {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([0x11; 32]),
            fleet_id: CanonicalId::from_bytes([0x22; 32]),
        },
        issuer: Principal::from_slice(&[1]),
        max_logins: 8,
        max_operations: 16,
        max_operations_per_account: 8,
        max_devices_per_account: 2,
        max_result_bytes: 1_000_000,
        issuance_burst: 20,
        issuance_refill_interval_ns: 100,
    }
}

fn request() -> IssueRequest {
    IssueRequest {
        epoch: 1,
        operation_id: [1; 32],
        login_epoch: 1,
        login_id: [3; 32],
        browser_public_key: [0x44; 32],
        grants: vec![DelegatedRoleGrant {
            target: "user".parse().unwrap(),
            scopes: vec!["read".parse().unwrap()],
        }],
    }
}

fn policy() -> IssuerPolicy {
    IssuerPolicy {
        ext: None,
        login_ttl_ns: 90,
        proof_ttl_ns: 10,
        receipt_timeout_ns: 10,
        reconciliation_retention_ns: 20,
    }
}

fn authority() -> IssuerAuthority {
    IssuerAuthority {
        signing: ic_auth::issuer_memory::IssuerSigning::Ed25519,
        generation: 1,
        key_binding_id: fixture("key-binding").try_into().unwrap(),
        expires_at_ns: 200,
    }
}

fn prepare(issuer: &mut MemoryIssuer) -> IssuerTicket {
    issuer
        .prepare_issue(account(), &request(), policy(), authority(), 10)
        .unwrap()
}

fn approve(issuer: &mut MemoryIssuer, ticket: IssuerTicket) {
    issuer
        .record_nonce(ticket, authority(), [0x55; 16], 11)
        .unwrap();
    issuer
        .record_approval(ticket, authority(), [9; 32], 12)
        .unwrap();
}

fn lookup(issuer: &MemoryIssuer, ticket: IssuerTicket) -> IssuerStatus<'_> {
    issuer
        .reconcile(ticket.account(), ticket.epoch(), ticket.operation_id())
        .unwrap()
        .unwrap()
}

#[test]
fn canonical_output_matches_independent_signed_reference() {
    let mut issuer = fresh_issuer(config(), 0).unwrap();
    prepare(&mut issuer); // Establish original login at 10; no effect needed.
    let mut next = request();
    next.operation_id = [2; 32];
    let t = issuer
        .prepare_issue(account(), &next, policy(), authority(), 20)
        .unwrap();
    issuer.record_nonce(t, authority(), [0x55; 16], 21).unwrap();
    issuer.record_approval(t, authority(), [9; 32], 22).unwrap();
    let input = issuer.dispatch(t, authority(), 23).unwrap();
    assert_eq!(input.ticket(), t);
    assert_eq!(input.claims(), fixture("claims"));
    // Fixture length is an independent reservation oracle, including the nonce
    // and future signature, even while the first operation is still reserved.
    assert_eq!(
        issuer.usage().result_bytes,
        2 * (fixture("claims").len() as u64 + 64)
    );
    assert_eq!(issuer.usage().logins, 1);
    assert_eq!(issuer.usage().operations, 2);
    let result = issuer
        .complete(t, authority(), fixture("signature").try_into().unwrap(), 25)
        .unwrap();
    assert_eq!(result.claims, fixture("claims"));
    assert_eq!(
        result.proof,
        ic_auth::issuer_memory::IssuedProof::Ed25519(&fixture("signature").try_into().unwrap())
    );
    assert_eq!(result.admission_deadline_ns, 30);
    #[cfg(feature = "ed25519-verification")]
    verify_result(result);
    assert_eq!(issuer.available_credit(), 18);
}

#[cfg(feature = "ed25519-verification")]
fn verify_result(result: IssuedCredential<'_>) {
    let cfg = config();
    let grants = AuthorityGrants::new(request().grants).unwrap();
    let host = VerificationContext {
        audience: &cfg.audience,
        issuer: cfg.issuer,
        grants: &grants,
        mechanism: PortableMechanism::Ed25519,
        key_binding_id: authority().key_binding_id,
        caller: Principal::from_slice(&fixture("caller")),
        now_ns: 25,
        authority_not_before_ns: 0,
        authority_expires_at_ns: 200,
        enrollment_expires_at_ns: 150,
        min_login_started_at_ns: 0,
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: 100,
        max_proof_ttl_ns: 60,
    };
    let key = Ed25519Key::new(fixture("public-key").try_into().unwrap()).unwrap();
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
    let ic_auth::issuer_memory::IssuedProof::Ed25519(signature) = result.proof else {
        panic!("Ed25519 result")
    };
    let verified = verify_ed25519(result.claims, signature, &host, &key, &mut budget).unwrap();
    assert_eq!(verified.fingerprint().as_slice(), fixture("digest"));
    assert_eq!(verified.caller(), host.caller);
}

#[test]
fn every_interruption_retains_one_dispatch_and_exact_result() {
    // Pause after each external boundary. Reconciliation is lookup-only; replay
    // the same callbacks, then simulate loss of the first signing response.
    for pause_after in 0..=4 {
        let mut issuer = fresh_issuer(config(), 0).unwrap();
        let ticket = prepare(&mut issuer);
        let mut effects = Vec::new();
        if pause_after >= 1 {
            issuer
                .record_nonce(ticket, authority(), [0x55; 16], 10)
                .unwrap();
        }
        if pause_after >= 2 {
            issuer
                .record_approval(ticket, authority(), [9; 32], 10)
                .unwrap();
        }
        if pause_after >= 3 {
            effects.push(issuer.dispatch(ticket, authority(), 10).unwrap());
        }
        if pause_after >= 4 {
            issuer.complete(ticket, authority(), [7; 64], 10).unwrap();
        }
        let usage = issuer.usage();
        for _ in 0..3 {
            assert!(issuer.reconcile(account(), 1, [99; 32]).unwrap().is_none());
            assert_eq!(
                issuer.prepare_issue(account(), &request(), policy(), authority(), 10),
                Ok(ticket)
            );
            issuer
                .record_nonce(ticket, authority(), [0x55; 16], 10)
                .unwrap();
            issuer
                .record_approval(ticket, authority(), [9; 32], 10)
                .unwrap();
            match issuer.dispatch(ticket, authority(), 10) {
                Ok(input) => effects.push(input),
                Err(error) => assert_eq!(error, E::Dispatched),
            }
            assert_eq!(issuer.usage(), usage);
            assert_eq!(issuer.available_credit(), 19);
        }
        assert_eq!(
            effects.len(),
            1,
            "signing response loss must not repeat the effect"
        );
        if pause_after < 4 {
            assert_eq!(lookup(&issuer, ticket), IssuerStatus::Unknown(ticket));
        }
        let original = issuer
            .complete(ticket, authority(), [7; 64], 11)
            .unwrap()
            .claims
            .to_vec();
        assert_eq!(
            issuer
                .complete(ticket, authority(), [7; 64], 12)
                .unwrap()
                .claims,
            original
        );
        assert_eq!(
            issuer.complete(ticket, authority(), [8; 64], 12),
            Err(E::Conflict)
        );
    }
}

#[test]
fn stage_refusals_and_conflicting_callbacks_preserve_original_evidence() {
    let mut issuer = fresh_issuer(config(), 0).unwrap();
    let t = prepare(&mut issuer);
    let usage = issuer.usage();
    assert_eq!(
        issuer.record_approval(t, authority(), [1; 32], 10),
        Err(E::Pending)
    );
    assert_eq!(issuer.dispatch(t, authority(), 10).unwrap_err(), E::Pending);
    assert_eq!(
        issuer.complete(t, authority(), [0; 64], 10),
        Err(E::Pending)
    );
    issuer.record_nonce(t, authority(), [0; 16], 11).unwrap(); // Zero is a valid nonce.
    assert_eq!(
        issuer.record_nonce(t, authority(), [1; 16], 11),
        Err(E::Conflict)
    );
    issuer.record_approval(t, authority(), [2; 32], 12).unwrap();
    assert_eq!(
        issuer.record_approval(t, authority(), [3; 32], 12),
        Err(E::Conflict)
    );
    assert_eq!(
        issuer.complete(t, authority(), [0; 64], 12),
        Err(E::Pending)
    );
    let intent = issuer.intent(t).unwrap();
    assert_eq!(intent.nonce, Some([0; 16]));
    assert_eq!(intent.approval, Some([2; 32]));
    assert_eq!(intent.browser_public_key, request().browser_public_key);
    assert_eq!(intent.grants, request().grants);
    assert_eq!(intent.admission_deadline_ns, 20);
    assert_eq!(intent.receipt_deadline_ns, 20);
    assert_eq!(issuer.usage(), usage);
    assert_eq!(issuer.available_credit(), 19);
}

#[test]
fn stale_authority_restart_and_revocation_fence_pending_callbacks() {
    for fence in 0..5 {
        let mut issuer = fresh_issuer(config(), 0).unwrap();
        let t = prepare(&mut issuer);
        approve(&mut issuer, t);
        issuer.dispatch(t, authority(), 13).unwrap();
        let mut current = authority();
        match fence {
            0 => current.generation += 1,
            1 => current.key_binding_id[0] ^= 1,
            2 => current.expires_at_ns += 1,
            3 => issuer.restart(14).unwrap(),
            4 => issuer.revoke(account(), 1, request().login_id, 14).unwrap(),
            _ => unreachable!(),
        }
        assert_eq!(issuer.complete(t, current, [1; 64], 15), Err(E::Denied));
        assert_eq!(
            issuer.record_nonce(t, current, [0x55; 16], 15),
            Err(E::Denied)
        );
        assert_eq!(
            issuer.record_approval(t, current, [9; 32], 15),
            Err(E::Denied)
        );
        assert_eq!(issuer.dispatch(t, current, 15).unwrap_err(), E::Denied);
        assert_eq!(lookup(&issuer, t), IssuerStatus::Unknown(t));
        assert_eq!(issuer.available_credit(), 19);
    }
}

#[test]
fn completed_recovery_survives_expiry_revocation_and_restart_without_new_authority() {
    let mut issuer = fresh_issuer(config(), 0).unwrap();
    let t = prepare(&mut issuer);
    approve(&mut issuer, t);
    issuer.dispatch(t, authority(), 13).unwrap();
    let bytes = issuer
        .complete(t, authority(), [1; 64], 14)
        .unwrap()
        .claims
        .to_vec();
    issuer.revoke(account(), 1, request().login_id, 15).unwrap();
    issuer.restart(16).unwrap();
    let mut current = authority();
    current.generation += 1;
    assert_eq!(
        issuer.prepare_issue(account(), &request(), policy(), current, 500),
        Ok(t)
    );
    assert_eq!(
        issuer.complete(t, current, [1; 64], 500).unwrap().claims,
        bytes
    );
    assert!(matches!(lookup(&issuer, t), IssuerStatus::Completed(_)));
    assert_eq!(issuer.dispatch(t, current, 500).unwrap_err(), E::Denied);
    assert_eq!(
        issuer.available_credit(),
        19,
        "recovery does not even refill the budget"
    );
    assert_eq!(issuer.epoch(), 2);
    assert!(
        issuer
            .reconcile(Principal::from_slice(&[3]), 1, request().operation_id)
            .unwrap()
            .is_none()
    );
}

#[test]
fn exclusive_deadlines_are_captured_and_cannot_be_extended_on_retry() {
    for (proof, receipt, authority_expiry, expected) in
        [(10, 5, 200, 15), (5, 10, 200, 15), (10, 10, 14, 14)]
    {
        let mut issuer = fresh_issuer(config(), 0).unwrap();
        let p = IssuerPolicy {
            proof_ttl_ns: proof,
            receipt_timeout_ns: receipt,
            ..policy()
        };
        let a = IssuerAuthority {
            expires_at_ns: authority_expiry,
            ..authority()
        };
        let t = issuer
            .prepare_issue(account(), &request(), p.clone(), a, 10)
            .unwrap();
        issuer.record_nonce(t, a, [1; 16], 10).unwrap();
        issuer.record_approval(t, a, [1; 32], 10).unwrap();
        issuer.dispatch(t, a, expected - 1).unwrap();
        assert_eq!(issuer.complete(t, a, [1; 64], expected), Err(E::Expired));
        assert_eq!(
            issuer.prepare_issue(account(), &request(), p, a, expected),
            Err(E::Expired)
        );
        assert_eq!(lookup(&issuer, t), IssuerStatus::Unknown(t));
    }
}

#[test]
fn capacity_refusals_preserve_existing_reservations_and_fresh_intent_credit() {
    let result_bytes = fixture("claims").len() as u64 + 64;
    for limit in 0..4 {
        let mut cfg = config();
        match limit {
            0 => {
                cfg.max_operations = 1;
                cfg.max_operations_per_account = 1;
            }
            1 => cfg.max_operations_per_account = 1,
            2 => cfg.max_logins = 1,
            3 => cfg.max_result_bytes = result_bytes,
            _ => unreachable!(),
        }
        let mut issuer = fresh_issuer(cfg, 0).unwrap();
        let t = prepare(&mut issuer);
        let usage = issuer.usage();
        let mut other = request();
        other.operation_id = [2; 32];
        other.login_id = [4; 32];
        other.browser_public_key = [5; 32];
        assert_eq!(
            issuer.prepare_issue(account(), &other, policy(), authority(), 11),
            Err(E::Capacity)
        );
        assert_eq!(issuer.usage(), usage);
        assert_eq!(issuer.available_credit(), 19);
        assert!(
            issuer
                .reconcile(account(), 1, other.operation_id)
                .unwrap()
                .is_none()
        );
        assert_eq!(issuer.intent(t).unwrap().nonce, None);
        assert_eq!(
            issuer.prepare_issue(account(), &request(), policy(), authority(), 11),
            Ok(t)
        );
        if limit == 1 {
            for id in 3..25 {
                other.operation_id = [id; 32];
                assert_eq!(
                    issuer.prepare_issue(account(), &other, policy(), authority(), 11),
                    Err(E::Capacity)
                );
            }
            issuer
                .prepare_issue(
                    Principal::from_slice(&[3]),
                    &other,
                    policy(),
                    authority(),
                    11,
                )
                .unwrap();
            assert_eq!(issuer.available_credit(), 18);
        }
        let usage = issuer.usage();
        // Physical expiry does not free reservations implicitly.
        assert_eq!(
            issuer.prepare_issue(account(), &other, policy(), authority(), 100),
            Err(E::Capacity)
        );
        assert_eq!(issuer.usage(), usage);
    }
    let mut cfg = config();
    cfg.max_result_bytes = result_bytes - 1;
    let mut issuer = fresh_issuer(cfg, 0).unwrap();
    assert_eq!(
        issuer.prepare_issue(account(), &request(), policy(), authority(), 10),
        Err(E::Capacity)
    );
    assert_eq!(issuer.usage().operations, 0);
    assert_eq!(issuer.usage().logins, 0);
    assert_eq!(issuer.usage().result_bytes, 0);
}

#[test]
fn account_and_device_selection_is_bounded_and_isolated() {
    let mut issuer = fresh_issuer(config(), 0).unwrap();
    let t = prepare(&mut issuer);
    let mut duplicate_browser = request();
    duplicate_browser.operation_id = [2; 32];
    duplicate_browser.login_id = [4; 32];
    assert_eq!(
        issuer.prepare_issue(account(), &duplicate_browser, policy(), authority(), 10),
        Err(E::Conflict)
    );
    let mut second = duplicate_browser;
    second.browser_public_key = [5; 32];
    issuer
        .prepare_issue(account(), &second, policy(), authority(), 10)
        .unwrap();
    let mut third = second;
    third.operation_id = [3; 32];
    third.login_id = [6; 32];
    third.browser_public_key = [7; 32];
    assert_eq!(
        issuer.prepare_issue(account(), &third, policy(), authority(), 10),
        Err(E::Capacity)
    );
    let other_account = Principal::from_slice(&[3]);
    issuer
        .prepare_issue(other_account, &request(), policy(), authority(), 10)
        .unwrap();
    issuer
        .revoke(other_account, 1, request().login_id, 10)
        .unwrap();
    approve(&mut issuer, t);
    issuer.dispatch(t, authority(), 13).unwrap();
    assert_eq!(issuer.usage().logins, 3);
    assert_eq!(issuer.usage().operations, 3);
}

#[test]
fn exact_identity_includes_binding_grants_and_policy() {
    let mut issuer = fresh_issuer(config(), 0).unwrap();
    let t = prepare(&mut issuer);
    for field in 0..4 {
        let mut r = request();
        match field {
            0 => r.browser_public_key[0] ^= 1,
            1 => r.login_id[0] ^= 1,
            2 => r.login_epoch += 1,
            3 => r.grants[0].scopes = vec!["write".parse().unwrap()],
            _ => unreachable!(),
        }
        assert_eq!(
            issuer.prepare_issue(account(), &r, policy(), authority(), 11),
            Err(E::Conflict)
        );
    }
    assert_eq!(
        issuer.prepare_issue(
            account(),
            &request(),
            IssuerPolicy {
                proof_ttl_ns: 11,
                ..policy()
            },
            authority(),
            11
        ),
        Err(E::Conflict)
    );
    assert_eq!(issuer.available_credit(), 19);
    assert_eq!(lookup(&issuer, t), IssuerStatus::Pending(t));
}

#[test]
fn invalid_requests_clocks_throttling_and_epoch_fences_never_create_authority() {
    let mut cfg = config();
    cfg.issuance_burst = 1;
    let mut issuer = fresh_issuer(cfg, 0).unwrap();
    assert_eq!(
        issuer.prepare_issue(Principal::anonymous(), &request(), policy(), authority(), 5),
        Err(E::Invalid)
    );
    assert_eq!(issuer.observed_at_ns(), 5);
    assert_eq!(issuer.available_credit(), 1);
    assert_eq!(
        issuer.prepare_issue(account(), &request(), policy(), authority(), 4),
        Err(E::Clock)
    );
    let t = prepare(&mut issuer);
    let mut r = request();
    r.operation_id = [2; 32];
    assert_eq!(
        issuer.prepare_issue(account(), &r, policy(), authority(), 11),
        Err(E::Throttled)
    );
    assert_eq!(issuer.available_credit(), 0);
    issuer.restart(12).unwrap();
    assert_eq!(
        issuer.prepare_issue(account(), &r, policy(), authority(), 12),
        Err(E::EpochClosed)
    );
    assert_eq!(
        issuer.record_nonce(t, authority(), [1; 16], 12),
        Err(E::Denied)
    );
    r.epoch = 2;
    r.login_epoch = 2;
    r.login_id = [4; 32];
    r.browser_public_key = [5; 32];
    issuer
        .prepare_issue(account(), &r, policy(), authority(), 100)
        .unwrap();
    assert_eq!(issuer.available_credit(), 0);
    issuer.close();
    assert_eq!(issuer.restart(101), Err(E::Closed));
    assert_eq!(
        issuer.reconcile(account(), 1, request().operation_id),
        Err(E::Closed)
    );
    assert_eq!(
        issuer.prepare_issue(account(), &r, policy(), authority(), 101),
        Err(E::Closed)
    );
}

#[test]
fn bad_configuration_and_overflow_refuse_without_partial_records() {
    for field in 0..7 {
        let mut cfg = config();
        match field {
            0 => cfg.max_logins = 0,
            1 => cfg.max_operations = 0,
            2 => cfg.max_operations_per_account = cfg.max_operations + 1,
            3 => cfg.max_devices_per_account = 9,
            4 => cfg.max_result_bytes = 0,
            5 => cfg.issuance_burst = 0,
            6 => cfg.issuance_refill_interval_ns = 0,
            _ => unreachable!(),
        }
        assert!(matches!(
            fresh_issuer(cfg, 0),
            Err(ic_auth::stable_upgrade::UpgradeError::Configuration)
        ));
    }
    let mut issuer = fresh_issuer(config(), u64::MAX - 2).unwrap();
    let a = IssuerAuthority {
        expires_at_ns: u64::MAX,
        ..authority()
    };
    assert_eq!(
        issuer.prepare_issue(account(), &request(), policy(), a, u64::MAX - 1),
        Err(E::Invalid)
    );
    assert_eq!(issuer.usage().operations, 0);
    assert_eq!(issuer.usage().logins, 0);
}

#[test]
fn malformed_requests_and_unavailable_authority_refuse_before_charging() {
    let mut issuer = fresh_issuer(config(), 0).unwrap();
    for field in 0..9 {
        let mut r = request();
        let mut p = policy();
        match field {
            0 => r.operation_id = [0; 32],
            1 => r.login_id = [0; 32],
            2 => r.browser_public_key = [0; 32],
            3 => r.epoch = 0,
            4 => r.login_epoch = 0,
            5 => r.grants[0].scopes.push("read".parse().unwrap()),
            6 => p.login_ttl_ns = 90 * 86_400_000_000_000 + 1,
            7 => p.proof_ttl_ns = 60_000_000_001,
            8 => p.receipt_timeout_ns = 0,
            _ => unreachable!(),
        }
        assert_eq!(
            issuer.prepare_issue(account(), &r, p, authority(), 10),
            Err(E::Invalid)
        );
        assert_eq!(issuer.available_credit(), 20);
        assert_eq!(issuer.usage().operations, 0);
    }
    for field in 0..3 {
        let mut a = authority();
        match field {
            0 => a.generation = 0,
            1 => a.key_binding_id = [0; 32],
            2 => a.expires_at_ns = 10,
            _ => unreachable!(),
        }
        assert_eq!(
            issuer.prepare_issue(account(), &request(), policy(), a, 10),
            Err(E::Denied)
        );
        assert_eq!(issuer.available_credit(), 20);
        assert_eq!(issuer.usage().logins, 0);
    }
}
