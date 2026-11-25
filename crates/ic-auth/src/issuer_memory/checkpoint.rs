//! Stable issuer checkpoint format and upgrade lifecycle.

use super::{
    Account, Approval, BTreeMap, BTreeSet, Completion, Expiry, IssuerAuthority, IssuerConfig,
    IssuerPolicy, IssuerSigning, IssuerTicket, IssuerUsage, MemoryIssuer, Phase, Principal,
    WorkBucket, expiry,
};
use crate::stable_upgrade::{Checkpoint, UpgradeError};
use ic_storage::{DefaultMemoryImpl, Memory, RuntimeMemory};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const CHECKPOINT_DOMAIN: &[u8] = b"ic-auth.issuer.v3";

///
/// PolicyLimits
///
/// Encodes retained policy with a bounded application extension.
///

#[derive(Serialize, Deserialize)]
#[serde(remote = "IssuerPolicy")]
pub(super) struct PolicyLimits {
    #[serde(with = "crate::stable_upgrade::extension")]
    ext: Option<Vec<u8>>,
    login_ttl_ns: u64,
    proof_ttl_ns: u64,
    receipt_timeout_ns: u64,
    reconciliation_retention_ns: u64,
}

///
/// TicketFields
///
/// Defines the serialized account, epoch and operation ID of an issuer ticket.
///

#[derive(Serialize, Deserialize)]
#[serde(remote = "IssuerTicket")]
pub(super) struct TicketFields {
    account: Principal,
    epoch: u64,
    operation_id: [u8; 32],
    authority: IssuerAuthority,
    boot: u64,
}

///
/// Fields
///
/// Defines the serialized checkpoint fields for the in-memory state.
///

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

///
/// SavedRef
///
/// Borrows the in-memory state for checkpoint encoding.
///

#[derive(Serialize)]
struct SavedRef<'a>(#[serde(with = "Fields")] &'a MemoryIssuer);

///
/// Saved
///
/// Reconstructs owned state from the checkpoint fields.
///

#[derive(Deserialize)]
struct Saved(#[serde(with = "Fields")] MemoryIssuer);

///
/// StableIssuer
///
/// Owns a single-use upgrade checkpoint and the indexed [MemoryIssuer] used
/// for ordinary operations between upgrades.
///

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
    /// Supply the trusted post-upgrade canister version. Only the immediate successor
    /// of the sealing version can reopen. Abort the upgrade transaction on refusal.
    /// Success advances the issuer epoch and fences callbacks from the previous boot.
    pub fn reopen(
        memory: RuntimeMemory<M>,
        config: IssuerConfig,
        now_ns: u64,
        canister_version: u64,
    ) -> Result<Self, UpgradeError> {
        let mut checkpoint = Checkpoint::load(memory)?;
        let (Saved(mut inner), commitment): (Saved, [u8; 32]) =
            Checkpoint::<M>::decode(checkpoint.payload(CHECKPOINT_DOMAIN, canister_version)?)?;
        if inner.config != config {
            return Err(UpgradeError::Configuration);
        }
        if inner.validate_checkpoint()? != commitment {
            return Err(UpgradeError::InvalidCheckpoint);
        }
        inner.restart(now_ns).map_err(|_| UpgradeError::Lifecycle)?;
        checkpoint.consume();

        Ok(Self { inner, checkpoint })
    }

    /// Save in the host's pre-upgrade transaction. Success closes the running
    /// instance. Supply the trusted canister version from the pre-upgrade hook.
    /// The encoded checkpoint is limited to 8 MiB. On error the host
    /// must abort the upgrade, retaining the heap.
    pub fn seal(&mut self, canister_version: u64) -> Result<(), UpgradeError> {
        if !self.inner.open {
            return Err(UpgradeError::Lifecycle);
        }
        self.checkpoint.save(
            CHECKPOINT_DOMAIN,
            canister_version,
            &(SavedRef(&self.inner), self.inner.renewal_digest()),
        )?;
        self.inner.close();

        Ok(())
    }

    /// Access the current Auth instance under the host transaction.
    pub fn state(&mut self) -> &mut MemoryIssuer {
        &mut self.inner
    }
}

impl<M: Memory> AsRef<MemoryIssuer> for StableIssuer<M> {
    fn as_ref(&self) -> &MemoryIssuer {
        &self.inner
    }
}

fn hash_renewal(
    digest: &mut Sha256,
    account: Principal,
    (epoch, id): &super::Identity,
    login: &super::Login,
) {
    use super::RenewalProgress;
    let tag = match login.renewal {
        RenewalProgress::Initial => 0,
        RenewalProgress::InFlight(_) => 1,
        RenewalProgress::Completed(_) => 2,
        RenewalProgress::InitialOutcomeUnknown => 3,
        RenewalProgress::PrunedRenewal(_) => 4,
    };
    digest.update([account.as_slice().len() as u8]);
    digest.update(account.as_slice());
    digest.update(epoch.to_le_bytes());
    digest.update(id);
    digest.update([tag]);
    digest.update(login.renewal.sequence().to_le_bytes());
}

