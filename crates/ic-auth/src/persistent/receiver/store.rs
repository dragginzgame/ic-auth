use super::{
    CallerRecord, Cow, Header, LoginRecordState, LoginRevocationChange, Memory, Principal,
    ReceiverStore, Replay, Session, SessionError, SessionReader, SessionStore, StoreError, Writes,
    binding, keys, session_error,
};
use crate::session::{SessionLimits, SessionOccupancy};

impl<M: Memory> SessionReader for ReceiverStore<M> {
    fn generation(&self) -> u64 {
        self.header.generation
    }
    fn limits(&self) -> SessionLimits {
        self.header
            .config
            .limits()
            .expect("configuration admitted before opening receiver")
    }
    fn session(&self, caller: Principal) -> Result<Option<Cow<'_, Session>>, SessionError> {
        self.read_session(caller)
            .map(|row| row.map(Cow::Owned))
            .map_err(session_error)
    }
    fn replay_exists(&self, fingerprint: [u8; 32]) -> Result<bool, SessionError> {
        Ok(self.records.contains(&keys::replay(&fingerprint)))
    }
    fn occupancy(&self, account: (Principal, Principal)) -> Result<SessionOccupancy, SessionError> {
        let counts = self.header.counts;
        let replays_for_account = self
            .records
            .get(&keys::replay_count(account))
            .map_err(session_error)?
            .unwrap_or(0);
        Ok(SessionOccupancy {
            sessions: counts.sessions,
            replays: counts.replays,
            replays_for_account,
        })
    }
}
impl<M: Memory> SessionStore for ReceiverStore<M> {
    fn read<T>(
        &mut self,
        operation: impl FnOnce(&dyn SessionReader) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        operation(self)
    }
    fn admit(
        &mut self,
        session: Session,
        replay: Replay,
        login: &LoginRevocationChange,
    ) -> Result<bool, SessionError> {
        session.check_admission_pair(&replay, login, self.header.generation)?;
        if session.audience() != self.header.config.audience {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        let owner: Option<Principal> = self
            .records
            .get(&keys::slot(
                keys::SESSION_SLOT,
                session.issuer(),
                session.subject(),
                session.device_slot(),
            ))
            .map_err(session_error)?;
        if let Some(owner) = owner.filter(|old| *old != session.caller()) {
            let previous = self
                .read_session(owner)
                .map_err(session_error)?
                .ok_or(SessionError::StorageFailure)?;
            if login.displaced_caller() != Some(owner)
                || previous.device_generation() >= session.device_generation()
            {
                return Err(SessionError::InvalidPolicyOrRecord);
            }
        }
        if self.replay_exists(replay.fingerprint())? {
            return Err(SessionError::ReplayConflict);
        }
        let removed = login.displaced_caller().unwrap_or(session.caller());
        let previous = self.read_session(removed).map_err(session_error)?;
        if removed != session.caller()
            && self
                .read_session(session.caller())
                .map_err(session_error)?
                .is_some()
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        let replacing = previous.is_some();
        let occupancy = SessionReader::occupancy(self, replay.account())?;
        SessionReader::limits(self).check_capacity(occupancy, replacing)?;
        let session_bytes = session.encode()?;
        let replay_bytes = replay.encode()?;
        let old_bytes = previous
            .as_ref()
            .map(|old| old.encode().map(|bytes| bytes.len()))
            .transpose()?
            .unwrap_or(0);
        let mut header = self.header.clone();
        header.counts.encoded_bytes = header
            .counts
            .encoded_bytes
            .checked_sub(old_bytes)
            .ok_or(SessionError::StorageFailure)?
            .checked_add(session_bytes.len())
            .and_then(|bytes| bytes.checked_add(replay_bytes.len()))
            .filter(|bytes| *bytes <= header.config.sessions[3])
            .ok_or(SessionError::ByteCapacity)?;
        header.counts.sessions += usize::from(!replacing);
        header.counts.replays += 1;
        if occupancy.replays_for_account == 0 {
            header.replay_accounts += 1;
        }
        let mut writes = Writes::new();
        if let Some(previous) = previous {
            self.delete_session(&previous, &mut writes)
                .map_err(session_error)?;
        }
        self.stage_login(login, Some(session_bytes), &mut header, &mut writes)
            .map_err(session_error)?;
        writes
            .put(
                keys::slot(
                    keys::SESSION_SLOT,
                    session.issuer(),
                    session.subject(),
                    session.device_slot(),
                ),
                &session.caller(),
            )
            .map_err(session_error)?;
        writes
            .put(keys::session_expiry(&session), &session.caller())
            .map_err(session_error)?;
        writes
            .put(
                keys::replay(&replay.fingerprint()),
                &serde_bytes::Bytes::new(&replay_bytes),
            )
            .map_err(session_error)?;
        writes
            .put(keys::replay_expiry(&replay), &replay.fingerprint())
            .map_err(session_error)?;
        writes
            .put(
                keys::replay_count(replay.account()),
                &(occupancy.replays_for_account + 1),
            )
            .map_err(session_error)?;
        self.publish(header, writes).map_err(session_error)?;
        Ok(replacing)
    }
    fn logout(&mut self, login: &LoginRevocationChange) -> Result<(), SessionError> {
        if !matches!(login.state(), LoginRecordState::Revoked(_)) {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        let previous = self.read_session(login.caller()).map_err(session_error)?;
        let mut header = self.header.clone();
        let mut writes = Writes::new();
        if let Some(previous) = previous {
            previous.check_login_change(login)?;
            header.counts.sessions = header
                .counts
                .sessions
                .checked_sub(1)
                .ok_or(SessionError::StorageFailure)?;
            header.counts.encoded_bytes = header
                .counts
                .encoded_bytes
                .checked_sub(previous.encode()?.len())
                .ok_or(SessionError::StorageFailure)?;
            self.delete_session(&previous, &mut writes)
                .map_err(session_error)?;
        }
        self.stage_login(login, None, &mut header, &mut writes)
            .map_err(session_error)?;
        self.publish(header, writes).map_err(session_error)
    }
}
impl<M: Memory> ReceiverStore<M> {
    pub(super) fn delete_session(
        &self,
        session: &Session,
        writes: &mut Writes,
    ) -> Result<(), StoreError> {
        let fence = LoginRevocationChange::from_session(session)
            .encode()
            .map_err(|_| StoreError::Corrupt)?;
        writes.put(keys::login(session.caller()), &CallerRecord::Fence(fence))?;
        writes.delete(keys::slot(
            keys::SESSION_SLOT,
            session.issuer(),
            session.subject(),
            session.device_slot(),
        ));
        writes.delete(keys::session_expiry(session));
        Ok(())
    }
    fn stage_login(
        &self,
        login: &LoginRevocationChange,
        session: Option<Vec<u8>>,
        header: &mut Header,
        writes: &mut Writes,
    ) -> Result<(), StoreError> {
        if login.audience() != header.config.audience {
            return Err(StoreError::Invalid);
        }
        let previous = self.login(login.caller())?;
        let displaced = login
            .displaced_caller()
            .map(|caller| self.login(caller)?.ok_or(StoreError::Corrupt))
            .transpose()?;
        if let Some(old) = displaced.as_ref().or(previous.as_ref()) {
            login
                .check_replacement(old, header.observed_ns)
                .map_err(|_| StoreError::Invalid)?;
        }
        if let Some(binding) = binding(login) {
            if binding.device_slot >= header.config.logins.devices_per_account {
                return Err(StoreError::Invalid);
            }
            let owner: Option<Principal> = self.records.get(&keys::slot(
                keys::LOGIN_SLOT,
                binding.issuer,
                binding.subject,
                binding.device_slot,
            ))?;
            if owner
                != displaced.as_ref().map(|old| old.caller()).or_else(|| {
                    previous
                        .as_ref()
                        .filter(|old| super::binding(old) == Some(binding))
                        .map(|old| old.caller())
                })
                && owner != Some(login.caller())
            {
                return Err(StoreError::Corrupt);
            }
        }
        for old in [previous, displaced].into_iter().flatten() {
            writes.delete(keys::login(old.caller()));
            if let Some(binding) = binding(&old) {
                header.counts.admitted_logins = header
                    .counts
                    .admitted_logins
                    .checked_sub(1)
                    .ok_or(StoreError::Corrupt)?;
                writes.delete(keys::slot(
                    keys::LOGIN_SLOT,
                    binding.issuer,
                    binding.subject,
                    binding.device_slot,
                ));
            } else {
                header.counts.unknown_denials = header
                    .counts
                    .unknown_denials
                    .checked_sub(1)
                    .ok_or(StoreError::Corrupt)?;
                writes.delete(keys::login_expiry(&old));
            }
        }
        if let Some(binding) = binding(login) {
            header.counts.admitted_logins += 1;
            writes.put(
                keys::slot(
                    keys::LOGIN_SLOT,
                    binding.issuer,
                    binding.subject,
                    binding.device_slot,
                ),
                &login.caller(),
            )?;
        } else {
            header.counts.unknown_denials += 1;
            writes.put(keys::login_expiry(login), &login.caller())?;
        }
        if header.counts.admitted_logins > header.config.logins.admitted
            || header.counts.unknown_denials > header.config.logins.unknown
        {
            return Err(StoreError::Capacity);
        }
        writes.put(
            keys::login(login.caller()),
            &match session {
                Some(bytes) => CallerRecord::Session(bytes),
                None => CallerRecord::Fence(login.encode().map_err(|_| StoreError::Invalid)?),
            },
        )?;
        Ok(())
    }
}

// One operation-local owned decode. Never retained across a callback, authority
// change or await; every later application call reads current protected storage.
pub(super) struct SelectedReader<'a, M: Memory> {
    pub store: &'a ReceiverStore<M>,
    pub caller: Principal,
    pub selected: Option<Session>,
}
impl<M: Memory> SessionReader for SelectedReader<'_, M> {
    fn generation(&self) -> u64 {
        self.store.header.generation
    }
    fn limits(&self) -> SessionLimits {
        SessionReader::limits(self.store)
    }
    fn session(&self, caller: Principal) -> Result<Option<Cow<'_, Session>>, SessionError> {
        if caller == self.caller {
            Ok(self.selected.as_ref().map(Cow::Borrowed))
        } else {
            SessionReader::session(self.store, caller)
        }
    }
    fn replay_exists(&self, fingerprint: [u8; 32]) -> Result<bool, SessionError> {
        SessionReader::replay_exists(self.store, fingerprint)
    }
    fn occupancy(&self, account: (Principal, Principal)) -> Result<SessionOccupancy, SessionError> {
        SessionReader::occupancy(self.store, account)
    }
}
