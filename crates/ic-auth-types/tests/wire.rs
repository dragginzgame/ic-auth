use candid::{CandidType, decode_one, encode_one};
use ic_auth_types::*;
use serde::Deserialize;

// Independent description of Canic's current Candid boundary: identifiers and
// roles are text, despite being validated value types inside the library.
#[derive(CandidType, Debug, Deserialize, PartialEq)]
enum SourceAudience {
    Fleet(SourceKey),
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceKey {
    canonical_network_id: String,
    fleet_id: String,
}

#[derive(CandidType, Debug, Deserialize, PartialEq)]
struct SourceGrant {
    target: String,
    scopes: Vec<String>,
}

#[test]
fn checked_audience_preserves_existing_candid_bytes() {
    let source = SourceAudience::Fleet(SourceKey {
        canonical_network_id: "ab".repeat(32),
        fleet_id: "cd".repeat(32),
    });
    let bytes = encode_one(&source).unwrap();
    let projected: DelegationAudience = decode_one(&bytes).unwrap();
    assert_eq!(encode_one(&projected).unwrap(), bytes);
    assert_eq!(
        decode_one::<SourceAudience>(&encode_one(projected).unwrap()).unwrap(),
        source
    );
}

#[test]
fn checked_role_preserves_existing_candid_bytes() {
    let source = SourceGrant {
        target: "project_instance".into(),
        scopes: vec!["app:read".into()],
    };
    let bytes = encode_one(&source).unwrap();
    let grant: DelegatedRoleGrant = decode_one(&bytes).unwrap();
    assert_eq!(grant.target.as_str(), "project_instance");
    assert_eq!(encode_one(grant).unwrap(), bytes);
}

#[test]
fn decoding_cannot_bypass_identifier_validation() {
    for bad in ["", "ab", &"AB".repeat(32), &"g0".repeat(32)] {
        assert_eq!(
            bad.parse::<CanonicalId>(),
            Err(IdentifierError::NonCanonicalId)
        );
        assert!(decode_one::<CanonicalId>(&encode_one(bad).unwrap()).is_err());
    }
    for bad in ["", "Root", "app/read", "é", "read "] {
        assert_eq!(bad.parse::<AuthRole>(), Err(IdentifierError::InvalidRole));
        assert!(decode_one::<AuthRole>(&encode_one(bad).unwrap()).is_err());
    }
}

#[test]
fn identities_keep_network_and_audience_distinct() {
    let original = AudienceId {
        canonical_network_id: CanonicalId::from_bytes([1; 32]),
        fleet_id: CanonicalId::from_bytes([2; 32]),
    };
    assert_ne!(
        original,
        AudienceId {
            canonical_network_id: CanonicalId::from_bytes([3; 32]),
            ..original
        }
    );
    assert_ne!(
        original,
        AudienceId {
            fleet_id: CanonicalId::from_bytes([3; 32]),
            ..original
        }
    );
    assert_eq!(
        original,
        decode_one::<AudienceId>(&encode_one(original).unwrap()).unwrap()
    );
}
