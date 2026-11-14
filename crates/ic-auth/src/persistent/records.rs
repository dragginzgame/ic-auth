use super::StoreMemory;
use ic_storage::{
    Memory, RuntimeMemory,
    adapter::{StableBTreeMap, StableCell},
    storable::{Bound, CodecError, Storable},
};
use serde::{Serialize, de::DeserializeOwned};
use std::borrow::Cow;

const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 73_728,
    allocation: 1_048_576,
    nodes: 16_384,
    depth: 16,
};
const RECORD: u64 = 0x41555448504401;

/// Persistent record refusal; storage IO/codec faults at the collection boundary
/// trap so the host cannot commit a partial operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum StoreError {
    /// Backing is foreign, missing or internally inconsistent; no reset is implied.
    #[error("foreign, missing or inconsistent Auth storage")]
    Corrupt,
    /// Supplied policy or input violates the store contract.
    #[error("Auth storage configuration or input is invalid")]
    Invalid,
    /// A bounded record or byte reservation cannot be admitted.
    #[error("Auth storage capacity exhausted")]
    Capacity,
    /// The issuer-wide fresh intent allowance is exhausted; no new intent exists.
    #[error("fresh issuance budget exhausted")]
    Throttled,
    /// Trusted time is earlier than the durable observation fence.
    #[error("Auth storage clock moved backwards")]
    Clock,
    /// A retry or callback differs from the retained original operation.
    #[error("operation conflicts with its original intent")]
    Conflict,
    /// The selected operation or login deadline has passed.
    #[error("operation or login is expired")]
    Expired,
    /// The request addresses an operation epoch no longer open for new work.
    #[error("operation epoch is closed")]
    EpochClosed,
    /// The live identity binding or protected authority does not authorize the work.
    #[error("authentication binding or authority is no longer current")]
    Denied,
    /// A required earlier stage has not completed.
    #[error("operation has not reached the required stage")]
    Pending,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Key(pub Vec<u8>);
impl Key {
    pub fn new(kind: u8) -> Self {
        Self(vec![kind])
    }
    pub fn principal(mut self, principal: ic_auth_protocol_types::Principal) -> Self {
        let bytes = principal.as_slice();
        self.0.push(bytes.len() as u8);
        self.0.extend_from_slice(bytes);
        self.0.resize(self.0.len() + 29 - bytes.len(), 0);
        self
    }
    pub fn number(mut self, number: u64) -> Self {
        self.0.extend_from_slice(&number.to_be_bytes());
        self
    }
    pub fn bytes(mut self, bytes: &[u8]) -> Self {
        self.0.extend_from_slice(bytes);
        self
    }
    pub fn principal_value(
        &self,
        kind: u8,
    ) -> Result<ic_auth_protocol_types::Principal, StoreError> {
        if self.0.len() != 31 || self.0[0] != kind || self.0[1] > 29 {
            return Err(StoreError::Corrupt);
        }
        let principal =
            ic_auth_protocol_types::Principal::from_slice(&self.0[2..2 + usize::from(self.0[1])]);
        if Self::new(kind).principal(principal) != *self {
            return Err(StoreError::Corrupt);
        }
        Ok(principal)
    }
    pub fn time(&self) -> Result<u64, StoreError> {
        Ok(u64::from_be_bytes(
            self.0
                .get(1..9)
                .ok_or(StoreError::Corrupt)?
                .try_into()
                .map_err(|_| StoreError::Corrupt)?,
        ))
    }
}
impl Storable for Key {
    const BOUND: Bound = Bound::Bounded {
        max_size: 128,
        is_fixed_size: false,
    };
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        Ok(Cow::Borrowed(&self.0))
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
        if bytes.is_empty() {
            return Err(CodecError::Invalid("empty Auth key"));
        }
        Ok(Self(bytes.into_owned()))
    }
}

pub(super) struct Blob(Vec<u8>);
impl Storable for Blob {
    const BOUND: Bound = Bound::Bounded {
        max_size: LIMITS.bytes as u32,
        is_fixed_size: false,
    };
    fn to_bytes(&self) -> Result<Cow<'_, [u8]>, CodecError> {
        Ok(Cow::Borrowed(&self.0))
    }
    fn from_bytes(bytes: Cow<'_, [u8]>) -> Result<Self, CodecError> {
        Ok(Self(bytes.into_owned()))
    }
}

pub(super) fn encode<T: Serialize>(value: &T) -> Result<Blob, StoreError> {
    ic_wire::native::encode_request(RECORD, value, LIMITS)
        .map(Blob)
        .map_err(|_| StoreError::Capacity)
}
pub(super) fn size<T: Serialize>(value: &T) -> Result<u64, StoreError> {
    ic_wire::native::encoded_frame_len(value, LIMITS)
        .map(|n| n as u64)
        .map_err(|_| StoreError::Capacity)
}

/// Small pre-encoded write set. No fallible application work after publication.
pub(super) struct Writes(Vec<(Key, Option<Blob>)>);
impl Writes {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn put<T: Serialize>(&mut self, key: Key, value: &T) -> Result<(), StoreError> {
        self.0.push((key, Some(encode(value)?)));
        Ok(())
    }
    pub fn delete(&mut self, key: Key) {
        self.0.push((key, None));
    }
}

