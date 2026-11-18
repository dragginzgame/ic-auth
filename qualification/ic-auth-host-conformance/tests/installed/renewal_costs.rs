//! Browser-authenticated renewal and current certificate/witness assembly costs.
use super::{control, frame, install, observe, start, wasm};
use ic_auth::{
    claims::decode_claims,
    issuer_signature::{IssuerSignaturePolicy, verify_issuer_signature},
};
use ic_auth_host_fixture::{
    Command, Fault, MEASURE, Measurement, OBSERVE, Observation, REQUEST, RESPONSE, Refusal, SEED,
    WITNESS_MEASURE, WitnessMeasurement, hash_bytes,
};
use ic_auth_protocol_types::Principal;
use ic_platform::certificate::{PreparedRootKey, construction::canister_key_der};
use ic_testkit::pocket_ic::{CanisterId, nonblocking::PocketIc};
use sha2::{Digest, Sha256};
use std::time::Duration;

async fn witness(
    pic: &PocketIc,
    id: CanisterId,
    account: CanisterId,
    expected: &Observation,
    name: &str,
) {
    let request = ic_wire::encode(
        &(),
        frame(WITNESS_MEASURE, ic_wire::Direction::Request),
        REQUEST,
    )
    .unwrap();
    let reply = pic
        .query_call(id, account, "auth_fixture_witness_measure", request.clone())
        .await
        .unwrap();
    let measured: WitnessMeasurement = ic_wire::decode(
        &reply,
        frame(WITNESS_MEASURE, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap();
    let view = &measured.observation;
    assert_eq!(view.claims, expected.claims);
    assert_eq!(view.root, expected.root);
    assert_eq!(view.usage, expected.usage);
    assert_eq!(view.credit, expected.credit);
    let root = PreparedRootKey::from_der(&pic.root_key().await.unwrap()).unwrap();
    let key = canister_key_der(Principal::from_slice(id.as_slice()), SEED).unwrap();
    verify_issuer_signature(
        hash_bytes(&view.claims),
        &view.signature,
        &key,
        &IssuerSignaturePolicy {
            signing_canister: Principal::from_slice(id.as_slice()),
            seed_hash: hash_bytes(SEED),
            ic_root_public_key_raw: &root.as_der()[37..],
            now_ns: view.now_ns,
            max_certificate_age_ns: 1_000_000_000,
            max_signature_bytes: RESPONSE.bytes,
        },
    )
    .unwrap();
    let ordinary =
        ic_wire::encode(view, frame(OBSERVE, ic_wire::Direction::Response), RESPONSE).unwrap();
    assert_eq!(measured.response_bytes, ordinary.len() as u64);
    assert!(measured.instructions > 0);
    assert!(measured.heap_pages.1 >= measured.heap_pages.0);
    println!(
        "witness-cost {name} instructions={} heap_pages_before={} heap_pages_after={} request_bytes={} response_bytes={} certificate_bytes={} signature_bytes={}",
        measured.instructions,
        measured.heap_pages.0,
        measured.heap_pages.1,
        request.len(),
        measured.response_bytes,
        view.certificate.len(),
        view.signature.len()
    );
}

#[tokio::test]
#[ignore = "requires prepared build tools and PocketIC 16.x"]
async fn renewal_witness_cost_observations() {
    let owned = start().await;
    let pic = &owned;
    let wasm = wasm();
    println!(
        "renewal-cost-fixture wasm_bytes={} sha256={:02x?}",
        wasm.len(),
        Sha256::digest(&wasm)
    );
    let id = install(pic, &wasm).await;
    let account = CanisterId::from_slice(&[9]);
    for command in [Command::Prepare, Command::Complete(Fault::None)] {
        assert_eq!(control(pic, id, account, command).await.unwrap(), Ok(()));
    }
    let original = observe(pic, id, account).await;
    witness(pic, id, account, &original, "initial-witness").await;
    let original_claims = decode_claims(&original.claims).unwrap();
    let der = [
        b"\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00".as_slice(),
        &[0x44; 32],
    ]
    .concat();
    let browser = CanisterId::from_slice(&[sha2::Sha224::digest(&der).as_slice(), &[2]].concat());
    pic.advance_time(Duration::from_secs(5)).await;
    let mut renewal = None;
    for (name, caller, command, expected) in [
        (
            "wrong-browser-renewal",
            account,
            Command::Renew,
            Err(Refusal::Denied),
        ),
        ("prepare-approved-renewal", browser, Command::Renew, Ok(())),
        (
            "complete-certified-renewal",
            account,
            Command::Complete(Fault::None),
            Ok(()),
        ),
        (
            "renewal-completed-retry",
            account,
            Command::Complete(Fault::Root),
            Ok(()),
        ),
    ] {
        let before = observe(pic, id, account).await;
        let request = ic_wire::encode(
            &command,
            frame(MEASURE, ic_wire::Direction::Request),
            REQUEST,
        )
        .unwrap();
        let reply = pic
            .update_call(id, caller, "auth_fixture_measure", request.clone())
            .await
            .unwrap();
        let sample: Measurement = ic_wire::decode(
            &reply,
            frame(MEASURE, ic_wire::Direction::Response),
            RESPONSE,
        )
        .unwrap();
        assert_eq!(sample.result, expected, "{name}");
        let after = observe(pic, id, account).await;
        if name == "wrong-browser-renewal" {
            assert_eq!(before.claims, after.claims);
            assert_eq!(before.usage, after.usage);
            assert_eq!(before.root, after.root);
            assert_eq!(before.credit, after.credit);
        } else if name == "prepare-approved-renewal" {
            assert_eq!((after.usage.0, after.usage.1), (1, 2));
            assert_eq!(after.credit, 3); // Five elapsed seconds refill to four, one renewal spends one.
            assert_eq!(after.root, original.root);
        } else {
            let claims = decode_claims(&after.claims).unwrap();
            assert_eq!(
                claims.login_started_at_ns,
                original_claims.login_started_at_ns
            );
            assert_eq!(claims.login_deadline_ns, original_claims.login_deadline_ns);
            assert!(claims.issued_at_ns > original_claims.issued_at_ns);
            assert_ne!(after.claims, original.claims);
            if let Some(previous) = &renewal {
                let previous: &Observation = previous;
                assert_eq!(after.claims, previous.claims);
                assert_eq!(after.root, previous.root);
                assert_eq!(after.usage, previous.usage);
                assert_eq!(after.credit, previous.credit);
            }
            renewal = Some(after);
        }
        assert!(sample.instructions > 0);
        assert!(sample.heap_pages.1 >= sample.heap_pages.0);
        let ordinary = ic_wire::encode(
            &expected,
            frame(ic_auth_host_fixture::CONTROL, ic_wire::Direction::Response),
            RESPONSE,
        )
        .unwrap();
        assert_eq!(sample.response_bytes, ordinary.len() as u64);
        println!(
            "renewal-cost {name} instructions={} heap_pages_before={} heap_pages_after={} request_bytes={} response_bytes={}",
            sample.instructions,
            sample.heap_pages.0,
            sample.heap_pages.1,
            request.len(),
            sample.response_bytes
        );
    }
    witness(
        pic,
        id,
        account,
        renewal.as_ref().unwrap(),
        "renewed-witness",
    )
    .await;
    // A later certificate wraps the same retained claims without extending the login.
    pic.advance_time(Duration::from_secs(1)).await;
    witness(
        pic,
        id,
        account,
        renewal.as_ref().unwrap(),
        "refreshed-witness",
    )
    .await;
    owned.drop().await;
}
