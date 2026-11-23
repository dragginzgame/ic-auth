//! Retained issuer evidence and callback fencing across host upgrades.

use super::{IssuerStatus, approve, authority, config, lookup, prepare};
use ic_auth::{
    issuer_memory::{IssuedProof, StableIssuer},
    stable_upgrade::UpgradeError,
};
use ic_storage::{RuntimeMemory, StableKey, VectorMemory, testing::fixture_runtime};

fn issuer_lease(backing: &VectorMemory) -> RuntimeMemory<VectorMemory> {
    let key = "auth.issuer.v1";
    let runtime =
        fixture_runtime(backing.clone(), "auth", "auth.", &[key], Default::default()).unwrap();

    runtime
        .open_memory(&StableKey::parse(key).unwrap())
        .unwrap()
}

#[test]
fn fresh_login_uses_the_reported_epoch_after_stable_upgrade() {
    use super::{account, policy, request};
    use ic_auth::issuer_memory::IssuerError;

    let backing = VectorMemory::default();
    let mut owner = StableIssuer::new_fresh(issuer_lease(&backing), config(), 0).unwrap();
    let mut request = request();
    request.epoch = owner.as_ref().epoch();
    request.login_epoch = request.epoch;
    owner.seal().unwrap();
    drop(owner);

    let mut owner = StableIssuer::reopen(issuer_lease(&backing), config(), 10).unwrap();
    let issuer = owner.state();
    assert_eq!(
        issuer.prepare_issue(account(), &request, policy(), authority(), 10),
        Err(IssuerError::EpochClosed)
    );
    request.epoch = issuer.epoch();
    request.login_epoch = request.epoch;
    let ticket = issuer
        .prepare_issue(account(), &request, policy(), authority(), 10)
        .unwrap();
    approve(issuer, ticket);
    issuer.dispatch(ticket, authority(), 13).unwrap();
    issuer.complete(ticket, authority(), [17; 64], 14).unwrap();
    assert!(matches!(lookup(issuer, ticket), IssuerStatus::Completed(_)));
}

#[test]
fn stable_upgrade_preserves_uncertain_effect_and_fences_old_callback() {
    let backing = VectorMemory::default();
    let lease = || issuer_lease(&backing);
    let mut owner = StableIssuer::new_fresh(lease(), config(), 0).unwrap();
    let ticket = prepare(owner.state());
    approve(owner.state(), ticket);
    owner.state().dispatch(ticket, authority(), 13).unwrap();
    let credit = owner.state().available_credit();
    owner.seal().unwrap();
    drop(owner);
    let mut changed = config();
    changed.max_logins += 1;
    assert!(matches!(
        StableIssuer::reopen(lease(), changed, 14),
        Err(UpgradeError::Configuration)
    ));
    let mut owner = StableIssuer::reopen(lease(), config(), 14).unwrap();
    assert!(matches!(lookup(owner.state(), ticket), IssuerStatus::Unknown(t) if t == ticket));
    assert_eq!(owner.state().available_credit(), credit);
    assert!(
        owner
            .state()
            .complete(ticket, authority(), [1; 64], 15)
            .is_err()
    );
    drop(owner);
    assert!(matches!(
        StableIssuer::reopen(lease(), config(), 16),
        Err(UpgradeError::InvalidCheckpoint)
    ));
    assert!(matches!(
        StableIssuer::new_fresh(lease(), config(), 16),
        Err(UpgradeError::Allocation)
    ));
}

#[test]
fn stable_completed_result_survives_and_corrupt_checkpoint_refuses_without_reset() {
    let backing = VectorMemory::default();
    let lease = || issuer_lease(&backing);
    let mut owner = StableIssuer::new_fresh(lease(), config(), 0).unwrap();
    let ticket = prepare(owner.state());
    approve(owner.state(), ticket);
    let claims = owner
        .state()
        .dispatch(ticket, authority(), 13)
        .unwrap()
        .claims()
        .to_vec();
    owner
        .state()
        .complete(ticket, authority(), [17; 64], 14)
        .unwrap();
    owner.seal().unwrap();
    drop(owner);
    let mut owner = StableIssuer::reopen(lease(), config(), 15).unwrap();
    let IssuerStatus::Completed(result) = lookup(owner.state(), ticket) else {
        panic!("lost completion")
    };
    assert_eq!(result.claims, claims);
    assert_eq!(result.proof, IssuedProof::Ed25519(&[17; 64]));
    owner.seal().unwrap();
    drop(owner);
    // Damage the Auth digest while leaving the supplier's cell header intact.
    let memory = lease();
    let offset = ic_storage::STABLE_CELL_HEADER_SIZE as u64;
    let mut byte = [0];
    memory.read(offset, &mut byte);
    byte[0] ^= 1;
    memory.write(offset, &byte);
    drop(memory);
    assert!(matches!(
        StableIssuer::reopen(lease(), config(), 16),
        Err(UpgradeError::InvalidCheckpoint)
    ));
    let memory = lease();
    let mut retained = [0];
    memory.read(offset, &mut retained);
    assert_eq!(retained, byte);
}

