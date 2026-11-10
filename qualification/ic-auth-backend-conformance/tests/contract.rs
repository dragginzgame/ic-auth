//! One scenario owner, exercised through two different persistence mechanics.
use ic_auth::{
    portable::{LoginRecordState, LoginRevocationChange},
    session::*,
};
use ic_auth_backend_conformance::{
    Backend, assert_admission_retry, assert_authorization, assert_backend_contract,
    assert_capacity, assert_logout_reopen, assert_maximum_shape, assert_metadata_reopen,
    assert_ninety_day_renewal, assert_pruning, assert_replacement_generation,
    assert_unknown_logout_reopen,
};
use ic_auth_protocol_types::Principal;
use std::{borrow::Cow, collections::BTreeMap};

struct Reference {
    store: MemorySessionStore,
    logins: BTreeMap<Principal, Vec<u8>>,
    fail: bool,
    observed_ns: u64,
}
impl SessionStore for Reference {
    fn read<T>(
        &mut self,
        operation: impl FnOnce(&dyn SessionReader) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        self.store.read(operation)
    }
    fn admit(
        &mut self,
        session: Session,
        replay: Replay,
        login: &LoginRevocationChange,
    ) -> Result<bool, SessionError> {
        let bytes = login.encode().unwrap();
        let fail = std::mem::take(&mut self.fail);
        self.store.admit_with_commit(session, replay, login, |_| {
            if fail {
                return Err(SessionError::StorageFailure);
            }
            if let Some(old) = login.displaced_caller() {
                self.logins.remove(&old);
            }
            self.logins.insert(login.caller(), bytes);
            Ok(())
        })
    }
    fn logout(&mut self, login: &LoginRevocationChange) -> Result<(), SessionError> {
        let bytes = login.encode().unwrap();
        let fail = std::mem::take(&mut self.fail);
        self.store.logout_with_commit(login, |_| {
            if fail {
                return Err(SessionError::StorageFailure);
            }
            self.logins.insert(login.caller(), bytes);
            Ok(())
        })
    }
}
impl Backend for Reference {
    fn fresh(generation: u64, limits: SessionLimits) -> Self {
        Self {
            store: MemorySessionStore::new(generation, limits),
            logins: BTreeMap::new(),
            fail: false,
            observed_ns: 0,
        }
    }
    fn fail_next_commit(&mut self) {
        self.fail = true;
    }
    fn reopen(self) -> Self {
        // Force the actual record codecs and complete-record restore boundary.
        let sessions: Vec<_> = self
            .store
            .session_records()
            .map(|s| Session::decode(&s.encode().unwrap()).unwrap())
            .collect();
        let replays: Vec<_> = self
            .store
            .replay_records()
            .map(|r| Replay::decode(&r.encode().unwrap()).unwrap())
            .collect();
        let store = MemorySessionStore::new(self.store.generation(), self.store.limits())
            .restore_records(
                (sessions.len(), replays.len()),
                self.store.encoded_bytes(),
                sessions,
                replays,
            )
            .unwrap();
        Self { store, ..self }
    }
    fn advance_generation(&mut self) -> Result<u64, SessionError> {
        let fail = std::mem::take(&mut self.fail);
        self.store.advance_generation_with_commit(|_| {
            if fail {
                Err(SessionError::StorageFailure)
            } else {
                Ok(())
            }
        })
    }
    fn prune(&mut self, now_ns: u64, budget: usize) -> Result<SessionPruned, SessionError> {
        self.store.prune_with_commit(now_ns, budget, |_| {
            if std::mem::take(&mut self.fail) {
                Err(SessionError::StorageFailure)
            } else {
                Ok(())
            }
        })
    }
    fn observe_clock(&mut self, now_ns: u64) -> Result<(), SessionError> {
        if now_ns < self.observed_ns {
            return Err(ic_auth::portable::LoginRevocationError::Clock.into());
        }
        self.observed_ns = now_ns;
        Ok(())
    }
    fn observed_at_ns(&self) -> u64 {
        self.observed_ns
    }
    fn retained_logins(&self) -> Vec<LoginRevocationChange> {
        self.logins
            .values()
            .map(|v| LoginRevocationChange::decode(v).unwrap())
            .collect()
    }
}

