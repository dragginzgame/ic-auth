//! Installed synchronous IC transactions, without a native snapshot/undo model.
mod capacity_costs;
mod certified_costs;
mod costs;
mod delegated;
mod enrollment;
mod independent_capacity;
mod issuer_costs;
mod limit_costs;
mod mixed_costs;
mod population_costs;
mod receiver;
mod receiver_support;
mod renewal_costs;

use super::support;
use support::{install, start, wasm};

use ic_auth::{
    claims::decode_claims,
    issuer_signature::{IssuerSignaturePolicy, verify_issuer_signature},
};
use ic_auth_host_fixture::{
    CONTROL, Command, Fault, OBSERVE, Observation, Progress, REQUEST, RESPONSE, Refusal, SEED,
};
use ic_auth_protocol_types::Principal;
use ic_platform::certificate::{
    CertificateDecodeLimits, PreparedRootKey, construction::LookupResult, decode_certificate,
};
use ic_testkit::pocket_ic::{CanisterId, nonblocking::PocketIc};
use sha2::{Digest, Sha256};
use std::time::Duration;

async fn install_with_checkpoint_loss(pic: &PocketIc, wasm: &[u8]) -> CanisterId {
    let id = pic.create_canister().await;
    pic.add_cycles(id, 5_000_000_000_000).await;
    let init = ic_wire::encode(
        &ic_auth_host_fixture::Installation {
            authority: Vec::new(),
            certified_network_key: Vec::new(),
            skip_auth_checkpoint: true,
            receiver_capacity: None,
        },
        ic_wire::Encoding::Payload,
        ic_auth_host_fixture::enrollment::LIMITS,
    )
    .unwrap();
    pic.install_canister(id, wasm.to_vec(), init, None).await;
    id
}

fn frame(method: u64, direction: ic_wire::Direction) -> ic_wire::Encoding {
    ic_wire::Encoding::Frame { method, direction }
}