pub(super) struct Records<M: Memory> {
    rows: StableBTreeMap<Key, Blob, M>,
    metadata: StableCell<Blob, M>,
    point_index: Option<(u8, StableBTreeMap<Key, Blob, M>)>,
}
impl<M: Memory> Records<M> {
    pub fn create(memory: StoreMemory<M>) -> Result<Self, StoreError> {
        if memory.records.size() != 0 || memory.metadata.size() != 0 {
            return Err(StoreError::Corrupt);
        }
        let empty = encode(&())?;
        let rows = StableBTreeMap::create(memory.records).map_err(|_| StoreError::Capacity)?;
        let metadata = StableCell::create(memory.metadata, empty)
            .expect("metadata initialization requires host rollback on failure");
        Ok(Self {
            rows,
            metadata,
            point_index: None,
        })
    }
    pub fn open(memory: StoreMemory<M>) -> Result<Self, StoreError> {
        let rows = StableBTreeMap::load(memory.records).map_err(|_| StoreError::Corrupt)?;
        let metadata =
            StableCell::load(memory.metadata, LIMITS.bytes).map_err(|_| StoreError::Corrupt)?;
        Ok(Self {
            rows,
            metadata,
            point_index: None,
        })
    }
    pub fn create_with_index(
        memory: StoreMemory<M>,
        index: RuntimeMemory<M>,
        kind: u8,
    ) -> Result<Self, StoreError> {
        if index.size() != 0 {
            return Err(StoreError::Corrupt);
        }
        let mut records = Self::create(memory)?;
        records.point_index = Some((
            kind,
            StableBTreeMap::create(index)
                .expect("point index initialization requires host rollback on failure"),
        ));
        Ok(records)
    }
    pub fn open_with_index(
        memory: StoreMemory<M>,
        index: RuntimeMemory<M>,
        kind: u8,
    ) -> Result<Self, StoreError> {
        let mut records = Self::open(memory)?;
        records.point_index = Some((
            kind,
            StableBTreeMap::load(index).map_err(|_| StoreError::Corrupt)?,
        ));
        Ok(records)
    }
    fn table(&self, key: &Key) -> &StableBTreeMap<Key, Blob, M> {
        self.point_index
            .as_ref()
            .filter(|(kind, _)| key.0.first() == Some(kind))
            .map_or(&self.rows, |(_, table)| table)
    }
    pub fn get<T: DeserializeOwned>(&self, key: &Key) -> Result<Option<T>, StoreError> {
        if key.0 == [0] {
            return ic_wire::native::decode_request(RECORD, &self.metadata.get().0, LIMITS)
                .map(Some)
                .map_err(|_| StoreError::Corrupt);
        }
        self.table(key)
            .get(key)
            .map(|blob| {
                ic_wire::native::decode_request(RECORD, &blob.0, LIMITS)
                    .map_err(|_| StoreError::Corrupt)
            })
            .transpose()
    }
    pub fn contains(&self, key: &Key) -> bool {
        self.table(key).contains_key(key)
    }
    pub fn range<T: DeserializeOwned>(
        &self,
        kind: u8,
        after: Option<Key>,
    ) -> impl Iterator<Item = Result<(Key, T), StoreError>> {
        use std::ops::Bound::{Excluded, Included};
        let first = Key::new(kind);
        self.table(&first)
            .range((
                after.map_or(Included(first), Excluded),
                Excluded(Key::new(kind + 1)),
            ))
            .map(|entry| {
                let (key, blob) = entry.into_pair();
                ic_wire::native::decode_request(RECORD, &blob.0, LIMITS)
                    .map(|row| (key, row))
                    .map_err(|_| StoreError::Corrupt)
            })
    }
    pub fn expired(&self, kind: u8, now_ns: u64, budget: usize) -> Result<Vec<Key>, StoreError> {
        let mut selected = Vec::new();
        for key in self
            .rows
            .keys_range(Key::new(kind)..Key::new(kind + 1))
            .take(budget)
        {
            if key.time()? > now_ns {
                break;
            }
            selected.push(key);
        }
        Ok(selected)
    }
    pub fn len(&self) -> u64 {
        self.rows.len()
            + 1
            + self
                .point_index
                .as_ref()
                .map_or(0, |(_, table)| table.len())
    }
    pub fn commit(&mut self, writes: Writes) {
        // The prepared operation has already admitted its required old state.
        // Keep Storage's structural checks without materializing unused payloads.
        for (key, value) in writes.0 {
            if key.0 == [0] {
                self.metadata
                    .set(value.expect("metadata cannot be deleted"));
            } else if let Some((kind, table)) = &mut self.point_index
                && key.0.first() == Some(kind)
            {
                if let Some(value) = value {
                    table.insert_discard(key, value);
                } else {
                    table.remove_discard(&key);
                }
            } else if let Some(value) = value {
                self.rows.insert_discard(key, value);
            } else {
                self.rows.remove_discard(&key);
            }
        }
    }
}
