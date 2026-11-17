//! Installed refusal costs with independent state and recovery assertions.
use super::{control, frame, install, observe, receiver, receiver_support, start, unchanged, wasm};
use ic_auth_host_fixture::{
    Command, Fault, MEASURE, Measurement, Progress, REQUEST, RESPONSE, Refusal, enrollment,
    receiver as wire,
};
use ic_testkit::pocket_ic::CanisterId;
use sha2::{Digest, Sha256};

#[tokio::test]
#[ignore = "requires prepared build tools, independent vectors and PocketIC 16.x"]
async fn limit_cost_observations() {
    let vectors = receiver::vectors();
    let caller = CanisterId::from_slice(&vectors["initial-caller"]);
    let owned = start().await;
    let pic = &owned;
    receiver::time(pic, 0).await;
    let wasm = wasm();
    println!(
        "limit-cost-fixture wasm_bytes={} sha256={:02x?}",
        wasm.len(),
        Sha256::digest(&wasm)
    );
    let id = install(pic, &wasm).await;
    receiver::time(pic, 25).await;
    assert_eq!(
        receiver::control(
            pic,
            id,
            caller,
            &wire::Command::Enroll(enrollment::Fault::None)
        )
        .await
        .unwrap(),
        Ok(wire::Outcome::Enrolled)
    );
    let mut invalid = receiver::credential(&vectors, "initial", false);
    if let wire::Command::Admit { signature, .. } = &mut invalid {
        signature[0] ^= 1;
    }
    for remaining in [8, 6, 4, 2, 0] {
        assert_eq!(
            receiver::control(pic, id, caller, &invalid).await.unwrap(),
            Err(wire::Refusal::Proof)
        );
        assert_eq!(receiver::observe(pic, id).await.credit.0, remaining);
    }
    let before = receiver::observe(pic, id).await;
    assert_eq!(before.usage, (0, 0, 0));
    let valid = receiver::credential(&vectors, "initial", false);
    let (sample, request_bytes) = receiver_support::measure(pic, id, caller, &valid).await;
    // The fixture coarsens budget refusal to Proof; exhausted credit and recovery
    // after the fixed refill independently establish the reason for refusal.
    assert_eq!(sample.result, Err(wire::Refusal::Proof));
    let refused = receiver::observe(pic, id).await;
    assert_eq!(refused.credit.0, 0);
    assert_eq!(refused.usage, before.usage);
    assert_eq!(refused.logins, before.logins);
    assert_eq!(refused.application_writes, 0);
    println!(
        "limit-cost exhausted-verification instructions={} heap_pages_before={} heap_pages_after={} request_bytes={request_bytes} response_bytes={}",
        sample.instructions, sample.heap_pages.0, sample.heap_pages.1, sample.response_bytes
    );
    pic.advance_time(std::time::Duration::from_secs(1)).await;
    assert_eq!(
        receiver::control(pic, id, caller, &valid).await.unwrap(),
        Err(wire::Refusal::Proof)
    );
    assert_eq!(receiver::observe(pic, id).await.credit.0, 1);
    pic.advance_time(std::time::Duration::from_secs(1)).await;
    assert!(matches!(
        receiver::control(pic, id, caller, &valid).await.unwrap(),
        Ok(wire::Outcome::Admitted(false, _))
    ));
    let recovered = receiver::observe(pic, id).await;
    assert_eq!(recovered.credit.0, 0);
    assert_eq!((recovered.usage.0, recovered.usage.1), (1, 1));

    let issuer = install(pic, &wasm).await;
    let account = CanisterId::from_slice(&[9]);
    assert_eq!(
        control(pic, issuer, account, Command::Prepare)
            .await
            .unwrap(),
        Ok(())
    );
    assert_eq!(
        control(pic, issuer, account, Command::Revoke)
            .await
            .unwrap(),
        Ok(())
    );
    let before = observe(pic, issuer, account).await;
    assert_eq!(before.progress, Progress::Pending);
    assert!(before.claims.is_empty());
    // A root-publication fault must not execute for a revoked pending operation.
    let bytes = ic_wire::encode(
        &Command::Complete(Fault::Root),
        frame(MEASURE, ic_wire::Direction::Request),
        REQUEST,
    )
    .unwrap();
    let request_bytes = bytes.len();
    let reply = pic
        .update_call(issuer, account, "auth_fixture_measure", bytes)
        .await
        .unwrap();
    let sample: Measurement = ic_wire::decode(
        &reply,
        frame(MEASURE, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap();
    assert_eq!(sample.result, Err(Refusal::Denied));
    assert!(sample.instructions > 0);
    assert!(sample.heap_pages.1 >= sample.heap_pages.0);
    assert_eq!(
        sample.response_bytes,
        ic_wire::encode(
            &Err::<(), _>(Refusal::Denied),
            frame(ic_auth_host_fixture::CONTROL, ic_wire::Direction::Response),
            RESPONSE
        )
        .unwrap()
        .len() as u64
    );
    unchanged(&before, &observe(pic, issuer, account).await);
    println!(
        "limit-cost revoked-pending-completion instructions={} heap_pages_before={} heap_pages_after={} request_bytes={request_bytes} response_bytes={}",
        sample.instructions, sample.heap_pages.0, sample.heap_pages.1, sample.response_bytes
    );
    owned.drop().await;
}