async fn control(
    pic: &PocketIc,
    id: CanisterId,
    account: CanisterId,
    command: Command,
) -> Result<Result<(), Refusal>, ic_testkit::pocket_ic::RejectResponse> {
    let bytes = ic_wire::encode(
        &command,
        frame(CONTROL, ic_wire::Direction::Request),
        REQUEST,
    )
    .unwrap();
    let reply = pic
        .update_call(id, account, "auth_fixture_control", bytes)
        .await?;
    Ok(ic_wire::decode(
        &reply,
        frame(CONTROL, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap())
}

async fn observe(pic: &PocketIc, id: CanisterId, account: CanisterId) -> Observation {
    let bytes = ic_wire::encode(&(), frame(OBSERVE, ic_wire::Direction::Request), REQUEST).unwrap();
    let reply = pic
        .query_call(id, account, "auth_fixture_observe", bytes)
        .await
        .unwrap();
    let view: Observation = ic_wire::decode(
        &reply,
        frame(OBSERVE, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap();
    let certificate = decode_certificate(
        &view.certificate,
        CertificateDecodeLimits {
            bytes: RESPONSE.bytes,
        },
    )
    .unwrap();
    assert_eq!(
        certificate.unverified().tree.lookup_path([
            b"canister".as_slice(),
            id.as_slice(),
            b"certified_data".as_slice()
        ]),
        LookupResult::Found(view.root.as_slice())
    );
    view
}

fn unchanged(before: &Observation, after: &Observation) {
    assert_eq!(after.progress, before.progress);
    assert_eq!(after.usage, before.usage);
    assert_eq!(after.credit, before.credit);
    assert_eq!(after.claims, before.claims);
    assert_eq!(after.root, before.root);
    assert!(after.signature.is_empty());
}

fn hash(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[tokio::test]
#[ignore = "requires prepared build tools and an explicitly provided PocketIC 16.x server"]
async fn traps_restore_issuer_leaf_and_system_root_then_original_completion_verifies() {
    let wasm = wasm();
    let owned = start().await;
    let pic = &owned;
    let id = install(pic, &wasm).await;
    let account = CanisterId::from_slice(&[9]);
    assert_eq!(
        control(pic, id, account, Command::Prepare).await.unwrap(),
        Ok(())
    );
    let before = observe(pic, id, account).await;
    assert_eq!(before.progress, Progress::Pending);
    assert_eq!((before.usage.0, before.usage.1), (1, 1));
    assert_eq!(before.credit, 3);
    for fault in [
        Fault::Leaf,
        Fault::Root,
        Fault::HostError,
        Fault::Completion,
    ] {
        let refusal = control(pic, id, account, Command::Complete(fault))
            .await
            .unwrap_err();
        assert!(refusal.reject_message.contains("after"), "{refusal:?}");
        unchanged(&before, &observe(pic, id, account).await);
    }
    assert_eq!(
        control(
            pic,
            id,
            CanisterId::from_slice(&[8]),
            Command::Complete(Fault::None)
        )
        .await
        .unwrap(),
        Err(Refusal::Denied)
    );
    unchanged(&before, &observe(pic, id, account).await);
    assert_eq!(
        control(pic, id, account, Command::Complete(Fault::None))
            .await
            .unwrap(),
        Ok(())
    );
    let complete = observe(pic, id, account).await;
    assert_eq!(complete.progress, Progress::Completed);
    assert_eq!(complete.usage, before.usage);
    assert_eq!(complete.credit, before.credit);
    assert_ne!(complete.root, before.root);
    let claims = decode_claims(&complete.claims).unwrap();
    assert_eq!(claims.issuer.as_slice(), id.as_slice());
    assert_eq!(claims.account_subject.as_slice(), account.as_slice());
    let root_der = pic.root_key().await.unwrap();
    let prepared = PreparedRootKey::from_der(&root_der).unwrap();
    let root = &prepared.as_der()[37..];
    let key = ic_platform::certificate::construction::canister_key_der(
        Principal::from_slice(id.as_slice()),
        SEED,
    )
    .unwrap();
    verify_issuer_signature(
        hash(&complete.claims),
        &complete.signature,
        &key,
        &IssuerSignaturePolicy {
            signing_canister: Principal::from_slice(id.as_slice()),
            seed_hash: hash(SEED),
            ic_root_public_key_raw: root,
            now_ns: complete.now_ns,
            max_certificate_age_ns: 1_000_000_000,
            max_signature_bytes: RESPONSE.bytes,
        },
    )
    .unwrap();
    // The leaf/root fault must not run on an exact completed retry.
    assert_eq!(
        control(pic, id, account, Command::Complete(Fault::Root))
            .await
            .unwrap(),
        Ok(())
    );
    let retry = observe(pic, id, account).await;
    assert_eq!(retry.claims, complete.claims);
    assert_eq!(retry.root, complete.root);
    assert_eq!(retry.usage, complete.usage);
    owned.drop().await;
}

#[tokio::test]
#[ignore = "requires built fixture and an explicitly provided PocketIC 16.x server"]
async fn actual_time_revocation_and_upgrade_loss_never_publish_or_recreate_authority() {
    let wasm = wasm();
    let owned = start().await;
    let pic = &owned;
    let account = CanisterId::from_slice(&[9]);
    for revoked in [false, true] {
        let id = install(pic, &wasm).await;
        assert_eq!(
            control(pic, id, account, Command::Prepare).await.unwrap(),
            Ok(())
        );
        let before = observe(pic, id, account).await;
        let expected = if revoked {
            assert_eq!(
                control(pic, id, account, Command::Revoke).await.unwrap(),
                Ok(())
            );
            Refusal::Denied
        } else {
            pic.advance_time(Duration::from_secs(11)).await;
            Refusal::Expired
        };
        assert_eq!(
            control(pic, id, account, Command::Complete(Fault::Root))
                .await
                .unwrap(),
            Err(expected)
        );
        unchanged(&before, &observe(pic, id, account).await);
    }
    let id = install_with_checkpoint_loss(pic, &wasm).await;
    assert_eq!(
        control(pic, id, account, Command::Prepare).await.unwrap(),
        Ok(())
    );
    assert_eq!(
        control(pic, id, account, Command::Complete(Fault::None))
            .await
            .unwrap(),
        Ok(())
    );
    let completed = observe(pic, id, account).await;
    assert_eq!(completed.progress, Progress::Completed);
    pic.stop_canister(id, None).await.unwrap();
    pic.upgrade_canister(id, wasm, vec![], None).await.unwrap();
    pic.start_canister(id, None).await.unwrap();
    let lost = observe(pic, id, account).await;
    assert_eq!(lost.progress, Progress::Closed);
    assert!(lost.claims.is_empty());
    assert!(lost.signature.is_empty());
    assert_ne!(lost.root, completed.root);
    for command in [
        Command::Prepare,
        Command::Complete(Fault::None),
        Command::Revoke,
    ] {
        assert_eq!(
            control(pic, id, account, command).await.unwrap(),
            Err(Refusal::Closed)
        );
    }
    owned.drop().await;
}
