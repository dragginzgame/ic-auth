use super::*;
use ic_auth_protocol_types::{AudienceId, AuthRole, CanonicalId, DelegatedRoleGrant, ScopeLabel};
use ic_storage::{
    StableKey, StorageConfig, VectorMemory,
    testing::{MeteredMemory, fixture_runtime},
};

type Backing = MeteredMemory<VectorMemory>;
fn config() -> IssuerConfig {
    IssuerConfig {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: Principal::from_slice(&[1]),
        max_logins: 100_000,
        max_operations: 200_000,
        max_operations_per_account: 64,
        max_reserved_bytes: 512 * 1024 * 1024,
        max_devices_per_account: 8,
    }
}
fn memory(backing: Backing) -> crate::persistent::StoreMemory<Backing> {
    let runtime = fixture_runtime(
        backing,
        "auth",
        "auth.",
        &["auth.issuer.v1", "auth.metadata.v1"],
        StorageConfig::default(),
    )
    .unwrap();
    crate::persistent::StoreMemory {
        records: runtime
            .open_memory(&StableKey::parse("auth.issuer.v1").unwrap())
            .unwrap(),
        metadata: runtime
            .open_memory(&StableKey::parse("auth.metadata.v1").unwrap())
            .unwrap(),
    }
}
fn authority() -> IssuerAuthority {
    IssuerAuthority {
        generation: 1,
        mechanism: PortableMechanism::Ed25519,
        key_binding_id: [7; 32],
        expires_at_ns: 10_000_000_000,
    }
}
fn policy() -> IssuerPolicy {
    IssuerPolicy {
        login_ttl_ns: 1000,
        proof_ttl_ns: 100,
        receipt_timeout_ns: 100,
        reconciliation_retention_ns: 100,
    }
}
fn grants() -> Vec<DelegatedRoleGrant> {
    vec![DelegatedRoleGrant {
        target: "member".parse::<AuthRole>().unwrap(),
        scopes: vec!["read".parse::<ScopeLabel>().unwrap()],
    }]
}
fn request(epoch: u64, n: u64) -> IssueRequest {
    let mut id = [1; 32];
    id[..8].copy_from_slice(&n.to_be_bytes());
    IssueRequest {
        epoch,
        operation_id: id,
        login_epoch: epoch,
        login_id: id,
        browser_public_key: id,
        grants: grants(),
    }
}
fn account(n: u64) -> Principal {
    Principal::from_slice(&n.to_be_bytes())
}
fn pending(status: IssuerStatus) -> IssuerTicket {
    match status {
        IssuerStatus::Pending(ticket) => ticket,
        _ => panic!("expected pending"),
    }
}
fn finish(store: &mut IssuerStore<Backing>, ticket: &IssuerTicket, now: u64) -> IssuedCredential {
    store.challenge(ticket, authority(), [4; 16], now).unwrap();
    store.approve(ticket, authority(), [5; 32], now).unwrap();
    store
        .complete(ticket, authority(), Some(vec![6; 64]), now)
        .unwrap()
}

