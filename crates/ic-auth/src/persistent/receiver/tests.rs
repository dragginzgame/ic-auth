use super::*;
use crate::session::SessionLimits;
use ic_auth_protocol_types::{AudienceId, CanonicalId};
use ic_storage::{
    StableKey, StorageConfig, VectorMemory,
    testing::{MeteredMemory, fixture_runtime},
};

type Backing = MeteredMemory<VectorMemory>;
fn config() -> ReceiverConfig {
    ReceiverConfig::new(
        AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        SessionLimits::new(100_000, 200_000, 64, 512 * 1024 * 1024).unwrap(),
        LoginLimits {
            admitted: 100_000,
            unknown: 100,
            maximum_ttl_ns: 10_000_000_000,
            devices_per_account: 8,
        },
        BudgetPolicy {
            burst: 10,
            refill_interval_ns: 1,
        },
        BudgetPolicy {
            burst: 10,
            refill_interval_ns: 1,
        },
    )
    .unwrap()
}
fn memory(backing: Backing) -> crate::persistent::receiver::ReceiverMemory<Backing> {
    let runtime = fixture_runtime(
        backing,
        "auth",
        "auth.",
        &["auth.receiver.v1", "auth.metadata.v1", "auth.callers.v1"],
        StorageConfig::default(),
    )
    .unwrap();
    let open = |key| {
        runtime
            .open_memory(&StableKey::parse(key).unwrap())
            .unwrap()
    };
    crate::persistent::receiver::ReceiverMemory {
        state: crate::persistent::StoreMemory {
            records: open("auth.receiver.v1"),
            metadata: open("auth.metadata.v1"),
        },
        callers: open("auth.callers.v1"),
    }
}
#[test]
fn unknown_denials_budget_clock_reopen_and_bounded_pruning_are_durable() {
    let backing = Backing::new(VectorMemory::default());
    let mut store = ReceiverStore::create(memory(backing.clone()), config(), 0, 0).unwrap();
    let caller = Principal::self_authenticating([9; 32]);
    assert!(matches!(
        store.logout(caller, 1),
        LogoutResult::Unconfirmed { .. }
    ));
    assert_eq!(store.occupancy().unknown_denials, 1);
    assert!(matches!(
        store.logout(caller, 2),
        LogoutResult::Unconfirmed { .. }
    ));
    assert_eq!(store.occupancy().unknown_denials, 1);
    drop(store);
    backing.reset();
    let mut store = ReceiverStore::open(memory(backing.clone()), &config(), 2).unwrap();
    assert!(backing.counts().read_bytes < 65_536);
    assert_eq!(store.occupancy().unknown_denials, 1);
    assert!(matches!(
        store.logout(caller, 1),
        LogoutResult::Unavailable(LogoutUnavailable::Clock)
    ));
    assert_eq!(store.prune(10_000_000_002, 1).unwrap().unknown_denials, 1);
    assert_eq!(store.occupancy(), ReceiverOccupancy::default());
}

#[test]
fn inspection_bounds_fence_scans_and_advances_empty_pages_without_writes() {
    let backing = Backing::new(VectorMemory::default());
    let mut config = config();
    config.logins.unknown = 1024;
    config.logout.burst = 1024;
    let mut store = ReceiverStore::create(memory(backing.clone()), config, 0, 0).unwrap();
    let mut callers: Vec<_> = (0u32..512)
        .map(|n| Principal::self_authenticating(n.to_be_bytes()))
        .collect();
    for caller in &callers {
        let result = store.logout(*caller, 1024);
        assert!(
            matches!(result, LogoutResult::Unconfirmed { .. }),
            "caller {caller}: {result:?}; {:?}",
            store.occupancy()
        );
    }
    callers.sort();
    backing.reset();
    let page = store.inspect_sessions(None, 2, 4, 1025).unwrap();
    assert!(page.sessions.is_empty());
    assert_eq!((page.scanned, page.next_after), (4, Some(callers[3])));
    assert_eq!((page.observed_at_ns, store.observed_at_ns()), (1025, 1024));
    assert!(
        backing.counts().read_bytes < 65_536,
        "bounded scan: {:?}",
        backing.counts()
    );
    assert_eq!((backing.counts().writes, backing.counts().grows), (0, 0));
    let mut after = page.next_after;
    let mut scanned = usize::from(page.scanned);
    while after.is_some() {
        let page = store.inspect_sessions(after, 2, 4, 1025).unwrap();
        assert!(page.sessions.is_empty());
        assert!(page.scanned <= 4);
        if let Some(next) = page.next_after {
            assert!(Some(next) > after);
        }
        after = page.next_after;
        scanned += usize::from(page.scanned);
    }
    assert_eq!(scanned, callers.len());
    for (limit, budget) in [(0, 4), (33, 128), (1, 0), (2, 1), (1, 129)] {
        assert_eq!(
            store.inspect_sessions(None, limit, budget, 1025),
            Err(StoreError::Invalid)
        );
    }
    assert_eq!(
        store.inspect_sessions(None, 1, 1, 0),
        Err(StoreError::Clock)
    );
    assert_eq!(backing.counts().writes, 0);

    let mut writes = Writes::new();
    writes
        .put(keys::login(callers[0]), &CallerRecord::Fence(vec![0]))
        .unwrap();
    store.records.commit(writes);
    backing.reset();
    assert_eq!(
        store.inspect_sessions(None, 1, 1, 1025),
        Err(StoreError::Corrupt)
    );
    assert_eq!(backing.counts().writes, 0);
}
