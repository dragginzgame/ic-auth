//! Closed sparse roots for private qualification controls.
use ic_wire::{
    Encoding, Error, Limits,
    evolution::{self, Projected},
};
use serde::{Serialize, de::DeserializeOwned};

/// Encode a record with its Serde core and an empty extension block.
pub fn encode<T: Serialize + ?Sized>(
    value: &T,
    encoding: Encoding,
    limits: Limits,
) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    evolution::encode_projected_into(&mut out, encoding, limits, |_| Ok(Projected::empty(value)))?;
    Ok(out)
}

/// Decode a closed record and refuse unexpected extension fields.
pub fn decode<T: DeserializeOwned>(
    bytes: &[u8],
    encoding: Encoding,
    limits: Limits,
) -> Result<T, Error> {
    evolution::decode_projected(bytes, encoding, limits, |record, context| {
        record
            .extensions
            .visit(context, |_, _| Err(Error::Incompatible))?;
        Ok(record.core)
    })
}

/// Measure a record including its sparse root.
pub fn encoded_len<T: Serialize + ?Sized>(
    value: &T,
    encoding: Encoding,
    limits: Limits,
) -> Result<usize, Error> {
    evolution::projected_len(&Projected::empty(value), encoding, limits)
}
