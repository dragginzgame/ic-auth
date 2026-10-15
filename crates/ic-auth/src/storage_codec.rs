//! Private framing for trusted-storage records; callers own semantic validation.

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