#[test]
fn original_intent_results_and_windows_survive_retries_and_reopen() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = IssuerStore::create(memory(backing.clone()), config(), 1).unwrap();
    let req = request(1, 1);
    let ticket = pending(
        store
            .prepare(account(1), req.clone(), policy(), authority(), 1)
            .unwrap(),
    );
    assert_eq!(
        store.claims_for_signing(&ticket, authority(), 1),
        Err(StoreError::Pending)
    );
    store.challenge(&ticket, authority(), [4; 16], 1).unwrap();
    assert_eq!(
        store.challenge(&ticket, authority(), [8; 16], 1),
        Err(StoreError::Conflict)
    );
    let result = finish(&mut store, &ticket, 2);
    assert_eq!(
        store.prepare(account(1), req.clone(), policy(), authority(), 3),
        Ok(IssuerStatus::Completed(result.clone()))
    );
    let claims = crate::portable::decode_claims(&result.claims).unwrap();
    assert_eq!(
        (
            claims.login_started_at_ns,
            claims.login_deadline_ns,
            claims.issued_at_ns
        ),
        (1, 1001, 1)
    );
    let occupancy = store.occupancy();
    drop(store);
    let mut store = IssuerStore::open(memory(backing.clone()), &config(), 4).unwrap();
    assert_eq!(store.occupancy(), occupancy);
    assert_eq!(
        store.retrieve(account(1), 1, req.operation_id),
        Ok(Some(IssuerStatus::Completed(result.clone())))
    );
    store.restart(4).unwrap();
    assert_eq!(
        store.complete(&ticket, authority(), Some(vec![6; 64]), 4),
        Ok(result)
    );
    assert_eq!(
        store.challenge(&ticket, authority(), [4; 16], 4),
        Err(StoreError::Denied)
    );
    assert_eq!(store.retrieve(account(2), 1, req.operation_id), Ok(None));
    assert_eq!(
        store.prepare(account(1), request(1, 2), policy(), authority(), 4),
        Err(StoreError::EpochClosed)
    );
    assert_eq!(
        store.prepare(account(1), request(2, 2), policy(), authority(), 3),
        Err(StoreError::Clock)
    );
}

#[test]
fn account_device_limit_duplicate_key_capacity_and_byte_reservations_are_atomic() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = IssuerStore::create(memory(backing), config(), 1).unwrap();
    for device in 1..=8 {
        store
            .prepare(account(1), request(1, device), policy(), authority(), 1)
            .unwrap();
    }
    let occupancy = store.occupancy();
    assert_eq!(
        store.prepare(account(1), request(1, 9), policy(), authority(), 1),
        Err(StoreError::Capacity)
    );
    let mut duplicate = request(1, 10);
    duplicate.browser_public_key = request(1, 1).browser_public_key;
    assert_eq!(
        store.prepare(account(1), duplicate, policy(), authority(), 1),
        Err(StoreError::Conflict)
    );
    assert_eq!(store.occupancy(), occupancy);
    store
        .prepare(account(2), request(1, 9), policy(), authority(), 1)
        .unwrap();
    let backing = Backing::new(VectorMemory::default());
    let mut small = config();
    small.max_reserved_bytes = 1;
    let mut store = IssuerStore::create(memory(backing), small, 1).unwrap();
    assert_eq!(
        store.prepare(account(1), request(1, 1), policy(), authority(), 1),
        Err(StoreError::Capacity)
    );
    assert_eq!(store.occupancy(), IssuerOccupancy::default());
}

#[test]
fn renew_authenticates_original_browser_and_cannot_extend_or_repeat_sequence() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = IssuerStore::create(memory(backing), config(), 1).unwrap();
    let request = request(1, 1);
    let ticket = pending(
        store
            .prepare(account(1), request.clone(), policy(), authority(), 1)
            .unwrap(),
    );
    finish(&mut store, &ticket, 1);
    let input = RenewRequest {
        account: account(1),
        login_epoch: 1,
        login_id: request.login_id,
        sequence: 1,
        grants: grants(),
    };
    assert_eq!(
        store.renew(account(9), input.clone(), authority(), 2),
        Err(StoreError::Denied)
    );
    let browser = crate::portable::browser_principal(&request.browser_public_key);
    let renewal = pending(store.renew(browser, input.clone(), authority(), 2).unwrap());
    assert_eq!(
        store.renew(browser, input.clone(), authority(), 3),
        Ok(IssuerStatus::Pending(renewal.clone()))
    );
    let result = finish(&mut store, &renewal, 3);
    let claims = crate::portable::decode_claims(&result.claims).unwrap();
    assert_eq!(
        (
            claims.login_started_at_ns,
            claims.login_deadline_ns,
            claims.device_generation
        ),
        (1, 1001, 1)
    );
    assert_eq!(
        store.renew(browser, input.clone(), authority(), 4),
        Ok(IssuerStatus::Completed(result))
    );
    let mut skipped = input.clone();
    skipped.sequence = 3;
    assert_eq!(
        store.renew(browser, skipped, authority(), 4),
        Err(StoreError::Conflict)
    );
    store.revoke(account(1), 1, request.login_id, 4).unwrap();
    assert_eq!(
        store.renew(browser, input, authority(), 4),
        Err(StoreError::Denied)
    );
}

