use super::{account, approve, authority, config, fixture, policy, prepare, request};
use ic_auth::{
    claims::decode_claims,
    issuer_memory::{IssuerError as E, IssuerStatus, IssuerTicket, MemoryIssuer, RenewRequest},
};
use ic_auth_protocol_types::Principal;

fn browser() -> Principal {
    Principal::from_slice(&fixture("caller"))
}

fn input(sequence: u64) -> RenewRequest {
    RenewRequest {
        account: account(),
        login_epoch: 1,
        login_id: request().login_id,
        sequence,
        grants: request().grants,
    }
}

fn completed_login() -> (MemoryIssuer, IssuerTicket) {
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    let t = prepare(&mut issuer);
    approve(&mut issuer, t);
    issuer.dispatch(t, authority(), 13).unwrap();
    issuer.complete(t, authority(), [7; 64], 14).unwrap();
    (issuer, t)
}

fn finish(issuer: &mut MemoryIssuer, ticket: IssuerTicket, now: u64) {
    issuer
        .record_nonce(ticket, authority(), [0x55; 16], now)
        .unwrap();
    issuer
        .record_approval(ticket, authority(), [9; 32], now)
        .unwrap();
    issuer.dispatch(ticket, authority(), now).unwrap();
    issuer.complete(ticket, authority(), [1; 64], now).unwrap();
}

fn id(name: &str) -> [u8; 32] {
    let (_, hex) = include_str!("../fixtures/issuer-renewal.tsv")
        .lines()
        .map(|l| l.split_once('\t').unwrap())
        .find(|(key, _)| *key == name)
        .unwrap();
    let bytes: Vec<_> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect();
    bytes.try_into().unwrap()
}

#[test]
fn renewal_preserves_reference_identity_signed_bytes_and_original_binding() {
    let (mut issuer, _) = completed_login();
    let usage = issuer.usage();
    let selection = issuer.preflight_renewal(browser(), &input(1)).unwrap();
    assert_eq!(selection.account, account());
    assert_eq!(selection.request.operation_id, id("base-id"));
    assert_eq!(
        selection.request.browser_public_key,
        request().browser_public_key
    );
    assert_eq!(selection.policy, policy());
    assert_eq!(issuer.usage(), usage);
    assert_eq!(issuer.observed_at_ns(), 14);
    assert_eq!(issuer.available_credit(), 19);
    let t = issuer
        .prepare_renewal(browser(), &input(1), authority(), 20)
        .unwrap();
    assert_eq!(t.operation_id(), id("base-id"));
    issuer.record_nonce(t, authority(), [0x55; 16], 21).unwrap();
    issuer.record_approval(t, authority(), [9; 32], 22).unwrap();
    let signing = issuer.dispatch(t, authority(), 23).unwrap();
    assert_eq!(signing.claims(), fixture("claims"));
    let result = issuer
        .complete(t, authority(), fixture("signature").try_into().unwrap(), 25)
        .unwrap();
    #[cfg(feature = "ed25519-verification")]
    super::verify_result(result);
    assert_eq!(result.claims, fixture("claims"));
    assert_eq!(issuer.usage().logins, 1);
    assert_eq!(issuer.usage().operations, 2);
    assert_eq!(issuer.usage().result_bytes, usage.result_bytes * 2);
    assert_eq!(issuer.available_credit(), 18);
    assert_eq!(
        issuer
            .preflight_renewal(browser(), &input(2))
            .unwrap()
            .request
            .operation_id,
        id("next-id")
    );
}

