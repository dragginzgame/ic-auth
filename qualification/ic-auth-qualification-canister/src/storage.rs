//! Fixture-owned checkpoint allocation through the consolidated Storage runtime.
//! Writes run synchronously in pre_upgrade; IC message rollback owns trap atomicity.
use ic_storage::adapter::StableCell;
use ic_storage::{
    DefaultMemoryImpl, Memory, RuntimeMemory, StableKey, StorageConfig, testing::fixture_runtime,
};
use std::cell::RefCell;

thread_local! {
    static CELL: RefCell<Option<StableCell<Vec<u8>, DefaultMemoryImpl>>> = const { RefCell::new(None) };
}

const KEY: &str = "ic_auth_qualification.checkpoint.v1";
const MAGIC: &[u8; 8] = b"AUTHSLOT";
const MAX_BYTES: usize = 131_072;

fn checkpoint_allocation(backing: DefaultMemoryImpl) -> RuntimeMemory<DefaultMemoryImpl> {
    let key = StableKey::parse(KEY).expect("checkpoint key admission");
    fixture_runtime(
        backing,
        "ic_auth_qualification",
        "ic_auth_qualification.",
        &[KEY],
        StorageConfig::default(),
    )
    .expect("checkpoint runtime admission")
    .open_memory(&key)
    .expect("checkpoint allocation admission")
}

pub(crate) fn bootstrap(backing: DefaultMemoryImpl, restore: bool) {
    CELL.with(|state| {
        let mut state = state.borrow_mut();
        assert!(state.is_none(), "checkpoint already initialized");
        if restore {
            assert!(backing.size() > 0, "missing checkpoint backing");
        }
        let memory = checkpoint_allocation(backing);
        let cell = if restore {
            StableCell::load(memory, MAX_BYTES + MAGIC.len()).expect("checkpoint cell admission")
        } else {
            assert_eq!(memory.size(), 0, "checkpoint allocation already occupied");
            StableCell::create(memory, Vec::new()).expect("checkpoint cell creation")
        };
        *state = Some(cell);
    });
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
