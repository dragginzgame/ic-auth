//! Stable receiver checkpoint format and upgrade lifecycle.

use super::{
    Account, AccountIndex, BTreeMap, BTreeSet, CallerRecord, Lifecycle, MemoryConfig,
    MemoryReceiver, Principal, Replay, VerificationBudget, WorkBucket, enrollment, policy,
};
use crate::stable_upgrade::{Checkpoint, DIGEST_BYTES, UpgradeError};
use ic_storage::{DefaultMemoryImpl, Memory, RuntimeMemory};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAX_PAYLOAD: usize = 128 * 1024 * 1024;
const CHUNK_ROWS: usize = 256;
const CHUNK_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 2 * 1024 * 1024,
    allocation: 64 * 1024 * 1024,
    nodes: 2 * 1024 * 1024,
    depth: 64,
};
const CHECKPOINT_DOMAIN: &[u8] = b"ic-auth.checkpoint.receiver.v1";

// Bound the caller key, retained login fields and session option tag, including
// worst-case principal lengths and varints. Session and replay bodies are
// accounted using their exact checkpoint encoding.
const CALLER_OVERHEAD: usize = 256;
const CHUNK_OVERHEAD: usize = 8;
const HEADER_ALLOWANCE: usize = 4096;

fn validate_checkpoint_budget(config: MemoryConfig) -> Result<(), UpgradeError> {
    let callers = config
        .login
        .max_retained_device_slots
        .saturating_add(config.unknown_logout.max_denials);
    let chunks = callers
        .div_ceil(CHUNK_ROWS)
        .saturating_add(config.capacity.max_replays.div_ceil(CHUNK_ROWS));
    let maximum = HEADER_ALLOWANCE
        .saturating_add(callers.saturating_mul(CALLER_OVERHEAD))
        .saturating_add(chunks.saturating_mul(CHUNK_OVERHEAD))
        .saturating_add(config.capacity.max_encoded_bytes);
    if maximum > MAX_PAYLOAD {
        return Err(UpgradeError::Configuration);
    }
    Ok(())
}

///
/// Fields
///
/// Defines the serialized checkpoint fields for the in-memory state.
///

#[derive(Serialize, Deserialize)]
#[serde(remote = "MemoryReceiver")]
struct Fields {
    config: MemoryConfig,
    lifecycle: Lifecycle,
    generation: u64,
    enrollment: enrollment::Enrollment,
    observed_at_ns: u64,
    #[serde(with = "crate::verification::RetainedBudget")]
    budget: VerificationBudget,
    unknown_budget: WorkBucket,
    unknown: usize,
    #[serde(skip)]
    slot_expiry: BTreeSet<(u64, Principal)>,
    #[serde(skip)]
    unknown_expiry: BTreeSet<(u64, Principal)>,
    #[serde(skip)]
    callers: BTreeMap<Principal, CallerRecord>,
    #[serde(skip)]
    replays: BTreeMap<[u8; 32], Replay>,
    #[serde(skip)]
    accounts: BTreeMap<Account, AccountIndex>,
    #[serde(skip)]
    session_expiry: BTreeSet<(u64, Principal)>,
    #[serde(skip)]
    replay_expiry: BTreeSet<(u64, [u8; 32])>,
    sessions: usize,
    encoded_bytes: usize,
}

///
/// SavedRef
///
/// Borrows the in-memory state for checkpoint encoding.
///

#[derive(Serialize)]
struct SavedRef<'a>(#[serde(with = "Fields")] &'a MemoryReceiver);

///
/// Saved
///
/// Reconstructs owned state from the checkpoint fields.
///

#[derive(Deserialize)]
struct Saved(#[serde(with = "Fields")] MemoryReceiver);

// Persist authoritative rows in bounded, ordered chunks. Derived indexes are
// restored from the same rows, and a single outer digest commits all chunks.
fn write_chunk<T: Serialize + ?Sized>(out: &mut Vec<u8>, value: &T) -> Result<(), UpgradeError> {
    let bytes = ic_wire::encode(value, ic_wire::Encoding::Payload, CHUNK_LIMITS)
        .map_err(|_| UpgradeError::Capacity)?;
    if out.len() + 4 + bytes.len() > DIGEST_BYTES + MAX_PAYLOAD {
        return Err(UpgradeError::Capacity);
    }
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&bytes);
    Ok(())
}

