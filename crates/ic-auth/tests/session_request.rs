//! Request identity vectors and refusal/lifetime contracts, independent of storage.

use ic_auth::session_request::{SessionRequest, SessionRequestError, SessionTtlPolicy};
use ic_auth_protocol_types::ScopeLabelError;

fn request(scopes: &[&str], ttl: Option<u64>) -> SessionRequest {
    SessionRequest::new(scopes.iter().map(|s| (*s).to_owned()).collect(), ttl).unwrap()
}

#[test]
fn retained_reference_hashes_match() {
    let mut checked = 0;
    for line in include_str!("fixtures/session-requests.tsv").lines() {
        if line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('|').collect();
        assert_eq!(fields.len(), 3);
        let scopes: Vec<_> = fields[0].split(',').collect();
        let ttl = (fields[1] != "-").then(|| fields[1].parse().unwrap());
        let value = request(&scopes, ttl);
        let hex: String = value.hash().iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, fields[2]);
        assert_eq!(value.ttl_secs(), ttl);
        checked += 1;
    }
    assert_eq!(checked, 4);
}

#[test]
fn unsigned_order_normalizes_but_identity_never_loses_scope_or_ttl() {
    let value = request(&["write", "read"], None);
    assert_eq!(value, request(&["read", "write"], None));
    assert_eq!(
        value
            .scopes()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
        ["read", "write"]
    );
    for changed in [
        request(&["read"], None),
        request(&["read", "writes"], None),
        request(&["readwrite"], None),
        request(&["read", "write"], Some(30)),
    ] {
        assert_ne!(value.hash(), changed.hash());
    }
}

#[test]
fn request_limits_and_error_order_are_preserved() {
    use SessionRequestError as E;
    for scopes in [vec![], vec!["INVALID".into(); 17]] {
        assert_eq!(SessionRequest::new(scopes, Some(0)), Err(E::InvalidRequest));
    }
    assert_eq!(
        SessionRequest::new(vec!["read".into(), "Bad".into(), "read".into()], Some(0)),
        Err(E::Scope(ScopeLabelError::InvalidSegmentStart { offset: 0 }))
    );
    assert_eq!(
        SessionRequest::new(vec!["read".into(); 2], None),
        Err(E::InvalidRequest)
    );
    assert_eq!(
        SessionRequest::new(vec!["read".into()], Some(0)),
        Err(E::InvalidRequest)
    );
    let scopes: Vec<_> = (0..16)
        .map(|i| format!("s{i:02}{}", "a".repeat(61)))
        .collect();
    assert_eq!(scopes.iter().map(String::len).sum::<usize>(), 1024);
    assert!(SessionRequest::new(scopes, Some(u64::MAX)).is_ok());
    assert_eq!(
        SessionRequest::new(vec!["a".repeat(65)], None),
        Err(E::Scope(ScopeLabelError::TooLong { bytes: 65, max: 64 }))
    );
}

#[test]
fn protected_policy_admits_bounds_before_deadline_arithmetic() {
    use SessionRequestError::InvalidLifetime;
    for (default, maximum) in [(0, 1), (1, 0), (2, 1), (1, 1801), (u64::MAX, u64::MAX)] {
        assert_eq!(
            SessionTtlPolicy::new(default, maximum),
            Err(InvalidLifetime)
        );
    }
    let policy = SessionTtlPolicy::new(30, 1800).unwrap();
    assert_eq!(policy.default_secs(), 30);
    assert_eq!(policy.maximum_secs(), 1800);
    assert_eq!(
        request(&["read"], None).expires_at_ns(7, policy),
        Ok(30_000_000_007)
    );
    assert_eq!(
        request(&["read"], Some(1800)).expires_at_ns(7, policy),
        Ok(1_800_000_000_007)
    );
    for ttl in [1801, u64::MAX] {
        assert_eq!(
            request(&["read"], Some(ttl)).expires_at_ns(0, policy),
            Err(InvalidLifetime)
        );
    }
    let one = request(&["read"], Some(1));
    assert_eq!(
        one.expires_at_ns(u64::MAX - 1_000_000_000, policy),
        Ok(u64::MAX)
    );
    assert_eq!(
        one.expires_at_ns(u64::MAX - 999_999_999, policy),
        Err(InvalidLifetime)
    );
}
