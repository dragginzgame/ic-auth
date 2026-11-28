//! Mixed deadline indexes and maximum scope count/width under installed host time.
use super::{
    receiver,
    receiver_support::{enroll, install, measured},
    start, wasm,
};
use ic_auth_host_fixture::receiver::{Capacity, Command, Outcome, Refusal};
use ic_testkit::pocket_ic::CanisterId;
use sha2::{Digest, Sha256};

#[tokio::test]
#[ignore = "requires prepared build tools, independent vectors and PocketIC 16.x"]
async fn mixed_expiry_cost_observations() {
    let vectors = receiver::vectors();
    let wasm = wasm();
    println!(
        "mixed-cost-fixture wasm_bytes={} sha256={:02x?}",
        wasm.len(),
        Sha256::digest(&wasm)
    );
    let owned = start().await;
    let pic = &owned;
    receiver::time(pic, 0).await;
    let id = install(
        pic,
        &wasm,
        Capacity {
            sessions: 4,
            replays: 8,
            account_replays: 2,
            logins: 4,
            ..Capacity::default()
        },
    )
    .await;
    let names = ["mixed-early", "mixed-middle", "mixed-late"];
    let callers = names.map(|name| CanisterId::from_slice(&vectors[&format!("{name}-caller")]));
    receiver::time(pic, 25).await;
    enroll(pic, id, callers[0]).await;
    let mut deadlines = Vec::new();
    for (name, caller) in names.iter().zip(callers) {
        let command = receiver::credential(&vectors, name, false);
        let Outcome::Admitted(false, deadline) = receiver::control(pic, id, caller, &command)
            .await
            .unwrap()
            .unwrap()
        else {
            panic!("expected fresh admission")
        };
        deadlines.push(deadline);
    }
    assert_eq!(deadlines[0], ic_auth_host_fixture::certified::at(28));
    assert_eq!(deadlines[1], ic_auth_host_fixture::certified::at(45));
    assert!(deadlines[2] > ic_auth_host_fixture::certified::at(85));
    let unknown = CanisterId::from_slice(&vectors["population-0-caller"]);
    let Outcome::Unconfirmed(deny_until) =
        receiver::control(pic, id, unknown, &Command::Logout(false))
            .await
            .unwrap()
            .unwrap()
    else {
        panic!("unknown logout must remain unconfirmed")
    };
    assert!(deny_until > ic_auth_host_fixture::certified::at(125));
    let mut previous_time = 25;
    let mut known = 3;
    for (time, limit, name, expected, usage, unknowns) in [
        (
            26,
            2,
            "mixed-not-yet-expired",
            Outcome::Pruned(0, 0, 0, 0),
            (3, 3),
            1,
        ),
        (
            31,
            1,
            "mixed-expired-session",
            Outcome::Pruned(1, 0, 0, 0),
            (2, 3),
            1,
        ),
        (
            31,
            2,
            "mixed-expired-replays",
            Outcome::Pruned(0, 2, 0, 0),
            (2, 1),
            1,
        ),
        (
            41,
            2,
            "mixed-later-replay",
            Outcome::Pruned(0, 1, 0, 0),
            (2, 0),
            1,
        ),
        (
            46,
            2,
            "mixed-middle-session",
            Outcome::Pruned(1, 0, 0, 0),
            (1, 0),
            1,
        ),
        (
            110,
            2,
            "mixed-late-session",
            Outcome::Pruned(1, 0, 0, 1),
            (0, 0),
            1,
        ),
        (
            130,
            2,
            "mixed-unknown-denial",
            Outcome::Pruned(0, 0, 1, 1),
            (0, 0),
            0,
        ),
        (
            130,
            2,
            "mixed-last-slot",
            Outcome::Pruned(0, 0, 0, 1),
            (0, 0),
            0,
        ),
    ] {
        if time > previous_time {
            receiver::time(pic, time).await;
            previous_time = time;
        }
        let before = receiver::observe(pic, id).await;
        assert_eq!(
            measured(pic, id, unknown, &Command::Prune(limit), 3, name).await,
            expected
        );
        let after = receiver::observe(pic, id).await;
        assert_eq!((after.usage.0, after.usage.1), usage);
        if let Outcome::Pruned(_, _, _, slots) = expected {
            known -= u64::from(slots);
        }
        assert_eq!(after.logins, (known, unknowns));
        assert_eq!(after.credit, before.credit);
        assert_eq!(after.application_writes, before.application_writes);
        if name == "mixed-not-yet-expired"
            || name == "mixed-last-slot"
            || name == "mixed-unknown-denial"
        {
            assert_eq!(after.usage.2, before.usage.2);
        } else {
            assert!(after.usage.2 < before.usage.2);
        }
        if name == "mixed-expired-replays" || name == "mixed-later-replay" {
            // Still-live sessions retain original exact retry after proof/replay expiry.
            for i in 1..3 {
                let command = receiver::credential(&vectors, names[i], false);
                assert_eq!(
                    receiver::control(pic, id, callers[i], &command)
                        .await
                        .unwrap(),
                    Ok(Outcome::Admitted(true, deadlines[i]))
                );
                assert_eq!(
                    receiver::control(pic, id, callers[i], &Command::Authorize(false))
                        .await
                        .unwrap(),
                    Ok(Outcome::Authorized)
                );
            }
            assert_eq!(
                receiver::control(pic, id, callers[0], &Command::Authorize(false))
                    .await
                    .unwrap(),
                Err(Refusal::Missing)
            );
        }
    }
    let final_state = receiver::observe(pic, id).await;
    assert_eq!(final_state.usage, (0, 0, 0));
    assert_eq!(final_state.logins, (0, 0));
    owned.drop().await;
}

