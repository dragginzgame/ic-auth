//! Private framing for trusted-storage records; callers own semantic validation.

struct ByteCount(usize);

impl std::io::Write for ByteCount {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("record size overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn encoded_len<T: serde::Serialize, E>(
    value: &T,
    domain: &[u8],
    maximum: usize,
    invalid: E,
    capacity: E,
) -> Result<usize, E> {
    let mut count = ByteCount(domain.len());
    serde_cbor::to_writer(&mut count, value).map_err(|_| invalid)?;
    if count.0 > maximum {
        return Err(capacity);
    }
    Ok(count.0)
}

pub(crate) fn encode<T: serde::Serialize, E>(
    value: &T,
    domain: &[u8],
    maximum: usize,
    invalid: E,
    capacity: E,
) -> Result<Vec<u8>, E> {
    let mut bytes = domain.to_vec();
    bytes.extend(serde_cbor::to_vec(value).map_err(|_| invalid)?);
    if bytes.len() > maximum {
        return Err(capacity);
    }
    Ok(bytes)
}

#[cfg(feature = "session-restore")]
pub(crate) fn decode<T: serde::de::DeserializeOwned, E>(
    bytes: &[u8],
    domain: &[u8],
    maximum: usize,
    invalid: impl Fn() -> E,
    capacity: E,
) -> Result<T, E> {
    // Reject oversized input before inspecting framing or allocating CBOR values.
    if bytes.len() > maximum {
        return Err(capacity);
    }
    let payload = bytes.strip_prefix(domain).ok_or_else(&invalid)?;
    serde_cbor::from_slice(payload).map_err(|_| invalid())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counted_framing_matches_cbor_at_length_and_capacity_boundaries() {
        for length in [0, 1, 23, 24, 255, 256, 512, 2048] {
            let value = (vec!["x".repeat(length)], u64::MAX, Some(42_u64));
            for domain in [b"".as_slice(), b"record\0"] {
                let encoded = encode(&value, domain, usize::MAX, "invalid", "capacity").unwrap();
                for maximum in [encoded.len() - 1, encoded.len(), encoded.len() + 1] {
                    assert_eq!(
                        encoded_len(&value, domain, maximum, "invalid", "capacity"),
                        encode(&value, domain, maximum, "invalid", "capacity")
                            .map(|bytes| bytes.len())
                    );
                }
            }
        }
    }

    struct Refused;
    impl serde::Serialize for Refused {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("refused"))
        }
    }

    #[test]
    fn serialization_failure_keeps_precedence_and_count_overflow_refuses() {
        use std::io::Write;
        assert_eq!(
            encoded_len(&Refused, b"domain", 0, "invalid", "capacity"),
            Err("invalid")
        );
        assert_eq!(
            encode(&Refused, b"domain", 0, "invalid", "capacity"),
            Err("invalid")
        );
        let mut count = ByteCount(usize::MAX);
        assert!(count.write(&[1]).is_err());
        assert_eq!(count.0, usize::MAX);
    }
}
