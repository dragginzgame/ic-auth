#![cfg(feature = "sessions")]
use ic_auth::{
    canonical::CanonicalAuthError,
    session::{
        MemorySessionStore, RequiredScopes, SessionError, SessionLimits, SessionOccupancy,
        SessionStore,
    },
};

#[test]
fn required_scope_admission_preserves_operation_semantics_and_enforces_bounds() {
    assert!(RequiredScopes::NONE.as_slice().is_empty());
    assert_eq!(
        RequiredScopes::new(vec![]).unwrap().as_slice(),
        &[] as &[ic_auth_protocol_types::ScopeLabel]
    );
    let scopes = vec!["write".into(), "read".into(), "write".into()];
    assert_eq!(
        RequiredScopes::new(scopes.clone())
            .unwrap()
            .as_slice()
            .iter()
            .map(|scope| scope.as_str())
            .collect::<Vec<_>>(),
        scopes
    );
    assert!(RequiredScopes::new(vec!["a".repeat(64); 16]).is_ok());
    assert_eq!(
        RequiredScopes::new(vec!["read".into(); 17]),
        Err(SessionError::InvalidPolicyOrRecord)
    );
    assert_eq!(
        RequiredScopes::new(vec![String::new()]),
        Err(SessionError::Canonical(CanonicalAuthError::Scope(
            ic_auth_protocol_types::ScopeLabelError::Empty
        )))
    );
    for scope in ["a::b".into(), "a".repeat(65), "read🚫".into()] {
        assert!(matches!(
            RequiredScopes::new(vec![scope]),
            Err(SessionError::Canonical(CanonicalAuthError::Scope(_)))
        ));
    }
}

#[test]
fn limits_refuse_each_invalid_dimension_before_store_construction() {
    let selected = [100_000, 200_000, 1024, 512 * 1024 * 1024];
    for index in 0..4 {
        let mut values = selected;
        values[index] = 0;
        assert_eq!(
            SessionLimits::new(values[0], values[1], values[2], values[3]),
            Err(SessionError::InvalidPolicyOrRecord)
        );
    }
    for values in [[1; 4], selected, [usize::MAX; 4]] {
        let limits = SessionLimits::new(values[0], values[1], values[2], values[3]).unwrap();
        assert_eq!(
            [
                limits.max_sessions(),
                limits.max_replays(),
                limits.max_replays_per_account(),
                limits.max_encoded_bytes()
            ],
            values
        );
        let mut store = MemorySessionStore::new(u64::MAX, limits);
        assert_eq!(store.read(|reader| Ok(reader.limits())).unwrap(), limits);
        assert_eq!(store.generation(), u64::MAX);
        assert_eq!(store.encoded_bytes(), 0);
    }
}

#[test]
fn host_selected_capacity_refuses_each_exhausted_dimension_and_preserves_replacement() {
    let limits = SessionLimits::new(100_000, 200_000, 1024, 512 * 1024 * 1024).unwrap();
    let available = SessionOccupancy {
        sessions: 99_999,
        replays: 199_999,
        replays_for_account: 1023,
    };
    assert_eq!(limits.check_capacity(available, false), Ok(()));
    let sessions_full = SessionOccupancy {
        sessions: 100_000,
        ..available
    };
    assert_eq!(
        limits.check_capacity(sessions_full, false),
        Err(SessionError::Capacity)
    );
    assert_eq!(limits.check_capacity(sessions_full, true), Ok(()));
    for full in [
        SessionOccupancy {
            replays: 200_000,
            ..available
        },
        SessionOccupancy {
            replays_for_account: 1024,
            ..available
        },
    ] {
        for replacing in [false, true] {
            assert_eq!(
                limits.check_capacity(full, replacing),
                Err(SessionError::Capacity)
            );
        }
    }
    let maximum = SessionLimits::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX).unwrap();
    assert_eq!(
        maximum.check_capacity(
            SessionOccupancy {
                sessions: usize::MAX - 1,
                replays: usize::MAX - 1,
                replays_for_account: usize::MAX - 1
            },
            false
        ),
        Ok(())
    );
    assert_eq!(
        maximum.check_capacity(
            SessionOccupancy {
                sessions: usize::MAX,
                replays: usize::MAX,
                replays_for_account: usize::MAX
            },
            true
        ),
        Err(SessionError::Capacity)
    );
}
