//! Installed known-login capacity: valid control, no eviction and no crypto charge.
use super::{frame, install, receiver, start, wasm};
use ic_auth_host_fixture::{
    RESPONSE,
    enrollment::Fault,
    receiver::{CONTROL, Command, MEASURE, Measurement, Outcome, REQUEST, Refusal},
};
use ic_testkit::pocket_ic::CanisterId;
use sha2::{Digest, Sha256};

#[tokio::test]
#[ignore = "requires prepared build tools, independent vectors and PocketIC 16.x"]
async fn capacity_cost_observations() {
    let vectors = receiver::vectors();
    let original = CanisterId::from_slice(&vectors["initial-caller"]);
    let candidate = CanisterId::from_slice(&vectors["other-device-caller"]);
    let owned = start().await;
    let pic = &owned;
    receiver::time(pic, 0).await;
    let wasm = wasm();
    println!(
        "capacity-cost-fixture wasm_bytes={} sha256={:02x?}",
        wasm.len(),
        Sha256::digest(&wasm)
    );
    let full = install(pic, &wasm).await;
    let empty = install(pic, &wasm).await;
    receiver::time(pic, 25).await;
    for id in [full, empty] {
        assert_eq!(
            receiver::control(pic, id, original, &Command::Enroll(Fault::None))
                .await
                .unwrap(),
            Ok(Outcome::Enrolled)
        );
    }
    let initial = receiver::credential(&vectors, "initial", false);
    let Outcome::Admitted(false, deadline) = receiver::control(pic, full, original, &initial)
        .await
        .unwrap()
        .unwrap()
    else {
        panic!("initial admission failed")
    };
    let other = receiver::credential(&vectors, "other-device", false);
    // Same independent credential, caller, protected key/grants and proof window.
    // Empty receiver acceptance distinguishes capacity from bad input or authority.
    assert!(matches!(
        receiver::control(pic, empty, candidate, &other)
            .await
            .unwrap(),
        Ok(Outcome::Admitted(false, _))
    ));
    let before = receiver::observe(pic, full).await;
    assert_eq!(before.logins, (1, 0));
    assert_eq!((before.usage.0, before.usage.1), (1, 1));
    let bytes =
        ic_wire::encode(&other, frame(MEASURE, ic_wire::Direction::Request), REQUEST).unwrap();
    let request_bytes = bytes.len();
    let reply = pic
        .update_call(full, candidate, "receiver_fixture_measure", bytes)
        .await
        .unwrap();
    let sample: Measurement = ic_wire::decode(
        &reply,
        frame(MEASURE, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap();
    assert_eq!(sample.result, Err(Refusal::Other));
    assert!(sample.instructions > 0);
    assert!(sample.heap_pages.1 >= sample.heap_pages.0);
    assert_eq!(
        sample.response_bytes,
        ic_wire::encode(
            &Err::<Outcome, _>(Refusal::Other),
            frame(CONTROL, ic_wire::Direction::Response),
            RESPONSE
        )
        .unwrap()
        .len() as u64
    );
    let after = receiver::observe(pic, full).await;
    assert_eq!(after.usage, before.usage);
    assert_eq!(after.logins, before.logins);
    assert_eq!(after.credit, before.credit);
    assert_eq!(after.application_writes, 0);
    assert_eq!(
        receiver::control(pic, full, candidate, &Command::Authorize(false))
            .await
            .unwrap(),
        Err(Refusal::Missing)
    );
    assert_eq!(
        receiver::control(pic, full, original, &initial)
            .await
            .unwrap(),
        Ok(Outcome::Admitted(true, deadline))
    );
    assert_eq!(
        receiver::control(pic, full, original, &Command::Authorize(false))
            .await
            .unwrap(),
        Ok(Outcome::Authorized)
    );
    assert_eq!(receiver::observe(pic, full).await.application_writes, 1);
    // Logout releases a session but intentionally retains the login fence. It is
    // not quota eviction and cannot make a second device fit the one-login limit.
    assert!(matches!(
        receiver::control(pic, full, original, &Command::Logout(false))
            .await
            .unwrap(),
        Ok(Outcome::Revoked(_))
    ));
    let logged_out = receiver::observe(pic, full).await;
    assert_eq!(logged_out.logins, (1, 0));
    assert_eq!(logged_out.usage.0, 0);
    assert_eq!(
        receiver::control(pic, full, candidate, &other)
            .await
            .unwrap(),
        Err(Refusal::Other)
    );
    let retained = receiver::observe(pic, full).await;
    assert_eq!(retained.usage, logged_out.usage);
    assert_eq!(retained.logins, logged_out.logins);
    assert_eq!(retained.credit, logged_out.credit);
    println!(
        "capacity-cost known-login-full instructions={} heap_pages_before={} heap_pages_after={} request_bytes={request_bytes} response_bytes={}",
        sample.instructions, sample.heap_pages.0, sample.heap_pages.1, sample.response_bytes
    );
    owned.drop().await;
}

#[tokio::test]
#[ignore = "requires prepared build tools, independent vectors and PocketIC 16.x"]
async fn replay_capacity_cost_observations() {
    let vectors = receiver::vectors();
    let caller = CanisterId::from_slice(&vectors["initial-caller"]);
    assert_eq!(caller.as_slice(), vectors["renewal-caller"]);
    assert_eq!(caller.as_slice(), vectors["late-old-caller"]);
    let owned = start().await;
    let pic = &owned;
    receiver::time(pic, 0).await;
    let wasm = wasm();
    println!(
        "replay-capacity-cost-fixture wasm_bytes={} sha256={:02x?}",
        wasm.len(),
        Sha256::digest(&wasm)
    );
    let full = install(pic, &wasm).await;
    let empty = install(pic, &wasm).await;
    receiver::time(pic, 25).await;
    for id in [full, empty] {
        assert_eq!(
            receiver::control(pic, id, caller, &Command::Enroll(Fault::None))
                .await
                .unwrap(),
            Ok(Outcome::Enrolled)
        );
    }
    let initial = receiver::credential(&vectors, "initial", false);
    let renewed = receiver::credential(&vectors, "renewal", false);
    let Outcome::Admitted(false, first_expiry) = receiver::control(pic, full, caller, &initial)
        .await
        .unwrap()
        .unwrap()
    else {
        panic!("initial admission failed")
    };
    let Outcome::Admitted(false, original_expiry) = receiver::control(pic, full, caller, &renewed)
        .await
        .unwrap()
        .unwrap()
    else {
        panic!("renewal admission failed")
    };
    let before = receiver::observe(pic, full).await;
    assert!(original_expiry >= first_expiry);
    assert_eq!(original_expiry, before.observed_at_ns + 60_000_000_000);
    assert_eq!(before.logins, (1, 0));
    assert_eq!((before.usage.0, before.usage.1), (1, 2));
    // Earlier proofs have expired, but their replay rows remain physically charged
    // until explicit pruning. This later proof is independently valid now.
    receiver::time(pic, 35).await;
    let candidate = receiver::credential(&vectors, "late-old", false);
    assert!(matches!(
        receiver::control(pic, empty, caller, &candidate)
            .await
            .unwrap(),
        Ok(Outcome::Admitted(false, _))
    ));
    let bytes = ic_wire::encode(
        &candidate,
        frame(MEASURE, ic_wire::Direction::Request),
        REQUEST,
    )
    .unwrap();
    let request_bytes = bytes.len();
    let reply = pic
        .update_call(full, caller, "receiver_fixture_measure", bytes)
        .await
        .unwrap();
    let sample: Measurement = ic_wire::decode(
        &reply,
        frame(MEASURE, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap();
    assert_eq!(sample.result, Err(Refusal::Other));
    assert!(sample.instructions > 0);
    assert!(sample.heap_pages.1 >= sample.heap_pages.0);
    assert_eq!(
        sample.response_bytes,
        ic_wire::encode(
            &Err::<Outcome, _>(Refusal::Other),
            frame(CONTROL, ic_wire::Direction::Response),
            RESPONSE
        )
        .unwrap()
        .len() as u64
    );
    let after = receiver::observe(pic, full).await;
    assert_eq!(after.usage, before.usage);
    assert_eq!(after.logins, before.logins);
    assert_eq!(after.credit, before.credit);
    // Exact retry is retained-session evidence, even after the proof window. It
    // must not charge work, extend its lease or evict either replay row.
    assert_eq!(
        receiver::control(pic, full, caller, &renewed)
            .await
            .unwrap(),
        Ok(Outcome::Admitted(true, original_expiry))
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
    assert_eq!(receiver::observe(pic, full).await.application_writes, 1);
    assert!(matches!(
        receiver::control(pic, full, caller, &Command::Logout(false))
            .await
            .unwrap(),
        Ok(Outcome::Revoked(_))
    ));
    let logged_out = receiver::observe(pic, full).await;
    assert_eq!((logged_out.usage.0, logged_out.usage.1), (0, 2));
    assert_eq!(
        receiver::control(pic, full, caller, &renewed)
            .await
            .unwrap(),
        Err(Refusal::Revoked)
    );
    assert_eq!(receiver::observe(pic, full).await.usage, logged_out.usage);
    println!(
        "capacity-cost replay-slots-full instructions={} heap_pages_before={} heap_pages_after={} request_bytes={request_bytes} response_bytes={}",
        sample.instructions, sample.heap_pages.0, sample.heap_pages.1, sample.response_bytes
    );
    owned.drop().await;
}
