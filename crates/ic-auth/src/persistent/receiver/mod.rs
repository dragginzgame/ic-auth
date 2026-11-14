//! Durable caller-bound sessions using the existing verification/session engine.
//!
//! Per-operation ledger snapshots contain at most two selected records. Counts,
//! expiry indexes, slot fences and replay evidence remain durable and Auth-owned.
//! No whole-store heap mirror or upgrade-time reconstruction is required.

mod inspection;
mod keys;
mod model;
mod retention;
mod store;

use super::StoreMemory;
use super::{
    StoreError,
    records::{Key, Records, Writes},
};
use crate::{
    admission_budget::SessionAdmissionBudget,
    portable::{LoginBinding, LoginRecordState, LoginRevocationChange, MemoryLoginRevocations},
    session::{
        self, Replay, Session, SessionContext, SessionError, SessionPolicy, SessionReader,
        SessionStore,
    },
};
use ic_auth_protocol_types::{LogoutResult, LogoutUnavailable, Principal};
use ic_storage::Memory;
pub use inspection::{MAX_INSPECTION_PAGE, MAX_INSPECTION_SCAN, SessionPage};
pub use model::{BudgetPolicy, LoginLimits, ReceiverConfig, ReceiverOccupancy, ReceiverPruned};
use model::{CallerRecord, Header};
use std::borrow::Cow;

const SCHEMA: u64 = 0x41555448525304;

/// Three fixed checked leases. Caller records have their own ordered table so
/// frequent authentication does not traverse maintenance/replay index pages.
pub struct ReceiverMemory<M: Memory> {
    /// Exclusive checked leases for records and protected metadata.
    pub state: StoreMemory<M>,
    /// Separate exclusive checked lease for the caller point-lookup index.
    pub callers: ic_storage::RuntimeMemory<M>,
}

