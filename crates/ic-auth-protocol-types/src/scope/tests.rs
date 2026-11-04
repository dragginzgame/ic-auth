use super::*;

#[test]
fn owned_scope_admission_preserves_bytes_allocation_and_exact_grammar() {
    for text in ["read", "fleet:read_all-1", "0:1", "", "a::b", "Read", "é"] {
        let mut supplied = String::with_capacity(128);
        supplied.push_str(text);
        let original = supplied.as_ptr();
        let owned = ScopeLabel::try_from(supplied);
        assert_eq!(owned, text.parse());
        if let Ok(label) = owned {
            assert_eq!(label.as_str(), text);
            let recovered = label.into_string();
            assert_eq!(recovered.as_ptr(), original);
            assert_eq!(recovered, text);
        }
    }
    assert!(ScopeLabel::try_from("a".repeat(64)).is_ok());
    assert_eq!(
        ScopeLabel::try_from("a".repeat(65)),
        Err(ScopeLabelError::TooLong { bytes: 65, max: 64 })
    );
}
