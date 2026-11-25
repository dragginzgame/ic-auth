use super::{
    STOP_AFTER, WRITES, authority, completed_login, dispatch, issuer, policy, renewal_input,
    request,
};
use crate::{
    browser_identity::browser_principal,
    issuer_memory::{IssuerError as E, IssuerPruned, MemoryIssuer, RenewalProgress},
};
use ic_auth_protocol_types::Principal;
use std::{
    collections::BTreeSet,
    panic::{AssertUnwindSafe, catch_unwind},
};

// Recompute physical totals and relationships from canonical records, independently
// of publication/pruning updates. This deliberately does not call index constructors.
fn assert_consistent(issuer: &MemoryIssuer) {
    assert!(issuer.open);
    let mut operations = 0;
    let mut logins = 0;
    let mut bytes = 0;
    let mut expected_expiry = BTreeSet::new();
    let mut expected_idle = BTreeSet::new();
    for (account, owner) in &issuer.accounts {
        assert!(!owner.logins.is_empty(), "empty accounts must be reclaimed");
        let mut renewals = 0;
        for (id, op) in &owner.operations {
            operations += 1;
            bytes += op.result_bytes;
            assert!(owner.logins.contains_key(&op.login));
            assert_eq!(*id, (op.ticket.epoch(), op.ticket.operation_id()));
            assert_eq!(*account, op.ticket.account());
            assert!(expected_expiry.insert((
                op.remove_at,
                account.as_slice().len() as u8,
                *account,
                *id
            )));
            if let Some(n) = op.renewal {
                assert_eq!(owner.renewals.get(&(op.login, n)), Some(id));
                renewals += 1;
            }
        }
        assert_eq!(owner.renewals.len(), renewals);
        for (id, login) in &owner.logins {
            logins += 1;
            let references = owner
                .operations
                .values()
                .filter(|op| op.login == *id)
                .count();
            assert_eq!(login.references as usize, references);
            assert!(login.generation <= issuer.generation);
            if references == 0 {
                expected_idle.insert((
                    if login.revoked || matches!(login.renewal, RenewalProgress::Initial) {
                        login.started
                    } else {
                        login.deadline
                    },
                    account.as_slice().len() as u8,
                    *account,
                    *id,
                ));
            }
        }
    }
    assert_eq!(issuer.usage.operations, operations);
    assert_eq!(issuer.usage.logins, logins);
    assert_eq!(issuer.usage.result_bytes, bytes);
    assert_eq!(issuer.operation_expiry, expected_expiry);
    assert_eq!(issuer.idle_logins, expected_idle);
}

fn with_renewal() -> MemoryIssuer {
    let mut issuer = completed_login();
    let t = issuer
        .prepare_renewal(
            browser_principal(&request().browser_public_key),
            &renewal_input(1),
            authority(),
            2,
        )
        .unwrap();
    dispatch(&mut issuer, t);
    issuer.complete(t, authority(), [1; 64], 2).unwrap();
    assert_consistent(&issuer);
    issuer
}