fn read_u32(bytes: &mut &[u8]) -> Result<usize, UpgradeError> {
    let (value, rest) = bytes
        .split_first_chunk::<4>()
        .ok_or(UpgradeError::InvalidCheckpoint)?;
    *bytes = rest;
    Ok(u32::from_le_bytes(*value) as usize)
}

fn read_chunk<T: serde::de::DeserializeOwned>(bytes: &mut &[u8]) -> Result<T, UpgradeError> {
    let len = read_u32(bytes)?;
    let (chunk, rest) = bytes
        .split_at_checked(len)
        .ok_or(UpgradeError::InvalidCheckpoint)?;
    *bytes = rest;
    ic_wire::decode(chunk, ic_wire::Encoding::Payload, CHUNK_LIMITS)
        .map_err(|_| UpgradeError::InvalidCheckpoint)
}

fn write_rows<K: Serialize, V: Serialize>(
    out: &mut Vec<u8>,
    rows: &BTreeMap<K, V>,
) -> Result<(), UpgradeError> {
    let count = u32::try_from(rows.len()).map_err(|_| UpgradeError::Capacity)?;
    if out.len() + 4 > DIGEST_BYTES + MAX_PAYLOAD {
        return Err(UpgradeError::Capacity);
    }
    out.extend_from_slice(&count.to_le_bytes());
    let mut chunk = Vec::with_capacity(CHUNK_ROWS);
    for row in rows {
        chunk.push(row);
        if chunk.len() == CHUNK_ROWS {
            write_chunk(out, chunk.as_slice())?;
            chunk.clear();
        }
    }
    if !chunk.is_empty() {
        write_chunk(out, chunk.as_slice())?;
    }
    Ok(())
}

fn read_rows<K: Ord + serde::de::DeserializeOwned, V: serde::de::DeserializeOwned>(
    bytes: &mut &[u8],
    maximum: usize,
) -> Result<BTreeMap<K, V>, UpgradeError> {
    let count = read_u32(bytes)?;
    if count > maximum {
        return Err(UpgradeError::InvalidCheckpoint);
    }
    let mut rows = BTreeMap::new();
    while rows.len() < count {
        let chunk: ic_wire::List<(K, V), CHUNK_ROWS> = read_chunk(bytes)?;
        if chunk.as_slice().len() != (count - rows.len()).min(CHUNK_ROWS) {
            return Err(UpgradeError::InvalidCheckpoint);
        }
        for (key, value) in chunk.into_vec() {
            if rows
                .last_key_value()
                .is_some_and(|(previous, _)| previous >= &key)
            {
                return Err(UpgradeError::InvalidCheckpoint);
            }
            rows.insert(key, value);
        }
    }
    Ok(rows)
}

///
/// StableReceiver
///
/// Owns a single-use upgrade checkpoint and the indexed [MemoryReceiver] used
/// for ordinary operations between upgrades.
///

pub struct StableReceiver<M: Memory = DefaultMemoryImpl> {
    inner: MemoryReceiver,
    checkpoint: Checkpoint<M, MAX_PAYLOAD>,
}

impl<M: Memory> StableReceiver<M> {
    /// Initialize a genuinely fresh installation with a host-owned, empty lease.
    pub fn new_fresh(
        memory: RuntimeMemory<M>,
        config: MemoryConfig,
        now_ns: u64,
    ) -> Result<Self, UpgradeError> {
        validate_checkpoint_budget(config)?;
        let inner =
            MemoryReceiver::new_fresh(config, now_ns).map_err(|_| UpgradeError::Configuration)?;

        Ok(Self {
            inner,
            checkpoint: Checkpoint::create(memory)?,
        })
    }

