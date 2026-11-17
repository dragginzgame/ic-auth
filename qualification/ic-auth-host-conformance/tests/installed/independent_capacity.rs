//! One limiting dimension per installed refusal, with otherwise-identical controls.
use super::{
    receiver, receiver_support,
    receiver_support::{enroll, install},
    start, wasm,
};
use ic_auth_host_fixture::receiver::{Capacity, Command, Outcome, Refusal};
use ic_testkit::pocket_ic::{CanisterId, nonblocking::PocketIc};
use sha2::{Digest, Sha256};

async fn admit(pic: &PocketIc, id: CanisterId, caller: CanisterId, command: &Command) -> u64 {
    let Outcome::Admitted(false, until) = receiver::control(pic, id, caller, command)
        .await
        .unwrap()
        .unwrap()
    else {
        panic!("expected fresh admission")
    };
    until
}
async fn scenario(dimension: &str) {
    let vectors = receiver::vectors();
    let caller = CanisterId::from_slice(&vectors["initial-caller"]);
    let candidate = CanisterId::from_slice(&vectors["other-device-caller"]);
    let other_account = CanisterId::from_slice(&vectors["other-account-caller"]);
    let initial = receiver::credential(&vectors, "initial", false);
    let other = receiver::credential(&vectors, "other-device", false);
    let wasm = wasm();
    println!(
        "independent-capacity-fixture {dimension} wasm_bytes={} sha256={:02x?}",
        wasm.len(),
        Sha256::digest(&wasm)
    );
    let generous = Capacity {
        sessions: 4,
        replays: 8,
        account_replays: 8,
        bytes: 4096,
        logins: 4,
        ..Capacity::default()
    };
    // Observe the actual two-record occupancy on a separate positive control.
    // No production accounting helper computes the test's byte boundary.
    let scout = start().await;
    receiver::time(&scout, 0).await;
    let reference = install(&scout, &wasm, generous).await;
    receiver::time(&scout, 25).await;
    enroll(&scout, reference, caller).await;
    admit(&scout, reference, caller, &initial).await;
    let first_bytes = receiver::observe(&scout, reference).await.usage.2;
    admit(&scout, reference, candidate, &other).await;
    let reference_usage = receiver::observe(&scout, reference).await;
    assert_eq!((reference_usage.usage.0, reference_usage.usage.1), (2, 2));
    assert_eq!(reference_usage.account_replays, (2, 0));
    let two_bytes = u32::try_from(reference_usage.usage.2).unwrap();
    assert!(u64::from(two_bytes - 1) > first_bytes);
    scout.drop().await;
    let mut limits = generous;
    let mut relaxed_limits = generous;
    match dimension {
        "sessions" => {
            limits.sessions = 1;
            relaxed_limits.sessions = 2;
        }
        "bytes" => {
            limits.bytes = two_bytes - 1;
            relaxed_limits.bytes = two_bytes;
        }
        "global-replays" => {
            limits.replays = 1;
            relaxed_limits.replays = 2;
        }
        "account-replays" => {
            limits.account_replays = 1;
            relaxed_limits.account_replays = 2;
        }
        _ => unreachable!(),
    }
    let owned = start().await;
    let pic = &owned;
    receiver::time(pic, 0).await;
    let full = install(pic, &wasm, limits).await;
    let relaxed = install(pic, &wasm, relaxed_limits).await;
    let empty = install(pic, &wasm, limits).await;
    let account_control = if dimension == "global-replays" {
        Some(install(pic, &wasm, limits).await)
    } else {
        None
    };
    receiver::time(pic, 25).await;
    for id in [full, relaxed, empty] {
        enroll(pic, id, caller).await;
    }
    if let Some(id) = account_control {
        enroll(pic, id, other_account).await;
        admit(
            pic,
            id,
            other_account,
            &receiver::credential(&vectors, "other-account", false),
        )
        .await;
    }
    let deadline = admit(pic, full, caller, &initial).await;
    admit(pic, relaxed, caller, &initial).await;
    admit(pic, relaxed, candidate, &other).await;
    admit(pic, empty, candidate, &other).await;
    let relaxed_state = receiver::observe(pic, relaxed).await;
    assert_eq!(relaxed_state.usage, reference_usage.usage);
    assert_eq!(relaxed_state.logins, (2, 0));
    let before = receiver::observe(pic, full).await;
    assert_eq!(before.usage.2, first_bytes);
    assert_eq!(before.account_replays, (1, 0));
    let (sample, request_bytes) = receiver_support::measure(pic, full, candidate, &other).await;
    assert_eq!(sample.result, Err(Refusal::Other), "{dimension}");
    let after = receiver::observe(pic, full).await;
    assert_eq!(after.usage, before.usage);
    assert_eq!(after.logins, before.logins);
    assert_eq!(after.account_replays, before.account_replays);
    assert_eq!(after.credit, before.credit);
    assert_eq!(after.application_writes, before.application_writes);
    assert_eq!(
        receiver::control(pic, full, candidate, &Command::Authorize(false))
            .await
            .unwrap(),
        Err(Refusal::Missing)
    );
    assert_eq!(
        receiver::control(pic, full, caller, &initial)
            .await
            .unwrap(),
        Ok(Outcome::Admitted(true, deadline))
    );
    let retried = receiver::observe(pic, full).await;
    assert_eq!(retried.usage, before.usage);
    assert_eq!(retried.credit, before.credit);
    assert_eq!(
        receiver::control(pic, full, caller, &Command::Authorize(false))
            .await
            .unwrap(),
        Ok(Outcome::Authorized)
    );
    if dimension == "global-replays" {
        assert_eq!(
            receiver::control(
                pic,
                full,
                other_account,
                &receiver::credential(&vectors, "other-account", false)
            )
            .await
            .unwrap(),
            Err(Refusal::Other)
        );
        let cross = receiver::observe(pic, full).await;
        assert_eq!(cross.usage, before.usage);
        assert_eq!(cross.account_replays, (1, 0));
        assert_eq!(cross.credit, before.credit);
    }
    if dimension == "account-replays" {
        // Another subject can consume a global replay slot while the first is full.
        let credential = receiver::credential(&vectors, "other-account", false);
        admit(pic, full, other_account, &credential).await;
        let separate = receiver::observe(pic, full).await;
        assert_eq!((separate.usage.0, separate.usage.1), (2, 2));
        assert_eq!(separate.account_replays, (1, 1));
        assert_eq!(
            receiver::control(pic, full, other_account, &Command::Authorize(false))
                .await
                .unwrap(),
            Ok(Outcome::Authorized)
        );
    }
    assert!(matches!(
        receiver::control(pic, full, caller, &Command::Logout(false))
            .await
            .unwrap(),
        Ok(Outcome::Revoked(_))
    ));
    let logged_out = receiver::observe(pic, full).await;
    assert_eq!(logged_out.account_replays.0, 1);
    if matches!(dimension, "account-replays" | "global-replays") {
        assert_eq!(
            receiver::control(pic, full, candidate, &other)
                .await
                .unwrap(),
            Err(Refusal::Other)
        );
        let retained = receiver::observe(pic, full).await;
        assert_eq!(retained.usage, logged_out.usage);
        assert_eq!(retained.account_replays, logged_out.account_replays);
        assert_eq!(retained.credit, logged_out.credit);
    } else {
        admit(pic, full, candidate, &other).await;
        let recovered = receiver::observe(pic, full).await;
        assert_eq!((recovered.usage.0, recovered.usage.1), (1, 2));
        assert_eq!(recovered.logins, (2, 0));
        assert_eq!(recovered.account_replays, (2, 0));
        assert_eq!(
            receiver::control(pic, full, caller, &initial)
                .await
                .unwrap(),
            Err(Refusal::Revoked)
        );
    }
    println!(
        "independent-capacity-cost {dimension} instructions={} heap_pages_before={} heap_pages_after={} request_bytes={request_bytes} response_bytes={} first_bytes={first_bytes} two_bytes={two_bytes}",
        sample.instructions, sample.heap_pages.0, sample.heap_pages.1, sample.response_bytes
    );
    owned.drop().await;
}
#[tokio::test]
#[ignore = "requires prepared build tools and PocketIC 16.x"]
async fn session_capacity_cost_observations() {
    scenario("sessions").await;
}
#[tokio::test]
#[ignore = "requires prepared build tools and PocketIC 16.x"]
async fn byte_capacity_cost_observations() {
    scenario("bytes").await;
}
#[tokio::test]
#[ignore = "requires prepared build tools and PocketIC 16.x"]
async fn account_capacity_cost_observations() {
    scenario("account-replays").await;
}

#[tokio::test]
#[ignore = "requires prepared build tools and PocketIC 16.x"]
async fn global_replay_capacity_cost_observations() {
    scenario("global-replays").await;
}
