//! Fixture-owned checkpoint allocation through the consolidated Storage runtime.
//! Writes run synchronously in pre_upgrade; IC message rollback owns trap atomicity.
use ic_storage::adapter::StableCell;
use ic_storage::{
    DefaultMemoryImpl, Memory, MemoryRuntime, StableKey, StorageConfig, testing::fixture_runtime,
};
use std::cell::RefCell;

thread_local! {
    static CELL: RefCell<Option<StableCell<Vec<u8>, DefaultMemoryImpl>>> = const { RefCell::new(None) };
}

const KEY: &str = "ic_auth_qualification.checkpoint.v1";
const ISSUER_KEY: &str = "ic_auth_qualification.issuer.v1";
const ISSUER_META: &str = "ic_auth_qualification.issuer_meta.v1";
const RECEIVER_KEY: &str = "ic_auth_qualification.receiver.v1";
const RECEIVER_CALLERS: &str = "ic_auth_qualification.callers.v1";
const RECEIVER_META: &str = "ic_auth_qualification.receiver_meta.v1";
const MAGIC: &[u8; 8] = b"AUTHSLOT";
const MAX_BYTES: usize = 131_072;

fn runtime(backing: DefaultMemoryImpl) -> MemoryRuntime<DefaultMemoryImpl> {
    fixture_runtime(
        backing,
        "ic_auth_qualification",
        "ic_auth_qualification.",
        &[
            KEY,
            ISSUER_KEY,
            ISSUER_META,
            RECEIVER_KEY,
            RECEIVER_META,
            RECEIVER_CALLERS,
        ],
        StorageConfig::default(),
    )
    .expect("checkpoint runtime admission")
}

#[cfg(test)]
fn checkpoint_allocation(
    backing: DefaultMemoryImpl,
) -> ic_storage::RuntimeMemory<DefaultMemoryImpl> {
    runtime(backing)
        .open_memory(&StableKey::parse(KEY).expect("checkpoint key admission"))
        .expect("checkpoint allocation admission")
}

pub(crate) fn bootstrap(
    backing: DefaultMemoryImpl,
    restore: bool,
) -> (
    ic_auth::persistent::StoreMemory<DefaultMemoryImpl>,
    ic_auth::persistent::receiver::ReceiverMemory<DefaultMemoryImpl>,
) {
    CELL.with(|state| {
        let mut state = state.borrow_mut();
        assert!(state.is_none(), "checkpoint already initialized");
        if restore {
            assert!(backing.size() > 0, "missing checkpoint backing");
        }
        let runtime = runtime(backing);
        let memory = runtime
            .open_memory(&StableKey::parse(KEY).expect("checkpoint key admission"))
            .expect("checkpoint allocation admission");
        let cell = if restore {
            StableCell::load(memory, MAX_BYTES + MAGIC.len()).expect("checkpoint cell admission")
        } else {
            assert_eq!(memory.size(), 0, "checkpoint allocation already occupied");
            StableCell::create(memory, Vec::new()).expect("checkpoint cell creation")
        };
        *state = Some(cell);
        let open = |key| {
            runtime
                .open_memory(&StableKey::parse(key).expect("storage key admission"))
                .expect("allocation admission")
        };
        (
            ic_auth::persistent::StoreMemory {
                records: open(ISSUER_KEY),
                metadata: open(ISSUER_META),
            },
            ic_auth::persistent::receiver::ReceiverMemory {
                state: ic_auth::persistent::StoreMemory {
                    records: open(RECEIVER_KEY),
                    metadata: open(RECEIVER_META),
                },
                callers: open(RECEIVER_CALLERS),
            },
        )
    })
}

pub(crate) fn save(bytes: &[u8]) {
    assert!(bytes.len() <= MAX_BYTES, "checkpoint exceeds fixture limit");
    let mut framed = Vec::with_capacity(MAGIC.len() + bytes.len());
    framed.extend_from_slice(MAGIC);
    framed.extend_from_slice(bytes);
    CELL.with(|state| {
        state
            .borrow_mut()
            .as_mut()
            .expect("checkpoint not initialized")
            .set(framed);
    });
}

pub(crate) fn with_payload<T>(read: impl FnOnce(&[u8]) -> T) -> T {
    CELL.with(|state| {
        let state = state.borrow();
        let payload = state
            .as_ref()
            .expect("checkpoint not initialized")
            .get()
            .strip_prefix(MAGIC)
            .expect("missing checkpoint or wrong payload identity");
        read(payload)
    })
}

#[cfg(test)]
mod tests;