#[test]
fn every_prune_and_restart_publication_interruption_is_terminal() {
    // Operation removal includes a renewal index and both nonfinal/final reference
    // decrements. A later call exercises idle-login and empty-account reclamation.
    for action in 0..3 {
        let setup = || {
            let mut issuer = with_renewal();
            if action == 1 {
                issuer.prune(200, 32).unwrap();
            }
            issuer
        };
        let run = |issuer: &mut MemoryIssuer| {
            if action == 2 {
                issuer.restart(200).unwrap();
            } else {
                issuer.prune(200, 32).unwrap();
            }
        };
        let mut control = setup();
        WRITES.with(|n| n.set(0));
        run(&mut control);
        let writes = WRITES.with(|n| n.get());
        assert!(writes > 0);
        assert_consistent(&control);
        for stop in 0..writes {
            let mut issuer = setup();
            STOP_AFTER.with(|fault| fault.set(Some(stop)));
            let interrupted = catch_unwind(AssertUnwindSafe(|| run(&mut issuer)));
            STOP_AFTER.with(|fault| fault.set(None));
            assert!(interrupted.is_err());
            assert_eq!(issuer.prune(201, 32), Err(E::Closed));
            assert_eq!(issuer.restart(201), Err(E::Closed));
            assert_eq!(
                issuer.prepare_issue(
                    Principal::from_slice(&[2]),
                    &request(),
                    policy(),
                    authority(),
                    201
                ),
                Err(E::Closed)
            );
            let browser = browser_principal(&request().browser_public_key);
            assert_eq!(
                issuer.prepare_renewal(browser, &renewal_input(1), authority(), 201),
                Err(E::Closed)
            );
            assert!(matches!(
                issuer.reconcile_renewal(browser, &renewal_input(1)),
                Err(E::Closed)
            ));
            assert_eq!(
                issuer.reconcile(Principal::from_slice(&[2]), 1, request().operation_id),
                Err(E::Closed)
            );
        }
    }
}

#[test]
fn expiry_accounting_remains_consistent_across_repeated_slot_reclamation() {
    let mut issuer = issuer();
    let account = Principal::from_slice(&[2]);
    let mut current = authority();
    current.expires_at_ns = 100_000;
    for round in 1..=20 {
        let now = round * 200;
        let mut r = request();
        r.epoch = issuer.epoch();
        r.login_epoch = issuer.epoch();
        let t = issuer
            .prepare_issue(account, &r, policy(), current, now)
            .unwrap();
        issuer.record_nonce(t, current, [1; 16], now).unwrap();
        issuer.record_approval(t, current, [1; 32], now).unwrap();
        issuer.dispatch(t, current, now).unwrap();
        issuer.complete(t, current, [1; 64], now).unwrap();
        let mut renewal = renewal_input(1);
        renewal.login_epoch = r.login_epoch;
        issuer
            .prepare_renewal(
                browser_principal(&r.browser_public_key),
                &renewal,
                current,
                now + 1,
            )
            .unwrap();
        r.operation_id = [5; 32];
        issuer
            .prepare_issue(account, &r, policy(), current, now + 2)
            .unwrap();
        assert_consistent(&issuer);
        for (time, budget) in [
            (now + 30, 0),
            (now + 30, 1),
            (now + 31, 32),
            (now + 32, 32),
            (now + 100, 32),
        ] {
            issuer.prune(time, budget).unwrap();
            assert_consistent(&issuer);
        }
        assert_eq!(issuer.usage.logins, 0);
        assert_eq!(issuer.generation, round);
    }
}

#[test]
fn epoch_or_boot_overflow_refuses_without_forgetting_records_or_advancing_other_fences() {
    let mut issuer = issuer();
    issuer.epoch = u64::MAX;
    let mut r = request();
    r.epoch = u64::MAX;
    r.login_epoch = u64::MAX;
    issuer
        .prepare_issue(Principal::from_slice(&[2]), &r, policy(), authority(), 1)
        .unwrap();
    let usage = issuer.usage();
    let credit = issuer.available_credit();
    assert_eq!(issuer.prune(31, 0), Ok(IssuerPruned::default()));
    assert_eq!(issuer.prune(31, 32), Err(E::Capacity));
    assert_eq!(issuer.usage(), usage);
    assert_eq!(issuer.available_credit(), credit);
    assert_eq!(issuer.observed_at_ns(), 31);
    assert_consistent(&issuer);
    assert_eq!(issuer.restart(31), Err(E::Capacity));
    assert_eq!(issuer.boot, 1);
    issuer.epoch = 7;
    issuer.boot = u64::MAX;
    assert_eq!(issuer.restart(31), Err(E::Capacity));
    assert_eq!(issuer.epoch, 7);
    assert_eq!(issuer.boot, u64::MAX);
    assert_consistent(&issuer);
}
