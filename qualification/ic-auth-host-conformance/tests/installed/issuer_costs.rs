//! Installed certified-issuer control costs with retained-state evidence.
use super::{frame, install, observe, start, wasm};
use ic_auth_host_fixture::{
    Command, Fault, MEASURE, Measurement, Progress, REQUEST, RESPONSE, Refusal,
};
use ic_testkit::pocket_ic::CanisterId;
use sha2::{Digest, Sha256};

#[tokio::test]
#[ignore = "requires prepared build tools and PocketIC 16.x"]
async fn issuer_cost_observations() {
    let owned = start().await;
    let pic = &owned;
    let wasm = wasm();
    println!(
        "issuer-cost-fixture wasm_bytes={} sha256={:02x?}",
        wasm.len(),
        Sha256::digest(&wasm)
    );
    let id = install(pic, &wasm).await;
    let account = CanisterId::from_slice(&[9]);
    let mut completed = None;
    for (name, caller, command, expected) in [
        ("prepare-approved-intent", account, Command::Prepare, Ok(())),
        (
            "wrong-account-completion",
            CanisterId::from_slice(&[8]),
            Command::Complete(Fault::None),
            Err(Refusal::Denied),
        ),
        (
            "certified-completion",
            account,
            Command::Complete(Fault::None),
            Ok(()),
        ),
        // A Root fault would trap if exact retry tried to publish again.
        (
            "completed-retry",
            account,
            Command::Complete(Fault::Root),
            Ok(()),
        ),
        ("account-revocation", account, Command::Revoke, Ok(())),
        (
            "retained-retry-after-revocation",
            account,
            Command::Complete(Fault::Root),
            Ok(()),
        ),
    ] {
        let before = observe(pic, id, account).await;
        let bytes = ic_wire::encode(
            &command,
            frame(MEASURE, ic_wire::Direction::Request),
            REQUEST,
        )
        .unwrap();
        let request_bytes = bytes.len();
        let reply = pic
            .update_call(id, caller, "auth_fixture_measure", bytes)
            .await
            .unwrap();
        let sample: Measurement = ic_wire::decode(
            &reply,
            frame(MEASURE, ic_wire::Direction::Response),
            RESPONSE,
        )
        .unwrap();
        assert_eq!(sample.result, expected, "{name}");
        assert!(sample.instructions > 0);
        assert!(sample.heap_pages.1 >= sample.heap_pages.0);
        let ordinary = ic_wire::encode(
            &expected,
            frame(ic_auth_host_fixture::CONTROL, ic_wire::Direction::Response),
            RESPONSE,
        )
        .unwrap();
        assert_eq!(sample.response_bytes, ordinary.len() as u64);
        let after = observe(pic, id, account).await;
        match name {
            "prepare-approved-intent" => {
                assert_eq!(after.progress, Progress::Pending);
                assert_eq!((after.usage.0, after.usage.1, after.credit), (1, 1, 3));
                assert_eq!(before.root, after.root);
            }
            "certified-completion" => {
                assert_eq!(after.progress, Progress::Completed);
                assert!(!after.claims.is_empty());
                assert!(!after.signature.is_empty());
                assert_ne!(before.root, after.root);
                completed = Some(after);
            }
            "wrong-account-completion" => super::unchanged(&before, &after),
            _ => {
                let completed = completed.as_ref().unwrap();
                assert_eq!(after.claims, completed.claims);
                assert_eq!(after.root, completed.root);
                assert_eq!(after.usage, completed.usage);
                assert_eq!(after.credit, completed.credit);
            }
        }
        println!(
            "issuer-cost {name} instructions={} heap_pages_before={} heap_pages_after={} request_bytes={request_bytes} response_bytes={}",
            sample.instructions, sample.heap_pages.0, sample.heap_pages.1, sample.response_bytes
        );
    }
    owned.drop().await;
}