    /// Consume a committed upgrade checkpoint from this module's named allocation.
    /// Supply the trusted post-upgrade canister version. Only the immediate successor
    /// of the sealing version can reopen. Abort the upgrade transaction on refusal.
    /// Obtain fresh authenticated enrollment before using retained sessions.
    pub fn reopen(
        memory: RuntimeMemory<M>,
        config: MemoryConfig,
        now_ns: u64,
        canister_version: u64,
    ) -> Result<Self, UpgradeError> {
        validate_checkpoint_budget(config)?;
        let mut checkpoint = Checkpoint::load(memory)?;
        let mut payload = checkpoint.payload(CHECKPOINT_DOMAIN, canister_version)?;
        let (Saved(mut inner), commitment): (Saved, [u8; 32]) = read_chunk(&mut payload)?;
        let saved_config = inner.config;
        saved_config
            .login
            .validate()
            .map_err(|_| UpgradeError::InvalidCheckpoint)?;
        saved_config
            .capacity
            .validate()
            .map_err(|_| UpgradeError::InvalidCheckpoint)?;
        let mut permitted = saved_config;
        permitted.login.max_retained_device_slots = permitted
            .login
            .max_retained_device_slots
            .max(config.login.max_retained_device_slots);
        permitted.capacity.max_sessions = permitted
            .capacity
            .max_sessions
            .max(config.capacity.max_sessions);
        permitted.capacity.max_replays = permitted
            .capacity
            .max_replays
            .max(config.capacity.max_replays);
        permitted.capacity.max_encoded_bytes = permitted
            .capacity
            .max_encoded_bytes
            .max(config.capacity.max_encoded_bytes);
        if permitted != config {
            return Err(UpgradeError::Configuration);
        }
        inner.callers = read_rows(
            &mut payload,
            saved_config
                .login
                .max_retained_device_slots
                .saturating_add(config.unknown_logout.max_denials),
        )?;
        inner.replays = read_rows(&mut payload, saved_config.capacity.max_replays)?;
        if !payload.is_empty() {
            return Err(UpgradeError::InvalidCheckpoint);
        }
        if inner.lifecycle == Lifecycle::Closed {
            return Err(UpgradeError::Lifecycle);
        }
        if inner.rebuild_checkpoint_indexes()? != commitment {
            return Err(UpgradeError::InvalidCheckpoint);
        }
        inner.config = config;
        inner.observe(now_ns).map_err(|_| UpgradeError::Lifecycle)?;
        inner.lifecycle = Lifecycle::Fresh;
        inner.enrollment.discard_pending();
        checkpoint.consume();

        Ok(Self { inner, checkpoint })
    }

    /// Save in the host's pre-upgrade transaction. Success closes the running
    /// instance. Supply the trusted canister version from the pre-upgrade hook.
    /// The encoded checkpoint is limited to 128 MiB. On error the host
    /// must abort the upgrade, retaining the heap.
    pub fn seal(&mut self, canister_version: u64) -> Result<(), UpgradeError> {
        if self.inner.lifecycle == Lifecycle::Closed {
            return Err(UpgradeError::Lifecycle);
        }
        let mut image = vec![0; DIGEST_BYTES];
        write_chunk(
            &mut image,
            &(SavedRef(&self.inner), self.inner.extension_digest()),
        )?;
        write_rows(&mut image, &self.inner.callers)?;
        write_rows(&mut image, &self.inner.replays)?;
        self.checkpoint
            .save_image(CHECKPOINT_DOMAIN, canister_version, image)?;
        self.inner.close();

        Ok(())
    }

    /// Access the current Auth instance under the host transaction.
    pub fn state(&mut self) -> &mut MemoryReceiver {
        &mut self.inner
    }
}

impl<M: Memory> std::ops::Deref for StableReceiver<M> {
    type Target = MemoryReceiver;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<M: Memory> std::ops::DerefMut for StableReceiver<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<M: Memory> AsRef<MemoryReceiver> for StableReceiver<M> {
    fn as_ref(&self) -> &MemoryReceiver {
        &self.inner
    }
}

fn hash_extension(digest: &mut Sha256, facts: &super::SessionFacts<'_>) {
    if let Some(ext) = facts.ext {
        digest.update([facts.caller.as_slice().len() as u8]);
        digest.update(facts.caller.as_slice());
        digest.update(facts.fingerprint);
        digest.update((ext.len() as u32).to_le_bytes());
        digest.update(ext);
    }
}

impl MemoryReceiver {
    // Byte accounting cannot distinguish equal-length extensions exchanged by an encoder.
    fn extension_digest(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        for facts in self
            .callers
            .values()
            .filter_map(CallerRecord::session_facts)
        {
            hash_extension(&mut digest, &facts);
        }
        digest.finalize().into()
    }