#[test]
fn registering_an_existing_login_preserves_each_signed_operation_across_upgrade() {
    use super::{account, policy, request};
    use ic_auth::issuer_memory::{IssuerError, IssuerPolicy, RenewRequest};
    use ic_auth_protocol_types::Principal;

    let backing = VectorMemory::default();
    let mut owner = StableIssuer::new_fresh(issuer_lease(&backing), config(), 0).unwrap();
    let issuer = owner.state();
    let original = prepare(issuer);
    approve(issuer, original);
    let input = issuer.dispatch(original, authority(), 13).unwrap();
    assert_eq!(
        ic_auth::claims::decode_claims(input.claims()).unwrap().ext,
        None
    );
    issuer.complete(original, authority(), [1; 64], 13).unwrap();

    let oversized = IssuerPolicy {
        ext: Some(vec![0; 4097]),
        ..policy()
    };
    let credit = issuer.available_credit();
    let usage = issuer.usage();
    assert_eq!(
        issuer.prepare_issue(account(), &request(), oversized, authority(), 20),
        Err(IssuerError::Invalid)
    );
    assert_eq!(issuer.available_credit(), credit);
    assert_eq!(issuer.usage(), usage);

    let registered = IssuerPolicy {
        ext: Some(vec![0xab; 16]),
        ..policy()
    };
    assert_eq!(
        issuer.prepare_issue(account(), &request(), registered.clone(), authority(), 20),
        Err(IssuerError::Conflict)
    );
    let mut next = request();
    next.operation_id = [2; 32];
    let ticket = issuer
        .prepare_issue(account(), &next, registered.clone(), authority(), 20)
        .unwrap();
    issuer
        .record_nonce(ticket, authority(), [0x55; 16], 20)
        .unwrap();
    issuer
        .record_approval(ticket, authority(), [9; 32], 20)
        .unwrap();

    let vector = |name| super::fixture_bytes(include_str!("../fixtures/memory.tsv"), name);
    assert_eq!(
        issuer.dispatch(ticket, authority(), 20).unwrap().claims(),
        vector("registered-claims")
    );
    issuer
        .complete(
            ticket,
            authority(),
            vector("registered-signature").try_into().unwrap(),
            20,
        )
        .unwrap();
    assert_eq!(
        issuer
            .prepare_issue(account(), &request(), policy(), authority(), 20)
            .unwrap(),
        original
    );
    next.operation_id = [4; 32];
    for id in [None, Some(vec![0xcd; 16])] {
        assert_eq!(
            issuer.prepare_issue(
                account(),
                &next,
                IssuerPolicy {
                    ext: id,
                    ..registered
                },
                authority(),
                20
            ),
            Err(IssuerError::Conflict)
        );
    }

    owner.seal().unwrap();
    drop(owner);
    let mut owner = StableIssuer::reopen(issuer_lease(&backing), config(), 21).unwrap();
    let issuer = owner.state();
    let browser = Principal::from_slice(&vector("registered-caller"));
    let renew = RenewRequest {
        account: account(),
        login_epoch: 1,
        login_id: next.login_id,
        sequence: 1,
        grants: next.grants,
    };
    let renewed = issuer
        .prepare_renewal(browser, &renew, authority(), 21)
        .unwrap();
    issuer
        .record_nonce(renewed, authority(), [0x56; 16], 21)
        .unwrap();
    issuer
        .record_approval(renewed, authority(), [9; 32], 21)
        .unwrap();
    assert_eq!(
        issuer.dispatch(renewed, authority(), 21).unwrap().claims(),
        vector("registered-renewal-claims")
    );
    issuer
        .complete(
            renewed,
            authority(),
            vector("registered-renewal-signature").try_into().unwrap(),
            21,
        )
        .unwrap();
    assert_eq!(
        issuer
            .prepare_renewal(browser, &renew, authority(), 22)
            .unwrap(),
        renewed
    );
}
