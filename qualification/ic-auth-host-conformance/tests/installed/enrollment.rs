//! Authenticated IC replies and atomic single-key enrollment installation.
use super::{
    frame, install,
    receiver::{control, credential, observe, time, vectors},
    start, wasm,
};
use ic_auth_host_fixture::{
    enrollment::{CONFIGURE, Fault, LIMITS, Scenario},
    receiver::{Command, Observation, Outcome, Refusal, TIME_ORIGIN_NS},
};
use ic_testkit::pocket_ic::{CanisterId, RejectResponse, nonblocking::PocketIc};

async fn configure(
    pic: &PocketIc,
    authority: CanisterId,
    caller: CanisterId,
    scenario: Scenario,
) -> Result<(), RejectResponse> {
    let bytes = ic_wire::encode(
        &scenario,
        frame(CONFIGURE, ic_wire::Direction::Request),
        LIMITS,
    )
    .unwrap();
    let reply = pic
        .update_call(authority, caller, "authority_fixture_configure", bytes)
        .await?;
    ic_wire::decode::<()>(
        &reply,
        frame(CONFIGURE, ic_wire::Direction::Response),
        LIMITS,
    )
    .unwrap();
    Ok(())
}
async fn install_receiver(pic: &PocketIc, wasm: &[u8], authority: CanisterId) -> CanisterId {
    let id = pic.create_canister().await;
    pic.add_cycles(id, 5_000_000_000_000).await;
    let init = ic_wire::encode(
        &authority.as_slice().to_vec(),
        ic_wire::Encoding::Payload,
        LIMITS,
    )
    .unwrap();
    pic.install_canister(id, wasm.to_vec(), init, None).await;
    id
}
fn retained(before: &Observation, after: &Observation) {
    assert_eq!(after.enrollment, before.enrollment);
    assert_eq!(after.revoked, before.revoked);
    assert_eq!(after.usage, before.usage);
    assert_eq!(after.logins, before.logins);
    assert_eq!(after.credit, before.credit);
    assert_eq!(after.application_writes, before.application_writes);
    // Request submission is a separate committed message: its clock/challenge
    // remain after a reply refusal or callback trap. Only callback writes undo.
    assert!(after.observed_at_ns >= before.observed_at_ns);
}

#[tokio::test]
#[ignore = "requires prepared build tools, independent vectors and PocketIC 16.x"]
async fn installed_authority_binds_replies_and_revocation_commits_with_receiver_metadata() {
    let wasm = wasm();
    let vectors = vectors();
    let caller = CanisterId::from_slice(&vectors["initial-caller"]);
    let admin = CanisterId::anonymous();
    let owned = start().await;
    let pic = &owned;
    time(pic, 0).await;
    let authority = install(pic, &wasm).await;
    let id = install_receiver(pic, &wasm, authority).await;
    assert_eq!(pic.get_subnet(authority).await, pic.get_subnet(id).await);
    assert_eq!(pic.topology().await.subnet_configs.len(), 1);
    time(pic, 25).await;
    let refresh = Command::Enroll(Fault::None);
    let admit = credential(&vectors, "initial", false);
    assert_eq!(
        control(pic, id, caller, &admit).await.unwrap(),
        Err(Refusal::Closed)
    );
    // Application callers cannot select authority behavior through the control plane.
    assert!(
        configure(pic, authority, caller, Scenario::Revoke)
            .await
            .is_err()
    );
    for scenario in [
        Scenario::WrongReceiver,
        Scenario::WrongChallenge,
        Scenario::Expired,
        Scenario::WrongKey,
        Scenario::Unavailable,
    ] {
        configure(pic, authority, admin, scenario).await.unwrap();
        let before = observe(pic, id).await;
        let expected = if matches!(scenario, Scenario::Unavailable) {
            Refusal::Unavailable
        } else {
            Refusal::Enrollment
        };
        assert_eq!(
            control(pic, id, caller, &refresh).await.unwrap(),
            Err(expected)
        );
        retained(&before, &observe(pic, id).await);
        assert_eq!(
            control(pic, id, caller, &admit).await.unwrap(),
            Err(Refusal::Closed)
        );
    }
    configure(pic, authority, admin, Scenario::Initial)
        .await
        .unwrap();
    assert_eq!(
        control(pic, id, caller, &refresh).await.unwrap(),
        Ok(Outcome::Enrolled)
    );
    let outcome = control(pic, id, caller, &admit).await.unwrap().unwrap();
    let Outcome::Admitted(false, deadline) = outcome else {
        panic!("{outcome:?}")
    };
    configure(pic, authority, admin, Scenario::Refresh)
        .await
        .unwrap();
    assert_eq!(
        control(pic, id, caller, &refresh).await.unwrap(),
        Ok(Outcome::Enrolled)
    );
    assert_eq!(
        observe(pic, id).await.enrollment,
        Some((2, 7, TIME_ORIGIN_NS + 180_000_000_000))
    );
    assert_eq!(
        control(pic, id, caller, &admit).await.unwrap(),
        Ok(Outcome::Admitted(true, deadline))
    );
    configure(pic, authority, admin, Scenario::Regression)
        .await
        .unwrap();
    let before = observe(pic, id).await;
    assert_eq!(
        control(pic, id, caller, &refresh).await.unwrap(),
        Err(Refusal::Enrollment)
    );
    retained(&before, &observe(pic, id).await);

    configure(pic, authority, admin, Scenario::Revoke)
        .await
        .unwrap();
    for fault in [Fault::Metadata, Fault::Entry] {
        let before = observe(pic, id).await;
        let reject = control(pic, id, caller, &Command::Enroll(fault))
            .await
            .unwrap_err();
        assert!(
            reject.reject_message.contains("after enrollment"),
            "{reject:?}"
        );
        retained(&before, &observe(pic, id).await);
        assert_eq!(
            control(pic, id, caller, &Command::Authorize(false))
                .await
                .unwrap(),
            Ok(Outcome::Authorized)
        );
    }
    assert_eq!(
        control(pic, id, caller, &refresh).await.unwrap(),
        Ok(Outcome::Enrolled)
    );
    let revoked = observe(pic, id).await;
    assert_eq!(
        revoked.enrollment,
        Some((3, 8, TIME_ORIGIN_NS + 150_000_000_000))
    );
    assert_eq!(revoked.revoked, Some(true));
    assert!(
        control(pic, id, caller, &Command::Authorize(false))
            .await
            .unwrap()
            .is_err()
    );
    assert!(control(pic, id, caller, &admit).await.unwrap().is_err());
    assert_eq!(
        observe(pic, id).await.application_writes,
        revoked.application_writes
    );
    configure(pic, authority, admin, Scenario::Reenable)
        .await
        .unwrap();
    let before = observe(pic, id).await;
    assert_eq!(
        control(pic, id, caller, &refresh).await.unwrap(),
        Err(Refusal::Enrollment)
    );
    retained(&before, &observe(pic, id).await);
    assert_eq!(
        control(pic, id, caller, &Command::Logout(false))
            .await
            .unwrap(),
        Ok(Outcome::Revoked(TIME_ORIGIN_NS + 100_000_000_000))
    );
    pic.stop_canister(id, None).await.unwrap();
    pic.upgrade_canister(id, wasm, vec![], None).await.unwrap();
    pic.start_canister(id, None).await.unwrap();
    assert_eq!(
        control(pic, id, caller, &refresh).await.unwrap(),
        Err(Refusal::Closed)
    );
    assert!(!observe(pic, id).await.present);
    owned.drop().await;
}
