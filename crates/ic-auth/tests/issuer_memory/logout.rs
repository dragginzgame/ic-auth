use super::{account, approve, authority, config, fixture, lookup, policy, prepare, request};
use ic_auth::issuer_memory::{IssuerError as E, IssuerStatus, MemoryIssuer, RenewRequest};
use ic_auth_protocol_types::Principal;

fn browser() -> Principal {
    Principal::from_slice(&fixture("caller"))
}

fn renewal() -> RenewRequest {
    RenewRequest {
        account: account(),
        login_epoch: 1,
        login_id: request().login_id,
        sequence: 1,
        grants: request().grants,
    }
}

#[test]
fn browser_logout_authenticates_every_selector_without_charging_or_revoking_another_login() {
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    let original = prepare(&mut issuer);
    let mut other = request();
    other.login_id = [4; 32];
    other.operation_id = [2; 32];
    other.browser_public_key = [5; 32];
    let second = issuer
        .prepare_issue(account(), &other, policy(), authority(), 10)
        .unwrap();
    let other_account = Principal::from_slice(&[3]);
    let third = issuer
        .prepare_issue(other_account, &request(), policy(), authority(), 10)
        .unwrap();
    let usage = issuer.usage();
    let credit = issuer.available_credit();
    for (caller, owner, epoch, id) in [
        (Principal::anonymous(), account(), 1, request().login_id),
        (account(), account(), 1, request().login_id),
        (
            browser(),
            Principal::from_slice(&[7]),
            1,
            request().login_id,
        ),
        (browser(), account(), 2, request().login_id),
        (browser(), account(), 1, [8; 32]),
        (browser(), account(), 1, other.login_id),
    ] {
        assert_eq!(
            issuer.revoke_browser(caller, owner, epoch, id, 10),
            Err(E::Denied)
        );
        assert_eq!(issuer.usage(), usage);
        assert_eq!(issuer.available_credit(), credit);
    }
    assert_eq!(
        issuer.revoke_browser(browser(), account(), 1, request().login_id, 10),
        Ok(100)
    );
    assert_eq!(
        issuer.record_nonce(original, authority(), [1; 16], 10),
        Err(E::Denied)
    );
    issuer
        .record_nonce(second, authority(), [1; 16], 10)
        .unwrap();
    issuer
        .record_nonce(third, authority(), [1; 16], 10)
        .unwrap();
    assert_eq!(issuer.usage(), usage);
    assert_eq!(issuer.available_credit(), credit);
}

#[test]
fn browser_logout_fences_initial_work_at_each_stage_and_preserves_uncertainty() {
    for stage in 0..4 {
        let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
        let ticket = prepare(&mut issuer);
        if stage >= 1 {
            issuer
                .record_nonce(ticket, authority(), [1; 16], 10)
                .unwrap();
        }
        if stage >= 2 {
            issuer
                .record_approval(ticket, authority(), [2; 32], 10)
                .unwrap();
        }
        if stage >= 3 {
            issuer.dispatch(ticket, authority(), 10).unwrap();
        }
        let before = if stage == 3 {
            IssuerStatus::Unknown(ticket)
        } else {
            IssuerStatus::Pending(ticket)
        };
        assert_eq!(
            issuer.revoke_browser(browser(), account(), 1, request().login_id, 11),
            Ok(100)
        );
        assert_eq!(lookup(&issuer, ticket), before);
        assert_eq!(
            issuer.record_nonce(ticket, authority(), [1; 16], 11),
            Err(E::Denied)
        );
        assert_eq!(
            issuer.record_approval(ticket, authority(), [2; 32], 11),
            Err(E::Denied)
        );
        assert_eq!(
            issuer.dispatch(ticket, authority(), 11).unwrap_err(),
            E::Denied
        );
        assert_eq!(
            issuer.complete(ticket, authority(), [1; 64], 11),
            Err(E::Denied)
        );
    }
}

#[test]
fn browser_logout_fences_renewal_completion_but_retains_original_results_and_deadline() {
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    let initial = prepare(&mut issuer);
    approve(&mut issuer, initial);
    issuer.dispatch(initial, authority(), 13).unwrap();
    let original = issuer
        .complete(initial, authority(), [1; 64], 14)
        .unwrap()
        .claims
        .to_vec();
    let input = renewal();
    let pending = issuer
        .prepare_renewal(browser(), &input, authority(), 15)
        .unwrap();
    issuer
        .record_nonce(pending, authority(), [2; 16], 15)
        .unwrap();
    issuer
        .record_approval(pending, authority(), [2; 32], 15)
        .unwrap();
    issuer.dispatch(pending, authority(), 15).unwrap();
    let usage = issuer.usage();
    let credit = issuer.available_credit();
    assert_eq!(
        issuer.revoke_browser(browser(), account(), 1, input.login_id, 16),
        Ok(100)
    );
    assert_eq!(
        issuer.complete(pending, authority(), [2; 64], 16),
        Err(E::Denied)
    );
    assert_eq!(
        issuer.prepare_renewal(browser(), &input, authority(), 16),
        Err(E::Denied)
    );
    assert!(matches!(
        issuer.preflight_renewal(browser(), &input),
        Err(E::Denied)
    ));
    assert_eq!(
        issuer
            .reconcile_renewal(browser(), &input)
            .unwrap()
            .unwrap()
            .status,
        IssuerStatus::Unknown(pending)
    );
    match lookup(&issuer, initial) {
        IssuerStatus::Completed(result) => assert_eq!(result.claims, original),
        other => panic!("lost original result: {other:?}"),
    }
    issuer.restart(17).unwrap();
    assert_eq!(
        issuer.revoke_browser(browser(), account(), 1, input.login_id, 500),
        Ok(100)
    );
    assert_eq!(issuer.usage(), usage);
    assert_eq!(issuer.available_credit(), credit);
}

#[test]
fn browser_logout_refuses_clock_regression_closed_state_and_pruned_or_lost_bindings() {
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    prepare(&mut issuer);
    assert_eq!(
        issuer.revoke_browser(browser(), account(), 1, request().login_id, 9),
        Err(E::Clock)
    );
    issuer.close();
    assert_eq!(
        issuer.revoke_browser(browser(), account(), 1, request().login_id, 10),
        Err(E::Closed)
    );
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    assert_eq!(
        issuer.revoke_browser(browser(), account(), 1, request().login_id, 10),
        Err(E::Denied)
    );
    prepare(&mut issuer);
    issuer.prune(500, 32).unwrap();
    issuer.prune(500, 32).unwrap();
    assert_eq!(
        issuer.revoke_browser(browser(), account(), 1, request().login_id, 500),
        Err(E::Denied)
    );
}