#[tokio::test]
#[ignore = "requires prepared build tools, independent vectors and PocketIC 16.x"]
async fn wide_scope_cost_observations() {
    let vectors = receiver::vectors();
    let command = receiver::credential(&vectors, "wide-scopes", false);
    let caller = CanisterId::from_slice(&vectors["wide-scopes-caller"]);
    let claims = ic_auth::claims::decode_claims(&vectors["wide-scopes-claims"]).unwrap();
    assert_eq!(claims.grants.len(), 1);
    assert_eq!(claims.grants[0].target.as_str().len(), 64);
    assert_eq!(claims.grants[0].scopes.len(), 32);
    assert!(
        claims.grants[0]
            .scopes
            .iter()
            .all(|s| s.as_str().len() == 64)
    );
    let original_allowance = ic_wire::Limits {
        allocation: 131_072,
        ..ic_auth_host_fixture::receiver::REQUEST
    };
    assert!(
        ic_wire::encode(
            &command,
            super::frame(
                ic_auth_host_fixture::receiver::CONTROL,
                ic_wire::Direction::Request
            ),
            original_allowance
        )
        .is_err()
    );
    let encoding = super::frame(
        ic_auth_host_fixture::receiver::CONTROL,
        ic_wire::Direction::Request,
    );
    let original_nodes = ic_wire::Limits {
        nodes: 8192,
        ..ic_auth_host_fixture::receiver::REQUEST
    };
    assert!(ic_wire::encoded_len(&command, encoding, original_nodes).is_err());
    assert_eq!(
        ic_wire::encoded_len(&command, encoding, ic_auth_host_fixture::receiver::REQUEST).unwrap(),
        2558
    );
    let wasm = wasm();
    println!(
        "wide-cost-fixture wasm_bytes={} sha256={:02x?}",
        wasm.len(),
        Sha256::digest(&wasm)
    );
    let owned = start().await;
    let pic = &owned;
    receiver::time(pic, 0).await;
    // Baseline narrow session and replay cost 173 bytes at these exact principal/time widths.
    // A 64-byte role adds 60; sixteen 64-byte scopes replace one four-byte scope:
    // 16 * (1 + 64) - (1 + 4) = 1035. Both length/count prefixes fit one byte.
    const LOGICAL_BYTES: u64 = 173 + 60 + 1035;
    let id = install(
        pic,
        &wasm,
        Capacity {
            wide_scopes: true,
            bytes: LOGICAL_BYTES as u32,
            ..Capacity::default()
        },
    )
    .await;
    let short = install(
        pic,
        &wasm,
        Capacity {
            wide_scopes: true,
            bytes: LOGICAL_BYTES as u32 - 1,
            ..Capacity::default()
        },
    )
    .await;
    receiver::time(pic, 25).await;
    for receiver_id in [id, short] {
        enroll(pic, receiver_id, caller).await;
    }
    let before = receiver::observe(pic, short).await;
    assert_eq!(
        receiver::control(pic, short, caller, &command)
            .await
            .unwrap(),
        Err(Refusal::Other)
    );
    let refused = receiver::observe(pic, short).await;
    assert_eq!(refused.usage, (0, 0, 0));
    assert_eq!(refused.logins, (0, 0));
    assert_eq!(refused.credit, before.credit);
    let Outcome::Admitted(false, until) =
        measured(pic, id, caller, &command, 1, "wide-first-admission").await
    else {
        panic!("wide admission refused")
    };
    let admitted = receiver::observe(pic, id).await;
    assert_eq!(admitted.usage, (1, 1, LOGICAL_BYTES));
    assert_eq!(
        measured(pic, id, caller, &command, 1, "wide-exact-retry").await,
        Outcome::Admitted(true, until)
    );
    assert_eq!(
        measured(
            pic,
            id,
            caller,
            &Command::Authorize(false),
            1,
            "wide-authorize-sixteen-scopes"
        )
        .await,
        Outcome::Authorized
    );
    assert_eq!(
        receiver::control(pic, id, caller, &Command::AuthorizeUnrequested)
            .await
            .unwrap(),
        Err(Refusal::Other)
    );
    let checked = receiver::observe(pic, id).await;
    assert_eq!(checked.application_writes, 1);
    assert_eq!(checked.usage, admitted.usage);
    assert_eq!(checked.credit, admitted.credit);
    receiver::time(pic, 110).await;
    assert_eq!(
        measured(
            pic,
            id,
            caller,
            &Command::Prune(2),
            1,
            "wide-prune-session-and-replay"
        )
        .await,
        Outcome::Pruned(1, 1, 0, 0)
    );
    let pruned = receiver::observe(pic, id).await;
    assert_eq!(pruned.usage, (0, 0, 0));
    assert_eq!(pruned.logins, (1, 0));
    assert_eq!(pruned.credit, admitted.credit);
    owned.drop().await;
}
