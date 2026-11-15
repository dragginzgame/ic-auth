//! Literal count/byte scenarios from the pinned session-store contract.
//! Arithmetic inputs are not an encoded backend or corruption/rollback oracle.
use ic_auth::receiver_policy::{
    AdmissionFootprint, SessionCapacity, SessionPolicyError as E, SessionUsage, assess_capacity,
};
use ic_auth_protocol_types::Principal;
fn account() -> (Principal, Principal) {
    (Principal::from_slice(&[1]), Principal::from_slice(&[3]))
}
fn limits() -> SessionCapacity {
    SessionCapacity {
        max_sessions: 2,
        max_replays: 10,
        max_replays_per_account: 3,
        max_encoded_bytes: 1000,
    }
}
fn usage() -> SessionUsage {
    SessionUsage {
        account: account(),
        sessions: 1,
        replays: 1,
        replays_for_account: 1,
        encoded_bytes: 200,
    }
}
fn footprint() -> AdmissionFootprint {
    AdmissionFootprint {
        session_bytes: 100,
        replay_bytes: 50,
        replaced_session_bytes: None,
    }
}

#[test]
fn fresh_admission_and_replacement_have_literal_physical_totals() {
    let initial = usage();
    assert_eq!(
        assess_capacity(account(), initial, footprint(), limits()),
        Ok(SessionUsage {
            account: account(),
            sessions: 2,
            replays: 2,
            replays_for_account: 2,
            encoded_bytes: 350,
        })
    );
    let full = SessionUsage {
        sessions: 2,
        replays: 2,
        replays_for_account: 2,
        encoded_bytes: 350,
        ..initial
    };
    let replacement = AdmissionFootprint {
        replaced_session_bytes: Some(100),
        ..footprint()
    };
    assert_eq!(
        assess_capacity(account(), full, replacement, limits()),
        Ok(SessionUsage {
            account: account(),
            sessions: 2,
            replays: 3,
            replays_for_account: 3,
            encoded_bytes: 400,
        })
    );
    // Login/slot replacement without a retained session frees no session capacity.
    assert_eq!(
        assess_capacity(account(), full, footprint(), limits()),
        Err(E::Capacity)
    );
    assert_eq!(initial, usage(), "all accounting inputs remain unchanged");
}

#[test]
fn each_physical_count_limit_refuses_growth_and_replacement_never_refunds_replay() {
    for (sessions, replays, local, replacing, expected) in [
        (1, 9, 2, false, true),
        (2, 9, 2, false, false),
        (2, 9, 2, true, true),
        (2, 10, 2, true, false),
        (2, 9, 3, true, false),
        (1, 10, 2, false, false),
        (1, 9, 3, false, false),
    ] {
        let current = SessionUsage {
            sessions,
            replays,
            replays_for_account: local,
            encoded_bytes: 500,
            ..usage()
        };
        let change = AdmissionFootprint {
            replaced_session_bytes: replacing.then_some(100),
            ..footprint()
        };
        assert_eq!(
            assess_capacity(account(), current, change, limits()).is_ok(),
            expected
        );
    }
    let saturated_account = SessionUsage {
        sessions: 0,
        replays: 3,
        replays_for_account: 3,
        encoded_bytes: 150,
        ..usage()
    };
    // All sessions may have expired or logged out. Retained replay is still charged.
    assert_eq!(
        assess_capacity(account(), saturated_account, footprint(), limits()),
        Err(E::Capacity)
    );
}

#[test]
fn replay_quota_is_issuer_qualified_and_cannot_be_selected_by_a_new_browser() {
    let saturated = SessionUsage {
        sessions: 1,
        replays: 3,
        replays_for_account: 3,
        encoded_bytes: 250,
        ..usage()
    };
    assert_eq!(
        assess_capacity(account(), saturated, footprint(), limits()),
        Err(E::Capacity)
    );
    for other in [
        (Principal::from_slice(&[2]), account().1),
        (account().0, Principal::from_slice(&[5])),
    ] {
        assert_eq!(
            assess_capacity(other, saturated, footprint(), limits()),
            Err(E::InvalidFacts)
        );
        let other_lookup = SessionUsage {
            account: other,
            replays_for_account: 0,
            ..saturated
        };
        assert_eq!(
            assess_capacity(other, other_lookup, footprint(), limits()),
            Ok(SessionUsage {
                account: other,
                sessions: 2,
                replays: 4,
                replays_for_account: 1,
                encoded_bytes: 400,
            })
        );
    }
}

