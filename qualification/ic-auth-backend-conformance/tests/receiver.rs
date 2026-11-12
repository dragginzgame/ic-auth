#![cfg(feature = "persistent")]
use ic_auth::persistent::{
    StoreMemory,
    receiver::{ReceiverConfig, ReceiverMemory, ReceiverStore},
};
use ic_auth_backend_conformance::receiver_tests::{
    CommitRejected, ReceiverHost, assert_receiver_contract,
};
use ic_storage::{StableKey, StorageConfig, VectorMemory, testing::fixture_runtime};

struct Host<const BROKEN: u8> {
    backing: VectorMemory,
    store: Option<ReceiverStore<VectorMemory>>,
    config: ReceiverConfig,
    fail: bool,
    failures: u8,
}
fn memory(backing: VectorMemory) -> ReceiverMemory<VectorMemory> {
    let runtime = fixture_runtime(
        backing,
        "receiver",
        "receiver.",
        &[
            "receiver.rows.v1",
            "receiver.metadata.v1",
            "receiver.callers.v1",
        ],
        StorageConfig::default(),
    )
    .unwrap();
    let open = |key| {
        runtime
            .open_memory(&StableKey::parse(key).unwrap())
            .unwrap()
    };
    ReceiverMemory {
        state: StoreMemory {
            records: open("receiver.rows.v1"),
            metadata: open("receiver.metadata.v1"),
        },
        callers: open("receiver.callers.v1"),
    }
}
impl<const BROKEN: u8> ReceiverHost for Host<BROKEN> {
    type Memory = VectorMemory;
    fn fresh(config: ReceiverConfig, generation: u64, now: u64) -> Self {
        let backing = VectorMemory::default();
        let store = ReceiverStore::create(memory(backing.clone()), config.clone(), generation, now)
            .unwrap();
        Self {
            backing,
            store: Some(store),
            config,
            fail: false,
            failures: 0,
        }
    }
    fn store(&mut self) -> &mut ReceiverStore<VectorMemory> {
        self.store.as_mut().unwrap()
    }
    fn reopen(mut self, now: u64) -> Self {
        self.store.take();
        self.store =
            Some(ReceiverStore::open(memory(self.backing.clone()), &self.config, now).unwrap());
        self
    }
    fn fail_next_commit(&mut self) {
        self.fail = true;
    }
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut ReceiverStore<VectorMemory>) -> T,
    ) -> Result<T, CommitRejected> {
        let before = self.backing.borrow().clone();
        let now = self.store().observed_at_ns();
        let result = operation(self.store());
        if std::mem::take(&mut self.fail) {
            self.failures += 1;
            let broken = self.failures == BROKEN;
            let now = if broken {
                self.store().observed_at_ns()
            } else {
                now
            };
            self.store.take();
            if !broken {
                *self.backing.borrow_mut() = before;
            }
            self.store =
                Some(ReceiverStore::open(memory(self.backing.clone()), &self.config, now).unwrap());
            Err(CommitRejected)
        } else {
            Ok(result)
        }
    }
}
#[test]
fn checked_backing_complete_owner_conforms() {
    assert_receiver_contract::<Host<0>>();
}
#[test]
#[should_panic(expected = "failed admission leaked records")]
fn committed_failure_is_detected() {
    assert_receiver_contract::<Host<1>>();
}

#[test]
#[should_panic(expected = "failed logout leaked records")]
fn logout_commit_failure_is_detected() {
    ic_auth_backend_conformance::receiver_tests::assert_receiver_lifecycle::<Host<2>>();
}
#[test]
#[should_panic(expected = "failed generation leaked authority")]
fn generation_commit_failure_is_detected() {
    ic_auth_backend_conformance::receiver_tests::assert_receiver_lifecycle::<Host<3>>();
}
#[test]
#[should_panic(expected = "failed renewal leaked records")]
fn renewal_commit_failure_is_detected() {
    ic_auth_backend_conformance::receiver_tests::assert_receiver_ninety_day_renewal::<Host<1>>();
}
#[test]
#[should_panic(expected = "failed prune leaked records")]
fn prune_commit_failure_is_detected() {
    ic_auth_backend_conformance::receiver_tests::assert_receiver_ninety_day_renewal::<Host<2>>();
}
