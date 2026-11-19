use super::{
    Account, Approval, BTreeMap, BTreeSet, Completion, Expiry, IssuerAuthority, IssuerConfig,
    IssuerSigning, IssuerTicket, IssuerUsage, MemoryIssuer, Phase, Principal, WorkBucket, expiry,
};
use crate::stable_upgrade::{Checkpoint, UpgradeError};
use ic_storage::{DefaultMemoryImpl, Memory, RuntimeMemory};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(remote = "IssuerTicket")]
pub(super) struct TicketFields {
    account: Principal,
    epoch: u64,
    operation_id: [u8; 32],
    authority: IssuerAuthority,
    boot: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "MemoryIssuer")]
struct Fields {
    config: IssuerConfig,
    accounts: BTreeMap<Principal, Account>,
    operation_expiry: BTreeSet<Expiry>,
    idle_logins: BTreeSet<Expiry>,
    usage: IssuerUsage,
    budget: WorkBucket,
    observed: u64,
    epoch: u64,
    boot: u64,
    generation: u64,
    open: bool,
}

#[derive(Serialize)]
struct SavedRef<'a>(#[serde(with = "Fields")] &'a MemoryIssuer);
#[derive(Deserialize)]
struct Saved(#[serde(with = "Fields")] MemoryIssuer);

/// Issuer with an exclusive, single-use upgrade checkpoint.
/// Ordinary operations use the same indexed state as [MemoryIssuer].
pub struct StableIssuer<M: Memory = DefaultMemoryImpl> {
    inner: MemoryIssuer,
    checkpoint: Checkpoint<M>,
}

impl<M: Memory> StableIssuer<M> {
    /// Initialize a genuinely fresh installation with a host-owned, empty lease.
    pub fn new_fresh(
        memory: RuntimeMemory<M>,
        config: IssuerConfig,
        now_ns: u64,
    ) -> Result<Self, UpgradeError> {
        let inner =
            MemoryIssuer::new_fresh(config, now_ns).map_err(|_| UpgradeError::Configuration)?;
        Ok(Self {
            inner,
            checkpoint: Checkpoint::create(memory)?,
        })
    }

    /// Consume a committed upgrade checkpoint from this module's named allocation.
    /// The host must abort the upgrade transaction on any refusal.
    pub fn reopen(
        memory: RuntimeMemory<M>,
        config: IssuerConfig,
        now_ns: u64,
    ) -> Result<Self, UpgradeError> {
        let mut checkpoint = Checkpoint::load(memory)?;
        let Saved(mut inner) = checkpoint.read(b"ic-auth.issuer.v1")?;
        if inner.config != config {
            return Err(UpgradeError::Configuration);
        }
        inner.validate_checkpoint()?;
        inner.restart(now_ns).map_err(|_| UpgradeError::Lifecycle)?;
        checkpoint.consume();
        Ok(Self { inner, checkpoint })
    }

    /// Save in the host's pre-upgrade transaction. Success closes the running
    /// instance. On error the host must abort the upgrade, retaining the heap.
    pub fn seal(&mut self) -> Result<(), UpgradeError> {
        if !self.inner.open {
            return Err(UpgradeError::Lifecycle);
        }
        self.checkpoint
            .save(b"ic-auth.issuer.v1", &SavedRef(&self.inner))?;
        self.inner.close();
        Ok(())
    }

    /// Access the current Auth instance under the host transaction.
    pub fn state(&mut self) -> &mut MemoryIssuer {
        &mut self.inner
    }
}

impl MemoryIssuer {
    fn validate_checkpoint(&self) -> Result<(), UpgradeError> {
        let bad = UpgradeError::InvalidCheckpoint;
        let cfg = self.config;
        if self.epoch == 0
            || self.boot == 0
            || self.usage.operations as usize != self.operation_expiry.len()
            || self.usage.operations > cfg.max_operations
            || self.usage.logins > cfg.max_logins
            || self.usage.result_bytes > cfg.max_result_bytes
            || !self.budget.matches(
                cfg.issuance_burst,
                cfg.issuance_refill_interval_ns,
                self.observed,
            )
        {
            return Err(bad);
        }
        let mut totals = IssuerUsage::default();
        let mut idle = 0;
        for (&account, owner) in &self.accounts {
            if owner.logins.is_empty()
                || owner.logins.len() > usize::from(cfg.max_devices_per_account)
                || owner.operations.len() > cfg.max_operations_per_account as usize
            {
                return Err(bad);
            }
            let mut references = BTreeMap::new();
            let mut renewals = 0;
            for (&id, op) in &owner.operations {
                let login = owner.logins.get(&op.login).ok_or(bad)?;
                if account != op.ticket.account
                    || id != (op.ticket.epoch, op.ticket.operation_id)
                    || op.ticket.boot == 0
                    || op.ticket.boot > self.boot
                    || op.ticket.epoch > self.epoch
                    || !self
                        .operation_expiry
                        .contains(&expiry(op.remove_at, account, id))
                {
                    return Err(bad);
                }
                if let Some(sequence) = op.renewal {
                    if owner.renewals.get(&(op.login, sequence)) != Some(&id) {
                        return Err(bad);
                    }
                    renewals += 1;
                }
                let retained = match &op.phase {
                    Phase::Approved(a) | Phase::Dispatched(a) | Phase::Completed(a, _) => Some(a),
                    _ => None,
                };
                let nonce = retained.map_or([0; 16], |a: &Approval| a.nonce);
                let claims = self.claims(op, login, nonce).map_err(|_| bad)?;
                let size = claims.len() as u64
                    + if op.ticket.authority.signing == IssuerSigning::Ed25519 {
                        64
                    } else {
                        0
                    };
                if size != op.result_bytes
                    || retained.is_some_and(|a| a.claims != claims)
                    || matches!(
                        (&op.phase, op.ticket.authority.signing),
                        (
                            Phase::Completed(_, Completion::Certified),
                            IssuerSigning::Ed25519
                        ) | (
                            Phase::Completed(_, Completion::Ed25519(_)),
                            IssuerSigning::Certified { .. }
                        )
                    )
                {
                    return Err(bad);
                }
                *references.entry(op.login).or_insert(0u32) += 1;
                totals.operations = totals.operations.checked_add(1).ok_or(bad)?;
                totals.result_bytes = totals.result_bytes.checked_add(size).ok_or(bad)?;
            }
            if renewals != owner.renewals.len() {
                return Err(bad);
            }
            let mut slots = BTreeSet::new();
            for (&id, login) in &owner.logins {
                if login.slot >= cfg.max_devices_per_account
                    || !slots.insert(login.slot)
                    || login.started.checked_add(login.policy.login_ttl_ns) != Some(login.deadline)
                    || login.generation == 0
                    || login.generation > self.generation
                    || login.references != references.get(&id).copied().unwrap_or(0)
                {
                    return Err(bad);
                }
                if login.references == 0 {
                    if !self
                        .idle_logins
                        .contains(&expiry(login.deadline, account, id))
                    {
                        return Err(bad);
                    }
                    idle += 1;
                }
                totals.logins = totals.logins.checked_add(1).ok_or(bad)?;
            }
        }
        if totals != self.usage || idle != self.idle_logins.len() {
            return Err(bad);
        }
        Ok(())
    }
}

impl<M: Memory> AsRef<MemoryIssuer> for StableIssuer<M> {
    fn as_ref(&self) -> &MemoryIssuer {
        &self.inner
    }
}
