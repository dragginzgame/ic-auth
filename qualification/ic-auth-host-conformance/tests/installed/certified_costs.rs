//! Full ECDSA/BLS admission in installed Wasm with independent signed claims.
use super::{frame, receiver::time, start, wasm};
use ic_auth_host_fixture::{
    Installation, RESPONSE,
    certified::{self as fixture, Command, Measurement, at},
    hash_bytes,
    receiver::{Outcome, Refusal},
};
use ic_platform::{
    certificate::construction::{
        Certificate, encode_canister_signature, encode_certificate, fork, labeled, leaf,
    },
    fixture_signing::PrivateKey,
};
use ic_testkit::pocket_ic::CanisterId;
use sha2::{Digest, Sha256};

fn signature(key: &PrivateKey, digest: [u8; 32]) -> Vec<u8> {
    let message = [b"\x1bic-auth-portable-credential".as_slice(), &digest].concat();
    let witness = labeled(
        b"sig".to_vec(),
        labeled(
            hash_bytes(fixture::SEED),
            labeled(hash_bytes(&message), leaf(Vec::new())),
        ),
    );
    let mut n = at(150);
    let mut encoded_time = Vec::new();
    loop {
        let b = (n & 127) as u8;
        n >>= 7;
        encoded_time.push(b | if n == 0 { 0 } else { 128 });
        if n == 0 {
            break;
        }
    }
    let tree = fork(
        labeled(
            b"canister".to_vec(),
            labeled(
                vec![2; 10],
                labeled(b"certified_data".to_vec(), leaf(witness.digest())),
            ),
        ),
        labeled(b"time".to_vec(), leaf(encoded_time)),
    );
    let signed = [b"\x0Dic-state-root".as_slice(), &tree.digest()].concat();
    let cert = Certificate {
        tree,
        signature: key.sign(&signed).serialize().to_vec(),
        delegation: None,
    };
    encode_canister_signature(&encode_certificate(&cert, 16384).unwrap(), &witness, 16384).unwrap()
}

#[tokio::test]
#[ignore = "requires prepared build tools and PocketIC 16.x"]
async fn certified_receiver_cost_observations() {
    let owned = start().await;
    let pic = &owned;
    time(pic, 0).await;
    let wasm = wasm();
    println!(
        "certified-cost-fixture wasm_bytes={} sha256={:02x?}",
        wasm.len(),
        Sha256::digest(&wasm)
    );
    let mut scalar = [0; 32];
    scalar[31] = 1;
    let key = PrivateKey::deserialize(&scalar).unwrap();
    let id = pic.create_canister().await;
    pic.add_cycles(id, 5_000_000_000_000).await;
    let init = ic_wire::encode(
        &Installation {
            authority: Vec::new(),
            certified_network_key: key.public_key().serialize().to_vec(),
            receiver_capacity: None,
        },
        ic_wire::Encoding::Payload,
        ic_auth_host_fixture::enrollment::LIMITS,
    )
    .unwrap();
    assert!(init.len() <= 256);
    pic.install_canister(id, wasm, init, None).await;
    // Fill the initially empty work bucket and stay away from its next refill boundary.
    time(pic, 155).await;
    let caller = CanisterId::from_slice(&fixture::bytes("caller"));
    assert_eq!(
        hash_bytes(fixture::bytes("claims")),
        fixture::hash("claims_digest")
    );
    let good = signature(&key, fixture::hash("claims_digest"));
    let bad = signature(&key, [0; 32]);
    for (name, command, expected, usage, credit) in [
        (
            "wrong-message-proof",
            Command::Admit(bad),
            Err(Refusal::Proof),
            (0, 0),
            7,
        ),
        (
            "first-certified-admission",
            Command::Admit(good.clone()),
            Ok(Outcome::Admitted(false, at(190))),
            (1, 1),
            4,
        ),
        (
            "certified-exact-retry",
            Command::Admit(good),
            Ok(Outcome::Admitted(true, at(190))),
            (1, 1),
            4,
        ),
        (
            "certified-authorization",
            Command::Authorize,
            Ok(Outcome::Authorized),
            (1, 1),
            4,
        ),
        (
            "certified-logout",
            Command::Logout,
            Ok(Outcome::Revoked(at(190))),
            (0, 1),
            4,
        ),
        (
            "certified-revoked-authorization",
            Command::Authorize,
            Err(Refusal::Revoked),
            (0, 1),
            4,
        ),
    ] {
        let bytes = ic_wire::encode(
            &command,
            frame(fixture::MEASURE, ic_wire::Direction::Request),
            RESPONSE,
        )
        .unwrap();
        let request_bytes = bytes.len();
        let reply = pic
            .update_call(id, caller, "certified_fixture_measure", bytes)
            .await
            .unwrap();
        let sample: Measurement = ic_wire::decode(
            &reply,
            frame(fixture::MEASURE, ic_wire::Direction::Response),
            RESPONSE,
        )
        .unwrap();
        assert_eq!(sample.result, expected, "{name}");
        assert_eq!(sample.usage, usage, "{name}");
        assert_eq!(sample.credit, credit, "{name}");
        assert!(sample.instructions > 0);
        assert!(sample.heap_pages.1 >= sample.heap_pages.0);
        let ordinary = ic_wire::encode(
            &expected,
            frame(fixture::MEASURE, ic_wire::Direction::Response),
            RESPONSE,
        )
        .unwrap();
        assert_eq!(sample.response_bytes, ordinary.len() as u64);
        println!(
            "certified-cost {name} instructions={} heap_pages_before={} heap_pages_after={} request_bytes={request_bytes} response_bytes={}",
            sample.instructions, sample.heap_pages.0, sample.heap_pages.1, sample.response_bytes
        );
    }
    owned.drop().await;
}
