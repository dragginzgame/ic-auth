//! Real receiver commits, ordinary refusal charging and terminal heap-loss refusal.
use super::{frame, install, start, wasm};
use ic_auth_host_fixture::{
    RESPONSE,
    receiver::{CONTROL, Command, OBSERVE, Observation, Outcome, REQUEST, Refusal, TIME_ORIGIN_NS},
};
use ic_testkit::pocket_ic::{CanisterId, RejectResponse, Time, nonblocking::PocketIc};
use std::{collections::BTreeMap, time::Duration};

pub(super) fn vectors() -> BTreeMap<String, Vec<u8>> {
    let path = std::path::PathBuf::from(
        std::env::var_os("AUTH_HOST_VECTORS").expect("run make qualify-host"),
    );
    let parent = ic_host_fs::Directory::open(path.parent().unwrap(), 0)
        .unwrap()
        .unwrap();
    let file = parent
        .open_file(
            path.file_name().unwrap(),
            ic_host_fs::Access::Read,
            ic_host_fs::FilePolicy {
                forbidden: 0,
                single_link: true,
            },
        )
        .unwrap()
        .unwrap();
    let (bytes, _) =
        ic_host_artifacts::read(file, ic_host_artifacts::Length::AtMost(65_536)).unwrap();
    std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|line| {
            let (name, hex) = line.split_once('\t').unwrap();
            assert_eq!(hex.len() % 2, 0);
            (
                name.into(),
                hex.as_bytes()
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                    .collect(),
            )
        })
        .collect()
}
pub(super) fn credential(
    vectors: &BTreeMap<String, Vec<u8>>,
    name: &str,
    trap_after: bool,
) -> Command {
    Command::Admit {
        claims: vectors[&format!("{name}-claims")].clone(),
        signature: vectors[&format!("{name}-signature")].clone(),
        trap_after,
    }
}
pub(super) async fn control(
    pic: &PocketIc,
    id: CanisterId,
    caller: CanisterId,
    command: &Command,
) -> Result<Result<Outcome, Refusal>, RejectResponse> {
    let bytes = ic_wire::encode(
        command,
        frame(CONTROL, ic_wire::Direction::Request),
        REQUEST,
    )
    .unwrap();
    let reply = pic
        .update_call(id, caller, "receiver_fixture_control", bytes)
        .await?;
    Ok(ic_wire::decode(
        &reply,
        frame(CONTROL, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap())
}
pub(super) async fn observe(pic: &PocketIc, id: CanisterId) -> Observation {
    let bytes = ic_wire::encode(&(), frame(OBSERVE, ic_wire::Direction::Request), REQUEST).unwrap();
    let reply = pic
        .query_call(
            id,
            CanisterId::anonymous(),
            "receiver_fixture_observe",
            bytes,
        )
        .await
        .unwrap();
    ic_wire::decode(
        &reply,
        frame(OBSERVE, ic_wire::Direction::Response),
        RESPONSE,
    )
    .unwrap()
}
pub(super) async fn time(pic: &PocketIc, seconds: u64) {
    pic.set_time(Time::from_nanos_since_unix_epoch(
        TIME_ORIGIN_NS + seconds * 1_000_000_000,
    ))
    .await;
}
async fn trapped(pic: &PocketIc, id: CanisterId, caller: CanisterId, command: &Command) {
    let before = observe(pic, id).await;
    let rejection = control(pic, id, caller, command).await.unwrap_err();
    assert!(
        rejection.reject_message.contains("after receiver"),
        "{rejection:?}"
    );
    assert_eq!(observe(pic, id).await, before);
}

#[tokio::test]
#[ignore = "requires prepared build tools, independent vectors and PocketIC 16.x"]
async fn receiver_transactions_restore_auth_and_application_but_normal_refusals_charge() {
    let vectors = vectors();
    let caller = CanisterId::from_slice(&vectors["initial-caller"]);
    let owned = start().await;
    let pic = &owned;
    time(pic, 0).await;
    let id = install(pic, &wasm()).await;
    time(pic, 25).await;
    let admit = credential(&vectors, "initial", false);
    assert_eq!(
        control(pic, id, caller, &admit).await.unwrap(),
        Err(Refusal::Closed)
    );
    assert_eq!(
        control(
            pic,
            id,
            caller,
            &Command::Enroll(ic_auth_host_fixture::enrollment::Fault::None)
        )
        .await
        .unwrap(),
        Ok(Outcome::Enrolled)
    );
    assert!(
        control(pic, id, CanisterId::from_slice(&[8]), &admit)
            .await
            .unwrap()
            .is_err()
    );
    let mut bad = credential(&vectors, "initial", true);
    if let Command::Admit { signature, .. } = &mut bad {
        signature[0] ^= 1;
    }
    // Ordinary bad proof returns (even with a requested post-success trap), so
    // the attempted-work charge survives the IC message boundary.
    assert_eq!(
        control(pic, id, caller, &bad).await.unwrap(),
        Err(Refusal::Proof)
    );
    let refused = observe(pic, id).await;
    assert_eq!(refused.usage, (0, 0, 0));
    assert_eq!(refused.credit.0, 8);
    trapped(pic, id, caller, &credential(&vectors, "initial", true)).await;
    let outcome = control(pic, id, caller, &admit).await.unwrap().unwrap();
    let Outcome::Admitted(false, deadline) = outcome else {
        panic!("{outcome:?}")
    };
    let admitted = observe(pic, id).await;
    assert_eq!((admitted.usage.0, admitted.usage.1), (1, 1));
    assert_eq!(admitted.logins, (1, 0));
    assert_eq!(admitted.credit.0, 6);
    trapped(pic, id, caller, &Command::Authorize(true)).await;
    assert_eq!(
        control(pic, id, caller, &Command::Authorize(false))
            .await
            .unwrap(),
        Ok(Outcome::Authorized)
    );
    assert_eq!(observe(pic, id).await.application_writes, 1);
    pic.advance_time(Duration::from_secs(11)).await; // Proof expires; original session remains.
    assert_eq!(
        control(pic, id, caller, &admit).await.unwrap(),
        Ok(Outcome::Admitted(true, deadline))
    );
    assert_eq!(observe(pic, id).await.credit.0, admitted.credit.0);
    trapped(pic, id, caller, &Command::Logout(true)).await;
    assert_eq!(
        control(pic, id, caller, &Command::Authorize(false))
            .await
            .unwrap(),
        Ok(Outcome::Authorized)
    );
    assert_eq!(
        control(pic, id, caller, &Command::Logout(false))
            .await
            .unwrap(),
        Ok(Outcome::Revoked(TIME_ORIGIN_NS + 100_000_000_000))
    );
    assert_eq!(
        control(pic, id, caller, &Command::Authorize(false))
            .await
            .unwrap(),
        Err(Refusal::Revoked)
    );
    let revoked = observe(pic, id).await;
    assert_eq!((revoked.usage.0, revoked.usage.1), (0, 1));
    assert_eq!(revoked.logins, (1, 0));
    assert_eq!(revoked.application_writes, 2);
    owned.drop().await;
}

#[tokio::test]
#[ignore = "requires built fixture, independent vectors and PocketIC 16.x"]
async fn receiver_upgrade_loss_cannot_reenroll_or_confirm_unknown_logout() {
    let vectors = vectors();
    let wasm = wasm();
    for known in [true, false] {
        let owned = start().await;
        let pic = &owned;
        time(pic, 0).await;
        let id = install(pic, &wasm).await;
        let intact = install(pic, &wasm).await;
        time(pic, 25).await;
        let name = if known { "initial" } else { "future" };
        let caller = CanisterId::from_slice(&vectors[&format!("{name}-caller")]);
        assert_eq!(
            control(
                pic,
                id,
                caller,
                &Command::Enroll(ic_auth_host_fixture::enrollment::Fault::None)
            )
            .await
            .unwrap(),
            Ok(Outcome::Enrolled)
        );
        assert_eq!(
            control(
                pic,
                intact,
                caller,
                &Command::Enroll(ic_auth_host_fixture::enrollment::Fault::None)
            )
            .await
            .unwrap(),
            Ok(Outcome::Enrolled)
        );
        if !known {
            assert!(matches!(
                control(pic, intact, caller, &Command::Logout(false))
                    .await
                    .unwrap(),
                Ok(Outcome::Unconfirmed(_))
            ));
        }
        if known {
            assert!(matches!(
                control(pic, id, caller, &credential(&vectors, name, false))
                    .await
                    .unwrap(),
                Ok(Outcome::Admitted(false, _))
            ));
        }
        trapped(pic, id, caller, &Command::Logout(true)).await;
        let logout = control(pic, id, caller, &Command::Logout(false))
            .await
            .unwrap()
            .unwrap();
        if known {
            assert_eq!(logout, Outcome::Revoked(TIME_ORIGIN_NS + 100_000_000_000));
        } else {
            assert!(matches!(logout, Outcome::Unconfirmed(_)));
        }
        assert_eq!(
            control(pic, id, caller, &Command::Logout(false))
                .await
                .unwrap(),
            Ok(logout)
        );
        let before = observe(pic, id).await;
        assert_eq!(before.logins, if known { (1, 0) } else { (0, 1) });
        pic.stop_canister(id, None).await.unwrap();
        pic.upgrade_canister(id, wasm.clone(), vec![], None)
            .await
            .unwrap();
        pic.start_canister(id, None).await.unwrap();
        let lost = observe(pic, id).await;
        assert!(!lost.present);
        time(pic, if known { 26 } else { 126 }).await;
        assert!(matches!(
            control(pic, intact, caller, &credential(&vectors, name, false))
                .await
                .unwrap(),
            Ok(Outcome::Admitted(false, _))
        ));
        // The independently signed credential is in its proof window now; a
        // fresh enrollment/timestamp cannot replace lost revocation evidence.
        for command in [
            Command::Enroll(ic_auth_host_fixture::enrollment::Fault::None),
            credential(&vectors, name, false),
            Command::Authorize(false),
            Command::Logout(false),
        ] {
            assert_eq!(
                control(pic, id, caller, &command).await.unwrap(),
                Err(Refusal::Closed)
            );
        }
        assert_eq!(observe(pic, id).await, lost);
        owned.drop().await;
    }
}
