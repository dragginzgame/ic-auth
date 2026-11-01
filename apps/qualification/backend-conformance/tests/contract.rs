//! One scenario owner, exercised through two different persistence mechanics.
use ic_auth::{
    portable::{LoginRecordState, LoginRevocationChange},
    session::*,
};
use ic_auth_backend_conformance::{Backend, assert_backend_contract};
use ic_auth_protocol_types::Principal;
use std::{borrow::Cow, collections::BTreeMap};

struct Reference {
    store: MemorySessionStore,
    logins: BTreeMap<Principal, Vec<u8>>,
    fail: bool,
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
    fn advance_generation(&mut self) {
        self.store
            .advance_generation_with_commit(|_| Ok(()))
            .unwrap();
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
    fn occupancy(&self, caller: Principal) -> Result<SessionOccupancy, SessionError> {
        let replays_for_caller = self
            .replays
            .values()
            .map(|bytes| Replay::decode(bytes))
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .filter(|r| r.caller() == caller)
            .count();
        Ok(SessionOccupancy {
            sessions: self.sessions.len(),
            replays: self.replays.len(),
            replays_for_caller,
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
        let replacing = self.sessions.contains_key(&session.caller());
        self.limits
            .check_capacity(self.occupancy(session.caller())?, replacing)?;
        let session_bytes = session.encode()?;
        let replay_bytes = replay.encode()?;
        let login_bytes = login.encode().unwrap();
        let old = self.sessions.get(&session.caller()).map_or(0, Vec::len);
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
        }
    }
    fn fail_next_commit(&mut self) {
        self.fail = true;
    }
    fn reopen(self) -> Self {
        self
    }
    fn advance_generation(&mut self) {
        self.generation = self.generation.checked_add(1).unwrap();
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
fn independent_encoded_records_conform() {
    assert_backend_contract::<Encoded>();
}

// Prove that the shared assertions detect a write-then-error atomicity bug.
struct Broken(Encoded);
impl SessionStore for Broken {
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
        let fail = std::mem::take(&mut self.0.fail);
        let result = self.0.admit(session, replay, login)?;
        if fail {
            Err(SessionError::StorageFailure)
        } else {
            Ok(result)
        }
    }
    fn logout(&mut self, login: &LoginRevocationChange) -> Result<(), SessionError> {
        self.0.logout(login)
    }
}
impl Backend for Broken {
    fn fresh(generation: u64, limits: SessionLimits) -> Self {
        Self(Encoded::fresh(generation, limits))
    }
    fn fail_next_commit(&mut self) {
        self.0.fail_next_commit();
    }
    fn reopen(self) -> Self {
        self
    }
    fn advance_generation(&mut self) {
        self.0.advance_generation();
    }
    fn retained_logins(&self) -> Vec<LoginRevocationChange> {
        self.0.retained_logins()
    }
}
#[test]
fn write_then_error_is_detected() {
    assert!(std::panic::catch_unwind(assert_backend_contract::<Broken>).is_err());
}
