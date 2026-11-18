//! Host effect traces are literal, including deliberately partial publication.
use super::{account, authority, config, lookup, policy, request};
use ic_auth::issuer_memory::{
    CertifiedCompletionError as C, IssuedProof, IssuerAuthority, IssuerError as E, IssuerSigning,
    IssuerStatus, IssuerTicket, MemoryIssuer, RenewRequest,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn certified() -> IssuerAuthority {
    IssuerAuthority {
        signing: IssuerSigning::Certified { seed_hash: [8; 32] },
        ..authority()
    }
}

fn approved() -> (MemoryIssuer, IssuerTicket) {
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    let t = issuer
        .prepare_issue(account(), &request(), policy(), certified(), 10)
        .unwrap();
    issuer.record_nonce(t, certified(), [0x55; 16], 11).unwrap();
    issuer.record_approval(t, certified(), [9; 32], 12).unwrap();
    (issuer, t)
}

#[test]
fn certified_result_capacity_is_reserved_before_publication_and_retry_is_free() {
    // Changing the one-byte mechanism tag does not change the independently
    // packed claims length; certified results retain no Ed25519 signature bytes.
    let bytes = super::fixture("claims").len() as u64;
    for limit in [bytes - 1, bytes] {
        let mut cfg = config();
        cfg.max_result_bytes = limit;
        let mut issuer = MemoryIssuer::new_fresh(cfg, 0).unwrap();
        let prepared = issuer.prepare_issue(account(), &request(), policy(), certified(), 10);
        if limit < bytes {
            assert_eq!(prepared, Err(E::Capacity));
            assert_eq!(issuer.usage().operations, 0);
            assert_eq!(issuer.usage().result_bytes, 0);
            continue;
        }
        let t = prepared.unwrap();
        assert_eq!(issuer.usage().result_bytes, bytes);
        issuer.record_nonce(t, certified(), [1; 16], 10).unwrap();
        issuer.record_approval(t, certified(), [2; 32], 10).unwrap();
        issuer
            .complete_certified(t, certified(), 10, |_| Ok::<_, ()>(()))
            .unwrap();
        let mut next = request();
        next.operation_id = [9; 32];
        assert_eq!(
            issuer.prepare_issue(account(), &next, policy(), certified(), 10),
            Err(E::Capacity)
        );
        issuer
            .complete_certified(t, certified(), 10, |_| -> Result<(), ()> {
                panic!("saturated retry must be read-only")
            })
            .unwrap();
        assert_eq!(issuer.usage().result_bytes, bytes);
    }
}

#[test]
fn certified_publication_is_once_and_retained_retries_never_recreate_a_leaf() {
    let (mut issuer, t) = approved();
    let before = issuer.usage();
    let credit = issuer.available_credit();
    let mut leaves = Vec::new();
    let result = issuer
        .complete_certified(t, certified(), 13, |p| {
            assert_eq!(p.ticket, t);
            assert_eq!(p.seed_hash, [8; 32]);
            assert_eq!(p.admission_deadline_ns, 20);
            leaves.push((p.seed_hash, p.message_hash));
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(result.proof, IssuedProof::Certified { seed_hash: [8; 32] });
    let original = result.claims.to_vec();
    assert_eq!(before.result_bytes, original.len() as u64);
    assert_eq!(leaves.len(), 1);
    assert_eq!(issuer.usage(), before);
    assert_eq!(issuer.available_credit(), credit);
    issuer.revoke(account(), 1, request().login_id, 14).unwrap();
    issuer.restart(15).unwrap();
    leaves.clear(); // Host has pruned/lost its leaf. Retained claims are not permission to sign.
    let stale = IssuerAuthority {
        generation: 99,
        ..certified()
    };
    for now in [15, 20, 100] {
        let result = issuer
            .complete_certified(t, stale, now, |_| -> Result<(), ()> {
                panic!("completed retry must not publish")
            })
            .unwrap();
        assert_eq!(result.claims, original);
        assert_eq!(result.admission_deadline_ns, 20);
        assert!(matches!(lookup(&issuer, t), IssuerStatus::Completed(_)));
    }
    assert!(leaves.is_empty());
    assert_eq!(issuer.prune(100, 1).unwrap().operations, 1);
    assert_eq!(
        issuer.complete_certified(t, certified(), 100, |_| Ok::<_, ()>(())),
        Err(C::Issuer(E::Denied))
    );
    assert_eq!(issuer.usage().result_bytes, 0);
}

#[test]
fn protected_mechanism_cannot_cross_effect_paths() {
    let (mut issuer, t) = approved();
    assert_eq!(issuer.dispatch(t, certified(), 13).unwrap_err(), E::Invalid);
    assert_eq!(
        issuer.complete(t, certified(), [7; 64], 13),
        Err(E::Invalid)
    );
    assert!(matches!(lookup(&issuer, t), IssuerStatus::Pending(_)));
    let mut ed = MemoryIssuer::new_fresh(config(), 0).unwrap();
    let et = super::prepare(&mut ed);
    super::approve(&mut ed, et);
    assert_eq!(
        ed.complete_certified(et, authority(), 13, |_| -> Result<(), ()> {
            panic!("Ed25519 cannot publish a certified leaf")
        }),
        Err(C::Issuer(E::Invalid))
    );
    ed.dispatch(et, authority(), 13).unwrap();
    ed.complete(et, authority(), [7; 64], 13).unwrap();
    assert_eq!(
        ed.complete_certified(et, certified(), 13, |_| Ok::<_, ()>(())),
        Err(C::Issuer(E::Invalid))
    );
}

#[test]
fn every_prepublication_refusal_skips_the_host_effect() {
    for change in 0..9 {
        let (mut issuer, t) = approved();
        let before = issuer.usage();
        let mut current = certified();
        let mut now = 13;
        let expected = match change {
            0 => {
                current.signing = IssuerSigning::Certified { seed_hash: [9; 32] };
                E::Denied
            }
            1 => {
                current.generation += 1;
                E::Denied
            }
            2 => {
                current.key_binding_id = [4; 32];
                E::Denied
            }
            3 => {
                current.signing = IssuerSigning::Ed25519;
                E::Denied
            }
            4 => {
                issuer.revoke(account(), 1, request().login_id, 13).unwrap();
                E::Denied
            }
            5 => {
                issuer.restart(13).unwrap();
                E::Denied
            }
            6 => {
                now = 20;
                E::Expired
            }
            7 => {
                now = 11;
                E::Clock
            }
            _ => {
                issuer.close();
                E::Closed
            }
        };
        assert_eq!(
            issuer.complete_certified(t, current, now, |_| -> Result<(), ()> {
                panic!("refusal invoked host publication")
            }),
            Err(C::Issuer(expected))
        );
        assert_eq!(issuer.usage(), before);
    }
    // Neither retained intent nor a nonce is approval.
    let mut issuer = MemoryIssuer::new_fresh(config(), 0).unwrap();
    let t = issuer
        .prepare_issue(account(), &request(), policy(), certified(), 10)
        .unwrap();
    for nonce in [false, true] {
        if nonce {
            issuer.record_nonce(t, certified(), [1; 16], 10).unwrap();
        }
        assert_eq!(
            issuer.complete_certified(t, certified(), 10, |_| -> Result<(), ()> {
                panic!("unapproved intent published")
            }),
            Err(C::Issuer(E::Pending))
        );
    }
}

#[test]
fn failed_or_unwound_host_publication_closes_without_claiming_rollback() {
    for unwind in [false, true] {
        let (mut issuer, t) = approved();
        let mut published = false;
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            issuer
                .complete_certified(t, certified(), 13, |_| {
                    published = true; // Deliberately broken host writes before failure.
                    assert!(!unwind, "interrupted host publication");
                    Err("host commit refused")
                })
                .map(|_| ())
        }));
        if unwind {
            assert!(outcome.is_err());
        } else {
            assert_eq!(outcome.unwrap(), Err(C::Publication("host commit refused")));
        }
        assert!(published); // Auth has not rolled back the host's write.
        assert_eq!(
            issuer.reconcile(account(), 1, t.operation_id()),
            Err(E::Closed)
        );
        assert_eq!(
            issuer.complete_certified(t, certified(), 13, |_| Ok::<_, ()>(())),
            Err(C::Issuer(E::Closed))
        );
        assert_eq!(issuer.restart(13), Err(E::Closed));
    }
}

