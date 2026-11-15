//! Independent grammar cases for the passive scope boundary.

use ic_auth_protocol_types::{ScopeLabel, ScopeLabelError};

#[test]
fn accepted_labels_preserve_exact_text_and_owned_storage() {
    for text in ["a", "0", "0:9", "a_b-c:d_", "read", "fleet:read_all-1"] {
        let value = text.to_owned();
        let allocation = value.as_ptr();
        let label = ScopeLabel::try_from(value).unwrap();
        assert_eq!(label.as_str(), text);
        assert_eq!(label.to_string(), text);
        assert_eq!(label, text.parse().unwrap());
        let value = label.into_string();
        assert_eq!(value.as_ptr(), allocation);
    }
    assert!("x".repeat(64).parse::<ScopeLabel>().is_ok());
}

#[test]
fn grammar_failures_keep_byte_offsets_and_precedence() {
    use ScopeLabelError as E;
    for (text, error) in [
        ("", E::Empty),
        (":a", E::EmptySegment),
        ("a:", E::EmptySegment),
        ("a::b", E::EmptySegment),
        ("_a", E::InvalidSegmentStart { offset: 0 }),
        ("a:-b", E::InvalidSegmentStart { offset: 2 }),
        ("é", E::InvalidSegmentStart { offset: 0 }),
        ("aé", E::InvalidByte { offset: 1 }),
        ("ab:cD:", E::InvalidByte { offset: 4 }),
    ] {
        assert_eq!(ScopeLabel::validate(text), Err(error));
        assert_eq!(text.parse::<ScopeLabel>(), Err(error));
    }
    assert_eq!(
        "!".repeat(65).parse::<ScopeLabel>(),
        Err(E::TooLong { bytes: 65, max: 64 })
    );
}

#[test]
fn every_ascii_byte_obeys_segment_position_rules() {
    for byte in 0_u8..=127 {
        let character = char::from(byte);
        let start = matches!(byte, b'a'..=b'z' | b'0'..=b'9');
        let continuation = start || matches!(byte, b'_' | b'-');
        assert_eq!(ScopeLabel::validate(&character.to_string()).is_ok(), start);
        assert_eq!(
            ScopeLabel::validate(&format!("a:{character}")).is_ok(),
            start
        );
        assert_eq!(
            ScopeLabel::validate(&format!("a{character}")).is_ok(),
            continuation
        );
    }
}
