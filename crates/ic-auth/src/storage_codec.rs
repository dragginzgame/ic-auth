//! Bounded native IC Wire records; callers own semantic restoration and custody.
//!
//! These identities are storage discriminators, never mounted RPC methods.
//! The old domain-prefixed CBOR records are retired, with no fallback reader.
use ic_wire::{Error, Limits, native};

pub(crate) const SESSION: u64 = 0x415554485311;
pub(crate) const REPLAY: u64 = 0x415554485302;
pub(crate) const LOGIN: u64 = 0x415554485323;

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
mod tests;

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