// Deliberately simple independent encoded-record adapter. No production claims:
// its scans and heap maps are only an independent check of the shared contract.
struct Encoded {
    generation: u64,
    limits: SessionLimits,
    sessions: BTreeMap<Principal, Vec<u8>>,
    replays: BTreeMap<[u8; 32], Vec<u8>>,
    logins: BTreeMap<Principal, Vec<u8>>,
    fail: bool,
    observed_ns: u64,
}
impl Encoded {
    fn bytes(&self) -> usize {
        self.sessions
            .values()
            .chain(self.replays.values())
            .map(Vec::len)
            .sum()
    }
    fn commit(&mut self) -> Result<(), SessionError> {
        if std::mem::take(&mut self.fail) {
            Err(SessionError::StorageFailure)
        } else {
            Ok(())
        }
    }
}
impl SessionReader for Encoded {
    fn generation(&self) -> u64 {
        self.generation
    }
    fn limits(&self) -> SessionLimits {
        self.limits
    }
    fn session(&self, caller: Principal) -> Result<Option<Cow<'_, Session>>, SessionError> {
        self.sessions
            .get(&caller)
            .map(|bytes| Session::decode(bytes).map(Cow::Owned))
            .transpose()
    }
    fn replay_exists(&self, fingerprint: [u8; 32]) -> Result<bool, SessionError> {
        Ok(self.replays.contains_key(&fingerprint))
    }
    fn occupancy(&self, account: (Principal, Principal)) -> Result<SessionOccupancy, SessionError> {
        let replays_for_account = self.replays.values().try_fold(0, |count, bytes| {
            Replay::decode(bytes).map(|replay| count + usize::from(replay.account() == account))
        })?;
        Ok(SessionOccupancy {
            sessions: self.sessions.len(),
            replays: self.replays.len(),
            replays_for_account,
        })
    }
}
impl SessionStore for Encoded {
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
        session.check_admission_pair(&replay, login, self.generation)?;
        if self.replay_exists(replay.fingerprint())? {
            return Err(SessionError::ReplayConflict);
        }
        let removed = login.displaced_caller().unwrap_or(session.caller());
        let replacing = self.sessions.contains_key(&removed);
        self.limits.check_capacity(
            self.occupancy((session.issuer(), session.subject()))?,
            replacing,
        )?;
        let session_bytes = session.encode()?;
        let replay_bytes = replay.encode()?;
        let login_bytes = login.encode().unwrap();
        let old = self.sessions.get(&removed).map_or(0, Vec::len);
        let total = self
            .bytes()
            .checked_sub(old)
            .and_then(|n| n.checked_add(session_bytes.len()))
            .and_then(|n| n.checked_add(replay_bytes.len()))
            .ok_or(SessionError::ByteCapacity)?;
        if total > self.limits.max_encoded_bytes() {
            return Err(SessionError::ByteCapacity);
        }
        self.commit()?;
        if let Some(old) = login.displaced_caller() {
            self.sessions.remove(&old);
            self.logins.remove(&old);
        }
        self.sessions.insert(session.caller(), session_bytes);
        self.replays.insert(replay.fingerprint(), replay_bytes);
        self.logins.insert(login.caller(), login_bytes);
        Ok(replacing)
    }
    fn logout(&mut self, login: &LoginRevocationChange) -> Result<(), SessionError> {
        if !matches!(login.state(), LoginRecordState::Revoked(_)) {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        if let Some(session) = self.session(login.caller())? {
            session.check_login_change(login)?;
        }
        let bytes = login.encode().unwrap();
        self.commit()?;
        self.sessions.remove(&login.caller());
        self.logins.insert(login.caller(), bytes);
        Ok(())
    }
}
impl Backend for Encoded {
    fn fresh(generation: u64, limits: SessionLimits) -> Self {
        Self {
            generation,
            limits,
            sessions: BTreeMap::new(),
            replays: BTreeMap::new(),
            logins: BTreeMap::new(),
            fail: false,
            observed_ns: 0,
        }
    }
    fn fail_next_commit(&mut self) {
        self.fail = true;
    }
    fn reopen(self) -> Self {
        self
    }
    fn advance_generation(&mut self) -> Result<u64, SessionError> {
        let next = self
            .generation
            .checked_add(1)
            .ok_or(SessionError::GenerationExhausted)?;
        self.commit()?;
        self.generation = next;
        Ok(next)
    }
    fn prune(&mut self, now_ns: u64, budget: usize) -> Result<SessionPruned, SessionError> {
        #[derive(Eq, Ord, PartialEq, PartialOrd)]
        enum Key {
            Session(Principal),
            Replay([u8; 32]),
        }
        let mut expired = Vec::new();
        for (caller, bytes) in &self.sessions {
            let deadline = Session::decode(bytes)?.expires_at_ns();
            if deadline <= now_ns {
                expired.push((deadline, Key::Session(*caller)));
            }
        }
        for (fingerprint, bytes) in &self.replays {
            let deadline = Replay::decode(bytes)?.remove_at_ns();
            if deadline <= now_ns {
                expired.push((deadline, Key::Replay(*fingerprint)));
            }
        }
        expired.sort_unstable();
        // A valid tighter host cap and reversed expired-row order exercise the
        // shared contract independently of the reference backend's scheduler.
        expired.reverse();
        expired.truncate(budget.min(32));
        if expired.is_empty() {
            return Ok(SessionPruned::default());
        }
        self.commit()?;
        let mut result = SessionPruned::default();
        for (_, key) in expired {
            match key {
                Key::Session(caller) => {
                    self.sessions.remove(&caller);
                    result.sessions += 1;
                }
                Key::Replay(fingerprint) => {
                    self.replays.remove(&fingerprint);
                    result.replays += 1;
                }
            }
        }
        Ok(result)
    }
    fn observe_clock(&mut self, now_ns: u64) -> Result<(), SessionError> {
        if now_ns < self.observed_ns {
            return Err(ic_auth::portable::LoginRevocationError::Clock.into());
        }
        self.observed_ns = now_ns;
        Ok(())
    }
    fn observed_at_ns(&self) -> u64 {
        self.observed_ns
    }
    fn retained_logins(&self) -> Vec<LoginRevocationChange> {
        self.logins
            .values()
            .map(|bytes| LoginRevocationChange::decode(bytes).unwrap())
            .collect()
    }
}

