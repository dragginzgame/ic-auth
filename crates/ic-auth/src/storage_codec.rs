//! Bounded native IC Wire records; callers own semantic restoration and custody.
//!
//! These identities are storage discriminators, never mounted RPC methods.
//! The old domain-prefixed CBOR records are retired, with no fallback reader.
use ic_wire::{Error, Limits, native};

pub(crate) const SESSION: u64 = 0x415554485301;
pub(crate) const REPLAY: u64 = 0x415554485302;
pub(crate) const LOGIN: u64 = 0x415554485303;

fn limits(maximum: usize) -> Limits {
    Limits {
        bytes: maximum,
        allocation: 65_536,
        nodes: 1024,
        depth: 16,
    }
}

fn failure<E>(error: Error, invalid: E, capacity: E) -> E {
    match error {
        Error::Limit | Error::Allocation => capacity,
        _ => invalid,
    }
}

pub(crate) fn encoded_len<T: serde::Serialize, E>(
    value: &T,
    maximum: usize,
    invalid: E,
    capacity: E,
) -> Result<usize, E> {
    native::encoded_frame_len(value, limits(maximum))
        .map_err(|error| failure(error, invalid, capacity))
}

pub(crate) fn encode<T: serde::Serialize, E>(
    value: &T,
    identity: u64,
    maximum: usize,
    invalid: E,
    capacity: E,
) -> Result<Vec<u8>, E> {
    native::encode_request(identity, value, limits(maximum))
        .map_err(|error| failure(error, invalid, capacity))
}

#[cfg(feature = "session-restore")]
pub(crate) fn decode<T: serde::de::DeserializeOwned, E>(
    bytes: &[u8],
    identity: u64,
    maximum: usize,
    invalid: E,
    capacity: E,
) -> Result<T, E> {
    native::decode_request(identity, bytes, limits(maximum))
        .map_err(|error| failure(error, invalid, capacity))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framed_counting_matches_emission_and_enforces_exact_capacity() {
        for length in [0, 1, 23, 24, 255, 256, 512, 2048] {
            let value = (vec!["x".repeat(length)], u64::MAX, Some(42_u64));
            for identity in [SESSION, REPLAY, LOGIN] {
                let encoded = encode(&value, identity, 8192, "invalid", "capacity").unwrap();
                for maximum in [encoded.len() - 1, encoded.len(), encoded.len() + 1] {
                    assert_eq!(
                        encoded_len(&value, maximum, "invalid", "capacity"),
                        encode(&value, identity, maximum, "invalid", "capacity")
                            .map(|bytes| bytes.len())
                    );
                }
            }
        }
    }

    #[cfg(feature = "session-restore")]
    #[test]
    fn framing_and_traversal_refusals_never_publish_a_record() {
        let encoded = encode(&[42_u8; 32], SESSION, 2048, "invalid", "capacity").unwrap();
        assert_eq!(
            decode::<[u8; 32], _>(&encoded, REPLAY, 2048, "invalid", "capacity"),
            Err("invalid")
        );
        assert_eq!(
            decode::<[u8; 32], _>(&encoded, SESSION, encoded.len() - 1, "invalid", "capacity"),
            Err("capacity")
        );
        for cut in 0..encoded.len() {
            assert!(
                decode::<[u8; 32], _>(&encoded[..cut], SESSION, 2048, "invalid", "capacity")
                    .is_err()
            );
        }
        let mut trailing = encoded;
        trailing.push(0);
        assert_eq!(
            decode::<[u8; 32], _>(&trailing, SESSION, 2048, "invalid", "capacity"),
            Err("invalid")
        );
        assert!(
            encode(&vec![0_u8; 1024], SESSION, 2048, "invalid", "capacity").is_err(),
            "traversal allowance is independent of bytes"
        );
        assert!(
            decode::<[u8; 32], _>(
                b"IC-AUTH-PORTABLE-SESSION\0\xa0",
                SESSION,
                2048,
                "invalid",
                "capacity"
            )
            .is_err()
        );
    }
}
