use super::{
    Replay, Session, SessionError, SessionLimits, SessionOccupancy, SessionReader, SessionStore,
};
use ic_auth_protocol_types::Principal;
use std::collections::{BTreeMap, BTreeSet};

struct Stored<T> {
    value: T,
    bytes: usize,
}

/// Bounded volatile reference backend for native and Wasm hosts. No stable
/// memory, globals, timers or certification-root effects. Transactions stage
/// only the touched pair; expiry indexes bound cleanup work without a store scan.
/// Losing this store also loses replay history. Before serving after a reset,
/// hosts must advance portable login/proof reset floors with qualified skew,
/// discard revocation state and authenticate fresh enrollment before readiness.
/// Persistent hosts retain sessions, replay and revocation state atomically.
pub struct MemorySessionStore {
    generation: u64,
    limits: SessionLimits,
    sessions: BTreeMap<Principal, Stored<Session>>,
    replays: BTreeMap<[u8; 32], Stored<Replay>>,
    replay_counts: BTreeMap<Principal, usize>,
    session_expiry: BTreeSet<(u64, Principal)>,
    replay_expiry: BTreeSet<(u64, [u8; 32])>,
    encoded_bytes: usize,
}

/// Bounded maintenance result. Expiry is exclusive for both record kinds.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SessionPruned {
    pub sessions: usize,
    pub replays: usize,
}

/// Validated write supplied only at the serialized host commit boundary.
/// The explicit write contains all touched records. Logout retains replay
/// history. Totals describe state after commit.
/// This is not a credential or a complete checkpoint. The host persists this
/// delta and protected metadata atomically before reporting success.
pub struct SessionCommit<'a> {
    pub write: SessionWrite<'a>,
    pub session_count: usize,
    pub replay_count: usize,
    pub encoded_bytes: usize,
}

/// Validated admission or logout at a host's atomic persistence boundary.
/// Every variant includes the matching login change; admission additionally
/// includes the exact session/replay pair to persist in the same transaction.
#[derive(Clone, Copy)]
pub enum SessionWrite<'a> {
    Admission {
        login: &'a crate::portable::LoginRevocationChange,
        session: &'a Session,
        replay: &'a Replay,
    },
    Logout {
        login: &'a crate::portable::LoginRevocationChange,
    },
}

/// Bounded expired-record deletion and resulting physical store totals.
/// Hosts delete these keys and persist the totals atomically before success.
pub struct SessionPrune<'a> {
    pub callers: &'a [Principal],
    pub fingerprints: &'a [[u8; 32]],
    pub session_count: usize,
    pub replay_count: usize,
    pub encoded_bytes: usize,
}

impl MemorySessionStore {
    /// Stream records under the host's synchronous checkpoint boundary. Persist
    /// both iterator lengths, encoded byte count, generation and the complete
    /// replay/login/authority state in one protected atomic storage generation.
    #[cfg(feature = "session-restore")]
    pub fn session_records(&self) -> impl ExactSizeIterator<Item = &Session> {
        self.sessions.values().map(|stored| &stored.value)
    }

    /// Replay records survive logout and generation changes; never export only
    /// records belonging to currently retained sessions.
    #[cfg(feature = "session-restore")]
    pub fn replay_records(&self) -> impl ExactSizeIterator<Item = &Replay> {
        self.replays.values().map(|stored| &stored.value)
    }

