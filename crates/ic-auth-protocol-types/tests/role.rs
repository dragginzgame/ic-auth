//! Role grammar is deliberately less restrictive than scope grammar.
use ic_auth_protocol_types::{AuthRole, CanonicalId, RoleError};

#[test]
fn roles_preserve_their_distinct_grammar_and_byte_bound() {
    for byte in 0_u8..=127 {
        let text = char::from(byte).to_string();
        let allowed = matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b':' | b'-');
        assert_eq!(text.parse::<AuthRole>().is_ok(), allowed);
    }
    for text in ["_", "::", "-:x", "user", "0:read"] {
        assert_eq!(text.parse::<AuthRole>().unwrap().as_str(), text);
        assert_eq!(AuthRole::try_from(text.to_owned()), text.parse());
    }
    for text in [String::new(), "é".into(), "USER".into(), "x".repeat(65)] {
        assert_eq!(AuthRole::try_from(text), Err(RoleError));
    }
    assert!("x".repeat(64).parse::<AuthRole>().is_ok());
    for bytes in [[0; 32], [255; 32]] {
        assert_eq!(CanonicalId::from_bytes(bytes).as_bytes(), &bytes);
    }
}
