#![cfg(feature = "sessions")]
use ic_auth::{
    canonical::CanonicalAuthError,
    session::{MemorySessionStore, RequiredScopes, SessionError, SessionLimits, SessionStore},
};

#[test]
fn required_scope_admission_preserves_operation_semantics_and_enforces_bounds() {
    assert!(RequiredScopes::NONE.as_slice().is_empty());
    assert_eq!(
        RequiredScopes::new(vec![]).unwrap().as_slice(),
        &[] as &[String]
    );
    let scopes = vec!["write".into(), "read".into(), "write".into()];
    assert_eq!(
        RequiredScopes::new(scopes.clone()).unwrap().as_slice(),
        scopes
    );
    assert!(RequiredScopes::new(vec!["a".repeat(64); 16]).is_ok());
    assert_eq!(
        RequiredScopes::new(vec!["read".into(); 17]),
        Err(SessionError::InvalidPolicyOrRecord)
    );
    assert_eq!(
        RequiredScopes::new(vec![String::new()]),
        Err(SessionError::Canonical(CanonicalAuthError::EmptyScope))
    );
    for scope in ["a::b".into(), "a".repeat(65), "read🚫".into()] {
        assert!(matches!(
            RequiredScopes::new(vec![scope]),
            Err(SessionError::Canonical(
                CanonicalAuthError::InvalidScope { .. }
            ))
        ));
    }
}

#[test]
fn limits_refuse_each_invalid_dimension_before_store_construction() {
    let maximum = [2048, 4096, 256, 8 * 1024 * 1024];
    for index in 0..4 {
        for invalid in [0, maximum[index] + 1] {
            let mut values = maximum;
            values[index] = invalid;
            assert_eq!(
                SessionLimits::new(values[0], values[1], values[2], values[3]),
                Err(SessionError::InvalidPolicyOrRecord)
            );
        }
    }
    for values in [[1; 4], maximum] {
        let limits = SessionLimits::new(values[0], values[1], values[2], values[3]).unwrap();
        assert_eq!(
            [
                limits.max_sessions(),
                limits.max_replays(),
                limits.max_replays_per_caller(),
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
