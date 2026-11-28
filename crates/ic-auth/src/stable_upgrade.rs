//! Single-use upgrade checkpoints for host-governed Auth allocations.
//!
//! The host supplies a different named allocation for each Auth owner. It must
//! trap on failure in either upgrade hook. A successful reopen consumes its
//! checkpoint before serving calls, fencing recovery from an outdated image.
use ic_storage::{
    Memory, RuntimeMemory,
    adapter::{Bound, StableCell, Storable},
};
#[cfg(feature = "issuer-stable")]
use serde::Serialize;
#[cfg(feature = "issuer-stable")]
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::{borrow::Cow, error::Error, fmt};

pub(crate) const DIGEST_BYTES: usize = 32;

pub(crate) const ISSUER_MAX_PAYLOAD: usize = 8 * 1024 * 1024;
#[cfg(feature = "issuer-stable")]
pub(crate) const ISSUER_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: ISSUER_MAX_PAYLOAD,
    allocation: 256 * 1024 * 1024,
    nodes: 8 * 1024 * 1024,
    depth: 64,
};

///
/// UpgradeError
///
/// Failure to create, commit or reopen a bounded Auth upgrade checkpoint.
///

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UpgradeError {
    /// Allocation is already occupied, missing, or has an incompatible cell header.
    Allocation,
    /// No sealed checkpoint exists or its format, digest or payload is invalid.
    InvalidCheckpoint,
    /// Encoding exceeds an upgrade resource allowance.
    Capacity,
    /// The host configuration is invalid or differs from retained configuration.
    Configuration,
    /// State is closed, time regressed, or restart fencing cannot advance.
    Lifecycle,
}

impl fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Auth upgrade checkpoint refused: {self:?}")
    }
}

impl Error for UpgradeError {}

/// Per-record extension with the same Wire byte bound on encode and decode.
pub(crate) mod extension {
    use ic_auth_protocol_types::MAX_TOKEN_EXT_BYTES;
    use ic_wire::{Bytes, BytesRef};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub(crate) fn serialize<S: Serializer>(
        value: &Option<Vec<u8>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value
            .as_deref()
            .map(BytesRef::<MAX_TOKEN_EXT_BYTES>::new)
            .transpose()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<u8>>, D::Error> {
        Ok(Option::<Bytes<MAX_TOKEN_EXT_BYTES>>::deserialize(deserializer)?.map(Bytes::into_vec))
    }
}

// Owner-specific checkpoint chunks and renewal identities have a closed sparse root.
pub(crate) mod record {
    use ic_wire::{
        Encoding, Error, Limits,
        evolution::{self, Projected},
    };
    use serde::{Serialize, de::DeserializeOwned};

    /// Encode a record with its Serde core and an empty extension block.
    pub(crate) fn encode<T: Serialize + ?Sized>(
        value: &T,
        encoding: Encoding,
        limits: Limits,
    ) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();
        evolution::encode_projected_into(&mut out, encoding, limits, |_| {
            Ok(Projected::empty(value))
        })?;
        Ok(out)
    }

    /// Decode a closed record and refuse unexpected extension fields.
    pub(crate) fn decode<T: DeserializeOwned>(
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
    pub(crate) fn encoded_len<T: Serialize + ?Sized>(
        value: &T,
        encoding: Encoding,
        limits: Limits,
    ) -> Result<usize, Error> {
        evolution::projected_len(&Projected::empty(value), encoding, limits)
    }
}

///
/// Image
///
/// Bounded bytes held in the stable checkpoint cell.
///

struct Image<const MAX_PAYLOAD: usize>(Vec<u8>);

impl<const MAX_PAYLOAD: usize> Storable for Image<MAX_PAYLOAD> {
    const BOUND: Bound = Bound::Bounded {
        max_size: (DIGEST_BYTES + MAX_PAYLOAD) as u32,
        is_fixed_size: false,
    };
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.0)
    }
    fn into_bytes(self) -> Vec<u8> {
        self.0
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        Self(bytes.into_owned())
    }
}

///
/// Checkpoint
///
/// Creates, seals and consumes the stable image for one Auth owner.
///

pub(crate) struct Checkpoint<M: Memory, const MAX_PAYLOAD: usize = ISSUER_MAX_PAYLOAD>(
    StableCell<Image<MAX_PAYLOAD>, M>,
);

impl<M: Memory, const MAX_PAYLOAD: usize> Checkpoint<M, MAX_PAYLOAD> {
    pub(crate) fn create(memory: RuntimeMemory<M>) -> Result<Self, UpgradeError> {
        if memory.size() != 0 {
            return Err(UpgradeError::Allocation);
        }

        Ok(Self(StableCell::create(memory, Image(Vec::new()))))
    }

    pub(crate) fn load(memory: RuntimeMemory<M>) -> Result<Self, UpgradeError> {
        StableCell::load(memory)
            .map(Self)
            .map_err(|_| UpgradeError::Allocation)
    }

    #[cfg(feature = "issuer-stable")]
    pub(crate) fn save<T: Serialize>(
        &mut self,
        domain: &[u8],
        canister_version: u64,
        value: &T,
        limits: ic_wire::Limits,
    ) -> Result<(), UpgradeError> {
        let payload_len = record::encoded_len(value, ic_wire::Encoding::Payload, limits)
            .map_err(|_| UpgradeError::Capacity)?;
        if payload_len > MAX_PAYLOAD {
            return Err(UpgradeError::Capacity);
        }
        let mut image = vec![0; DIGEST_BYTES + payload_len];
        let encoded = ic_wire::evolution::encode_projected_to_slice(
            &mut image[DIGEST_BYTES..],
            ic_wire::Encoding::Payload,
            limits,
            |_| Ok(ic_wire::evolution::Projected::empty(value)),
        )
        .map_err(|_| UpgradeError::Capacity)?;
        if encoded.len() != payload_len {
            return Err(UpgradeError::Capacity);
        }
        self.save_image(domain, canister_version, image)
    }