#[test]
fn actual_browser_and_exact_login_grants_are_required_without_mutating_preflight() {
    let (mut issuer, _) = completed_login();
    let usage = issuer.usage();
    for wrong in [
        Principal::anonymous(),
        account(),
        Principal::from_slice(&[3]),
    ] {
        assert!(matches!(
            issuer.preflight_renewal(wrong, &input(1)),
            Err(E::Denied)
        ));
        assert!(matches!(
            issuer.reconcile_renewal(wrong, &input(1)),
            Err(E::Denied)
        ));
        assert_eq!(
            issuer.prepare_renewal(wrong, &input(1), authority(), 20),
            Err(E::Denied)
        );
    }
    for field in 0..4 {
        let mut r = input(1);
        match field {
            0 => r.account = Principal::from_slice(&[3]),
            1 => r.login_epoch = 2,
            2 => r.login_id[0] ^= 1,
            3 => r.grants[0].scopes = vec!["write".parse().unwrap()],
            _ => unreachable!(),
        }
        assert!(matches!(
            issuer.preflight_renewal(browser(), &r),
            Err(E::Denied)
        ));
        assert_eq!(
            issuer.prepare_renewal(browser(), &r, authority(), 20),
            Err(E::Denied)
        );
    }
    assert_eq!(issuer.usage(), usage);
    assert_eq!(issuer.available_credit(), 19);
    assert_eq!(issuer.observed_at_ns(), 20);
    issuer.preflight_renewal(browser(), &input(1)).unwrap();
    issuer.reconcile_renewal(browser(), &input(1)).unwrap();
    assert_eq!(issuer.observed_at_ns(), 20);
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(1), authority(), 19),
        Err(E::Clock)
    );
}

#[test]
fn next_sequence_requires_completed_prior_work_and_unknown_is_lookup_only() {
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    let initial = prepare(&mut issuer);
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(0), authority(), 10),
        Err(E::Invalid)
    );
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(1), authority(), 10),
        Err(E::Conflict)
    );
    finish(&mut issuer, initial, 11);
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(2), authority(), 12),
        Err(E::Conflict)
    );
    let t = issuer
        .prepare_renewal(browser(), &input(1), authority(), 20)
        .unwrap();
    issuer.record_nonce(t, authority(), [1; 16], 20).unwrap();
    issuer.record_approval(t, authority(), [1; 32], 20).unwrap();
    issuer.dispatch(t, authority(), 20).unwrap();
    for _ in 0..3 {
        assert_eq!(
            issuer.prepare_renewal(browser(), &input(1), authority(), 21),
            Ok(t)
        );
        assert_eq!(
            issuer.prepare_renewal(browser(), &input(2), authority(), 21),
            Err(E::Conflict)
        );
        assert_eq!(
            issuer.dispatch(t, authority(), 21).unwrap_err(),
            E::Dispatched
        );
        let retained = issuer
            .reconcile_renewal(browser(), &input(1))
            .unwrap()
            .unwrap();
        assert_eq!(retained.status, IssuerStatus::Unknown(t));
        assert_eq!(retained.intent.nonce, Some([1; 16]));
        assert_eq!(retained.intent.approval, Some([1; 32]));
        assert_eq!(retained.intent.admission_deadline_ns, 30);
    }
    assert_eq!(issuer.usage().operations, 2);
    assert_eq!(issuer.available_credit(), 18);
    issuer.complete(t, authority(), [7; 64], 22).unwrap();
    let next = issuer
        .prepare_renewal(browser(), &input(2), authority(), 23)
        .unwrap();
    assert_eq!(next.operation_id(), id("next-id"));
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(1), authority(), 23),
        Ok(t)
    );
    assert_eq!(issuer.available_credit(), 17);
}

#[test]
fn renewed_proof_is_clamped_to_original_absolute_window() {
    let (mut issuer, _) = completed_login();
    let t = issuer
        .prepare_renewal(browser(), &input(1), authority(), 95)
        .unwrap();
    issuer.record_nonce(t, authority(), [1; 16], 96).unwrap();
    issuer.record_approval(t, authority(), [1; 32], 97).unwrap();
    let frame = issuer.dispatch(t, authority(), 98).unwrap();
    let claims = decode_claims(frame.claims()).unwrap();
    assert_eq!(claims.login_started_at_ns, 10);
    assert_eq!(claims.login_deadline_ns, 100);
    assert_eq!(claims.issued_at_ns, 95);
    assert_eq!(claims.admission_expires_at_ns, 100);
    assert_eq!(claims.device_slot, 0);
    assert_eq!(claims.device_generation, 1);
    assert_eq!(claims.browser_public_key, [0x44; 32]);
    assert_eq!(claims.account_subject, account());
    assert_eq!(
        issuer.complete(t, authority(), [1; 64], 100),
        Err(E::Expired)
    );
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(1), authority(), 100),
        Err(E::Expired)
    );
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(2), authority(), 100),
        Err(E::Expired)
    );
    assert!(matches!(
        issuer
            .reconcile_renewal(browser(), &input(1))
            .unwrap()
            .unwrap()
            .status,
        IssuerStatus::Unknown(_)
    ));
}

