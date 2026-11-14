//! Indexed issuer intent, multi-device login, renewal and original-result custody.
//!
//! One operation reads its selected login/intent and at most eight account slots.
//! No routine operation exports, validates or encodes unrelated retained history.
//! Authentication of account/authority and grant approval stay explicit host
//! inputs. Nonces, approval and completed results are recorded once, before the
//! next external effect. Private signing keys are never stored here.

mod keys;
mod model;
mod prepare;
mod renewal;
mod retention;

use super::StoreMemory;
use super::{
    StoreError,
    records::{Key, Records, Writes, size},
};
use ic_auth_protocol_types::{PortableClaims, PortableMechanism, Principal};
use ic_storage::Memory;
use model::{Header, Login, LoginId, Operation};
pub use model::{
    IssueRequest, IssuedCredential, IssuerAuthority, IssuerConfig, IssuerIntent, IssuerOccupancy,
    IssuerPolicy, IssuerPruned, IssuerStatus, IssuerTicket, RenewRequest, RenewalSelection,
    RetainedRenewal,
};

const SCHEMA: u64 = 0x41555448495303;

/// Auth owns the schema/algorithms; the host supplies one exclusive checked lease
/// and the transaction boundary. All returned errors preserve records and totals,
/// except trusted-clock observations and fresh intent work charges. Collection faults trap.
pub struct IssuerStore<M: Memory> {
    records: Records<M>,
    header: Header,
}
impl<M: Memory> IssuerStore<M> {
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn create(
        memory: StoreMemory<M>,
        config: IssuerConfig,
        now_ns: u64,
    ) -> Result<Self, StoreError> {
        Self::check_config(&config)?;
        let header = Header {
            issuance_budget: crate::admission_budget::SessionAdmissionBudget::for_issuance(
                config.issuance_burst,
                config.issuance_refill_interval_ns,
                now_ns,
            )
            .map_err(|_| StoreError::Invalid)?,
            schema: SCHEMA,
            config,
            epoch: 1,
            boot: 1,
            generation: 0,
            observed_ns: now_ns,
            occupancy: IssuerOccupancy::default(),
            renewal_indexes: 0,
            idle_logins: 0,
            operation_accounts: 0,
        };
        let mut writes = Writes::new();
        writes.put(Key::new(keys::HEADER), &header)?;
        let mut records = Records::create(memory)?;
        records.commit(writes);
        Ok(Self { records, header })
    }