    fn rebuild_checkpoint_indexes(&mut self) -> Result<[u8; 32], UpgradeError> {
        let bad = UpgradeError::InvalidCheckpoint;
        let cfg = self.config;
        if !self
            .enrollment
            .matches(cfg.enrollment, cfg.audience, self.generation)
            || self.unknown > cfg.unknown_logout.max_denials
            || self
                .callers
                .len()
                .checked_sub(self.unknown)
                .is_none_or(|known| known > cfg.login.max_retained_device_slots)
            || self.sessions > cfg.capacity.max_sessions
            || self.replays.len() > cfg.capacity.max_replays
            || self.encoded_bytes > cfg.capacity.max_encoded_bytes
            || !self.budget.matches(
                cfg.verification_costs,
                cfg.verification_capacity,
                cfg.refill_interval_ns,
                self.observed_at_ns,
            )
            || !self.unknown_budget.matches(
                cfg.unknown_logout.capacity,
                cfg.unknown_logout.refill_interval_ns,
                self.observed_at_ns,
            )
        {
            return Err(bad);
        }
        let mut digest = Sha256::new();
        let mut bytes = 0usize;
        let mut sessions = 0;
        let mut unknown = 0;
        for (caller, row) in &mut self.callers {
            policy::check_record(&row.login, cfg.audience).map_err(|_| bad)?;
            if caller != &row.login.caller {
                return Err(bad);
            }
            match row.slot() {
                Some(slot) if slot.2 < cfg.login.max_device_slots => {
                    self.slot_expiry
                        .insert((row.reclaim_at(cfg.login.max_login_ttl_ns), *caller));
                    if self
                        .accounts
                        .entry((slot.0, slot.1))
                        .or_default()
                        .set_slot(slot.2, *caller)
                        .is_some()
                    {
                        return Err(bad);
                    }
                }
                None if row.session.is_none() => {
                    self.unknown_expiry.insert((row.login.until_ns, *caller));
                    unknown += 1;
                }
                _ => return Err(bad),
            }
            if row.session.is_some() {
                if row.login.denied {
                    return Err(bad);
                }
                let facts = row.session_facts().ok_or(bad)?;
                policy::check_session_shape(&facts).map_err(|_| bad)?;
                if facts.established_at_ns > self.observed_at_ns
                    || facts.authority_generation > self.generation
                {
                    return Err(bad);
                }
                hash_extension(&mut digest, &facts);
                let session = row.session.as_mut().ok_or(bad)?;
                session.bytes = session.encoded_size().map_err(|_| bad)?;
                bytes = bytes.checked_add(session.bytes).ok_or(bad)?;
                self.session_expiry.insert((session.expires_at_ns, *caller));
                sessions += 1;
            }
        }
        for (fingerprint, replay) in &mut self.replays {
            replay.bytes = replay.encoded_size(fingerprint).map_err(|_| bad)?;
            bytes = bytes.checked_add(replay.bytes).ok_or(bad)?;
            self.replay_expiry
                .insert((replay.remove_at_ns, *fingerprint));
            let count = &mut self
                .accounts
                .entry(replay.account)
                .or_default()
                .replay_count;
            *count += 1;
            if *count > cfg.capacity.max_replays_per_account {
                return Err(bad);
            }
        }
        if bytes != self.encoded_bytes || sessions != self.sessions || unknown != self.unknown {
            return Err(bad);
        }
        Ok(digest.finalize().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_auth_protocol_types::CanonicalId;

    #[test]
    fn checkpoint_budget_accounts_for_dormant_slots_and_chunk_framing() {
        let mut config = MemoryConfig::starting_profile(
            ic_auth_protocol_types::AudienceId {
                canonical_network_id: CanonicalId::from_bytes([1; 32]),
                fleet_id: CanonicalId::from_bytes([2; 32]),
            },
            super::super::EnrollmentBinding {
                receiver: Principal::from_slice(&[10]),
                authority: Principal::from_slice(&[11]),
            },
        );
        assert_eq!(validate_checkpoint_budget(config), Ok(()));

        // 125,064 caller entries require 489 chunks. 400,000 replays need 1,563.
        let overhead = 4096 + 125_064 * 256 + (489 + 1563) * 8;
        config.capacity.max_encoded_bytes = MAX_PAYLOAD - overhead;
        assert_eq!(validate_checkpoint_budget(config), Ok(()));
        config.capacity.max_encoded_bytes += 1;
        assert_eq!(
            validate_checkpoint_budget(config),
            Err(UpgradeError::Configuration)
        );
        config.capacity.max_encoded_bytes -= 1;
        config.login.max_retained_device_slots += 1;
        assert_eq!(
            validate_checkpoint_budget(config),
            Err(UpgradeError::Configuration)
        );
        config.capacity.max_encoded_bytes = 1;
        config.login.max_retained_device_slots = usize::MAX;
        assert_eq!(
            validate_checkpoint_budget(config),
            Err(UpgradeError::Configuration)
        );
    }
}
