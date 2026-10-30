use ic_auth::canonical::{CanonicalAuthError, validate_scope_label, validate_scopes};
use ic_auth::certificate_issuance::{GrantValidationError, validate_grants};
use ic_auth_protocol_types::{DelegatedRoleGrant, ScopeLabel, ScopeLabelError};

// Compile-time consumers use the same detailed validator as runtime callers.
const STATIC_SCOPE: Result<(), ScopeLabelError> = ScopeLabel::validate("fleet:read_all-1");
const STATIC_INVALID: Result<(), ScopeLabelError> = ScopeLabel::validate("fleet:_read");

fn grant(role: &str, scopes: &[&str]) -> DelegatedRoleGrant {
    DelegatedRoleGrant {
        target: role.parse().unwrap(),
        scopes: scopes.iter().map(|value| (*value).into()).collect(),
    }
}

#[test]
fn const_and_runtime_scope_checks_preserve_detailed_byte_causes_and_canonical_projection() {
    assert_eq!(STATIC_SCOPE, Ok(()));
    assert_eq!(
        STATIC_INVALID,
        Err(ScopeLabelError::InvalidSegmentStart { offset: 6 })
    );
    let cases = [
        ("", ScopeLabelError::Empty),
        (":read", ScopeLabelError::EmptySegment),
        ("read:", ScopeLabelError::EmptySegment),
        ("read::write", ScopeLabelError::EmptySegment),
        ("Read", ScopeLabelError::InvalidSegmentStart { offset: 0 }),
        (
            "read:_write",
            ScopeLabelError::InvalidSegmentStart { offset: 5 },
        ),
        ("read:é", ScopeLabelError::InvalidSegmentStart { offset: 5 }),
        ("readé", ScopeLabelError::InvalidByte { offset: 4 }),
        ("read/write:", ScopeLabelError::InvalidByte { offset: 4 }),
        (
            "read:Write::",
            ScopeLabelError::InvalidSegmentStart { offset: 5 },
        ),
    ];
    for (scope, expected) in cases {
        assert_eq!(ScopeLabel::validate(scope), Err(expected));
        assert_eq!(
            validate_scope_label(scope),
            Err(CanonicalAuthError::Scope(expected))
        );
    }
    for scope in ["a", "0", "a_b-c", "0:1:a-b_c", "fleet:read_all-1"] {
        assert_eq!(ScopeLabel::validate(scope), Ok(()));
        assert_eq!(validate_scope_label(scope), Ok(()));
    }
    assert_eq!(ScopeLabel::validate(&"a".repeat(64)), Ok(()));
    let oversized = format!("_{}", "a".repeat(64));
    assert_eq!(
        ScopeLabel::validate(&oversized),
        Err(ScopeLabelError::TooLong { bytes: 65, max: 64 })
    );
    assert_eq!(
        validate_scope_label(&oversized),
        Err(CanonicalAuthError::Scope(
            ic_auth_protocol_types::ScopeLabelError::TooLong {
                bytes: oversized.len(),
                max: 64
            }
        ))
    );
}

#[test]
fn ordered_scopes_allow_empty_and_report_grammar_before_order_at_each_label() {
    assert_eq!(validate_scopes(&[]), Ok(()));
    assert_eq!(
        validate_scopes(&["a".into(), "a:b".into(), "b".into()]),
        Ok(())
    );
    for scopes in [["b", "a"], ["a", "a"]] {
        assert_eq!(
            validate_scopes(&scopes.map(String::from)),
            Err(CanonicalAuthError::NonCanonicalScopes)
        );
    }
    assert_eq!(
        validate_scopes(&["b".into(), "Bad".into()]),
        Err(CanonicalAuthError::Scope(
            ic_auth_protocol_types::ScopeLabelError::InvalidSegmentStart { offset: 0 }
        ))
    );
}

#[test]
fn standalone_grants_enforce_explicit_bounds_without_a_certificate() {
    use GrantValidationError as E;
    assert_eq!(validate_grants(&[], 0, 0), Err(E::Empty));
    let grants = [grant("a", &["read", "write"]), grant("b", &["read"])];
    assert_eq!(validate_grants(&grants, 2, 2), Ok(()));
    assert_eq!(
        validate_grants(&grants, 1, 2),
        Err(E::TooMany { found: 2, max: 1 })
    );
    assert_eq!(
        validate_grants(&grants[..1], 0, 2),
        Err(E::TooMany { found: 1, max: 0 })
    );
    for max in [0, 1] {
        assert_eq!(
            validate_grants(&grants, 2, max),
            Err(E::TooManyScopes {
                role: "a".parse().unwrap(),
                found: 2,
                max,
            })
        );
    }
    assert_eq!(
        validate_grants(&[grant("a", &[])], 1, 0),
        Err(E::EmptyScopes {
            role: "a".parse().unwrap()
        })
    );
}

#[test]
fn standalone_grant_refusals_preserve_input_order_for_mixed_invalid_values() {
    use GrantValidationError as E;
    // Global count wins over the first malformed scope.
    let grants = [grant("b", &["Bad"]), grant("a", &[])];
    assert_eq!(
        validate_grants(&grants, 1, 1),
        Err(E::TooMany { found: 2, max: 1 })
    );
    // Earlier scope failure wins over later role-order and empty-scope failures.
    assert_eq!(
        validate_grants(&grants, 2, 1),
        Err(E::Canonical(CanonicalAuthError::Scope(
            ic_auth_protocol_types::ScopeLabelError::InvalidSegmentStart { offset: 0 }
        )))
    );
    for role in ["a", "b"] {
        assert_eq!(
            validate_grants(&[grant("b", &["read"]), grant(role, &[])], 2, 1),
            Err(E::Canonical(CanonicalAuthError::NonCanonicalRoles))
        );
    }
    assert_eq!(
        validate_grants(&[grant("a", &["Bad", "Bad"])], 1, 1),
        Err(E::TooManyScopes {
            role: "a".parse().unwrap(),
            found: 2,
            max: 1
        })
    );
    for scopes in [["write", "read"], ["read", "read"]] {
        assert_eq!(
            validate_grants(&[grant("a", &scopes)], 1, 2),
            Err(E::Canonical(CanonicalAuthError::NonCanonicalScopes))
        );
    }
}