    pub(crate) fn save_image(
        &mut self,
        domain: &[u8],
        canister_version: u64,
        mut image: Vec<u8>,
    ) -> Result<(), UpgradeError> {
        if !(DIGEST_BYTES..=DIGEST_BYTES + MAX_PAYLOAD).contains(&image.len()) {
            return Err(UpgradeError::Capacity);
        }
        let successor = canister_version
            .checked_add(1)
            .ok_or(UpgradeError::Lifecycle)?;
        let digest = Sha256::new()
            .chain_update(domain)
            .chain_update(successor.to_le_bytes())
            .chain_update(&image[DIGEST_BYTES..])
            .finalize();
        image[..DIGEST_BYTES].copy_from_slice(&digest);
        self.0.set(Image(image));

        Ok(())
    }

    pub(crate) fn payload(
        &self,
        domain: &[u8],
        canister_version: u64,
    ) -> Result<&[u8], UpgradeError> {
        let (expected, payload) = self
            .0
            .get()
            .0
            .split_first_chunk::<DIGEST_BYTES>()
            .ok_or(UpgradeError::InvalidCheckpoint)?;

        let digest: [u8; 32] = Sha256::new()
            .chain_update(domain)
            .chain_update(canister_version.to_le_bytes())
            .chain_update(payload)
            .finalize()
            .into();
        if digest != *expected {
            return Err(UpgradeError::InvalidCheckpoint);
        }

        Ok(payload)
    }

    #[cfg(feature = "issuer-stable")]
    pub(crate) fn decode<T: DeserializeOwned>(
        payload: &[u8],
        limits: ic_wire::Limits,
    ) -> Result<T, UpgradeError> {
        record::decode(payload, ic_wire::Encoding::Payload, limits)
            .map_err(|_| UpgradeError::InvalidCheckpoint)
    }

    pub(crate) fn consume(&mut self) {
        self.0.set(Image(Vec::new()));
    }
}

#[cfg(test)]
mod tests {
    use super::{Checkpoint, ISSUER_MAX_PAYLOAD, Image, UpgradeError};
    use ic_storage::{StableKey, VectorMemory, adapter::StableCell, testing::fixture_runtime};
    use sha2::{Digest, Sha256};

    #[test]
    fn storage_079_cell_refuses_reopen_and_fresh_initialization() {
        let name = "auth.checkpoint.v1";
        let runtime = fixture_runtime(
            VectorMemory::default(),
            "auth",
            "auth.",
            &[name],
            Default::default(),
        )
        .unwrap();
        let key = StableKey::parse(name).unwrap();
        let lease = || runtime.open_memory(&key).unwrap();
        // Storage 0.79 Cell framing for an empty (consumed) checkpoint.
        let previous = *b"SCL\x01\0\0\0\0";
        let memory = lease();
        assert_eq!(memory.grow(1).unwrap(), 0);
        memory.write(0, &previous);
        drop(memory);
        assert!(matches!(
            Checkpoint::<_>::load(lease()),
            Err(UpgradeError::Allocation)
        ));
        assert!(matches!(
            Checkpoint::<_>::create(lease()),
            Err(UpgradeError::Allocation)
        ));
        let mut retained = [0; 8];
        lease().read(0, &mut retained);
        assert_eq!(retained, previous);
    }

    #[test]
    fn prior_format_digests_are_refused_before_payload_decoding() {
        let key = "auth.checkpoint.v1";
        const ISSUER: &str = "ic-auth.issuer.checkpoint.sparse.v1";
        const RECEIVER: &str = "ic-auth.checkpoint.receiver.sparse.v1";
        for (legacy, current, version_bound) in [
            ("ic-auth.issuer.checkpoint.v1", ISSUER, true),
            ("ic-auth.checkpoint.receiver.v1", RECEIVER, true),
            ("ic-auth.issuer.v1", ISSUER, false),
            ("ic-auth.issuer.v2", ISSUER, false),
            ("ic-auth.issuer.v3", ISSUER, true),
            ("ic-auth.receiver.v1", RECEIVER, false),
            ("ic-auth.receiver.v2", RECEIVER, false),
            ("ic-auth.receiver.v3", RECEIVER, true),
            ("ic-auth.receiver.v4", RECEIVER, true),
            ("ic-auth.receiver.checkpoint.v1", RECEIVER, true),
        ] {
            let backing = VectorMemory::default();
            let runtime =
                fixture_runtime(backing, "auth", "auth.", &[key], Default::default()).unwrap();
            let key = StableKey::parse(key).unwrap();
            let lease = || runtime.open_memory(&key).unwrap();
            let payload = [0x5a; 64];
            let mut digest = Sha256::new().chain_update(legacy);
            if version_bound {
                digest.update(1u64.to_le_bytes());
            }
            let mut image = digest.chain_update(payload).finalize().to_vec();
            image.extend_from_slice(&payload);
            drop(StableCell::create(
                lease(),
                Image::<ISSUER_MAX_PAYLOAD>(image),
            ));
            let checkpoint = Checkpoint::<_>::load(lease()).unwrap();
            assert_eq!(
                checkpoint.payload(current.as_bytes(), 1),
                Err(UpgradeError::InvalidCheckpoint)
            );
        }
    }
}
