use super::*;

#[cfg(feature = "session-restore")]
#[test]
fn auth_037_record_identities_are_retired_without_a_fallback_reader() {
    // Even an otherwise valid payload cannot cross the explicit format cut.
    // Empty checkpoints are rejected separately by the host envelope identity.
    let value = [42_u8; 32];
    for (retired, current) in [
        (0x415554485311, SESSION),
        (0x415554485302, REPLAY),
        (0x415554485323, LOGIN),
    ] {
        let old = encode(&value, retired, 2048).unwrap();
        assert_eq!(
            decode::<[u8; 32]>(&old, current, 2048),
            Err(Error::Identity)
        );
        let new = encode(&value, current, 2048).unwrap();
        assert_eq!(decode::<[u8; 32]>(&new, current, 2048), Ok(value));
        assert_eq!(
            decode::<[u8; 32]>(&new, retired, 2048),
            Err(Error::Identity)
        );
    }
}

#[test]
fn framed_counting_matches_emission_and_enforces_exact_capacity() {
    for length in [0, 1, 23, 24, 255, 256, 512, 2048] {
        let value = (vec!["x".repeat(length)], u64::MAX, Some(42_u64));
        for identity in [SESSION, REPLAY, LOGIN] {
            let encoded = encode(&value, identity, 8192).unwrap();
            for maximum in [encoded.len() - 1, encoded.len(), encoded.len() + 1] {
                assert_eq!(
                    encoded_len(&value, maximum),
                    encode(&value, identity, maximum).map(|bytes| bytes.len())
                );
            }
        }
    }
}

#[cfg(feature = "session-restore")]
#[test]
fn framing_and_traversal_refusals_never_publish_a_record() {
    let encoded = encode(&[42_u8; 32], SESSION, 2048).unwrap();
    assert_eq!(
        decode::<[u8; 32]>(&encoded, REPLAY, 2048),
        Err(Error::Identity)
    );
    assert_eq!(
        decode::<[u8; 32]>(&encoded, SESSION, encoded.len() - 1),
        Err(Error::Limit)
    );
    for cut in 0..encoded.len() {
        assert!(decode::<[u8; 32]>(&encoded[..cut], SESSION, 2048).is_err());
    }
    let mut trailing = encoded;
    trailing.push(0);
    assert_eq!(
        decode::<[u8; 32]>(&trailing, SESSION, 2048),
        Err(Error::Trailing)
    );
    assert!(
        encode(&vec![0_u8; 1024], SESSION, 2048).is_err(),
        "traversal allowance is independent of bytes"
    );
    assert!(decode::<[u8; 32]>(b"IC-AUTH-PORTABLE-SESSION\0\xa0", SESSION, 2048).is_err());
}

#[cfg(feature = "session-restore")]
#[test]
fn binary_audiences_are_fixed_length_and_retired_text_records_are_refused() {
    use ic_auth_protocol_types::{AudienceId, CanonicalId};
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Record(#[serde(with = "super::audience")] AudienceId);
    let audience = AudienceId {
        canonical_network_id: CanonicalId::from_bytes([0; 32]),
        fleet_id: CanonicalId::from_bytes([255; 32]),
    };
    let bytes = encode(&Record(audience), SESSION, 2048).unwrap();
    assert_eq!(
        decode::<Record>(&bytes, SESSION, 2048).unwrap(),
        Record(audience)
    );
    assert_eq!(bytes.len(), 11 + 2 * 33);
    for (old, current) in [(0x415554485301, SESSION), (0x415554485303, LOGIN)] {
        let retired = encode(&audience, old, 2048).unwrap();
        assert_eq!(
            decode::<Record>(&retired, current, 2048),
            Err(Error::Identity)
        );
        let relabeled = encode(&audience, current, 2048).unwrap();
        assert!(decode::<Record>(&relabeled, current, 2048).is_err());
    }
    for length in [0, 31, 33, 64] {
        let invalid = (
            serde_bytes::ByteBuf::from(vec![0; length]),
            serde_bytes::ByteArray::new([0; 32]),
        );
        let bytes = encode(&invalid, SESSION, 2048).unwrap();
        assert!(decode::<Record>(&bytes, SESSION, 2048).is_err());
    }
    // Public identifier text serialization is an independent unchanged contract.
    assert_eq!(
        encode(&audience.fleet_id, SESSION, 2048).unwrap(),
        encode(&"ff".repeat(32), SESSION, 2048).unwrap()
    );
}
