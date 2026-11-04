use super::*;

#[test]
fn owned_and_borrowed_roles_keep_the_same_grammar_and_allocation() {
    for label in ["user", "project_instance", "role:42-a", ""] {
        let owned = label.to_owned();
        let allocation = owned.as_ptr();
        let parsed = AuthRole::try_from(owned);
        assert_eq!(parsed, label.parse());
        if let Ok(role) = parsed {
            let recovered = role.into_string();
            assert_eq!(recovered, label);
            assert_eq!(recovered.as_ptr(), allocation);
        }
    }
    for label in ["USER", "a b", "é", "a/b", "a\0b"] {
        assert_eq!(
            AuthRole::try_from(label.to_owned()),
            Err(IdentifierError::InvalidRole)
        );
        assert_eq!(label.parse::<AuthRole>(), Err(IdentifierError::InvalidRole));
    }
    assert!("a".repeat(64).parse::<AuthRole>().is_ok());
    assert_eq!(
        "a".repeat(65).parse::<AuthRole>(),
        Err(IdentifierError::InvalidRole)
    );
    assert_eq!(
        AuthRole::try_from("a".repeat(65)),
        Err(IdentifierError::InvalidRole)
    );
    for text in ["a".repeat(64), "a".repeat(65)] {
        let decoded = AuthRole::deserialize(serde::de::value::StringDeserializer::<
            serde::de::value::Error,
        >::new(text.clone()));
        assert_eq!(decoded.ok(), text.parse().ok());
    }
}