    /// Open without a scan or reset. Protected backing provenance is host-owned;
    /// each selected row/index is checked when used. No signing key is restored.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn open(
        memory: StoreMemory<M>,
        expected: &IssuerConfig,
        now_ns: u64,
    ) -> Result<Self, StoreError> {
        Self::check_config(expected)?;
        let records = Records::open(memory)?;
        let header: Header = records
            .get(&Key::new(keys::HEADER))?
            .ok_or(StoreError::Corrupt)?;
        if header.schema != SCHEMA
            || !header.issuance_budget.is_operations()
            || !header.issuance_budget.stored_policy_valid(
                expected.issuance_burst,
                expected.issuance_refill_interval_ns,
                header.observed_ns,
            )
            || &header.config != expected
            || header.epoch == 0
            || header.boot == 0
            || header.occupancy.logins > expected.max_logins
            || header.occupancy.operations > expected.max_operations
            || header.occupancy.reserved_bytes > expected.max_reserved_bytes
            || header.renewal_indexes > header.occupancy.operations
            || header.idle_logins > header.occupancy.logins
            || header.operation_accounts > header.occupancy.operations
            || records.len()
                != 1 + 3 * header.occupancy.logins
                    + 2 * header.occupancy.operations
                    + header.renewal_indexes
                    + header.idle_logins
                    + header.operation_accounts
        {
            return Err(StoreError::Corrupt);
        }
        let mut store = Self { records, header };
        store.observe(now_ns)?;
        Ok(store)
    }

    fn check_config(config: &IssuerConfig) -> Result<(), StoreError> {
        if config.issuance_burst == 0
            || config.issuance_refill_interval_ns == 0
            || config.issuer == Principal::anonymous()
            || config.max_logins == 0
            || config.max_operations == 0
            || config.max_logins > u32::MAX as u64
            || config.max_operations > u32::MAX as u64
            || config.max_reserved_bytes == 0
            || config.max_operations_per_account == 0
            || config.max_operations_per_account > config.max_operations
            || !(1..=ic_auth_protocol_types::limits::MAX_DEVICE_SLOTS)
                .contains(&config.max_devices_per_account)
        {
            return Err(StoreError::Invalid);
        }
        Ok(())
    }

    #[must_use]
    /// Current operation epoch, fencing new work from closed epochs.
    pub fn epoch(&self) -> u64 {
        self.header.epoch
    }
    #[must_use]
    /// Read retained counters without scanning logins or operations.
    pub fn occupancy(&self) -> IssuerOccupancy {
        self.header.occupancy
    }
    #[must_use]
    /// Durable trusted-clock fence preserved across reopen and restart.
    pub fn observed_at_ns(&self) -> u64 {
        self.header.observed_ns
    }
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn intent(&self, ticket: &IssuerTicket) -> Result<IssuerIntent, StoreError> {
        let row = self
            .operation(ticket.account, ticket.epoch, &ticket.operation_id)?
            .ok_or(StoreError::Denied)?;
        if &row.ticket != ticket {
            return Err(StoreError::Denied);
        }
        Ok(Self::project_intent(row))
    }

    fn project_intent(row: Operation) -> IssuerIntent {
        IssuerIntent {
            ticket: row.ticket,
            request: row.request,
            policy: row.policy,
            nonce: row.nonce,
            approval: row.approval,
            admission_deadline_ns: row.admission_ns,
            receipt_deadline_ns: row.receipt_ns,
        }
    }

    /// Explicit lifecycle fence. Old completed results stay readable; callbacks
    /// from an earlier boot cannot create a new signature or completion.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn restart(&mut self, now_ns: u64) -> Result<(), StoreError> {
        self.observe(now_ns)?;
        let mut header = self.header.clone();
        header.boot = header.boot.checked_add(1).ok_or(StoreError::Capacity)?;
        header.epoch = header.epoch.checked_add(1).ok_or(StoreError::Capacity)?;
        self.publish(header, Writes::new())
    }

    /// Exact account-authenticated lookup. Never creates work or revives expired
    /// credentials. The host must bind `account` to its authenticated caller.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn retrieve(
        &self,
        account: Principal,
        epoch: u64,
        operation_id: [u8; 32],
    ) -> Result<Option<IssuerStatus>, StoreError> {
        self.operation(account, epoch, &operation_id)?
            .map(|row| self.status(&row))
            .transpose()
    }

    /// Persist a fresh random nonce before the authority/signing effect. Repeated
    /// callbacks keep the original nonce; conflicting callbacks refuse.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn challenge(
        &mut self,
        ticket: &IssuerTicket,
        current: IssuerAuthority,
        nonce: [u8; 16],
        now_ns: u64,
    ) -> Result<(), StoreError> {
        self.observe(now_ns)?;
        let mut row = self.pending(ticket, current, now_ns)?;
        if let Some(old) = row.nonce {
            return if old == nonce {
                Ok(())
            } else {
                Err(StoreError::Conflict)
            };
        }
        row.nonce = Some(nonce);
        self.write_operation(&row)
    }

    /// Host-verified response commitment for this exact intent/challenge. The
    /// caller verifies its authority-specific response before invoking this API.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn approve(
        &mut self,
        ticket: &IssuerTicket,
        current: IssuerAuthority,
        response_hash: [u8; 32],
        now_ns: u64,
    ) -> Result<(), StoreError> {
        self.observe(now_ns)?;
        let mut row = self.pending(ticket, current, now_ns)?;
        if row.nonce.is_none() {
            return Err(StoreError::Pending);
        }
        if let Some(old) = row.approval {
            return if old == response_hash {
                Ok(())
            } else {
                Err(StoreError::Conflict)
            };
        }
        row.approval = Some(response_hash);
        self.write_operation(&row)
    }

    /// Canonical original claims for a host-authorized signing effect. This is
    /// NOT receiver verification or application authorization.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn claims_for_signing(
        &mut self,
        ticket: &IssuerTicket,
        current: IssuerAuthority,
        now_ns: u64,
    ) -> Result<Vec<u8>, StoreError> {
        self.observe(now_ns)?;
        let row = self.pending(ticket, current, now_ns)?;
        if row.approval.is_none() {
            return Err(StoreError::Pending);
        }
        if row.result.is_some() {
            return Err(StoreError::Conflict);
        }
        crate::portable::encode_claims(&Self::claims(&self.header.config, &row)?)
            .map_err(|_| StoreError::Invalid)
    }

    /// Retain an original local signing result. Ed25519 supplies exactly 64 bytes;
    /// certified issuance supplies None and composes its leaf with the host's
    /// existing signature store in the same synchronous host transaction.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn complete(
        &mut self,
        ticket: &IssuerTicket,
        current: IssuerAuthority,
        signature: Option<Vec<u8>>,
        now_ns: u64,
    ) -> Result<IssuedCredential, StoreError> {
        self.observe(now_ns)?;
        let mut row = self
            .operation(ticket.account, ticket.epoch, &ticket.operation_id)?
            .ok_or(StoreError::Denied)?;
        if &row.ticket != ticket {
            return Err(StoreError::Denied);
        }
        if row.result.is_some() {
            let IssuerStatus::Completed(result) = self.status(&row)? else {
                unreachable!("completed row")
            };
            return if result.signature == signature {
                Ok(result)
            } else {
                Err(StoreError::Conflict)
            };
        }
        self.check_pending(&row, current, now_ns)?;
        if row.approval.is_none() {
            return Err(StoreError::Pending);
        }
        if !matches!(
            (row.ticket.authority.mechanism, &signature),
            (PortableMechanism::Certified, None)
        ) && !matches!((row.ticket.authority.mechanism, &signature), (PortableMechanism::Ed25519, Some(bytes)) if bytes.len() == 64)
        {
            return Err(StoreError::Invalid);
        }
        let result = IssuedCredential {
            claims: crate::portable::encode_claims(&Self::claims(&self.header.config, &row)?)
                .map_err(|_| StoreError::Invalid)?,
            signature,
            admission_deadline_ns: row.admission_ns,
        };
        row.result = Some(result.clone());
        if size(&row)? > row.reserved_bytes {
            return Err(StoreError::Corrupt);
        }
        let mut writes = Writes::new();
        writes.put(keys::ticket(ticket), &row)?;
        if let Some(sequence) = row.renewal {
            let mut login = self.login(&row.login)?.ok_or(StoreError::Corrupt)?;
            if login.sequence != sequence {
                return Err(StoreError::Conflict);
            }
            login.completed = true;
            writes.put(keys::login(&login.id), &login)?;
        } else {
            let mut login = self.login(&row.login)?.ok_or(StoreError::Corrupt)?;
            if login.sequence == 0 {
                login.completed = true;
                writes.put(keys::login(&login.id), &login)?;
            }
        }
        self.publish(self.header.clone(), writes)?;
        Ok(result)
    }

    fn operation(
        &self,
        account: Principal,
        epoch: u64,
        id: &[u8; 32],
    ) -> Result<Option<Operation>, StoreError> {
        let row: Option<Operation> = self.records.get(&keys::operation(account, epoch, id))?;
        if let Some(row) = &row
            && (row.ticket.account != account
                || row.ticket.epoch != epoch
                || &row.ticket.operation_id != id
                || row.login.account != account
                || row.ticket.operation_id != row.request.operation_id
                || row.ticket.epoch != row.request.epoch
                || row.login.epoch != row.request.login_epoch
                || row.login.id != row.request.login_id
                || row.slot >= self.header.config.max_devices_per_account
                || row.device_generation == 0
                || row.device_generation > self.header.generation
                || row.started_ns >= row.deadline_ns
                || row.issued_ns < row.started_ns
                || row.admission_ns <= row.issued_ns
                || row.admission_ns > row.deadline_ns
                || row.receipt_ns <= row.issued_ns
                || row.remove_ns < row.admission_ns.max(row.receipt_ns)
                || size(row)? > row.reserved_bytes
                || !self
                    .records
                    .contains(&keys::operation_expiry(row.remove_ns, &row.ticket)))
        {
            return Err(StoreError::Corrupt);
        }
        Ok(row)
    }
    fn login(&self, id: &LoginId) -> Result<Option<Login>, StoreError> {
        let login: Option<Login> = self.records.get(&keys::login(id))?;
        if let Some(login) = &login {
            let slot: Option<LoginId> = self.records.get(&keys::slot(id.account, login.slot))?;
            let browser: Option<LoginId> = self
                .records
                .get(&keys::browser(id.account, &login.browser_public_key))?;
            if &login.id != id
                || login.slot >= self.header.config.max_devices_per_account
                || login.generation == 0
                || login.generation > self.header.generation
                || login.started_ns >= login.deadline_ns
                || slot.as_ref() != Some(id)
                || browser.as_ref() != Some(id)
                || self.records.contains(&keys::login_expiry(login)) != (login.references == 0)
            {
                return Err(StoreError::Corrupt);
            }
        }
        Ok(login)
    }
    fn pending(
        &self,
        ticket: &IssuerTicket,
        current: IssuerAuthority,
        now_ns: u64,
    ) -> Result<Operation, StoreError> {
        let row = self
            .operation(ticket.account, ticket.epoch, &ticket.operation_id)?
            .ok_or(StoreError::Denied)?;
        if &row.ticket != ticket {
            return Err(StoreError::Denied);
        }
        self.check_pending(&row, current, now_ns)?;
        Ok(row)
    }
    fn check_pending(
        &self,
        row: &Operation,
        current: IssuerAuthority,
        now_ns: u64,
    ) -> Result<(), StoreError> {
        if row.ticket.boot != self.header.boot || row.ticket.authority != current {
            return Err(StoreError::Denied);
        }
        if now_ns
            >= row
                .receipt_ns
                .min(row.admission_ns)
                .min(current.expires_at_ns)
        {
            return Err(StoreError::Expired);
        }
        let login = self.login(&row.login)?.ok_or(StoreError::Corrupt)?;
        if login.revoked
            || login.generation != row.device_generation
            || login.slot != row.slot
            || login.browser_public_key != row.request.browser_public_key
            || login.started_ns != row.started_ns
            || login.deadline_ns != row.deadline_ns
        {
            return Err(StoreError::Denied);
        }
        Ok(())
    }
    fn status(&self, row: &Operation) -> Result<IssuerStatus, StoreError> {
        if let Some(result) = &row.result {
            if result.claims
                != crate::portable::encode_retained_claims(&Self::claims(&self.header.config, row)?)
                    .map_err(|_| StoreError::Corrupt)?
                || result.admission_deadline_ns != row.admission_ns
                || !matches!(
                    (row.ticket.authority.mechanism, &result.signature),
                    (PortableMechanism::Certified, None)
                ) && !matches!((row.ticket.authority.mechanism, &result.signature), (PortableMechanism::Ed25519, Some(bytes)) if bytes.len() == 64)
            {
                return Err(StoreError::Corrupt);
            }
            Ok(IssuerStatus::Completed(result.clone()))
        } else {
            Ok(IssuerStatus::Pending(row.ticket.clone()))
        }
    }
    fn claims(config: &IssuerConfig, row: &Operation) -> Result<PortableClaims, StoreError> {
        Ok(PortableClaims {
            mechanism: row.ticket.authority.mechanism,
            audience: config.audience,
            issuer: config.issuer,
            key_binding_id: row.ticket.authority.key_binding_id,
            browser_public_key: row.request.browser_public_key,
            account_subject: row.ticket.account,
            device_slot: row.slot,
            device_generation: row.device_generation,
            login_started_at_ns: row.started_ns,
            login_deadline_ns: row.deadline_ns,
            issued_at_ns: row.issued_ns,
            admission_expires_at_ns: row.admission_ns,
            nonce: row.nonce.ok_or(StoreError::Pending)?,
            grants: row.request.grants.clone(),
        })
    }
    fn write_operation(&mut self, row: &Operation) -> Result<(), StoreError> {
        if size(row)? > row.reserved_bytes {
            return Err(StoreError::Corrupt);
        }
        let mut writes = Writes::new();
        writes.put(keys::ticket(&row.ticket), row)?;
        self.publish(self.header.clone(), writes)
    }
    fn observe(&mut self, now_ns: u64) -> Result<(), StoreError> {
        if now_ns < self.header.observed_ns {
            return Err(StoreError::Clock);
        }
        if now_ns != self.header.observed_ns {
            let mut header = self.header.clone();
            header.observed_ns = now_ns;
            self.publish(header, Writes::new())?;
        }
        Ok(())
    }
    fn publish(&mut self, header: Header, mut writes: Writes) -> Result<(), StoreError> {
        writes.put(Key::new(keys::HEADER), &header)?;
        self.records.commit(writes);
        self.header = header;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
