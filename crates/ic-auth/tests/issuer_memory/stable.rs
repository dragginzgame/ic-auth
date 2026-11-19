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
