//! Stable receiver checkpoint format and upgrade lifecycle.

use super::{
    Account, BTreeMap, BTreeSet, CallerRecord, Lifecycle, MemoryConfig, MemoryReceiver, Principal,
    Replay, Slot, VerificationBudget, WorkBucket, enrollment, policy,
};
use crate::stable_upgrade::{Checkpoint, UpgradeError};
use ic_storage::{DefaultMemoryImpl, Memory, RuntimeMemory};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const CHECKPOINT_DOMAIN: &[u8] = b"ic-auth.receiver.v3";

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
    unknown_expiry: BTreeSet<(u64, Principal)>,
    callers: BTreeMap<Principal, CallerRecord>,
    slots: BTreeMap<Slot, Principal>,
    replays: BTreeMap<[u8; 32], Replay>,
    accounts: BTreeMap<Account, usize>,
    session_expiry: BTreeSet<(u64, Principal)>,
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

///
/// StableReceiver
///
/// Owns a single-use upgrade checkpoint and the indexed [MemoryReceiver] used
/// for ordinary operations between upgrades.
///

pub struct StableReceiver<M: Memory = DefaultMemoryImpl> {
    inner: MemoryReceiver,
    checkpoint: Checkpoint<M>,
}

impl<M: Memory> StableReceiver<M> {
    /// Initialize a genuinely fresh installation with a host-owned, empty lease.
    pub fn new_fresh(
        memory: RuntimeMemory<M>,
        config: MemoryConfig,
        now_ns: u64,
    ) -> Result<Self, UpgradeError> {
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
        let mut checkpoint = Checkpoint::load(memory)?;
        let (Saved(mut inner), commitment): (Saved, [u8; 32]) =
            Checkpoint::<M>::decode(checkpoint.payload(CHECKPOINT_DOMAIN, canister_version)?)?;
        if inner.config != config {
            return Err(UpgradeError::Configuration);
        }
        if inner.lifecycle == Lifecycle::Closed {
            return Err(UpgradeError::Lifecycle);
        }
        if inner.validate_checkpoint()? != commitment {
            return Err(UpgradeError::InvalidCheckpoint);
        }
        inner.observe(now_ns).map_err(|_| UpgradeError::Lifecycle)?;
        inner.lifecycle = Lifecycle::Fresh;
        inner.enrollment.discard_pending();
        checkpoint.consume();

        Ok(Self { inner, checkpoint })
    }

    /// Save in the host's pre-upgrade transaction. Success closes the running
    /// instance. Supply the trusted canister version from the pre-upgrade hook.
    /// The encoded checkpoint is limited to 8 MiB. On error the host
    /// must abort the upgrade, retaining the heap.
    pub fn seal(&mut self, canister_version: u64) -> Result<(), UpgradeError> {
        if self.inner.lifecycle == Lifecycle::Closed {
            return Err(UpgradeError::Lifecycle);
        }
        self.checkpoint.save(
            CHECKPOINT_DOMAIN,
            canister_version,
            &(SavedRef(&self.inner), self.inner.extension_digest()),
        )?;
        self.inner.close();

        Ok(())
    }

    /// Access the current Auth instance under the host transaction.
    pub fn state(&mut self) -> &mut MemoryReceiver {
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

    fn validate_checkpoint(&self) -> Result<[u8; 32], UpgradeError> {
        let bad = UpgradeError::InvalidCheckpoint;
        let cfg = self.config;
        if !self
            .enrollment
            .matches(cfg.enrollment, cfg.audience, self.generation)
            || self.sessions != self.session_expiry.len()
            || self.replays.len() != self.replay_expiry.len()
            || self.unknown != self.unknown_expiry.len()
            || self.unknown > cfg.unknown_logout.max_denials
            || self.callers.len().checked_sub(self.unknown) != Some(self.slots.len())
            || self.slots.len() > cfg.login.max_admitted_logins
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
        for (caller, row) in &self.callers {
            policy::check_record(&row.login, cfg.audience).map_err(|_| bad)?;
            if caller != &row.login.caller {
                return Err(bad);
            }
            match row.slot() {
                Some(slot)
                    if self.slots.get(&slot) == Some(caller)
                        && slot.2 < cfg.login.max_device_slots => {}
                None if row.session.is_none()
                    && self.unknown_expiry.contains(&(row.login.until_ns, *caller)) =>
                {
                    unknown += 1
                }
                _ => return Err(bad),
            }
            if let Some(s) = &row.session {
                if row.login.denied || !self.session_expiry.contains(&(s.expires_at_ns, *caller)) {
                    return Err(bad);
                }
                let facts = row.session_facts().ok_or(bad)?;
                policy::check_session_shape(&facts).map_err(|_| bad)?;
                if s.logical_bytes(*caller, facts.audience, facts.login, facts.scopes)
                    .map_err(|_| bad)?
                    != s.bytes
                    || facts.established_at_ns > self.observed_at_ns
                    || facts.authority_generation > self.generation
                {
                    return Err(bad);
                }
                bytes = bytes.checked_add(s.bytes).ok_or(bad)?;
                hash_extension(&mut digest, &facts);
                sessions += 1;
            }
        }
        let mut accounts = BTreeMap::new();
        for (fingerprint, replay) in &self.replays {
            if !self
                .replay_expiry
                .contains(&(replay.remove_at_ns, *fingerprint))
                || !(1..=512).contains(&replay.bytes)
            {
                return Err(bad);
            }
            bytes = bytes.checked_add(replay.bytes).ok_or(bad)?;
            let count = accounts.entry(replay.account).or_insert(0usize);
            *count += 1;
            if *count > cfg.capacity.max_replays_per_account {
                return Err(bad);
            }
        }
        if accounts != self.accounts
            || bytes != self.encoded_bytes
            || sessions != self.sessions
            || unknown != self.unknown
        {
            return Err(bad);
        }
        Ok(digest.finalize().into())
    }
}
