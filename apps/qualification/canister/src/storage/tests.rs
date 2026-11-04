use super::*;
use ic_storage::{RuntimeConstructionError, StableCellPayloadError, adapter::CollectionError};
use std::panic::AssertUnwindSafe;

fn load() -> Vec<u8> {
    with_payload(<[u8]>::to_vec)
}

#[test]
fn previous_memory_manager_refuses_without_clearing_application_backing() {
    let backing = DefaultMemoryImpl::default();
    let old_header: Vec<u8> = include_str!("../../fixtures/memory-063/manager-header.hex")
        .trim()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    // Isolated foreign-format fixture; never an alternate live store path.
    assert_eq!(backing.grow(2), 0);
    backing.write(0, &old_header);
    backing.write(65_536, b"retained application data");
    let bytes = || {
        let mut bytes = vec![0; backing.size() as usize * 65_536];
        backing.read(0, &mut bytes);
        bytes
    };
    let before = bytes();
    let error = fixture_runtime(
        backing.clone(),
        "ic_auth_qualification",
        "ic_auth_qualification.",
        &[KEY],
        StorageConfig::default(),
    )
    .err()
    .expect("old manager must be refused");
    assert!(matches!(
        error.downcast_ref::<RuntimeConstructionError>(),
        Some(RuntimeConstructionError::Directory(_))
    ));
    assert!(
        std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing.clone(), true))).is_err()
    );
    assert_eq!(bytes(), before, "refusal must not reset any retained bytes");
    CELL.with(|state| assert!(state.borrow().is_none()));
}

#[test]
fn checkpoint_bounds_and_replacement() {
    let backing = DefaultMemoryImpl::default();
    assert!(std::panic::catch_unwind(load).is_err());
    assert!(
        std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing.clone(), true))).is_err()
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
    let retained_bytes = || {
        let mut bytes = vec![0; backing.size() as usize * 65_536];
        backing.read(0, &mut bytes);
        bytes
    };
    // Runtime bootstrap owns its directory. Snapshot after obtaining
    // the lease to isolate the collection's fresh-only creation boundary.
    let allocation = checkpoint_allocation(backing.clone());
    let before = retained_bytes();
    assert!(matches!(
        StableCell::create(allocation, Vec::<u8>::new()),
        Err(CollectionError::NotEmpty)
    ));
    assert!(
        retained_bytes() == before,
        "collection creation must preserve occupied backing"
    );
    assert!(
        std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing.clone(), false))).is_err()
    );
    bootstrap(backing, true);
    assert_eq!(load(), b"short replacement");
    CELL.with(|state| {
        state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .set(b"BADMAGIC".to_vec())
            .unwrap()
    });
    assert!(std::panic::catch_unwind(load).is_err());
    save(&[]);
    assert!(load().is_empty());
}

#[test]
fn retired_candid_checkpoint_is_refused_without_clearing() {
    bootstrap(Default::default(), false);
    let old = b"AUTHCELLDIDL\x00\x00".to_vec();
    CELL.with(|state| {
        state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .set(old.clone())
            .unwrap()
    });
    assert!(std::panic::catch_unwind(load).is_err());
    CELL.with(|state| assert_eq!(*state.borrow().as_ref().unwrap().get(), old));
}

#[test]
fn retired_finite_wire_checkpoint_is_refused_without_clearing() {
    bootstrap(Default::default(), false);
    let old = b"AUTHWIR2IWold-finite-frame".to_vec();
    CELL.with(|state| {
        state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .set(old.clone())
            .unwrap()
    });
    assert!(std::panic::catch_unwind(load).is_err());
    CELL.with(|state| assert_eq!(*state.borrow().as_ref().unwrap().get(), old));
}

#[test]
fn retired_cbor_records_checkpoint_is_refused_without_clearing() {
    bootstrap(Default::default(), false);
    let old = b"AUTHNAT1IWold-cbor-records".to_vec();
    CELL.with(|state| {
        state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .set(old.clone())
            .unwrap()
    });
    assert!(std::panic::catch_unwind(load).is_err());
    CELL.with(|state| assert_eq!(*state.borrow().as_ref().unwrap().get(), old));
}

#[test]
fn retired_text_id_checkpoint_is_refused_without_clearing() {
    bootstrap(Default::default(), false);
    let old = b"AUTHNAT2IWold-text-id-records".to_vec();
    CELL.with(|state| {
        state
            .borrow_mut()
            .as_mut()
            .unwrap()
            .set(old.clone())
            .unwrap()
    });
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
        StableCell::<Vec<u8>, _>::load(
            checkpoint_allocation(backing.clone()),
            MAX_BYTES + MAGIC.len()
        ),
        Err(CollectionError::Cell(StableCellPayloadError::NotStableCell))
    ));
    assert!(
        std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing.clone(), true))).is_err()
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
    let assert_claimed = || {
        let error = fixture_runtime(
            backing.clone(),
            "ic_auth_qualification",
            "ic_auth_qualification.",
            &[KEY],
            StorageConfig::default(),
        )
        .err()
        .expect("live backing must remain claimed");
        assert!(matches!(
            error.downcast_ref::<RuntimeConstructionError>(),
            Some(RuntimeConstructionError::BackingAlreadyClaimed)
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
fn checkpoint_checksum_refusal_preserves_backing() {
    let backing = DefaultMemoryImpl::default();
    bootstrap(backing.clone(), false);
    save(b"checksummed checkpoint");
    CELL.with(|state| state.borrow_mut().take());
    let memory = checkpoint_allocation(backing.clone());
    // Isolated corruption injection through a checked lease after the cell
    // releases it. No parallel raw backing path exists in the fixture host.
    memory.write(ic_storage::STABLE_CELL_HEADER_SIZE as u64, b"X");
    drop(memory);
    let before = backing.borrow().clone();
    assert!(matches!(
        StableCell::<Vec<u8>, _>::load(
            checkpoint_allocation(backing.clone()),
            MAX_BYTES + MAGIC.len()
        ),
        Err(CollectionError::Checksum)
    ));
    assert!(
        std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing.clone(), true))).is_err()
    );
    assert_eq!(*backing.borrow(), before);
    CELL.with(|state| assert!(state.borrow().is_none()));
}

#[test]
fn checkpoint_retains_fixture_cell_format_and_rejects_oversized_restore() {
    for size in [MAX_BYTES, MAX_BYTES + 1] {
        let backing = DefaultMemoryImpl::default();
        let open = |restore| {
            let memory = checkpoint_allocation(backing.clone());
            if restore {
                StableCell::load(memory, MAX_BYTES + MAGIC.len() + 1)
            } else {
                StableCell::create(memory, Vec::new())
            }
            .unwrap()
        };
        let payload = vec![42; size];
        let mut expected = [MAGIC.as_slice(), &payload].concat();
        let mut original = open(false);
        original.set(expected.clone()).unwrap();
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
        assert_eq!(open(true).get(), &expected);
    }
}