impl MemoryIssuer {
    // Bind consumed sequences independently of operation rows that pruning can remove.
    fn renewal_digest(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        for (&account, owner) in &self.accounts {
            for (id, login) in &owner.logins {
                hash_renewal(&mut digest, account, id, login);
            }
        }
        digest.finalize().into()
    }

    fn validate_checkpoint(&self) -> Result<[u8; 32], UpgradeError> {
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
        let mut digest = Sha256::new();
        let mut generations = BTreeSet::new();
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
                let authority = op.ticket.authority;
                if authority.generation == 0
                    || authority.key_binding_id == [0; 32]
                    || op.issued < login.started
                    || op.issued >= login.deadline
                    || op.issued >= authority.expires_at_ns
                    || op.issued > self.observed
                    || login
                        .policy
                        .deadlines(op.issued, login.deadline, authority.expires_at_ns)
                        != Some((op.admission, op.receipt, op.remove_at))
                {
                    return Err(bad);
                }
                if op
                    .ext
                    .as_ref()
                    .is_some_and(|ext| Some(ext) != login.policy.ext.as_ref())
                    || account != op.ticket.account
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
                    let completed = matches!(op.phase, Phase::Completed(..));
                    if sequence == 0
                        || sequence > login.renewal.sequence()
                        || (!completed
                            && !matches!(login.renewal, super::RenewalProgress::InFlight(n) if n == sequence))
                        || (completed
                            && sequence == login.renewal.sequence()
                            && !matches!(login.renewal, super::RenewalProgress::Completed(_)))
                        || owner.renewals.get(&(op.login, sequence)) != Some(&id)
                    {
                        return Err(bad);
                    }
                    renewals += 1;
                }
                if op.renewal.is_none()
                    && matches!(op.phase, Phase::Completed(..))
                    && matches!(
                        login.renewal,
                        super::RenewalProgress::Initial
                            | super::RenewalProgress::InitialOutcomeUnknown
                    )
                {
                    return Err(bad);
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
                if matches!(login.renewal, super::RenewalProgress::InFlight(n) if n == 0 || !owner.renewals.contains_key(&(id, n)))
                    || matches!(login.renewal, super::RenewalProgress::PrunedRenewal(0))
                    || !login.policy.valid_limits()
                    || !generations.insert(login.generation)
                    || login.slot >= cfg.max_devices_per_account
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
                        .contains(&expiry(login.idle_prune_at(), account, id))
                    {
                        return Err(bad);
                    }
                    idle += 1;
                }
                hash_renewal(&mut digest, account, &id, login);
                totals.logins = totals.logins.checked_add(1).ok_or(bad)?;
            }
        }
        if totals != self.usage || idle != self.idle_logins.len() {
            return Err(bad);
        }
        Ok(digest.finalize().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_storage::{StableKey, VectorMemory, testing::fixture_runtime};

    #[test]
    fn lowered_sequence_refuses_even_after_its_operation_has_been_pruned() {
        let mut issuer = super::super::tests::completed_login();
        issuer.prune(50, 32).unwrap();
        assert_eq!(issuer.usage().operations, 0);
        let account = Principal::from_slice(&[2]);
        let id = (1, [2; 32]);
        let login = issuer
            .accounts
            .get_mut(&account)
            .unwrap()
            .logins
            .get_mut(&id)
            .unwrap();
        login.renewal = super::super::RenewalProgress::Completed(1);
        let commitment = issuer.renewal_digest();
        issuer
            .accounts
            .get_mut(&account)
            .unwrap()
            .logins
            .get_mut(&id)
            .unwrap()
            .renewal = super::super::RenewalProgress::Completed(0);
        let key = "auth.issuer.v1";
        let runtime = fixture_runtime(
            VectorMemory::default(),
            "auth",
            "auth.",
            &[key],
            Default::default(),
        )
        .unwrap();
        let key = StableKey::parse(key).unwrap();
        let lease = || runtime.open_memory(&key).unwrap();
        let mut cell = Checkpoint::create(lease()).unwrap();
        cell.save(CHECKPOINT_DOMAIN, 10, &(SavedRef(&issuer), commitment))
            .unwrap();
        drop(cell);
        assert!(matches!(
            StableIssuer::reopen(lease(), issuer.config, 51, 11),
            Err(UpgradeError::InvalidCheckpoint)
        ));
    }
}
