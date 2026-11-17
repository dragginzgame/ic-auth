//! Small installed populations and fixed-size maintenance batches, not fleet budgets.
use super::{
    receiver,
    receiver_support::{enroll, install, measured},
    start, wasm,
};
use ic_auth_host_fixture::receiver::{Capacity, Command, Outcome};
use ic_testkit::pocket_ic::CanisterId;
use sha2::{Digest, Sha256};
#[tokio::test]
#[ignore = "requires prepared build tools, independent vectors and PocketIC 16.x"]
async fn population_pruning_cost_observations() {
    let vectors = receiver::vectors();
    let wasm = wasm();
    println!(
        "population-cost-fixture wasm_bytes={} sha256={:02x?}",
        wasm.len(),
        Sha256::digest(&wasm)
    );
    for size in [1, 8, 32] {
        let owned = start().await;
        let pic = &owned;
        receiver::time(pic, 0).await;
        // Identical capacity/budget configuration at all three population sizes.
        let limits = Capacity {
            sessions: 32,
            replays: 64,
            account_replays: 2,
            bytes: 32768,
            logins: 32,
            verification: 128,
            refill_ns: 100_000_000,
            wide_scopes: false,
        };
        let id = install(pic, &wasm, limits).await;
        let first = CanisterId::from_slice(&vectors["population-0-caller"]);
        receiver::time(pic, 25).await;
        enroll(pic, id, first).await;
        let mut callers = Vec::new();
        let mut deadlines = Vec::new();
        for i in 0..size {
            let name = format!("population-{i}");
            let caller = CanisterId::from_slice(&vectors[&format!("{name}-caller")]);
            let command = receiver::credential(&vectors, &name, false);
            let result = if i + 1 == size {
                measured(pic, id, caller, &command, size, "last-admission").await
            } else {
                receiver::control(pic, id, caller, &command)
                    .await
                    .unwrap()
                    .unwrap()
            };
            let Outcome::Admitted(false, until) = result else {
                panic!("expected fresh population admission")
            };
            callers.push(caller);
            deadlines.push(until);
        }
        let populated = receiver::observe(pic, id).await;
        assert_eq!(
            (populated.usage.0, populated.usage.1),
            (u64::from(size), u64::from(size))
        );
        assert_eq!(populated.logins, (u64::from(size), 0));
        assert!(populated.usage.2 > 0);
        for (name, caller) in [
            ("authorize-first", first),
            ("authorize-last", *callers.last().unwrap()),
        ] {
            assert_eq!(
                measured(pic, id, caller, &Command::Authorize(false), size, name).await,
                Outcome::Authorized
            );
        }
        assert_eq!(receiver::observe(pic, id).await.application_writes, 2);
        // Proofs expire first. Pruning their replays must preserve live sessions.
        receiver::time(pic, 35).await;
        for phase in ["prune-replays", "prune-sessions"] {
            if phase == "prune-sessions" {
                receiver::time(pic, 110).await;
            }
            let mut remaining = size;
            let mut batches = 0;
            while remaining > 0 {
                let before = receiver::observe(pic, id).await;
                let command = Command::Prune(4);
                let result = if batches == 0 {
                    measured(pic, id, first, &command, size, phase).await
                } else {
                    receiver::control(pic, id, first, &command)
                        .await
                        .unwrap()
                        .unwrap()
                };
                let removed = remaining.min(4);
                let expected = if phase == "prune-replays" {
                    Outcome::Pruned(0, removed, 0)
                } else {
                    Outcome::Pruned(removed, 0, 0)
                };
                assert_eq!(result, expected);
                remaining -= removed;
                batches += 1;
                let after = receiver::observe(pic, id).await;
                let counts = if phase == "prune-replays" {
                    (u64::from(size), u64::from(remaining))
                } else {
                    (u64::from(remaining), 0)
                };
                assert_eq!((after.usage.0, after.usage.1), counts);
                assert!(after.usage.2 < before.usage.2);
                assert_eq!(after.logins, populated.logins);
                assert_eq!(after.credit, before.credit);
                assert_eq!(after.application_writes, before.application_writes);
            }
            assert_eq!(batches, size.div_ceil(4));
            if phase == "prune-replays" {
                // Every retained session retries after proof expiry and replay pruning.
                for (i, (&caller, &deadline)) in callers.iter().zip(&deadlines).enumerate() {
                    let command = receiver::credential(&vectors, &format!("population-{i}"), false);
                    assert_eq!(
                        receiver::control(pic, id, caller, &command).await.unwrap(),
                        Ok(Outcome::Admitted(true, deadline))
                    );
                }
                let retried = receiver::observe(pic, id).await;
                assert_eq!((retried.usage.0, retried.usage.1), (u64::from(size), 0));
                assert_eq!(retried.credit, populated.credit);
            }
        }
        let empty = receiver::observe(pic, id).await;
        assert_eq!(empty.usage, (0, 0, 0));
        assert_eq!(empty.logins, (u64::from(size), 0));
        assert_eq!(
            measured(pic, id, first, &Command::Prune(4), size, "empty-prune").await,
            Outcome::Pruned(0, 0, 0)
        );
        let after = receiver::observe(pic, id).await;
        assert_eq!(after.usage, empty.usage);
        assert_eq!(after.logins, empty.logins);
        assert_eq!(after.credit, empty.credit);
        owned.drop().await;
    }
}
