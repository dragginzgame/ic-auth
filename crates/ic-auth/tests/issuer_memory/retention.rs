use super::{account, approve, authority, config, fixture, policy, prepare, request};
use ic_auth::{
    claims::decode_claims,
    issuer_memory::{
        IssuerAuthority, IssuerError as E, IssuerPolicy, IssuerPruned, IssuerStatus, IssuerTicket,
        MemoryIssuer, RenewRequest,
    },
};
use ic_auth_protocol_types::Principal;

fn finish(issuer: &mut MemoryIssuer, t: IssuerTicket, now: u64) {
    issuer.record_nonce(t, authority(), [1; 16], now).unwrap();
    issuer
        .record_approval(t, authority(), [1; 32], now)
        .unwrap();
    issuer.dispatch(t, authority(), now).unwrap();
    issuer.complete(t, authority(), [1; 64], now).unwrap();
}

fn renewal(sequence: u64) -> RenewRequest {
    RenewRequest {
        account: account(),
        login_epoch: 1,
        login_id: request().login_id,
        sequence,
        grants: request().grants,
    }
}

fn browser() -> Principal {
    Principal::from_slice(&fixture("caller"))
}

fn pruned(operations: u32, logins: u32) -> IssuerPruned {
    IssuerPruned { operations, logins }
}

#[test]
fn prune_captures_later_deadline_and_keeps_exact_results_until_explicit_removal() {
    for complete in [false, true] {
        let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
        let p = IssuerPolicy {
            receipt_timeout_ns: 30,
            ..policy()
        };
        let t = issuer
            .prepare_issue(account(), &request(), p.clone(), authority(), 10)
            .unwrap();
        approve(&mut issuer, t);
        issuer.dispatch(t, authority(), 13).unwrap();
        if complete {
            issuer.complete(t, authority(), [1; 64], 14).unwrap();
        }
        let retained = issuer.usage();
        // admission=20, receipt=40, reconciliation retention=20 => remove at 60.
        assert_eq!(issuer.prune(59, u32::MAX), Ok(pruned(0, 0)));
        assert!(
            issuer
                .reconcile(account(), 1, request().operation_id)
                .unwrap()
                .is_some()
        );
        assert_eq!(issuer.usage(), retained);
        assert_eq!(issuer.epoch(), 1);
        assert_eq!(issuer.prune(60, 1), Ok(pruned(1, 0)));
        assert_eq!(issuer.epoch(), 2);
        assert_eq!(issuer.usage().result_bytes, 0);
        assert_eq!(issuer.usage().operations, 0);
        assert_eq!(issuer.usage().logins, 1);
        assert_eq!(issuer.available_credit(), 19);
        assert!(
            issuer
                .reconcile(account(), 1, request().operation_id)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            issuer.prepare_issue(account(), &request(), p, authority(), 60),
            Err(E::EpochClosed)
        );
        assert_eq!(issuer.complete(t, authority(), [1; 64], 60), Err(E::Denied));
        assert_eq!(issuer.prune(99, 32), Ok(pruned(0, 0)));
        assert_eq!(issuer.prune(100, 32), Ok(pruned(0, 1)));
        assert_eq!(issuer.usage().logins, 0);
        assert_eq!(
            issuer.available_credit(),
            19,
            "maintenance cannot refill credit"
        );
        assert_eq!(issuer.prune(99, 32), Err(E::Clock));
    }
}

#[test]
fn pruning_old_ids_does_not_invalidate_unrelated_live_callbacks() {
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    let old = prepare(&mut issuer);
    let mut later = request();
    later.operation_id = [2; 32];
    let live = issuer
        .prepare_issue(account(), &later, policy(), authority(), 35)
        .unwrap();
    // Both operations share a login; its second reference must survive removal.
    assert_eq!(issuer.prune(40, 32), Ok(pruned(1, 0)));
    assert_eq!(issuer.epoch(), 2);
    assert_eq!(issuer.usage().operations, 1);
    assert_eq!(issuer.usage().logins, 1);
    assert_eq!(
        issuer.prepare_issue(account(), &later, policy(), authority(), 40),
        Ok(live)
    );
    finish(&mut issuer, live, 41);
    assert!(matches!(
        issuer.reconcile(account(), 1, later.operation_id).unwrap(),
        Some(IssuerStatus::Completed(_))
    ));
    assert_eq!(
        issuer.record_nonce(old, authority(), [1; 16], 41),
        Err(E::Denied)
    );
    assert_eq!(issuer.prune(65, 32), Ok(pruned(1, 0)));
    assert_eq!(
        issuer.epoch(),
        2,
        "old epoch deletion must not close a new epoch"
    );
    assert_eq!(issuer.prune(100, 32), Ok(pruned(0, 1)));
}