#[test]
fn certified_renewal_shares_completion_watermarks_and_original_window() {
    let (mut issuer, t) = approved();
    let browser = ic_auth_protocol_types::Principal::from_slice(&super::fixture("caller"));
    let renewal = RenewRequest {
        account: account(),
        login_epoch: 1,
        login_id: request().login_id,
        sequence: 1,
        grants: request().grants,
    };
    assert_eq!(
        issuer.prepare_renewal(browser, &renewal, certified(), 13),
        Err(E::Conflict)
    );
    issuer
        .complete_certified(t, certified(), 13, |_| Ok::<_, ()>(()))
        .unwrap();
    let next = issuer
        .prepare_renewal(browser, &renewal, certified(), 95)
        .unwrap();
    issuer.record_nonce(next, certified(), [7; 16], 95).unwrap();
    issuer
        .record_approval(next, certified(), [8; 32], 95)
        .unwrap();
    let result = issuer
        .complete_certified(next, certified(), 95, |p| {
            assert_eq!(p.admission_deadline_ns, 100); // Absolute original login deadline.
            Ok::<_, ()>(())
        })
        .unwrap();
    let claims = ic_auth::claims::decode_claims(result.claims).unwrap();
    assert_eq!(claims.device_generation, 1);
    assert_eq!(claims.login_started_at_ns, 10);
    assert_eq!(claims.login_deadline_ns, 100);
    assert_eq!(
        claims.mechanism,
        ic_auth_protocol_types::PortableMechanism::Certified
    );
    issuer.prune(125, 32).unwrap();
    assert!(
        issuer
            .reconcile_renewal(browser, &renewal)
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        issuer.preflight_renewal(browser, &renewal),
        Err(E::EpochClosed)
    ));
}