#[test]
fn bounded_pruning_closes_consumed_epochs_and_preserves_referenced_logins() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = IssuerStore::create(memory(backing.clone()), config(), 1).unwrap();
    for n in 1..=40 {
        store
            .prepare(account(n), request(1, n), policy(), authority(), 1)
            .unwrap();
    }
    let pruned = store.prune(202, u32::MAX).unwrap();
    assert_eq!((pruned.operations, pruned.logins), (32, 0));
    assert_eq!(store.epoch(), 2);
    assert_eq!(store.occupancy().logins, 40);
    assert_eq!(
        store.prepare(account(1), request(1, 1), policy(), authority(), 202),
        Err(StoreError::EpochClosed)
    );
    assert_eq!(store.prune(202, 32).unwrap().operations, 8);
    drop(store);
    let mut store = IssuerStore::open(memory(backing), &config(), 1002).unwrap();
    assert_eq!(store.prune(1002, 3).unwrap().logins, 3);
    while store.occupancy().logins != 0 {
        store.prune(1002, 32).unwrap();
    }
    assert_eq!(store.occupancy(), IssuerOccupancy::default());
    let ticket = pending(
        store
            .prepare(account(1), request(2, 1), policy(), authority(), 1002)
            .unwrap(),
    );
    let claims = finish(&mut store, &ticket, 1002).claims;
    assert!(
        crate::portable::decode_claims(&claims)
            .unwrap()
            .device_generation
            > 40
    );
}

#[test]
fn late_callbacks_revocation_and_invalid_completion_do_not_publish_results() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = IssuerStore::create(memory(backing), config(), 1).unwrap();
    let req = request(1, 1);
    let ticket = pending(
        store
            .prepare(account(1), req.clone(), policy(), authority(), 1)
            .unwrap(),
    );
    store.challenge(&ticket, authority(), [4; 16], 1).unwrap();
    store.approve(&ticket, authority(), [5; 32], 1).unwrap();
    let occupancy = store.occupancy();
    assert_eq!(
        store.complete(&ticket, authority(), Some(vec![0; 63]), 1),
        Err(StoreError::Invalid)
    );
    let mut changed = authority();
    changed.generation += 1;
    assert_eq!(
        store.complete(&ticket, changed, Some(vec![0; 64]), 1),
        Err(StoreError::Denied)
    );
    store.revoke(account(1), 1, req.login_id, 1).unwrap();
    assert_eq!(
        store.complete(&ticket, authority(), Some(vec![0; 64]), 1),
        Err(StoreError::Denied)
    );
    assert_eq!(store.occupancy(), occupancy);
    assert!(matches!(
        store.retrieve(account(1), 1, req.operation_id).unwrap(),
        Some(IssuerStatus::Pending(_))
    ));
}

