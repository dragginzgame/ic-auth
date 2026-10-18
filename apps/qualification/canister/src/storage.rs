//! Fixture-owned checkpoint allocation. No library bootstraps a competing manager.
//! Writes run synchronously in pre_upgrade; IC message rollback owns trap atomicity.
use ic_memory::{DefaultMemoryImpl, MemoryManagerConfig, testing::FixtureCell};
use std::cell::RefCell;

thread_local! {
    static CELL: RefCell<Option<FixtureCell<DefaultMemoryImpl>>> = const { RefCell::new(None) };
}

const KEY: &str = "ic_auth_qualification.checkpoint.v1";
const MAGIC: &[u8; 8] = b"AUTHWIR2";
const MAX_BYTES: usize = 131_072;

pub(crate) fn bootstrap(backing: DefaultMemoryImpl, restore: bool) {
    CELL.with(|state| {
        let mut state = state.borrow_mut();
        assert!(state.is_none(), "checkpoint already initialized");
        let open = if restore {
            FixtureCell::open_bounded
        } else {
            FixtureCell::create_bounded
        };
        *state = Some(
            open(
                backing,
                "ic_auth_qualification",
                "ic_auth_qualification.",
                KEY,
                MemoryManagerConfig::default(),
                MAX_BYTES + MAGIC.len(),
            )
            .expect("checkpoint cell admission"),
        );
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
            .try_set(framed)
            .expect("checkpoint write")
    });
}

pub(crate) fn load() -> Vec<u8> {
    CELL.with(|state| {
        let state = state.borrow();
        state
            .as_ref()
            .expect("checkpoint not initialized")
            .bytes()
            .strip_prefix(MAGIC)
            .expect("missing checkpoint or wrong payload identity")
            .to_vec()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_memory::Memory;
    use std::panic::AssertUnwindSafe;

    #[test]
    fn checkpoint_bounds_and_replacement() {
        let backing = DefaultMemoryImpl::default();
        assert!(std::panic::catch_unwind(load).is_err());
        assert!(
            std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing.clone(), true)))
                .is_err()
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
                .try_set(b"BADMAGIC".to_vec())
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
                .try_set(old.clone())
                .unwrap()
        });
        assert!(std::panic::catch_unwind(load).is_err());
        CELL.with(|state| assert_eq!(state.borrow().as_ref().unwrap().bytes(), old));
    }

    #[test]
    fn previous_raw_checkpoint_format_is_refused_without_clearing() {
        let backing = DefaultMemoryImpl::default();
        bootstrap(backing.clone(), false);
        let memory = CELL.with(|state| state.borrow_mut().take().unwrap().into_memory());
        // The retired fixture wrote AUTHQCP1, a little-endian u64 length, then payload.
        let old = [b"AUTHQCP1".as_slice(), &3_u64.to_le_bytes(), b"old"].concat();
        memory.write(0, &old);
        assert!(std::panic::catch_unwind(AssertUnwindSafe(|| bootstrap(backing, true))).is_err());
        let mut retained = vec![0; old.len()];
        memory.read(0, &mut retained);
        assert_eq!(retained, old);
        CELL.with(|state| assert!(state.borrow().is_none()));
    }
}
