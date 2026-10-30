//! Fixture-owned checkpoint allocation. No library bootstraps a competing manager.
//! Writes run synchronously in pre_upgrade; IC message rollback owns trap atomicity.
use ic_memory::{
    DefaultMemoryImpl, Memory, MemoryManagerConfig, RuntimeMemory, stable_cell_payload_length,
    testing::fixture_runtime,
};
use ic_storage::adapter::StableCell;
use std::cell::RefCell;

thread_local! {
    static CELL: RefCell<Option<StableCell<Vec<u8>, DefaultMemoryImpl>>> = const { RefCell::new(None) };
}

const KEY: &str = "ic_auth_qualification.checkpoint.v1";
const MAGIC: &[u8; 8] = b"AUTHNAT3";
const MAX_BYTES: usize = 131_072;

fn checkpoint_allocation(backing: DefaultMemoryImpl) -> RuntimeMemory<DefaultMemoryImpl> {
    fixture_runtime(
        backing,
        "ic_auth_qualification",
        "ic_auth_qualification.",
        &[KEY],
        MemoryManagerConfig::default(),
    )
    .expect("checkpoint runtime admission")
    .open_memory(KEY)
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
            // Admit the retained size before the collection allocates/decodes it.
            stable_cell_payload_length(&memory, MAX_BYTES + MAGIC.len())
                .expect("checkpoint payload admission");
            StableCell::load(memory).expect("checkpoint cell admission")
        } else {
            assert_eq!(memory.size(), 0, "checkpoint allocation already occupied");
            StableCell::new(memory, Vec::new())
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

pub(crate) fn load() -> Vec<u8> {
    CELL.with(|state| {
        let state = state.borrow();
        state
            .as_ref()
            .expect("checkpoint not initialized")
            .get()
            .strip_prefix(MAGIC)
            .expect("missing checkpoint or wrong payload identity")
            .to_vec()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_memory::{
        RuntimeConstructionError, StableCellPayloadError,
        testing::{FixtureCell, FixtureCellError, FixtureRuntimeError},
    };
    use std::panic::AssertUnwindSafe;

    #[test]
    fn previous_memory_ledger_refuses_without_clearing_application_backing() {
        use ic_platform::stable_structures::{
            Cell,
            memory_manager::{MemoryId, MemoryManager},
        };

        let backing = DefaultMemoryImpl::default();
        let old_ledger: Vec<u8> =
            include_str!("../fixtures/memory-055/ledger_commit_store_single.cbor.hex")
                .trim()
                .as_bytes()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                .collect();
        // Isolated refusal fixture only: no live governed runtime or aliases.
        // Slot 100 is the application allocation recorded by the frozen ledger.
        {
            let manager = MemoryManager::init(backing.clone());
            Cell::new(manager.get(MemoryId::new(0)), old_ledger);
            Cell::new(
                manager.get(MemoryId::new(100)),
                b"retained application data".to_vec(),
            );
        }
        let bytes = || {
            let mut bytes = vec![0; backing.size() as usize * 65_536];
            backing.read(0, &mut bytes);
            bytes
        };
        let before = bytes();
        assert!(matches!(
            fixture_runtime(
                backing.clone(),
                "ic_auth_qualification",
                "ic_auth_qualification.",
                &[KEY],
                MemoryManagerConfig::default(),
            ),
            Err(FixtureRuntimeError::Bootstrap(error))
                if matches!(*error, ic_memory::RuntimeBootstrapError::StableCellLedger(
                    ic_memory::StableCellLedgerError::Record(_)
                ))
        ));
        assert!(
            std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing.clone(), true)))
                .is_err()
        );
        assert_eq!(bytes(), before, "refusal must not reset any retained bytes");
        CELL.with(|state| assert!(state.borrow().is_none()));
    }

    #[test]
    fn checkpoint_bounds_and_replacement() {
        let backing = DefaultMemoryImpl::default();
        assert!(std::panic::catch_unwind(load).is_err());
        assert!(
            std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing.clone(), true)))
                .is_err()
        );
        assert_eq!(
            backing.size(),
            0,
            "missing restore must not initialize backing"
        );
        bootstrap(backing.clone(), false);
        assert!(std::panic::catch_unwind(load).is_err());
        let payload = vec![42; MAX_BYTES];
        save(&payload);
        assert_eq!(load(), payload);
        assert!(std::panic::catch_unwind(|| save(&vec![0; MAX_BYTES + 1])).is_err());
        assert_eq!(load(), payload, "oversize refusal preserves checkpoint");
        save(b"short replacement");
        assert!(std::panic::catch_unwind(|| bootstrap(Default::default(), false)).is_err());
        assert_eq!(load(), b"short replacement");
        CELL.with(|state| state.borrow_mut().take());
        assert!(
            std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing.clone(), false)))
                .is_err()
        );
        bootstrap(backing, true);
        assert_eq!(load(), b"short replacement");
        CELL.with(|state| {
            state
                .borrow_mut()
                .as_mut()
                .unwrap()
                .set(b"BADMAGIC".to_vec())
        });
        assert!(std::panic::catch_unwind(load).is_err());
        save(&[]);
        assert!(load().is_empty());
    }

    #[test]
    fn retired_candid_checkpoint_is_refused_without_clearing() {
        bootstrap(Default::default(), false);
        let old = b"AUTHCELLDIDL\x00\x00".to_vec();
        CELL.with(|state| state.borrow_mut().as_mut().unwrap().set(old.clone()));
        assert!(std::panic::catch_unwind(load).is_err());
        CELL.with(|state| assert_eq!(*state.borrow().as_ref().unwrap().get(), old));
    }

    #[test]
    fn retired_finite_wire_checkpoint_is_refused_without_clearing() {
        bootstrap(Default::default(), false);
        let old = b"AUTHWIR2IWold-finite-frame".to_vec();
        CELL.with(|state| state.borrow_mut().as_mut().unwrap().set(old.clone()));
        assert!(std::panic::catch_unwind(load).is_err());
        CELL.with(|state| assert_eq!(*state.borrow().as_ref().unwrap().get(), old));
    }

    #[test]
    fn retired_cbor_records_checkpoint_is_refused_without_clearing() {
        bootstrap(Default::default(), false);
        let old = b"AUTHNAT1IWold-cbor-records".to_vec();
        CELL.with(|state| state.borrow_mut().as_mut().unwrap().set(old.clone()));
        assert!(std::panic::catch_unwind(load).is_err());
        CELL.with(|state| assert_eq!(*state.borrow().as_ref().unwrap().get(), old));
    }

    #[test]
    fn retired_text_id_checkpoint_is_refused_without_clearing() {
        bootstrap(Default::default(), false);
        let old = b"AUTHNAT2IWold-text-id-records".to_vec();
        CELL.with(|state| state.borrow_mut().as_mut().unwrap().set(old.clone()));
        assert!(std::panic::catch_unwind(load).is_err());
        CELL.with(|state| assert_eq!(*state.borrow().as_ref().unwrap().get(), old));
    }

    #[test]
    fn previous_raw_checkpoint_format_is_refused_without_clearing() {
        let backing = DefaultMemoryImpl::default();
        bootstrap(backing.clone(), false);
        CELL.with(|state| state.borrow_mut().take());
        let memory = checkpoint_allocation(backing.clone());
        // The retired fixture wrote AUTHQCP1, a little-endian u64 length, then payload.
        let old = [b"AUTHQCP1".as_slice(), &3_u64.to_le_bytes(), b"old"].concat();
        memory.write(0, &old);
        drop(memory);
        assert!(matches!(
            StableCell::<Vec<u8>, _>::load(checkpoint_allocation(backing.clone())),
            Err(StableCellPayloadError::NotStableCell)
        ));
        assert!(
            std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing.clone(), true)))
                .is_err()
        );
        let memory = checkpoint_allocation(backing);
        let mut retained = vec![0; old.len()];
        memory.read(0, &mut retained);
        assert_eq!(retained, old);
        CELL.with(|state| assert!(state.borrow().is_none()));
    }

    #[test]
    fn checkpoint_reopen_requires_release_of_cell_and_detached_handle() {
        let backing = DefaultMemoryImpl::default();
        bootstrap(backing.clone(), false);
        save(b"retained checkpoint");
        let open = || {
            FixtureCell::open_bounded(
                backing.clone(),
                "ic_auth_qualification",
                "ic_auth_qualification.",
                KEY,
                MemoryManagerConfig::default(),
                MAX_BYTES + MAGIC.len(),
            )
        };
        let assert_claimed = || {
            assert!(matches!(
                open(),
                Err(FixtureCellError::Setup(FixtureRuntimeError::Construction(
                    RuntimeConstructionError::BackingAlreadyClaimed
                )))
            ));
        };
        assert_claimed();
        assert_eq!(load(), b"retained checkpoint");
        CELL.with(|state| state.borrow_mut().take());
        let memory = checkpoint_allocation(backing.clone());
        assert_claimed();
        drop(memory);
        bootstrap(backing.clone(), true);
        assert_eq!(load(), b"retained checkpoint");
    }

    #[test]
    fn checkpoint_retains_fixture_cell_format_and_rejects_oversized_restore() {
        for size in [MAX_BYTES, MAX_BYTES + 1] {
            let backing = DefaultMemoryImpl::default();
            let open = |restore| {
                let constructor = if restore {
                    FixtureCell::open_bounded
                } else {
                    FixtureCell::create_bounded
                };
                constructor(
                    backing.clone(),
                    "ic_auth_qualification",
                    "ic_auth_qualification.",
                    KEY,
                    MemoryManagerConfig::default(),
                    MAX_BYTES + MAGIC.len() + 1,
                )
                .unwrap()
            };
            let payload = vec![42; size];
            let mut expected = [MAGIC.as_slice(), &payload].concat();
            let mut original = open(false);
            original.try_set(expected.clone()).unwrap();
            drop(original);
            if size > MAX_BYTES {
                assert!(
                    std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing.clone(), true)))
                        .is_err()
                );
                CELL.with(|state| assert!(state.borrow().is_none()));
            } else {
                bootstrap(backing.clone(), true);
                assert_eq!(load(), payload);
                save(b"storage replacement");
                expected = [MAGIC.as_slice(), b"storage replacement"].concat();
                CELL.with(|state| state.borrow_mut().take());
            }
            assert_eq!(open(true).bytes(), expected);
        }
    }
}