    /// Rebuild a fresh store from complete, checked, trusted-storage records.
    /// Counts and encoded bytes MUST come from the same atomic generation as
    /// the records. Failure consumes partial state. Old-generation and expired
    /// records remain physical occupancy until pruning; live authorization still
    /// requires the restored login ledger and freshly checked host authority.
    /// This does not authenticate snapshots or protect against coherent rollback.
    #[cfg(feature = "session-restore")]
    pub fn restore_records(
        mut self,
        expected_counts: (usize, usize),
        expected_encoded_bytes: usize,
        sessions: impl IntoIterator<Item = Session>,
        replays: impl IntoIterator<Item = Replay>,
    ) -> Result<Self, SessionError> {
        if !self.sessions.is_empty()
            || !self.replays.is_empty()
            || expected_counts.0 > self.limits.max_sessions
            || expected_counts.1 > self.limits.max_replays
            || expected_encoded_bytes > self.limits.max_encoded_bytes
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        for session in sessions {
            if self.sessions.len() >= expected_counts.0 {
                return Err(SessionError::Capacity);
            }
            if session.generation() > self.generation
                || self.sessions.contains_key(&session.caller())
            {
                return Err(SessionError::InvalidPolicyOrRecord);
            }
            let bytes = session.encoded_len()?;
            self.encoded_bytes = self
                .encoded_bytes
                .checked_add(bytes)
                .filter(|total| *total <= expected_encoded_bytes)
                .ok_or(SessionError::ByteCapacity)?;
            self.install_session(Stored {
                value: session,
                bytes,
            });
        }
        for replay in replays {
            if self.replays.len() >= expected_counts.1 {
                return Err(SessionError::Capacity);
            }
            let count = self
                .replay_counts
                .get(&replay.caller())
                .copied()
                .unwrap_or(0);
            if count >= self.limits.max_replays_per_caller {
                return Err(SessionError::Capacity);
            }
            if replay.generation() > self.generation
                || self.replays.contains_key(&replay.fingerprint())
            {
                return Err(SessionError::InvalidPolicyOrRecord);
            }
            let bytes = replay.encoded_len()?;
            self.encoded_bytes = self
                .encoded_bytes
                .checked_add(bytes)
                .filter(|total| *total <= expected_encoded_bytes)
                .ok_or(SessionError::ByteCapacity)?;
            self.install_replay(Stored {
                value: replay,
                bytes,
            });
        }
        if (self.sessions.len(), self.replays.len()) != expected_counts
            || self.encoded_bytes != expected_encoded_bytes
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        for stored in self.sessions.values() {
            // An expired proof's replay may already have been pruned while its
            // session remains live. If retained, the pair must still agree.
            if let Some(replay) = self.replays.get(&stored.value.fingerprint())
                && (replay.value.caller() != stored.value.caller()
                    || replay.value.generation() != stored.value.generation())
            {
                return Err(SessionError::InvalidPolicyOrRecord);
            }
        }
        Ok(self)
    }

    // Callers validate before reaching these infallible index updates. Durable
    // admission invokes them only after commit; restore admits each record first.
    fn install_session(&mut self, session: Stored<Session>) {
        let caller = session.value.caller();
        self.take_session(caller);
        self.session_expiry
            .insert((session.value.expires_at_ns(), caller));
        self.sessions.insert(caller, session);
    }

    fn install_replay(&mut self, replay: Stored<Replay>) {
        let caller = replay.value.caller();
        let fingerprint = replay.value.fingerprint();
        self.replay_expiry
            .insert((replay.value.remove_at_ns(), fingerprint));
        self.replays.insert(fingerprint, replay);
        *self.replay_counts.entry(caller).or_default() += 1;
    }

    pub fn new(generation: u64, limits: SessionLimits) -> Result<Self, SessionError> {
        limits.validate()?;
        Ok(Self {
            generation,
            limits,
            sessions: BTreeMap::new(),
            replays: BTreeMap::new(),
            replay_counts: BTreeMap::new(),
            session_expiry: BTreeSet::new(),
            replay_expiry: BTreeSet::new(),
            encoded_bytes: 0,
        })
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    /// Invalidate sessions without deleting replay history. A durable host must
    /// persist the generation and changed protected configuration atomically.
    pub fn advance_generation(&mut self) -> Result<u64, SessionError> {
        self.advance_generation_with_commit(|_| Ok(()))
    }
    /// Commit the proposed generation with protected host authority before
    /// publishing it in memory. Failure preserves the prior generation.
    pub fn advance_generation_with_commit(
        &mut self,
        commit: impl FnOnce(u64) -> Result<(), SessionError>,
    ) -> Result<u64, SessionError> {
        let next = self
            .generation
            .checked_add(1)
            .ok_or(SessionError::GenerationExhausted)?;
        commit(next)?;
        self.generation = next;
        Ok(next)
    }
    pub fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }
    // Replacement and deletion share the expiry-index invariant; callers own
    // their staged byte totals and update those only after durable success.
    fn take_session(&mut self, caller: Principal) -> Option<Stored<Session>> {
        let old = self.sessions.remove(&caller)?;
        self.session_expiry
            .remove(&(old.value.expires_at_ns(), caller));
        Some(old)
    }