/// Auth-owned durable receiver. The host serializes operations with current
/// authority and supplies the clock, caller, lifecycle and atomic transaction.
pub struct ReceiverStore<M: Memory> {
    records: Records<M>,
    header: Header,
}
impl<M: Memory> ReceiverStore<M> {
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn create(
        memory: ReceiverMemory<M>,
        config: ReceiverConfig,
        generation: u64,
        now_ns: u64,
    ) -> Result<Self, StoreError> {
        config.validate()?;
        let admission = SessionAdmissionBudget::for_verification(
            config.admission.burst,
            config.admission.refill_interval_ns,
            config.verification_costs,
            now_ns,
        )
        .map_err(|_| StoreError::Invalid)?;
        let logout = SessionAdmissionBudget::for_operations(
            config.logout.burst,
            config.logout.refill_interval_ns,
            now_ns,
        )
        .map_err(|_| StoreError::Invalid)?;
        let header = Header {
            schema: SCHEMA,
            config,
            generation,
            observed_ns: now_ns,
            counts: ReceiverOccupancy::default(),
            replay_accounts: 0,
            admission,
            logout,
        };
        let mut writes = Writes::new();
        writes.put(Key::new(keys::HEADER), &header)?;
        let mut records = Records::create_with_index(memory.state, memory.callers, keys::LOGIN)?;
        records.commit(writes);
        Ok(Self { records, header })
    }
    /// Opens only the current Auth schema. Missing/foreign data never initializes
    /// implicitly. Protected authority and backing provenance remain host-owned.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn open(
        memory: ReceiverMemory<M>,
        expected: &ReceiverConfig,
        now_ns: u64,
    ) -> Result<Self, StoreError> {
        expected.validate()?;
        let records = Records::open_with_index(memory.state, memory.callers, keys::LOGIN)?;
        let header: Header = records
            .get(&Key::new(keys::HEADER))?
            .ok_or(StoreError::Corrupt)?;
        let counts = header.counts;
        let rows = 1u64
            .checked_add(
                2u64.checked_mul(counts.sessions as u64)
                    .ok_or(StoreError::Corrupt)?,
            )
            .and_then(|n| n.checked_add(2u64.checked_mul(counts.replays as u64)?))
            .and_then(|n| {
                n.checked_add(2u64.checked_mul(
                    (counts.admitted_logins as u64).checked_add(counts.unknown_denials as u64)?,
                )?)
            })
            .and_then(|n| n.checked_add(header.replay_accounts as u64))
            .ok_or(StoreError::Corrupt)?;
        if header.schema != SCHEMA
            || !header.logout.is_operations()
            || !header.admission.has_costs(expected.verification_costs)
            || &header.config != expected
            || counts.sessions > expected.sessions[0]
            || counts.replays > expected.sessions[1]
            || counts.encoded_bytes > expected.sessions[3]
            || counts.admitted_logins > expected.logins.admitted
            || counts.unknown_denials > expected.logins.unknown
            || header.replay_accounts > counts.replays
            || records.len() != rows
            || !header.admission.stored_policy_valid(
                expected.admission.burst,
                expected.admission.refill_interval_ns,
                header.observed_ns,
            )
            || !header.logout.stored_policy_valid(
                expected.logout.burst,
                expected.logout.refill_interval_ns,
                header.observed_ns,
            )
        {
            return Err(StoreError::Corrupt);
        }
        let mut store = Self { records, header };
        store.observe(now_ns)?;
        Ok(store)
    }
    #[must_use]
    /// Read retained counters and byte accounting without scanning sessions.
    pub fn occupancy(&self) -> ReceiverOccupancy {
        self.header.counts
    }
    #[must_use]
    /// Current protected receiver authority generation.
    pub fn generation(&self) -> u64 {
        self.header.generation
    }
    #[must_use]
    /// Durable trusted-clock fence used to reject rollback.
    pub fn observed_at_ns(&self) -> u64 {
        self.header.observed_ns
    }

    /// Persist this increment in the same host transaction as any authority
    /// change. Retained sessions fail their generation check; replay is retained.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn advance_generation(&mut self) -> Result<u64, StoreError> {
        let mut header = self.header.clone();
        header.generation = header
            .generation
            .checked_add(1)
            .ok_or(StoreError::Capacity)?;
        let next = header.generation;
        self.publish(header, Writes::new())?;
        Ok(next)
    }

    /// Complete proof admission through the existing Auth verifier. The persistent
    /// receiver owns its two separate verification/logout rate budgets.
    #[cfg(any(
        feature = "portable-signature-verification",
        feature = "certified-proof-verification"
    ))]
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn admit(
        &mut self,
        credential: &ic_auth_protocol_types::PortableCredential,
        request: crate::session_request::SessionRequest,
        admission: &session::SessionAdmissionContext<'_>,
        policy: SessionPolicy,
    ) -> Result<session::SessionEstablished, SessionError> {
        let authority = admission.verification.claims().authority;
        self.observe(authority.now_ns).map_err(session_error)?;
        let decoded = crate::portable::DecodedPortableClaims::decode(&credential.claims)?;
        // Caller identity is immutable for this exact signed payload, including
        // retries. Refuse mismatches before touching caller/slot/session indexes.
        // Proof expiry stays in the engine so expired-proof retries still work.
        if decoded.browser_caller != authority.caller {
            return Err(crate::portable::PortableClaimsError::CallerMismatch.into());
        }
        let claims = &decoded.claims;
        let mut ledger = self
            .ledger(
                authority.caller,
                Some((claims.issuer, claims.account_subject, claims.device_slot)),
            )
            .map_err(session_error)?;
        let mut budget = self.header.admission.clone();
        let result = session::establish_decoded_session(
            self,
            &mut ledger,
            &mut budget,
            (decoded, &credential.proof),
            request,
            admission,
            policy,
        );
        // Observation above persists clock progress independently. Preserve every
        // changed budget (including failed verification), but exact retries and
        // repeated exhaustion at the same clock need no duplicate header write.
        if self.header.admission != budget {
            let mut header = self.header.clone();
            header.admission = budget;
            self.publish(header, Writes::new()).map_err(session_error)?;
        }
        result
    }

    /// Point authentication with current host authority, clock, revocation and
    /// operation scopes. Projection runs only after the existing engine accepts.
    /// No proof verification, renewal, scan or lease extension occurs here.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn authenticate<T>(
        &mut self,
        context: &SessionContext<'_>,
        policy: SessionPolicy,
        project: impl FnOnce(&Session) -> T,
    ) -> Result<T, SessionError> {
        context.validate(policy, self.header.config.audience)?;
        self.observe(context.authority.now_ns)
            .map_err(session_error)?;
        let caller = context.authority.caller;
        if !crate::portable::is_login_caller(caller) {
            return Err(crate::portable::LoginRevocationError::InvalidCaller.into());
        }
        let row = self.read_caller(caller).map_err(session_error)?;
        let (selected, login) = match row.as_ref() {
            Some(row) => {
                let (session, change) = self.decode_caller(caller, row).map_err(session_error)?;
                (session, change.live_binding(context.authority.now_ns)?)
            }
            None => (None, None),
        };
        let selected = store::SelectedReader {
            store: self,
            caller,
            selected,
        };
        session::authorize_snapshot(&selected, login, context, policy, project)
    }

    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn logout(&mut self, caller: Principal, now_ns: u64) -> LogoutResult {
        let result = (|| {
            self.observe(now_ns)?;
            let mut ledger = self.ledger(caller, None)?;
            let mut budget = self.header.logout.clone();
            let result = session::logout_session(self, &mut ledger, &mut budget, caller, now_ns);
            let mut header = self.header.clone();
            header.logout = budget;
            self.publish(header, Writes::new())?;
            Ok::<_, StoreError>(result)
        })();
        result.unwrap_or_else(|error| {
            LogoutResult::Unavailable(if error == StoreError::Clock {
                LogoutUnavailable::Clock
            } else {
                LogoutUnavailable::Storage
            })
        })
    }

    fn ledger(
        &self,
        caller: Principal,
        selected: Option<(Principal, Principal, u8)>,
    ) -> Result<MemoryLoginRevocations, StoreError> {
        let mut records = Vec::new();
        if let Some(record) = self.login(caller)? {
            records.push(record);
        }
        if let Some((issuer, subject, slot)) = selected {
            let owner: Option<Principal> =
                self.records
                    .get(&keys::slot(keys::LOGIN_SLOT, issuer, subject, slot))?;
            if let Some(owner) = owner.filter(|old| *old != caller) {
                records.push(self.login(owner)?.ok_or(StoreError::Corrupt)?);
            }
        }
        self.selected_ledger(records)
    }
    fn selected_ledger(
        &self,
        records: Vec<LoginRevocationChange>,
    ) -> Result<MemoryLoginRevocations, StoreError> {
        let config = self.header.config.logins;
        MemoryLoginRevocations::selected(
            self.header.config.audience,
            (
                config.admitted,
                config.unknown,
                config.maximum_ttl_ns,
                config.devices_per_account,
            ),
            self.header.observed_ns,
            (
                self.header.counts.admitted_logins,
                self.header.counts.unknown_denials,
            ),
            records,
        )
        .map_err(|_| StoreError::Corrupt)
    }
    fn login(&self, caller: Principal) -> Result<Option<LoginRevocationChange>, StoreError> {
        self.read_caller(caller)?
            .as_ref()
            .map(|row| {
                let change = self.login_from(caller, row)?;
                self.check_login_index(&change)?;
                Ok(change)
            })
            .transpose()
    }
    fn read_caller(&self, caller: Principal) -> Result<Option<CallerRecord>, StoreError> {
        self.records.get(&keys::login(caller))
    }
    fn login_from(
        &self,
        caller: Principal,
        row: &CallerRecord,
    ) -> Result<LoginRevocationChange, StoreError> {
        self.decode_caller(caller, row).map(|(_, login)| login)
    }
    fn decode_caller(
        &self,
        caller: Principal,
        row: &CallerRecord,
    ) -> Result<(Option<Session>, LoginRevocationChange), StoreError> {
        let (session, change) = match row {
            CallerRecord::Session(bytes) => {
                let session = Session::decode(bytes).map_err(|_| StoreError::Corrupt)?;
                if session.caller() != caller
                    || session.audience() != self.header.config.audience
                    || session.generation() > self.header.generation
                {
                    return Err(StoreError::Corrupt);
                }
                let login = LoginRevocationChange::from_session(&session);
                (Some(session), login)
            }
            CallerRecord::Fence(bytes) => (
                None,
                LoginRevocationChange::decode(bytes).map_err(|_| StoreError::Corrupt)?,
            ),
        };
        change
            .check_stored(
                self.header.observed_ns,
                self.header.config.logins.maximum_ttl_ns,
                self.header.config.logins.devices_per_account,
            )
            .map_err(|_| StoreError::Corrupt)?;
        if change.caller() != caller || change.audience() != self.header.config.audience {
            return Err(StoreError::Corrupt);
        }
        Ok((session, change))
    }
    // Slot/expiry indexes govern mutation and maintenance. Ordinary session
    // authentication uses the canonical caller fence, as MemoryLoginRevocations
    // does: replacement atomically deletes that caller's entire old record.
    fn check_login_index(&self, change: &LoginRevocationChange) -> Result<(), StoreError> {
        let caller = change.caller();
        if let Some(binding) = binding(change) {
            let owner: Option<Principal> = self.records.get(&keys::slot(
                keys::LOGIN_SLOT,
                binding.issuer,
                binding.subject,
                binding.device_slot,
            ))?;
            if owner != Some(caller) {
                return Err(StoreError::Corrupt);
            }
        } else if !self.records.contains(&keys::login_expiry(change)) {
            return Err(StoreError::Corrupt);
        }
        Ok(())
    }
    fn read_session(&self, caller: Principal) -> Result<Option<Session>, StoreError> {
        self.read_caller(caller)?
            .as_ref()
            .map(|row| self.session_from(caller, row))
            .transpose()
            .map(Option::flatten)
    }
    fn session_from(
        &self,
        caller: Principal,
        row: &CallerRecord,
    ) -> Result<Option<Session>, StoreError> {
        self.decode_caller(caller, row).map(|(session, _)| session)
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
fn binding(change: &LoginRevocationChange) -> Option<LoginBinding> {
    match change.state() {
        LoginRecordState::Reserved(binding) | LoginRecordState::Revoked(Some(binding)) => {
            Some(binding)
        }
        LoginRecordState::Revoked(None) => None,
    }
}
fn session_error(error: StoreError) -> SessionError {
    match error {
        StoreError::Clock => crate::portable::LoginRevocationError::Clock.into(),
        StoreError::Capacity => SessionError::Capacity,
        StoreError::Invalid => SessionError::InvalidPolicyOrRecord,
        _ => SessionError::StorageFailure,
    }
}

#[cfg(test)]
mod tests;