#[test]
fn reference_commit_callbacks_conform() {
    assert_backend_contract::<Reference>();
}
#[test]
fn independent_admission_retry_conforms() {
    assert_admission_retry::<Encoded>();
}
#[test]
fn independent_logout_reopen_conforms() {
    assert_logout_reopen::<Encoded>();
}
#[test]
fn independent_unknown_logout_reopen_conforms() {
    assert_unknown_logout_reopen::<Encoded>();
}
#[test]
fn independent_replacement_generation_conforms() {
    assert_replacement_generation::<Encoded>();
}
#[test]
fn independent_ninety_day_renewal_conforms() {
    assert_ninety_day_renewal::<Encoded>();
}
#[test]
fn independent_capacity_conforms() {
    assert_capacity::<Encoded>();
}
#[test]
fn independent_device_replacement_conforms() {
    ic_auth_backend_conformance::assert_device_replacement::<Encoded>();
}
#[test]
fn independent_authorization_conforms() {
    assert_authorization::<Encoded>();
}
#[test]
fn independent_maximum_shape_conforms() {
    assert_maximum_shape::<Encoded>();
}

// Select which write violates atomicity without copying the adapter mechanics.
struct Broken<const WRITE: u8>(Encoded);
impl<const WRITE: u8> Broken<WRITE> {
    fn write_then_error<T>(
        &mut self,
        write: impl FnOnce(&mut Encoded) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        let fail = std::mem::take(&mut self.0.fail);
        let result = write(&mut self.0)?;
        if fail {
            Err(SessionError::StorageFailure)
        } else {
            Ok(result)
        }
    }
}
impl<const WRITE: u8> SessionStore for Broken<WRITE> {
    fn read<T>(
        &mut self,
        operation: impl FnOnce(&dyn SessionReader) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        self.0.read(operation)
    }
    fn admit(
        &mut self,
        session: Session,
        replay: Replay,
        login: &LoginRevocationChange,
    ) -> Result<bool, SessionError> {
        if WRITE != 0 {
            self.0.admit(session, replay, login)
        } else {
            self.write_then_error(|store| store.admit(session, replay, login))
        }
    }
    fn logout(&mut self, login: &LoginRevocationChange) -> Result<(), SessionError> {
        if WRITE == 1 {
            self.write_then_error(|store| store.logout(login))
        } else {
            self.0.logout(login)
        }
    }
}
impl<const WRITE: u8> Backend for Broken<WRITE> {
    fn fresh(generation: u64, limits: SessionLimits) -> Self {
        Self(Encoded::fresh(generation, limits))
    }
    fn fail_next_commit(&mut self) {
        self.0.fail_next_commit();
    }
    fn reopen(mut self) -> Self {
        if WRITE == 4 {
            self.0.observed_ns = 0;
        }
        if WRITE == 6 {
            self.0.logins.retain(|_, bytes| {
                !matches!(
                    LoginRevocationChange::decode(bytes).unwrap().state(),
                    LoginRecordState::Revoked(None)
                )
            });
        }
        if WRITE == 7 {
            self.0.logins.clear();
        }
        self
    }
    fn advance_generation(&mut self) -> Result<u64, SessionError> {
        if WRITE == 3 {
            self.write_then_error(Encoded::advance_generation)
        } else {
            self.0.advance_generation()
        }
    }
    fn prune(&mut self, now_ns: u64, budget: usize) -> Result<SessionPruned, SessionError> {
        let nonempty = budget > 0
            && (self
                .0
                .sessions
                .values()
                .any(|b| Session::decode(b).unwrap().expires_at_ns() <= now_ns)
                || self
                    .0
                    .replays
                    .values()
                    .any(|b| Replay::decode(b).unwrap().remove_at_ns() <= now_ns));
        if WRITE == 2 && nonempty {
            self.write_then_error(|store| store.prune(now_ns, budget))
        } else {
            let mut result = self.0.prune(now_ns, budget)?;
            if WRITE == 5 && budget > 128 {
                loop {
                    let extra = self.0.prune(now_ns, 128)?;
                    result.sessions += extra.sessions;
                    result.replays += extra.replays;
                    if extra.sessions + extra.replays == 0 {
                        break;
                    }
                }
            }
            Ok(result)
        }
    }
    fn observe_clock(&mut self, now_ns: u64) -> Result<(), SessionError> {
        self.0.observe_clock(now_ns)
    }
    fn observed_at_ns(&self) -> u64 {
        self.0.observed_at_ns()
    }
    fn retained_logins(&self) -> Vec<LoginRevocationChange> {
        self.0.retained_logins()
    }
}
#[test]
#[should_panic(expected = "failed admission must preserve every record")]
fn admission_write_then_error_is_detected() {
    assert_admission_retry::<Broken<0>>();
}
#[test]
#[should_panic(expected = "failed logout must preserve session, replay and reservation")]
fn logout_write_then_error_is_detected() {
    assert_logout_reopen::<Broken<1>>();
}
#[test]
#[should_panic(expected = "failed unknown logout must preserve every record")]
fn unknown_logout_write_then_error_is_detected() {
    assert_unknown_logout_reopen::<Broken<1>>();
}
#[test]
#[should_panic(expected = "unknown denial must survive complete reopen")]
fn forgotten_unknown_denial_is_detected() {
    assert_unknown_logout_reopen::<Broken<6>>();
}
#[test]
#[should_panic(expected = "90-day login records must survive complete reopen")]
fn forgotten_ninety_day_login_is_detected() {
    assert_ninety_day_renewal::<Broken<7>>();
}

#[test]
fn independent_pruning_conforms() {
    assert_pruning::<Encoded>();
}
#[test]
fn independent_metadata_reopen_conforms() {
    assert_metadata_reopen::<Encoded>();
}
#[test]
#[should_panic(expected = "failed pruning must preserve every record")]
fn prune_write_then_error_is_detected() {
    assert_pruning::<Broken<2>>();
}
#[test]
#[should_panic(expected = "failed generation write must preserve authority")]
fn generation_write_then_error_is_detected() {
    assert_metadata_reopen::<Broken<3>>();
}
#[test]
#[should_panic(expected = "clock floor must survive reopen")]
fn forgotten_clock_floor_is_detected() {
    assert_metadata_reopen::<Broken<4>>();
}

#[test]
#[should_panic(expected = "cleanup must obey its 128-record work bound")]
fn unbounded_cleanup_is_detected() {
    assert_pruning::<Broken<5>>();
}
