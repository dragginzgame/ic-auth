//! Replay frozen issuer evidence and exercise a fresh NNS-to-application delegation.
use super::{certified_costs::install_receiver, frame, receiver::time, support, wasm};
use ic_auth_host_fixture::{
    RESPONSE,
    certified::{self as fixture, Command, Measurement, at},
    hash_bytes,
    receiver::Outcome,
};
use ic_platform::certificate::{
    CertificateDecodeLimits, PreparedRootKey,
    construction::{LookupResult, encode_canister_signature, labeled, leaf},
    decode_certificate,
};
use ic_testkit::pocket_ic::{CanisterId, nonblocking::PocketIc};

fn check_delegation(certificate: &[u8], subnet: CanisterId) {
    let limits = CertificateDecodeLimits {
        bytes: RESPONSE.bytes,
    };
    let decoded = decode_certificate(certificate, limits).unwrap();
    let delegation = decoded.unverified().delegation.as_ref().unwrap();
    assert_eq!(delegation.subnet_id.as_slice(), subnet.as_slice());
    let parent = decode_certificate(&delegation.certificate, limits).unwrap();
    assert_eq!(
        parent
            .unverified()
            .tree
            .lookup_path([b"subnet".as_slice(), subnet.as_slice(), b"type"]),
        LookupResult::Found(b"application")
    );
}

async fn admit(pic: &PocketIc, receiver: CanisterId, signature: Vec<u8>) {
    let request = ic_wire::encode(
        &Command::Admit(signature),
        frame(fixture::MEASURE, ic_wire::Direction::Request),
        RESPONSE,
    )
    .unwrap();
    let caller = CanisterId::from_slice(&fixture::bytes("caller"));
    let reply = pic
        .update_call(receiver, caller, "certified_fixture_measure", request)
        .await
        .unwrap();
    let measurement: Measurement = ic_wire::decode(
        &reply,
        frame(fixture::MEASURE, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap();
    assert_eq!(measurement.result, Ok(Outcome::Admitted(false, at(190))));
    assert_eq!(measurement.usage, (1, 1));
    println!(
        "delegated first admission instructions={}",
        measurement.instructions
    );
}

#[tokio::test]
#[ignore = "requires prepared build tools and PocketIC 16.x"]
async fn delegated_application_subnet_issuer() {
    let bytes = wasm();
    let pic = support::builder()
        .with_application_subnet()
        .build_async()
        .await;
    time(&pic, 0).await;
    let subnet = pic.topology().await.get_app_subnets()[0];
    let issuer = pic.create_canister_on_subnet(None, None, subnet).await;
    assert_eq!(issuer.as_slice(), fixture::ISSUER);
    let message = [
        b"\x1bic-auth-portable-credential".as_slice(),
        &fixture::hash("claims_digest"),
    ]
    .concat();
    let witness = labeled(
        b"sig".to_vec(),
        labeled(
            hash_bytes(fixture::SEED),
            labeled(hash_bytes(message), leaf(Vec::new())),
        ),
    );
    pic.install_canister(
        issuer,
        include_bytes!("../fixtures/delegated/issuer.wasm").to_vec(),
        witness.digest().to_vec(),
        None,
    )
    .await;
    let root = pic.root_key().await.unwrap();
    let root = PreparedRootKey::from_der(&root).unwrap();
    let receiver = install_receiver(&pic, bytes.clone(), &root.as_der()[37..]).await;
    let frozen =
        PreparedRootKey::from_der(include_bytes!("../fixtures/delegated/root.der")).unwrap();
    let replay = install_receiver(&pic, bytes, &frozen.as_der()[37..]).await;
    time(&pic, 155).await;
    pic.tick().await;
    let certificate = pic
        .query_call(issuer, CanisterId::anonymous(), "certificate", vec![])
        .await
        .unwrap();
    check_delegation(&certificate, subnet);
    let signature = encode_canister_signature(&certificate, &witness, RESPONSE.bytes).unwrap();
    let principal = ic_auth_protocol_types::Principal::from_slice(issuer.as_slice());
    let key =
        ic_platform::certificate::construction::canister_key_der(principal, fixture::SEED).unwrap();
    ic_auth::issuer_signature::verify_issuer_signature(
        fixture::hash("claims_digest"),
        &signature,
        &key,
        &ic_auth::issuer_signature::IssuerSignaturePolicy {
            signing_canister: principal,
            seed_hash: hash_bytes(fixture::SEED),
            ic_root_public_key_raw: &root.as_der()[37..],
            now_ns: at(156),
            max_certificate_age_ns: 20 * fixture::SECOND,
            max_signature_bytes: RESPONSE.bytes,
        },
    )
    .unwrap();
    time(&pic, 156).await;
    if let Some(directory) = std::env::var_os("AUTH_DELEGATED_CAPTURE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::write(directory.join("root.der"), root.as_der()).unwrap();
        std::fs::write(directory.join("issuer-signature.cbor"), &signature).unwrap();
    }
    admit(&pic, receiver, signature).await;
    admit(
        &pic,
        replay,
        include_bytes!("../fixtures/delegated/issuer-signature.cbor").to_vec(),
    )
    .await;
    pic.drop().await;
}
