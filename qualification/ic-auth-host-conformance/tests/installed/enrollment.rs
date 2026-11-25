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

async fn install_receiver(
    pic: &PocketIc,
    wasm: &[u8],
    authority: CanisterId,
    stable_auth: bool,
) -> CanisterId {
    let id = pic.create_canister().await;
    pic.add_cycles(id, 5_000_000_000_000).await;
    let init = ic_wire::encode(
        &ic_auth_host_fixture::Installation {
            authority: authority.as_slice().to_vec(),
            certified_network_key: Vec::new(),
            stable_auth,
            receiver_capacity: None,
        },
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
    let id = install_receiver(pic, &wasm, authority, false).await;
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

#[tokio::test]
#[ignore = "requires independent signed vectors and PocketIC 16.x"]
async fn stable_auth_upgrade_and_failed_reopen_preserve_session_logout_and_certified_result() {
    let wasm = wasm();
    let vectors = vectors();
    let caller = CanisterId::from_slice(&vectors["registered-caller"]);
    let pic = start().await;
    time(&pic, 0).await;
    let authority = install(&pic, &wasm).await;
    let id = install_receiver(&pic, &wasm, authority, true).await;
    time(&pic, 25).await;
    let enroll = Command::Enroll(Fault::None);
    let admit = credential(&vectors, "registered", false);
    assert_eq!(
        control(&pic, id, caller, &enroll).await.unwrap(),
        Ok(Outcome::Enrolled)
    );
    let Ok(Outcome::Admitted(false, deadline)) = control(&pic, id, caller, &admit).await.unwrap()
    else {
        panic!("initial admission")
    };
    use ic_auth_host_fixture::{Command as Issuance, Fault as IssuerFault, Progress};
    for operation in [Issuance::Prepare, Issuance::Complete(IssuerFault::None)] {
        assert_eq!(
            super::control(&pic, id, caller, operation).await.unwrap(),
            Ok(())
        );
    }
    let issued = super::observe(&pic, id, caller).await;
    assert_eq!(issued.progress, Progress::Completed);
    let before = observe(&pic, id).await;
    let err = pic
        .upgrade_canister(id, wasm.clone(), vec![1], None)
        .await
        .unwrap_err();
    assert!(
        err.reject_message.contains("after stable receiver reopen"),
        "{err:?}"
    );
    assert_eq!(observe(&pic, id).await, before);
    assert_eq!(
        control(&pic, id, caller, &Command::Authorize(false))
            .await
            .unwrap(),
        Ok(Outcome::AuthorizedUser([0xab; 16]))
    );

    pic.upgrade_canister(id, wasm.clone(), vec![], None)
        .await
        .unwrap();
    assert_eq!(
        control(&pic, id, caller, &Command::Authorize(false))
            .await
            .unwrap(),
        Err(Refusal::Closed)
    );
    configure(&pic, authority, CanisterId::anonymous(), Scenario::Refresh)
        .await
        .unwrap();
    assert_eq!(
        control(&pic, id, caller, &enroll).await.unwrap(),
        Ok(Outcome::Enrolled)
    );
    assert_eq!(
        control(&pic, id, caller, &admit).await.unwrap(),
        Ok(Outcome::Admitted(true, deadline))
    );
    assert_eq!(
        control(&pic, id, caller, &Command::Authorize(false))
            .await
            .unwrap(),
        Ok(Outcome::AuthorizedUser([0xab; 16]))
    );
    assert_eq!(observe(&pic, id).await.credit, before.credit);
    assert!(matches!(
        control(&pic, id, caller, &Command::Logout(false))
            .await
            .unwrap(),
        Ok(Outcome::Revoked(_))
    ));
    pic.upgrade_canister(id, wasm, vec![], None).await.unwrap();
    let after = observe(&pic, id).await;
    assert_eq!(after.usage.0, 0);
    assert_eq!(after.usage.1, 1);
    assert_eq!(after.logins, (1, 0));
    configure(
        &pic,
        authority,
        CanisterId::anonymous(),
        Scenario::SecondRefresh,
    )
    .await
    .unwrap();
    assert_eq!(
        control(&pic, id, caller, &enroll).await.unwrap(),
        Ok(Outcome::Enrolled)
    );
    assert_eq!(
        control(&pic, id, caller, &Command::Authorize(false))
            .await
            .unwrap(),
        Err(Refusal::Revoked)
    );
    assert!(control(&pic, id, caller, &admit).await.unwrap().is_err());
    let retained = super::observe(&pic, id, caller).await;
    assert_eq!(retained.progress, Progress::Completed);
    assert_eq!(retained.claims, issued.claims);
    assert_eq!(retained.root, issued.root);
    assert!(!retained.signature.is_empty());
    assert_eq!(retained.usage, issued.usage);
    assert_eq!(retained.credit, issued.credit);
    pic.drop().await;
}

#[tokio::test]
#[ignore = "requires PocketIC 16.x"]
async fn stable_auth_upgrade_skipping_reopen_fences_both_owners() {
    let wasm = wasm();
    let pic = start().await;
    time(&pic, 0).await;
    let authority = install(&pic, &wasm).await;
    let id = install_receiver(&pic, &wasm, authority, true).await;
    // A valid empty module has no upgrade hooks. Old pre-upgrade seals both
    // owners, then the intervening install leaves both allocations untouched.
    let empty_module = b"\0asm\x01\0\0\0".to_vec();
    pic.upgrade_canister(id, empty_module, vec![], None)
        .await
        .unwrap();
    let error = pic
        .upgrade_canister(id, wasm, vec![], None)
        .await
        .unwrap_err();
    assert!(
        error
            .reject_message
            .contains("receiver Some(InvalidCheckpoint), issuer Some(InvalidCheckpoint)"),
        "{error:?}"
    );
    pic.drop().await;
}

#[tokio::test]
#[ignore = "requires PocketIC 16.x and signed browser identity vectors"]
async fn fenced_renewal_recovers_after_upgrade_and_retention() {
    use ic_auth_host_fixture::{
        Command as Issuance, Fault as IssuerFault, Progress, Refusal as IssuerRefusal,
    };
    let pic = start().await;
    let wasm = wasm();
    time(&pic, 0).await;
    let authority = install(&pic, &wasm).await;
    let id = install_receiver(&pic, &wasm, authority, true).await;
    let account = CanisterId::from_slice(&[9]);
    let browser = CanisterId::from_slice(&vectors()["initial-caller"]);
    for (caller, command) in [
        (account, Issuance::Prepare),
        (account, Issuance::Complete(IssuerFault::None)),
        (browser, Issuance::Renew(1)),
    ] {
        assert_eq!(
            super::control(&pic, id, caller, command).await.unwrap(),
            Ok(())
        );
    }
    pic.upgrade_canister(id, wasm.clone(), vec![], None)
        .await
        .unwrap();
    assert_eq!(
        super::control(&pic, id, account, Issuance::Complete(IssuerFault::None))
            .await
            .unwrap(),
        Err(IssuerRefusal::Denied)
    );
    assert!(
        super::control(&pic, id, browser, Issuance::Renew(2))
            .await
            .unwrap()
            .is_err()
    );
    time(&pic, 41).await;
    assert_eq!(
        super::control(&pic, id, account, Issuance::Prune)
            .await
            .unwrap(),
        Ok(())
    );
    pic.upgrade_canister(id, wasm, vec![], None).await.unwrap();
    assert!(
        super::control(&pic, id, browser, Issuance::Renew(1))
            .await
            .unwrap()
            .is_err()
    );
    assert_eq!(
        super::control(&pic, id, browser, Issuance::Renew(2))
            .await
            .unwrap(),
        Ok(())
    );
    assert_eq!(
        super::control(&pic, id, account, Issuance::Complete(IssuerFault::None))
            .await
            .unwrap(),
        Ok(())
    );
    assert_eq!(
        super::observe(&pic, id, account).await.progress,
        Progress::Completed
    );
    pic.drop().await;
}
