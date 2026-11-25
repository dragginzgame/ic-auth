//! Single-use upgrade checkpoints for host-governed Auth allocations.
//!
//! The host supplies a different named allocation for each Auth owner. It must
//! trap on failure in either upgrade hook. A successful reopen consumes its
//! checkpoint before serving calls, fencing recovery from an outdated image.
use ic_storage::{
    Memory, RuntimeMemory,
    adapter::{Bound, StableCell, Storable},
};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{borrow::Cow, error::Error, fmt};

const MAX_PAYLOAD: usize = 8 * 1024 * 1024;
const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: MAX_PAYLOAD,
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

///
/// Image
///
/// Bounded bytes held in the stable checkpoint cell.
///

struct Image(Vec<u8>);

impl Storable for Image {
    const BOUND: Bound = Bound::Bounded {
        max_size: (32 + MAX_PAYLOAD) as u32,
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

pub(crate) struct Checkpoint<M: Memory>(StableCell<Image, M>);

impl<M: Memory> Checkpoint<M> {
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

    pub(crate) fn save<T: Serialize>(
        &mut self,
        domain: &[u8],
        canister_version: u64,
        value: &T,
    ) -> Result<(), UpgradeError> {
        let successor = canister_version
            .checked_add(1)
            .ok_or(UpgradeError::Lifecycle)?;
        let payload = ic_wire::encode(value, ic_wire::Encoding::Payload, LIMITS)
            .map_err(|_| UpgradeError::Capacity)?;
        let digest = Sha256::new()
            .chain_update(domain)
            .chain_update(successor.to_le_bytes())
            .chain_update(&payload)
            .finalize();

        let mut image = Vec::with_capacity(32 + payload.len());
        image.extend_from_slice(&digest);
        image.extend_from_slice(&payload);
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
            .split_first_chunk::<32>()
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

    pub(crate) fn decode<T: DeserializeOwned>(payload: &[u8]) -> Result<T, UpgradeError> {
        ic_wire::decode(payload, ic_wire::Encoding::Payload, LIMITS)
            .map_err(|_| UpgradeError::InvalidCheckpoint)
    }

    pub(crate) fn consume(&mut self) {
        self.0.set(Image(Vec::new()));
    }
}

#[cfg(test)]
mod tests {
    use super::{Checkpoint, Image, UpgradeError};
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
            Checkpoint::load(lease()),
            Err(UpgradeError::Allocation)
        ));
        assert!(matches!(
            Checkpoint::create(lease()),
            Err(UpgradeError::Allocation)
        ));
        let mut retained = [0; 8];
        lease().read(0, &mut retained);
        assert_eq!(retained, previous);
    }

    #[test]
    fn prior_format_digests_are_refused_before_payload_decoding() {
        let key = "auth.checkpoint.v1";
        for role in ["issuer", "receiver"] {
            for legacy in ["v1", "v2"] {
                let backing = VectorMemory::default();
                let runtime =
                    fixture_runtime(backing, "auth", "auth.", &[key], Default::default()).unwrap();
                let key = StableKey::parse(key).unwrap();
                let lease = || runtime.open_memory(&key).unwrap();
                // Independent old envelope: SHA-256(domain || payload), then payload.
                let payload = [0x5a; 64];
                let mut image = Sha256::new()
                    .chain_update(format!("ic-auth.{role}.{legacy}"))
                    .chain_update(payload)
                    .finalize()
                    .to_vec();
                image.extend_from_slice(&payload);
                drop(StableCell::create(lease(), Image(image)));
                let checkpoint = Checkpoint::load(lease()).unwrap();
                assert_eq!(
                    checkpoint.payload(format!("ic-auth.{role}.v3").as_bytes(), 1),
                    Err(UpgradeError::InvalidCheckpoint)
                );
            }
        }
    }
}