#[test]
fn preflight_cannot_survive_revocation_or_replace_current_authority_checks() {
    for revoke in [false, true] {
        let (mut issuer, _) = completed_login();
        issuer.preflight_renewal(browser(), &input(1)).unwrap();
        let mut current = authority();
        if revoke {
            issuer.revoke(account(), 1, request().login_id, 15).unwrap();
        } else {
            current.expires_at_ns = 15;
        }
        assert_eq!(
            issuer.prepare_renewal(browser(), &input(1), current, 16),
            Err(E::Denied)
        );
        assert_eq!(issuer.available_credit(), 19);
        assert_eq!(issuer.usage().operations, 1);
    }
    let (mut issuer, _) = completed_login();
    let t = issuer
        .prepare_renewal(browser(), &input(1), authority(), 20)
        .unwrap();
    issuer.record_nonce(t, authority(), [1; 16], 20).unwrap();
    issuer.record_approval(t, authority(), [1; 32], 20).unwrap();
    issuer.dispatch(t, authority(), 20).unwrap();
    let mut current = authority();
    current.generation += 1;
    assert_eq!(issuer.complete(t, current, [1; 64], 21), Err(E::Denied));
    issuer.revoke(account(), 1, request().login_id, 22).unwrap();
    assert_eq!(issuer.complete(t, authority(), [1; 64], 23), Err(E::Denied));
    assert_eq!(
        issuer
            .reconcile_renewal(browser(), &input(1))
            .unwrap()
            .unwrap()
            .status,
        IssuerStatus::Unknown(t)
    );
}

#[test]
fn retained_completion_survives_restart_expiry_and_revocation_without_reissuing() {
    let (mut issuer, _) = completed_login();
    let t = issuer
        .prepare_renewal(browser(), &input(1), authority(), 20)
        .unwrap();
    finish(&mut issuer, t, 21);
    let original = match issuer
        .reconcile_renewal(browser(), &input(1))
        .unwrap()
        .unwrap()
        .status
    {
        IssuerStatus::Completed(r) => r.claims.to_vec(),
        _ => panic!("completion missing"),
    };
    issuer.restart(22).unwrap();
    let selected = issuer.preflight_renewal(browser(), &input(1)).unwrap();
    assert_eq!(selected.request.epoch, 1);
    let next = issuer.preflight_renewal(browser(), &input(2)).unwrap();
    assert_eq!(next.request.epoch, 2);
    assert_eq!(next.request.login_epoch, 1);
    assert_eq!(next.request.operation_id, id("next-id"));
    let next_ticket = issuer
        .prepare_renewal(browser(), &input(2), authority(), 23)
        .unwrap();
    assert_eq!(next_ticket.epoch(), 2);
    finish(&mut issuer, next_ticket, 24);
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(1), authority(), 500),
        Ok(t)
    );
    assert_eq!(issuer.available_credit(), 17);
    issuer
        .revoke(account(), 1, request().login_id, 500)
        .unwrap();
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(1), authority(), 500),
        Err(E::Denied)
    );
    assert!(matches!(
        issuer.preflight_renewal(browser(), &input(1)),
        Err(E::Denied)
    ));
    let retained = issuer
        .reconcile_renewal(browser(), &input(1))
        .unwrap()
        .unwrap();
    assert_eq!(retained.intent.ticket, t);
    match retained.status {
        IssuerStatus::Completed(r) => assert_eq!(r.claims, original),
        _ => panic!("retained completion changed"),
    }
    assert_eq!(issuer.available_credit(), 17);
}

#[test]
fn account_issue_cannot_be_reclassified_as_a_renewal_with_the_same_id() {
    let (mut issuer, _) = completed_login();
    let mut collision = request();
    collision.operation_id = id("base-id");
    let t = issuer
        .prepare_issue(account(), &collision, policy(), authority(), 15)
        .unwrap();
    let usage = issuer.usage();
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(1), authority(), 16),
        Err(E::Conflict)
    );
    assert!(
        issuer
            .reconcile_renewal(browser(), &input(1))
            .unwrap()
            .is_none()
    );
    assert!(issuer.preflight_renewal(browser(), &input(1)).is_ok());
    assert_eq!(
        issuer.reconcile(account(), 1, id("base-id")).unwrap(),
        Some(IssuerStatus::Pending(t))
    );
    assert_eq!(issuer.usage(), usage);
    assert_eq!(issuer.available_credit(), 18);
}

