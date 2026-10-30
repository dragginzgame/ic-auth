//! Bounded native IC Wire records; callers own semantic restoration and custody.
//!
//! These identities are storage discriminators, never mounted RPC methods.
//! The old domain-prefixed CBOR records are retired, with no fallback reader.
use ic_wire::{Error, Limits, native};

pub(crate) const SESSION: u64 = 0x415554485311;
pub(crate) const REPLAY: u64 = 0x415554485302;
pub(crate) const LOGIN: u64 = 0x415554485313;

fn limits(maximum: usize) -> Limits {
    Limits {
        bytes: maximum,
        allocation: 65_536,
        nodes: 1024,
        depth: 16,
    }
}

pub(crate) fn encoded_len<T: serde::Serialize>(value: &T, maximum: usize) -> Result<usize, Error> {
    native::encoded_frame_len(value, limits(maximum))
}
pub(crate) fn encode<T: serde::Serialize>(
    value: &T,
    identity: u64,
    maximum: usize,
) -> Result<Vec<u8>, Error> {
    native::encode_request(identity, value, limits(maximum))
}
#[cfg(feature = "session-restore")]
pub(crate) fn decode<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    identity: u64,
    maximum: usize,
) -> Result<T, Error> {
    native::decode_request(identity, bytes, limits(maximum))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

// Private record representation; public identity text and signing encoders stay unchanged.
pub(crate) mod audience {
    use ic_auth_protocol_types::AudienceId;
    use serde::{Serialize, Serializer};
    pub fn serialize<S: Serializer>(value: &AudienceId, serializer: S) -> Result<S::Ok, S::Error> {
        (
            serde_bytes::Bytes::new(value.canonical_network_id.as_bytes()),
            serde_bytes::Bytes::new(value.fleet_id.as_bytes()),
        )
            .serialize(serializer)
    }
    #[cfg(feature = "session-restore")]
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<AudienceId, D::Error> {
        use ic_auth_protocol_types::CanonicalId;
        use serde::Deserialize;
        let (network, fleet): (serde_bytes::ByteArray<32>, serde_bytes::ByteArray<32>) =
            Deserialize::deserialize(deserializer)?;
        Ok(AudienceId {
            canonical_network_id: CanonicalId::from_bytes(network.into_array()),
            fleet_id: CanonicalId::from_bytes(fleet.into_array()),
        })
    }
}