#[test]
fn hundred_thousand_logins_keep_selected_work_bounded() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = IssuerStore::create(memory(backing.clone()), config(), 1).unwrap();
    let mut observations = Vec::new();
    for n in 1..=100_000 {
        store
            .prepare(account(n), request(1, n), policy(), authority(), 1)
            .unwrap();
        if [1, 1000, 10_000, 100_000].contains(&n) {
            backing.reset();
            let status = store
                .retrieve(account(n), 1, request(1, n).operation_id)
                .unwrap();
            let reads = backing.counts();
            let ticket = pending(status.unwrap());
            backing.reset();
            store.challenge(&ticket, authority(), [4; 16], 1).unwrap();
            let mutation = backing.counts();
            assert!(!reads.overflowed && !mutation.overflowed);
            assert!(
                reads.read_bytes < 32_768,
                "point lookup read unrelated history: {reads:?}"
            );
            assert!(
                mutation.read_bytes < 131_072,
                "selected mutation read unrelated history: {mutation:?}"
            );
            assert!(
                mutation.write_bytes < 32_768,
                "selected mutation rewrote unrelated history: {mutation:?}"
            );
            observations.push((n, reads, mutation));
        }
    }
    assert_eq!(store.occupancy().logins, 100_000);
    let before = store.occupancy();
    assert_eq!(
        store.prepare(
            account(100_001),
            request(1, 100_001),
            policy(),
            authority(),
            1
        ),
        Err(StoreError::Capacity)
    );
    assert_eq!(store.occupancy(), before);
    drop(store);
    backing.reset();
    let store = IssuerStore::open(memory(backing.clone()), &config(), 1).unwrap();
    let restored = backing.counts();
    assert!(
        restored.read_bytes < 65_536,
        "open scanned history: {restored:?}"
    );
    assert_eq!(store.occupancy(), before);
    eprintln!("Auth indexed issuer IO observations: {observations:?}; reopen: {restored:?}");
}

#[test]
fn one_account_cannot_fill_the_operation_table_with_one_login() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = IssuerStore::create(memory(backing), config(), 1).unwrap();
    let original = request(1, 1);
    for n in 1u64..=64 {
        let mut request = original.clone();
        request.operation_id[..8].copy_from_slice(&n.to_be_bytes());
        store
            .prepare(account(1), request, policy(), authority(), 1)
            .unwrap();
    }
    let counts = store.occupancy();
    let mut request = original;
    request.operation_id = [9; 32];
    assert_eq!(
        store.prepare(account(1), request, policy(), authority(), 1),
        Err(StoreError::Capacity)
    );
    assert_eq!(store.occupancy(), counts);
    assert_eq!(counts.logins, 1);
    store
        .prepare(
            account(2),
            super::tests::request(1, 1),
            policy(),
            authority(),
            1,
        )
        .unwrap();
}

#[test]
fn malformed_selected_records_refuse_before_publication() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = IssuerStore::create(memory(backing.clone()), config(), 1).unwrap();
    let request = request(1, 1);
    let ticket = pending(
        store
            .prepare(account(1), request.clone(), policy(), authority(), 1)
            .unwrap(),
    );
    finish(&mut store, &ticket, 1);
    let mut row = store
        .operation(account(1), 1, &request.operation_id)
        .unwrap()
        .unwrap();
    row.result.as_mut().unwrap().claims[0] ^= 1;
    let mut writes = Writes::new();
    writes.put(keys::ticket(&ticket), &row).unwrap();
    store.records.commit(writes);
    let counts = store.occupancy();
    assert_eq!(
        store.retrieve(account(1), 1, request.operation_id),
        Err(StoreError::Corrupt)
    );
    assert_eq!(
        store.complete(&ticket, authority(), Some(vec![6; 64]), 1),
        Err(StoreError::Corrupt)
    );
    assert_eq!(store.occupancy(), counts);
}

#[test]
fn completion_reservation_covers_longest_nonce_and_approval_encodings() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = IssuerStore::create(memory(backing), config(), 1).unwrap();
    let request = request(1, 1);
    let ticket = pending(
        store
            .prepare(account(1), request.clone(), policy(), authority(), 1)
            .unwrap(),
    );
    assert_eq!(store.intent(&ticket).unwrap().request, request);
    store.challenge(&ticket, authority(), [255; 16], 1).unwrap();
    store.approve(&ticket, authority(), [255; 32], 1).unwrap();
    let result = store
        .complete(&ticket, authority(), Some(vec![255; 64]), 1)
        .unwrap();
    assert_eq!(
        store.retrieve(account(1), 1, request.operation_id),
        Ok(Some(IssuerStatus::Completed(result)))
    );
}

mod renewal;
