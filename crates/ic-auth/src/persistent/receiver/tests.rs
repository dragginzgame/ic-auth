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