#[test]
fn fresh_epoch_cannot_replace_an_unknown_renewal_or_recover_a_lost_binding() {
    let (mut issuer, _) = completed_login();
    let t = issuer
        .prepare_renewal(browser(), &input(1), authority(), 20)
        .unwrap();
    issuer.restart(21).unwrap();
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(1), authority(), 22),
        Err(E::Denied)
    );
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(2), authority(), 22),
        Err(E::Conflict)
    );
    let selected = issuer.preflight_renewal(browser(), &input(1)).unwrap();
    assert_eq!(selected.request.epoch, 1);
    assert_eq!(
        issuer
            .reconcile_renewal(browser(), &input(1))
            .unwrap()
            .unwrap()
            .intent
            .ticket,
        t
    );
    let mut lost = MemoryIssuer::new_fresh(config(), 22).unwrap();
    assert_eq!(
        lost.prepare_renewal(browser(), &input(1), authority(), 23),
        Err(E::Denied)
    );
    assert!(matches!(
        lost.reconcile_renewal(browser(), &input(1)),
        Err(E::Denied)
    ));
    assert_eq!(lost.usage().logins, 0);
    assert_eq!(lost.available_credit(), 20);
}

#[test]
fn renewal_uses_shared_capacity_and_budget_without_consuming_refused_sequences() {
    for limit in 0..4 {
        let mut cfg = config();
        match limit {
            0 => {
                cfg.max_operations = 1;
                cfg.max_operations_per_account = 1;
            }
            1 => cfg.max_operations_per_account = 1,
            2 => cfg.max_result_bytes = fixture("claims").len() as u64 + 64,
            3 => cfg.issuance_burst = 1,
            _ => unreachable!(),
        }
        let mut issuer = MemoryIssuer::new_fresh(cfg, 0).unwrap();
        let t = prepare(&mut issuer);
        finish(&mut issuer, t, 11);
        let usage = issuer.usage();
        assert_eq!(
            issuer.prepare_renewal(browser(), &input(1), authority(), 20),
            Err(if limit == 3 {
                E::Throttled
            } else {
                E::Capacity
            })
        );
        assert_eq!(issuer.usage(), usage);
        assert_eq!(issuer.available_credit(), if limit == 3 { 0 } else { 19 });
        assert!(
            issuer
                .reconcile_renewal(browser(), &input(1))
                .unwrap()
                .is_none()
        );
        assert!(issuer.preflight_renewal(browser(), &input(1)).is_ok());
        if limit == 3 {
            let a = super::IssuerAuthority {
                expires_at_ns: 200,
                ..authority()
            };
            // At refill 100 the original window has expired: credit cannot revive it.
            assert_eq!(
                issuer.prepare_renewal(browser(), &input(1), a, 100),
                Err(E::Expired)
            );
        }
    }
}

#[test]
fn initial_completion_cannot_unlock_an_inflight_renewal_or_impersonate_its_identity() {
    let (mut issuer, initial) = completed_login();
    let mut extra = request();
    extra.operation_id = [8; 32];
    let old = issuer
        .prepare_issue(account(), &extra, policy(), authority(), 15)
        .unwrap();
    let t = issuer
        .prepare_renewal(browser(), &input(1), authority(), 20)
        .unwrap();
    // Completing another account-authenticated issue must not mark sequence one done.
    finish(&mut issuer, old, 21);
    issuer.complete(initial, authority(), [7; 64], 21).unwrap();
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(2), authority(), 21),
        Err(E::Conflict)
    );
    let selected = issuer.preflight_renewal(browser(), &input(1)).unwrap();
    assert_eq!(
        issuer.prepare_issue(
            account(),
            &selected.request,
            selected.policy,
            authority(),
            21
        ),
        Err(E::Conflict)
    );
    assert_eq!(
        issuer.prepare_renewal(browser(), &input(1), authority(), 21),
        Ok(t)
    );
    assert_eq!(issuer.available_credit(), 17);
}