#[test]
fn retained_renewal_watermarks_survive_real_result_pruning() {
    for complete in [false, true] {
        let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
        let initial = prepare(&mut issuer);
        finish(&mut issuer, initial, 11);
        let t = issuer
            .prepare_renewal(browser(), &renewal(1), authority(), 20)
            .unwrap();
        if complete {
            finish(&mut issuer, t, 21);
        } else {
            issuer.record_nonce(t, authority(), [1; 16], 21).unwrap();
            issuer.record_approval(t, authority(), [1; 32], 21).unwrap();
            issuer.dispatch(t, authority(), 21).unwrap();
        }
        assert_eq!(issuer.prune(50, 32), Ok(pruned(2, 0)));
        assert!(
            issuer
                .reconcile_renewal(browser(), &renewal(1))
                .unwrap()
                .is_none()
        );
        assert_eq!(
            issuer.prepare_renewal(browser(), &renewal(1), authority(), 51),
            Err(E::EpochClosed)
        );
        assert_eq!(issuer.usage().operations, 0);
        assert_eq!(issuer.usage().logins, 1);
        assert_eq!(issuer.available_credit(), 18);
        if complete {
            let next = issuer
                .prepare_renewal(browser(), &renewal(2), authority(), 51)
                .unwrap();
            assert_eq!(next.epoch(), 2);
            finish(&mut issuer, next, 52);
        } else {
            assert_eq!(
                issuer.prepare_renewal(browser(), &renewal(2), authority(), 51),
                Err(E::Conflict)
            );
            assert_eq!(issuer.complete(t, authority(), [1; 64], 51), Err(E::Denied));
        }
    }
}

#[test]
fn reusing_idle_login_removes_it_from_pruning_until_last_reference_expires() {
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    prepare(&mut issuer);
    assert_eq!(issuer.prune(40, 32), Ok(pruned(1, 0)));
    let mut later = request();
    later.epoch = issuer.epoch();
    later.operation_id = [2; 32];
    let live = issuer
        .prepare_issue(account(), &later, policy(), authority(), 90)
        .unwrap();
    // The login deadline is 100, but the new result must remain until 120.
    assert_eq!(issuer.prune(100, 32), Ok(pruned(0, 0)));
    assert_eq!(issuer.usage().logins, 1);
    assert_eq!(issuer.usage().operations, 1);
    assert_eq!(
        issuer.reconcile(account(), 2, later.operation_id).unwrap(),
        Some(IssuerStatus::Pending(live))
    );
    assert_eq!(issuer.prune(120, 32), Ok(pruned(1, 0)));
    assert_eq!(
        issuer.usage().logins,
        1,
        "newly idle logins wait for the next bounded call"
    );
    assert_eq!(issuer.prune(120, 32), Ok(pruned(0, 1)));
    assert_eq!(issuer.usage().result_bytes, 0);
}

#[test]
fn pruning_restores_capacity_but_never_old_login_identity_or_generation() {
    let mut cfg = config();
    cfg.max_logins = 1;
    cfg.max_devices_per_account = 1;
    cfg.max_operations = 1;
    cfg.max_operations_per_account = 1;
    cfg.max_result_bytes = fixture("claims").len() as u64 + 64;
    let mut issuer = MemoryIssuer::new_fresh(cfg, 0).unwrap();
    let old = prepare(&mut issuer);
    issuer.revoke(account(), 1, request().login_id, 11).unwrap();
    assert_eq!(issuer.prune(100, 32), Ok(pruned(1, 0)));
    let mut fresh = request();
    fresh.epoch = issuer.epoch();
    fresh.login_epoch = issuer.epoch();
    fresh.browser_public_key = [9; 32];
    assert_eq!(
        issuer.prepare_issue(account(), &fresh, policy(), authority(), 100),
        Err(E::Capacity)
    );
    assert_eq!(issuer.prune(100, 32), Ok(pruned(0, 1)));
    assert_eq!(
        issuer.prepare_issue(account(), &request(), policy(), authority(), 100),
        Err(E::EpochClosed)
    );
    let mut wrong_epoch = fresh.clone();
    wrong_epoch.login_epoch = 1;
    assert_eq!(
        issuer.prepare_issue(account(), &wrong_epoch, policy(), authority(), 100),
        Err(E::EpochClosed)
    );
    assert!(matches!(
        issuer.reconcile_renewal(browser(), &renewal(1)),
        Err(E::Denied)
    ));
    let new = issuer
        .prepare_issue(account(), &fresh, policy(), authority(), 101)
        .unwrap();
    issuer.record_nonce(new, authority(), [1; 16], 101).unwrap();
    issuer
        .record_approval(new, authority(), [1; 32], 101)
        .unwrap();
    let input = issuer.dispatch(new, authority(), 101).unwrap();
    let claims = decode_claims(input.claims()).unwrap();
    assert_eq!(claims.device_slot, 0);
    assert_eq!(claims.device_generation, 2);
    assert_eq!(claims.browser_public_key, [9; 32]);
    assert_eq!(claims.login_started_at_ns, 101);
    assert_eq!(
        issuer.record_nonce(old, authority(), [1; 16], 101),
        Err(E::Denied)
    );
    assert_eq!(issuer.usage().operations, 1);
    assert_eq!(issuer.usage().logins, 1);
    assert_eq!(issuer.usage().result_bytes, cfg.max_result_bytes);
}