#[test]
fn byte_limit_is_exclusive_of_heap_cost_but_inclusive_at_exact_encoded_total() {
    for (ceiling, expected) in [(349, Err(E::ByteCapacity)), (350, Ok(350)), (351, Ok(350))] {
        let bound = SessionCapacity {
            max_encoded_bytes: ceiling,
            ..limits()
        };
        assert_eq!(
            assess_capacity(account(), usage(), footprint(), bound).map(|u| u.encoded_bytes),
            expected
        );
    }
    let replacement = AdmissionFootprint {
        session_bytes: 100,
        replay_bytes: 50,
        replaced_session_bytes: Some(150),
    };
    let bound = SessionCapacity {
        max_encoded_bytes: 200,
        ..limits()
    };
    assert_eq!(
        assess_capacity(account(), usage(), replacement, bound).map(|u| u.encoded_bytes),
        Ok(200)
    );
    // Subtract only the displaced session, never its replay history or known fence.
    let replacement = AdmissionFootprint {
        replaced_session_bytes: Some(100),
        ..replacement
    };
    assert_eq!(
        assess_capacity(account(), usage(), replacement, bound),
        Err(E::ByteCapacity)
    );
}

#[test]
fn arithmetic_subtracts_before_adding_and_never_wraps_counts_or_bytes() {
    let bound = SessionCapacity {
        max_sessions: usize::MAX,
        max_replays: usize::MAX,
        max_replays_per_account: usize::MAX,
        max_encoded_bytes: usize::MAX,
    };
    let large = SessionUsage {
        encoded_bytes: usize::MAX,
        ..usage()
    };
    let replacement = AdmissionFootprint {
        session_bytes: 100,
        replay_bytes: 100,
        replaced_session_bytes: Some(200),
    };
    assert_eq!(
        assess_capacity(account(), large, replacement, bound).map(|u| u.encoded_bytes),
        Ok(usize::MAX)
    );
    let overflow = AdmissionFootprint {
        session_bytes: 101,
        ..replacement
    };
    assert_eq!(
        assess_capacity(account(), large, overflow, bound),
        Err(E::ByteCapacity)
    );
    assert_eq!(
        assess_capacity(
            account(),
            usage(),
            AdmissionFootprint {
                replaced_session_bytes: Some(201),
                ..replacement
            },
            bound
        ),
        Err(E::InvalidFacts)
    );
    for (sessions, replays, local) in [
        (usize::MAX, 1, 1),
        (1, usize::MAX, 1),
        (1, usize::MAX, usize::MAX),
    ] {
        let counts = SessionUsage {
            sessions,
            replays,
            replays_for_account: local,
            ..usage()
        };
        assert_eq!(
            assess_capacity(account(), counts, footprint(), bound),
            Err(E::Capacity)
        );
    }
    let counts = SessionUsage {
        sessions: usize::MAX - 1,
        replays: usize::MAX - 1,
        replays_for_account: usize::MAX - 1,
        ..usage()
    };
    assert_eq!(
        assess_capacity(account(), counts, footprint(), bound),
        Ok(SessionUsage {
            account: account(),
            sessions: usize::MAX,
            replays: usize::MAX,
            replays_for_account: usize::MAX,
            encoded_bytes: 350,
        })
    );
}

#[test]
fn bounded_record_sizes_and_inconsistent_selected_facts_refuse() {
    for change in [
        (|f: &mut AdmissionFootprint| f.session_bytes = 0) as fn(&mut AdmissionFootprint),
        |f| f.session_bytes = 2049,
        |f| f.replay_bytes = 0,
        |f| f.replay_bytes = 513,
        |f| f.replaced_session_bytes = Some(0),
        |f| f.replaced_session_bytes = Some(2049),
    ] {
        let mut size = footprint();
        change(&mut size);
        assert_eq!(
            assess_capacity(account(), usage(), size, limits()),
            Err(E::InvalidFacts)
        );
    }
    let maximum = AdmissionFootprint {
        session_bytes: 2048,
        replay_bytes: 512,
        replaced_session_bytes: None,
    };
    assert_eq!(
        assess_capacity(
            account(),
            usage(),
            maximum,
            SessionCapacity {
                max_encoded_bytes: 2760,
                ..limits()
            }
        )
        .map(|u| u.encoded_bytes),
        Ok(2760)
    );
    let replacement = AdmissionFootprint {
        replaced_session_bytes: Some(100),
        ..footprint()
    };
    assert_eq!(
        assess_capacity(
            account(),
            SessionUsage {
                sessions: 0,
                ..usage()
            },
            replacement,
            limits()
        ),
        Err(E::InvalidFacts)
    );
    assert_eq!(
        assess_capacity(
            account(),
            SessionUsage {
                replays_for_account: 2,
                ..usage()
            },
            footprint(),
            limits()
        ),
        Err(E::InvalidFacts)
    );
    for account in [
        (Principal::anonymous(), account().1),
        (account().0, Principal::anonymous()),
    ] {
        assert_eq!(
            assess_capacity(
                account,
                SessionUsage { account, ..usage() },
                footprint(),
                limits()
            ),
            Err(E::InvalidFacts)
        );
    }
    for change in [
        (|l: &mut SessionCapacity| l.max_sessions = 0) as fn(&mut SessionCapacity),
        |l| l.max_replays = 0,
        |l| l.max_replays_per_account = 0,
        |l| l.max_encoded_bytes = 0,
    ] {
        let mut limit = limits();
        change(&mut limit);
        assert_eq!(
            assess_capacity(account(), usage(), footprint(), limit),
            Err(E::InvalidContext)
        );
    }
}
