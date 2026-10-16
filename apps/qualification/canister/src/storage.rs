//! Fixture-owned checkpoint allocation. No library bootstraps a competing manager.
//! Writes run synchronously in pre_upgrade; IC message rollback owns trap atomicity.
use ic_memory::{
    GenericAllocationPolicy, MemoryAllocationPool, MemoryAuthority, MemoryRequest, MemoryRuntime,
    RuntimeMemory, SchemaMetadata, SealedDeclarationSnapshot,
    ic_stable_structures::{DefaultMemoryImpl, Memory},
};
use std::cell::RefCell;

thread_local! {
    static RUNTIME: RefCell<Option<MemoryRuntime<DefaultMemoryImpl>>> = const { RefCell::new(None) };
}

const KEY: &str = "ic_auth_qualification.checkpoint.v1";
const MAGIC: &[u8; 8] = b"AUTHQCP1";
const MAX_BYTES: usize = 131_072;
const HEADER: u64 = 16;

pub(crate) fn bootstrap(backing: DefaultMemoryImpl) {
    let declarations = SealedDeclarationSnapshot::new(vec![
        MemoryRequest::new("ic_auth_qualification", KEY, SchemaMetadata::default())
            .expect("checkpoint declaration"),
    ])
    .expect("checkpoint declarations");
    let pool = MemoryAllocationPool::new(
        vec![
            MemoryAuthority::new("ic_auth_qualification", "ic_auth_qualification.")
                .expect("checkpoint authority"),
        ],
        vec![],
    )
    .expect("checkpoint pool");
    RUNTIME.with(|state| {
        let mut state = state.borrow_mut();
        assert!(state.is_none(), "checkpoint runtime already initialized");
        let mut runtime = MemoryRuntime::new(backing).expect("checkpoint runtime");
        runtime
            .bootstrap(&declarations, &pool, &GenericAllocationPolicy)
            .expect("checkpoint bootstrap");
        *state = Some(runtime);
    });
}

fn checkpoint_memory() -> RuntimeMemory<DefaultMemoryImpl> {
    RUNTIME.with(|state| {
        state
            .borrow()
            .as_ref()
            .expect("checkpoint runtime not initialized")
            .open_memory(KEY)
            .expect("checkpoint memory")
    })
}

pub(crate) fn save(bytes: &[u8]) {
    assert!(bytes.len() <= MAX_BYTES, "checkpoint exceeds fixture limit");
    let memory = checkpoint_memory();
    let pages = (HEADER + bytes.len() as u64).div_ceil(65_536);
    if memory.size() < pages {
        memory
            .grow(pages - memory.size())
            .expect("checkpoint capacity");
    }
    memory.write(HEADER, bytes);
    memory.write(8, &(bytes.len() as u64).to_le_bytes());
    memory.write(0, MAGIC);
}

pub(crate) fn load() -> Vec<u8> {
    let memory = checkpoint_memory();
    assert!(memory.size() > 0, "missing checkpoint");
    let mut header = [0; HEADER as usize];
    memory.read(0, &mut header);
    assert_eq!(&header[..8], MAGIC, "checkpoint format mismatch");
    let len = u64::from_le_bytes(header[8..].try_into().expect("checkpoint length"));
    assert!(
        len <= MAX_BYTES as u64 && (HEADER + len).div_ceil(65_536) <= memory.size(),
        "invalid checkpoint length"
    );
    let mut bytes = vec![0; len as usize];
    memory.read(HEADER, &mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_bounds_and_replacement() {
        let backing = DefaultMemoryImpl::default();
        assert!(std::panic::catch_unwind(checkpoint_memory).is_err());
        bootstrap(backing.clone());
        assert!(std::panic::catch_unwind(load).is_err());
        let payload = vec![42; MAX_BYTES];
        save(&payload);
        assert_eq!(load(), payload);
        assert!(std::panic::catch_unwind(|| save(&vec![0; MAX_BYTES + 1])).is_err());
        assert_eq!(load(), payload, "oversize refusal preserves checkpoint");
        save(b"short replacement");
        assert_eq!(load(), b"short replacement");
        assert!(std::panic::catch_unwind(|| bootstrap(Default::default())).is_err());
        assert_eq!(load(), b"short replacement");
        // Reconstruct the one owner over retained backing, as post_upgrade does.
        let slot = RUNTIME.with(|state| state.borrow().as_ref().unwrap().memory_id(KEY).unwrap());
        RUNTIME.with(|state| state.borrow_mut().take());
        bootstrap(backing);
        assert_eq!(
            RUNTIME.with(|state| state.borrow().as_ref().unwrap().memory_id(KEY).unwrap()),
            slot
        );
        assert_eq!(load(), b"short replacement");
        let memory = checkpoint_memory();
        memory.write(0, b"BADMAGIC");
        assert!(std::panic::catch_unwind(load).is_err());
        memory.write(0, MAGIC);
        memory.write(8, &u64::MAX.to_le_bytes());
        assert!(std::panic::catch_unwind(load).is_err());
        save(&[]);
        assert!(load().is_empty());
    }
}
