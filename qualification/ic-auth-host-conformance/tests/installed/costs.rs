//! Narrow installed receiver measurements, not accepted production budgets.
use super::{frame, install, receiver, start, wasm};
use ic_auth_host_fixture::{
    RESPONSE,
    enrollment::Fault,
    receiver::{Command, MEASURE, Measurement, Outcome, REQUEST, Refusal},
};
use ic_testkit::pocket_ic::CanisterId;
use sha2::{Digest, Sha256};

#[tokio::test]
#[ignore = "requires prepared build tools, independent vectors and PocketIC 16.x"]
async fn receiver_cost_observations() {
    let vectors = receiver::vectors();
    let caller = CanisterId::from_slice(&vectors["initial-caller"]);
    let owned = start().await;
    let pic = &owned;
    receiver::time(pic, 0).await;
    let wasm = wasm();
    println!(
        "cost-fixture wasm_bytes={} sha256={:02x?}",
        wasm.len(),
        Sha256::digest(&wasm)
    );
    let id = install(pic, &wasm).await;
    receiver::time(pic, 25).await;
    assert_eq!(
        receiver::control(pic, id, caller, &Command::Enroll(Fault::None))
            .await
            .unwrap(),
        Ok(Outcome::Enrolled)
    );
    let mut bad = receiver::credential(&vectors, "initial", false);
    if let Command::Admit { signature, .. } = &mut bad {
        signature[0] ^= 1;
    }
    let deadline = ic_auth_host_fixture::receiver::TIME_ORIGIN_NS + 100_000_000_000;
    // Fixed ordered scenario; retry exercises retained evidence, not new verification.
    let mut admitted_until = None;
    for (name, command, mut expected) in [
        ("bad-proof", bad, Err(Refusal::Proof)),
        (
            "first-admission",
            receiver::credential(&vectors, "initial", false),
            Ok(Outcome::Admitted(false, deadline - 15_000_000_000)),
        ),
        (
            "exact-retry",
            receiver::credential(&vectors, "initial", false),
            Ok(Outcome::Admitted(true, deadline - 15_000_000_000)),
        ),
        (
            "authorize",
            Command::Authorize(false),
            Ok(Outcome::Authorized),
        ),
        (
            "logout",
            Command::Logout(false),
            Ok(Outcome::Revoked(deadline)),
        ),
        (
            "revoked-authorization",
            Command::Authorize(false),
            Err(Refusal::Revoked),
        ),
    ] {
        let bytes = ic_wire::encode(
            &command,
            frame(MEASURE, ic_wire::Direction::Request),
            REQUEST,
        )
        .unwrap();
        let request_bytes = bytes.len();
        let reply = pic
            .update_call(id, caller, "receiver_fixture_measure", bytes)
            .await
            .unwrap();
        let sample: Measurement = ic_wire::decode(
            &reply,
            frame(MEASURE, ic_wire::Direction::Response),
            RESPONSE,
        )
        .unwrap();
        if name == "first-admission" {
            let observed = receiver::observe(pic, id).await;
            let until = observed.observed_at_ns + 60_000_000_000;
            assert!(until < deadline);
            admitted_until = Some(until);
            expected = Ok(Outcome::Admitted(false, until));
        } else if name == "exact-retry" {
            expected = Ok(Outcome::Admitted(true, admitted_until.unwrap()));
        }
        assert_eq!(sample.result, expected, "{name}");
        assert!(sample.instructions > 0, "{name}");
        assert!(sample.heap_pages.1 >= sample.heap_pages.0, "{name}");
        let ordinary = ic_wire::encode(
            &expected,
            frame(
                ic_auth_host_fixture::receiver::CONTROL,
                ic_wire::Direction::Response,
            ),
            RESPONSE,
        )
        .unwrap();
        assert_eq!(sample.response_bytes, ordinary.len() as u64);
        println!(
            "cost {name} instructions={} heap_pages_before={} heap_pages_after={} request_bytes={request_bytes} response_bytes={}",
            sample.instructions, sample.heap_pages.0, sample.heap_pages.1, sample.response_bytes
        );
    }
    let observed = receiver::observe(pic, id).await;
    assert_eq!(observed.application_writes, 1);
    assert_eq!((observed.usage.0, observed.usage.1), (0, 1));
    owned.drop().await;
}