    /// Logout removes only this caller's session, preserving consumed proofs.
    fn clear(&mut self, caller: Principal) -> bool {
        if let Some(old) = self.take_session(caller) {
            self.encoded_bytes -= old.bytes;
            true
        } else {
            false
        }
    }
    /// Remove at most min(budget,128) expired records through ordered indexes.
    /// No implicit cleanup occurs during failed admission or authorization.
    pub fn prune(&mut self, now_ns: u64, budget: usize) -> SessionPruned {
        self.prune_with_commit(now_ns, budget, |_| Ok(()))
            .expect("volatile maintenance cannot fail")
    }
    /// Stage at most 128 expired records, then atomically persist their deletion.
    /// The callback is skipped for empty work. On returned error all heap records,
    /// indexes and accounting remain unchanged. No await/reentrancy is permitted.
    pub fn prune_with_commit(
        &mut self,
        now_ns: u64,
        budget: usize,
        commit: impl FnOnce(SessionPrune<'_>) -> Result<(), SessionError>,
    ) -> Result<SessionPruned, SessionError> {
        let mut sessions = self.session_expiry.iter().peekable();
        let mut replays = self.replay_expiry.iter().peekable();
        let mut callers = Vec::new();
        let mut fingerprints = Vec::new();
        let mut bytes = self.encoded_bytes;
        for _ in 0..budget.min(128) {
            let session = sessions
                .peek()
                .copied()
                .copied()
                .filter(|(time, _)| *time <= now_ns);
            let replay = replays
                .peek()
                .copied()
                .copied()
                .filter(|(time, _)| *time <= now_ns);
            if let Some((time, caller)) = session
                && replay.is_none_or(|(other, _)| time <= other)
            {
                sessions.next();
                bytes -= self.sessions[&caller].bytes;
                callers.push(caller);
            } else if let Some((_, fingerprint)) = replay {
                replays.next();
                bytes -= self.replays[&fingerprint].bytes;
                fingerprints.push(fingerprint);
            } else {
                break;
            }
        }
        let result = SessionPruned {
            sessions: callers.len(),
            replays: fingerprints.len(),
        };
        if callers.is_empty() && fingerprints.is_empty() {
            return Ok(result);
        }
        commit(SessionPrune {
            callers: &callers,
            fingerprints: &fingerprints,
            session_count: self.sessions.len() - callers.len(),
            replay_count: self.replays.len() - fingerprints.len(),
            encoded_bytes: bytes,
        })?;
        for caller in callers {
            self.clear(caller);
        }
        for fingerprint in fingerprints {
            let old = self
                .replays
                .remove(&fingerprint)
                .expect("owned expiry index");
            self.replay_expiry
                .remove(&(old.value.remove_at_ns(), fingerprint));
            self.encoded_bytes -= old.bytes;
            let caller = old.value.caller();
            let count = self
                .replay_counts
                .get_mut(&caller)
                .expect("owned replay count");
            *count -= 1;
            if *count == 0 {
                self.replay_counts.remove(&caller);
            }
        }
        Ok(result)
    }
}

impl SessionReader for MemorySessionStore {
    fn generation(&self) -> u64 {
        self.generation
    }
    fn limits(&self) -> SessionLimits {
        self.limits
    }
    fn session(
        &self,
        caller: Principal,
    ) -> Result<Option<std::borrow::Cow<'_, Session>>, SessionError> {
        Ok(self
            .sessions
            .get(&caller)
            .map(|s| std::borrow::Cow::Borrowed(&s.value)))
    }
    fn replay_exists(&self, fingerprint: [u8; 32]) -> Result<bool, SessionError> {
        Ok(self.replays.contains_key(&fingerprint))
    }
    fn occupancy(&self, caller: Principal) -> Result<SessionOccupancy, SessionError> {
        Ok(SessionOccupancy {
            sessions: self.sessions.len(),
            replays: self.replays.len(),
            replays_for_caller: self.replay_counts.get(&caller).copied().unwrap_or(0),
        })
    }
}
impl SessionStore for MemorySessionStore {
    fn read<T>(
        &mut self,
        operation: impl FnOnce(&dyn SessionReader) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        operation(self)
    }
    fn admit(
        &mut self,
        session: &Session,
        replay: &Replay,
        login: &crate::portable::LoginRevocationChange,
    ) -> Result<bool, SessionError> {
        self.admit_with_commit(session, replay, login, |_| Ok(()))
    }
    fn logout(
        &mut self,
        login: &crate::portable::LoginRevocationChange,
    ) -> Result<(), SessionError> {
        self.logout_with_commit(login, |_| Ok(()))
    }
}
impl MemorySessionStore {
    /// Validate and prepare exactly one session/replay pair, then call the host
    /// once to persist it atomically with the login reservation and totals.
    /// Returns true for replacement. Refusals never call persistence; a commit
    /// error preserves all heap records and accounting. Success must mean durable
    /// completion, with no await/reentrancy or error after durable success.
    pub fn admit_with_commit(
        &mut self,
        session: &Session,
        replay: &Replay,
        login: &crate::portable::LoginRevocationChange,
        commit: impl FnOnce(SessionCommit<'_>) -> Result<(), SessionError>,
    ) -> Result<bool, SessionError> {
        if !matches!(
            login.state(),
            crate::portable::LoginRecordState::Reserved(_)
        ) || session.caller() != replay.caller()
            || session.fingerprint() != replay.fingerprint()
            || session.generation() != replay.generation()
            || replay.remove_at_ns() == 0
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        session.check_login_change(login)?;
        if session.generation() != self.generation {
            return Err(SessionError::StaleAuthority);
        }
        if self.replay_exists(replay.fingerprint())? {
            return Err(SessionError::ReplayConflict);
        }
        let old = self.sessions.get(&session.caller());
        let replacing = old.is_some();
        self.limits
            .check_capacity(self.occupancy(session.caller())?, replacing)?;
        let session_bytes = session.encoded_len()?;
        let replay_bytes = replay.encoded_len()?;
        let total = (self.encoded_bytes - old.map_or(0, |s| s.bytes))
            .checked_add(session_bytes)
            .and_then(|b| b.checked_add(replay_bytes))
            .filter(|b| *b <= self.limits.max_encoded_bytes)
            .ok_or(SessionError::ByteCapacity)?;
        // Prepare the touched owned records before persistence, never clone the
        // whole store or allocate their payloads after durable success.
        let session = Stored {
            value: session.clone(),
            bytes: session_bytes,
        };
        let replay = Stored {
            value: replay.clone(),
            bytes: replay_bytes,
        };
        commit(SessionCommit {
            write: SessionWrite::Admission {
                login,
                session: &session.value,
                replay: &replay.value,
            },
            session_count: self.sessions.len() + usize::from(!replacing),
            replay_count: self.replays.len() + 1,
            encoded_bytes: total,
        })?;
        self.install_session(session);
        self.install_replay(replay);
        self.encoded_bytes = total;
        Ok(replacing)
    }

    /// Atomically persist a login tombstone and remove its caller's session.
    /// Replay evidence is retained. Invalid reservations and mismatched existing
    /// records refuse before persistence. A commit error preserves heap state.
    /// The host commit has the same durable-completion contract as admission.
    pub fn logout_with_commit(
        &mut self,
        login: &crate::portable::LoginRevocationChange,
        commit: impl FnOnce(SessionCommit<'_>) -> Result<(), SessionError>,
    ) -> Result<(), SessionError> {
        if !matches!(login.state(), crate::portable::LoginRecordState::Revoked(_)) {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        let caller = login.caller();
        let old = self.sessions.get(&caller);
        if let Some(session) = old {
            session.value.check_login_change(login)?;
        }
        commit(SessionCommit {
            write: SessionWrite::Logout { login },
            session_count: self.sessions.len() - usize::from(old.is_some()),
            replay_count: self.replays.len(),
            encoded_bytes: self.encoded_bytes - old.map_or(0, |s| s.bytes),
        })?;
        self.clear(caller);
        Ok(())
    }
}