#[test]
fn maintenance_is_capped_at_32_and_prioritizes_operations_over_idle_logins() {
    let mut cfg = config();
    cfg.max_logins = 64;
    cfg.max_operations = 64;
    cfg.issuance_burst = 100;
    let mut issuer = MemoryIssuer::new_fresh(cfg, 0).unwrap();
    for n in 1..=40 {
        issuer
            .prepare_issue(
                Principal::from_slice(&[1, n]),
                &request(),
                policy(),
                authority(),
                10,
            )
            .unwrap();
    }
    let live_account = Principal::from_slice(&[1, 41]);
    let current = IssuerAuthority {
        expires_at_ns: 2000,
        ..authority()
    };
    issuer
        .prepare_issue(live_account, &request(), policy(), current, 995)
        .unwrap();
    let credit = issuer.available_credit();
    let usage = issuer.usage();
    assert_eq!(issuer.prune(1000, 0), Ok(pruned(0, 0)));
    assert_eq!(issuer.usage(), usage);
    assert_eq!(issuer.observed_at_ns(), 1000);
    assert_eq!(issuer.prune(1000, u32::MAX), Ok(pruned(32, 0)));
    for n in 1..=40 {
        assert_eq!(
            issuer
                .reconcile(Principal::from_slice(&[1, n]), 1, request().operation_id)
                .unwrap()
                .is_none(),
            n <= 32
        );
    }
    assert_eq!(issuer.usage().operations, 9);
    assert_eq!(issuer.usage().logins, 41);
    assert_eq!(issuer.prune(1000, 10), Ok(pruned(8, 2)));
    assert_eq!(issuer.prune(1000, 32), Ok(pruned(0, 32)));
    assert_eq!(issuer.prune(1000, 32), Ok(pruned(0, 6)));
    assert_eq!(issuer.prune(1000, 32), Ok(pruned(0, 0)));
    assert_eq!(issuer.usage().operations, 1);
    assert_eq!(issuer.usage().logins, 1);
    assert_eq!(issuer.available_credit(), credit);
    assert!(
        issuer
            .reconcile(live_account, 1, request().operation_id)
            .unwrap()
            .is_some()
    );
}

#[test]
fn expiry_ties_use_reference_principal_length_then_bytes_and_operation_identity() {
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    let short = Principal::from_slice(&[255]);
    let long = Principal::from_slice(&[0, 0]);
    // Deliberately reverse insertion order. Length-first must choose short first.
    issuer
        .prepare_issue(long, &request(), policy(), authority(), 10)
        .unwrap();
    let mut second = request();
    second.operation_id = [2; 32];
    issuer
        .prepare_issue(short, &second, policy(), authority(), 10)
        .unwrap();
    issuer
        .prepare_issue(short, &request(), policy(), authority(), 10)
        .unwrap();
    assert_eq!(issuer.prune(40, 1), Ok(pruned(1, 0)));
    assert!(
        issuer
            .reconcile(short, 1, request().operation_id)
            .unwrap()
            .is_none()
    );
    assert!(
        issuer
            .reconcile(short, 1, second.operation_id)
            .unwrap()
            .is_some()
    );
    assert!(
        issuer
            .reconcile(long, 1, request().operation_id)
            .unwrap()
            .is_some()
    );
    assert_eq!(issuer.prune(40, 1), Ok(pruned(1, 0)));
    assert!(
        issuer
            .reconcile(short, 1, second.operation_id)
            .unwrap()
            .is_none()
    );
    assert_eq!(issuer.prune(40, 1), Ok(pruned(1, 0)));
    assert_eq!(issuer.prune(100, 1), Ok(pruned(0, 1)));
    // Short account is gone; long account still has its expired retained login.
    assert_eq!(
        issuer.revoke(short, 1, request().login_id, 100),
        Err(E::Denied)
    );
    assert_eq!(issuer.revoke(long, 1, request().login_id, 100), Ok(()));
}

#[test]
fn invalid_or_overflowing_retention_refuses_before_publishing_records() {
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    let invalid = IssuerPolicy {
        reconciliation_retention_ns: 0,
        ..policy()
    };
    assert_eq!(
        issuer.prepare_issue(account(), &request(), invalid, authority(), 10),
        Err(E::Invalid)
    );
    assert_eq!(issuer.available_credit(), 20);
    let overflow = IssuerPolicy {
        reconciliation_retention_ns: u64::MAX,
        ..policy()
    };
    assert_eq!(
        issuer.prepare_issue(account(), &request(), overflow, authority(), 10),
        Err(E::Invalid)
    );
    assert_eq!(issuer.available_credit(), 19);
    assert_eq!(issuer.usage().operations, 0);
    assert_eq!(issuer.usage().logins, 0);
    assert_eq!(issuer.prune(100, 32), Ok(pruned(0, 0)));
}
