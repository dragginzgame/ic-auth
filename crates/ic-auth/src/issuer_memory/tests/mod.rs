//! Publication interruption and exhausted monotonic fences cannot be driven
//! through the public API without infeasible allocations or u64 transitions.
mod retention;

use super::{
    IssueRequest, IssuerAuthority, IssuerConfig, IssuerError as E, IssuerPolicy, IssuerTicket,
    MemoryIssuer, RenewRequest, RenewalProgress,
};
use crate::browser_identity::browser_principal;
use ic_auth_protocol_types::{AudienceId, CanonicalId, DelegatedRoleGrant, Principal};
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};

thread_local! {
    static STOP_AFTER: Cell<Option<usize>> = const { Cell::new(None) };
    static WRITES: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn after_write() {
    WRITES.with(|n| n.set(n.get() + 1));
    STOP_AFTER.with(|stop| match stop.get() {
        Some(0) => {
            stop.set(None);
            panic!("injected issuer publication interruption");
        }
        Some(n) => stop.set(Some(n - 1)),
        None => {}
    });
}

fn issuer() -> MemoryIssuer {
    MemoryIssuer::new_fresh(
        IssuerConfig {
            audience: AudienceId {
                canonical_network_id: CanonicalId::from_bytes([1; 32]),
                fleet_id: CanonicalId::from_bytes([2; 32]),
            },
            issuer: Principal::from_slice(&[1]),
            max_logins: 2,
            max_operations: 3,
            max_operations_per_account: 3,
            max_devices_per_account: 2,
            max_result_bytes: 10_000,
            issuance_burst: 5,
            issuance_refill_interval_ns: 1,
        },
        0,
    )
    .unwrap()
}

fn request() -> IssueRequest {
    IssueRequest {
        epoch: 1,
        operation_id: [1; 32],
        login_epoch: 1,
        login_id: [2; 32],
        browser_public_key: [3; 32],
        grants: vec![DelegatedRoleGrant {
            target: "app".parse().unwrap(),
            scopes: vec!["read".parse().unwrap()],
        }],
    }
}

fn authority() -> IssuerAuthority {
    IssuerAuthority {
        signing: crate::issuer_memory::IssuerSigning::Ed25519,
        generation: 1,
        key_binding_id: [4; 32],
        expires_at_ns: 1_000,
    }
}

fn policy() -> IssuerPolicy {
    IssuerPolicy {
        ext: None,
        login_ttl_ns: 100,
        proof_ttl_ns: 10,
        receipt_timeout_ns: 10,
        reconciliation_retention_ns: 20,
    }
}

#[test]
fn certified_host_and_completion_writes_share_terminal_failure_containment() {
    for stop in 0..=3 {
        let mut issuer = issuer();
        let current = IssuerAuthority {
            signing: super::IssuerSigning::Certified { seed_hash: [8; 32] },
            ..authority()
        };
        let account = Principal::from_slice(&[2]);
        let ticket = issuer
            .prepare_issue(account, &request(), policy(), current, 1)
            .unwrap();
        issuer.record_nonce(ticket, current, [1; 16], 1).unwrap();
        issuer.record_approval(ticket, current, [2; 32], 1).unwrap();
        let mut leaf_published = false;
        STOP_AFTER.with(|fault| fault.set(Some(stop)));
        let result = catch_unwind(AssertUnwindSafe(|| {
            issuer
                .complete_certified(ticket, current, 1, |_| {
                    leaf_published = true;
                    Ok::<_, ()>(())
                })
                .map(|_| ())
        }));
        STOP_AFTER.with(|fault| fault.set(None));
        assert!(leaf_published);
        if stop < 3 {
            assert!(result.is_err());
            assert_eq!(
                issuer.reconcile(account, 1, ticket.operation_id()),
                Err(E::Closed)
            );
            assert_eq!(issuer.restart(1), Err(E::Closed));
        } else {
            assert_eq!(result.unwrap(), Ok(()));
            assert!(matches!(
                issuer.reconcile(account, 1, ticket.operation_id()).unwrap(),
                Some(super::IssuerStatus::Completed(_))
            ));
        }
    }
}

#[test]
fn publication_interruption_is_terminal_for_new_and_existing_bindings() {
    let account = Principal::from_slice(&[2]);
    for existing in [false, true] {
        for stop in 0..if existing { 6 } else { 7 } {
            let mut issuer = issuer();
            let first = issuer
                .prepare_issue(account, &request(), policy(), authority(), 1)
                .unwrap();
            let mut next = request();
            next.operation_id = [5; 32];
            if !existing {
                next.login_id = [6; 32];
                next.browser_public_key = [7; 32];
            }
            STOP_AFTER.with(|fault| fault.set(Some(stop)));
            let result = catch_unwind(AssertUnwindSafe(|| {
                issuer.prepare_issue(account, &next, policy(), authority(), 2)
            }));
            STOP_AFTER.with(|fault| fault.set(None));
            assert!(result.is_err());
            assert_eq!(issuer.restart(3), Err(E::Closed));
            assert_eq!(
                issuer.reconcile(account, 1, next.operation_id),
                Err(E::Closed)
            );
            assert_eq!(
                issuer.prepare_issue(account, &next, policy(), authority(), 3),
                Err(E::Closed)
            );
            assert_eq!(
                issuer.record_nonce(first, authority(), [1; 16], 3),
                Err(E::Closed)
            );
            assert_eq!(
                issuer.record_approval(first, authority(), [1; 32], 3),
                Err(E::Closed)
            );
            assert_eq!(
                issuer.dispatch(first, authority(), 3).unwrap_err(),
                E::Closed
            );
            assert_eq!(
                issuer.complete(first, authority(), [1; 64], 3),
                Err(E::Closed)
            );
            assert_eq!(
                issuer.revoke(account, 1, request().login_id, 3),
                Err(E::Closed)
            );
            assert!(matches!(issuer.intent(first), Err(E::Closed)));
        }
    }
    let mut normal = issuer();
    normal
        .prepare_issue(account, &request(), policy(), authority(), 1)
        .unwrap();
    assert_eq!(normal.usage().operations, 1);
    assert_eq!(normal.usage().logins, 1);
}

#[test]
fn exhausted_generations_and_epochs_do_not_wrap() {
    let mut issuer = issuer();
    issuer.generation = u64::MAX;
    assert_eq!(
        issuer.prepare_issue(
            Principal::from_slice(&[2]),
            &request(),
            policy(),
            authority(),
            1
        ),
        Err(E::Capacity)
    );
    assert_eq!(issuer.usage().logins, 0);
    assert_eq!(issuer.usage().operations, 0);
    issuer.epoch = u64::MAX;
    assert_eq!(issuer.restart(2), Err(E::Capacity));
    assert_eq!(issuer.epoch(), u64::MAX);
}

fn renewal_input(sequence: u64) -> RenewRequest {
    RenewRequest {
        account: Principal::from_slice(&[2]),
        login_epoch: 1,
        login_id: request().login_id,
        sequence,
        grants: request().grants,
    }
}

fn dispatch(issuer: &mut MemoryIssuer, ticket: IssuerTicket) {
    issuer
        .record_nonce(ticket, authority(), [1; 16], 2)
        .unwrap();
    issuer
        .record_approval(ticket, authority(), [1; 32], 2)
        .unwrap();
    issuer.dispatch(ticket, authority(), 2).unwrap();
}

fn completed_login() -> MemoryIssuer {
    let mut issuer = issuer();
    let t = issuer
        .prepare_issue(
            Principal::from_slice(&[2]),
            &request(),
            policy(),
            authority(),
            1,
        )
        .unwrap();
    dispatch(&mut issuer, t);
    issuer.complete(t, authority(), [1; 64], 2).unwrap();
    issuer
}

#[test]
fn interrupted_renewal_reservation_and_completion_close_every_entry_path() {
    let browser = browser_principal(&request().browser_public_key);
    // Eight publication boundaries in renewal preparation, two in completion.
    for completing in [false, true] {
        for stop in 0..if completing { 2 } else { 8 } {
            let mut issuer = completed_login();
            let request = renewal_input(1);
            let ticket = if completing {
                let t = issuer
                    .prepare_renewal(browser, &request, authority(), 2)
                    .unwrap();
                dispatch(&mut issuer, t);
                Some(t)
            } else {
                None
            };
            STOP_AFTER.with(|fault| fault.set(Some(stop)));
            let interrupted = catch_unwind(AssertUnwindSafe(|| {
                if let Some(t) = ticket {
                    issuer.complete(t, authority(), [1; 64], 2).unwrap();
                } else {
                    issuer
                        .prepare_renewal(browser, &request, authority(), 2)
                        .unwrap();
                }
            }));
            STOP_AFTER.with(|fault| fault.set(None));
            assert!(interrupted.is_err());
            assert_eq!(issuer.restart(3), Err(E::Closed));
            assert_eq!(
                issuer.prepare_renewal(browser, &request, authority(), 3),
                Err(E::Closed)
            );
            assert!(matches!(
                issuer.reconcile_renewal(browser, &request),
                Err(E::Closed)
            ));
            assert!(matches!(
                issuer.preflight_renewal(browser, &request),
                Err(E::Closed)
            ));
            assert_eq!(
                issuer.prepare_issue(
                    request.account,
                    &super::tests::request(),
                    policy(),
                    authority(),
                    3
                ),
                Err(E::Closed)
            );
        }
    }
    // Initial completion now also publishes the first renewal watermark.
    for stop in 0..2 {
        let mut issuer = issuer();
        let t = issuer
            .prepare_issue(
                Principal::from_slice(&[2]),
                &request(),
                policy(),
                authority(),
                1,
            )
            .unwrap();
        dispatch(&mut issuer, t);
        STOP_AFTER.with(|fault| fault.set(Some(stop)));
        let interrupted = catch_unwind(AssertUnwindSafe(|| {
            issuer.complete(t, authority(), [1; 64], 2).unwrap();
        }));
        STOP_AFTER.with(|fault| fault.set(None));
        assert!(interrupted.is_err());
        assert_eq!(issuer.restart(3), Err(E::Closed));
        assert_eq!(
            issuer.prepare_renewal(browser, &renewal_input(1), authority(), 3),
            Err(E::Closed)
        );
    }
}

#[test]
fn exhausted_renewal_watermarks_cannot_wrap() {
    let browser = browser_principal(&request().browser_public_key);
    let mut issuer = completed_login();
    issuer
        .accounts
        .get_mut(&Principal::from_slice(&[2]))
        .unwrap()
        .logins
        .get_mut(&(1, request().login_id))
        .unwrap()
        .renewal = RenewalProgress::Completed(u64::MAX);
    for sequence in [1, u64::MAX] {
        assert_eq!(
            issuer.prepare_renewal(browser, &renewal_input(sequence), authority(), 3),
            Err(E::EpochClosed)
        );
    }
    assert_eq!(
        issuer.prepare_renewal(browser, &renewal_input(0), authority(), 3),
        Err(E::Invalid)
    );
}

#[test]
fn renewal_operation_identity_matches_independent_wide_integer_and_grant_frame() {
    let input = RenewRequest {
        account: Principal::from_slice(&[255; 29]),
        login_epoch: 128,
        login_id: std::array::from_fn(|i| i as u8),
        sequence: u64::MAX,
        grants: vec![
            DelegatedRoleGrant {
                target: "app".parse().unwrap(),
                scopes: vec!["read".parse().unwrap(), "write".parse().unwrap()],
            },
            DelegatedRoleGrant {
                target: "user".parse().unwrap(),
                scopes: vec!["read".parse().unwrap()],
            },
        ],
    };
    let (_, hex) = include_str!("../../../tests/fixtures/issuer-renewal.tsv")
        .lines()
        .map(|l| l.split_once('\t').unwrap())
        .find(|(key, _)| *key == "wide-id")
        .unwrap();
    let expected: Vec<_> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect();
    assert_eq!(
        super::renewal::operation_id(&input).unwrap().as_slice(),
        expected
    );
}
