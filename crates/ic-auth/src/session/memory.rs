use super::{
    Replay, Session, SessionError, SessionLimits, SessionOccupancy, SessionStore,
    SessionTransaction,
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
/// `admission` contains the touched session/replay pair; `None` means logout of
/// `login.caller()`, retaining replay history. Totals describe state after commit.
/// This is not a credential or a complete checkpoint. The host persists this
/// delta and protected metadata atomically before reporting success.
pub struct SessionCommit<'a> {
    pub login: &'a crate::portable::LoginRevocationChange,
    pub admission: Option<(&'a Session, &'a Replay)>,
    pub session_count: usize,
    pub replay_count: usize,
    pub encoded_bytes: usize,
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
            self.session_expiry
                .insert((session.expires_at_ns(), session.caller()));
            self.sessions.insert(
                session.caller(),
                Stored {
                    value: session,
                    bytes,
                },
            );
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
            self.replay_expiry
                .insert((replay.remove_at_ns(), replay.fingerprint()));
            self.replay_counts.insert(replay.caller(), count + 1);
            self.replays.insert(
                replay.fingerprint(),
                Stored {
                    value: replay,
                    bytes,
                },
            );
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
    /// Logout removes only this caller's session, preserving consumed proofs.
    fn clear(&mut self, caller: Principal) -> bool {
        if let Some(old) = self.sessions.remove(&caller) {
            self.session_expiry
                .remove(&(old.value.expires_at_ns(), caller));
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

struct Transaction<'a> {
    store: &'a MemorySessionStore,
    staged: Option<(Stored<Session>, Stored<Replay>, usize)>,
    removed: Option<Principal>,
}
impl SessionTransaction for Transaction<'_> {
    fn generation(&self) -> u64 {
        self.store.generation
    }
    fn limits(&self) -> SessionLimits {
        self.store.limits
    }
    fn session(&self, caller: Principal) -> Result<Option<Session>, SessionError> {
        Ok(self.store.sessions.get(&caller).map(|s| s.value.clone()))
    }
    fn replay_exists(&self, fingerprint: [u8; 32]) -> Result<bool, SessionError> {
        Ok(self.store.replays.contains_key(&fingerprint))
    }
    fn occupancy(&self, caller: Principal) -> Result<SessionOccupancy, SessionError> {
        Ok(SessionOccupancy {
            sessions: self.store.sessions.len(),
            replays: self.store.replays.len(),
            replays_for_caller: self.store.replay_counts.get(&caller).copied().unwrap_or(0),
        })
    }
    fn remove(&mut self, caller: Principal) -> Result<(), SessionError> {
        if self.staged.is_some() || self.removed.is_some() {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        self.removed = Some(caller);
        Ok(())
    }
    fn stage(&mut self, session: Session, replay: Replay) -> Result<(), SessionError> {
        if self.staged.is_some()
            || self.removed.is_some()
            || session.caller() != replay.caller()
            || session.fingerprint() != replay.fingerprint()
            || session.generation() != replay.generation()
            || replay.remove_at_ns() == 0
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        if session.generation() != self.generation() {
            return Err(SessionError::StaleAuthority);
        }
        if self.replay_exists(replay.fingerprint())? {
            return Err(SessionError::ReplayConflict);
        }
        let old = self.store.sessions.get(&session.caller());
        let limits = self.limits();
        let count = self.occupancy(session.caller())?;
        limits.check_capacity(count, old.is_some())?;
        let session_bytes = session.encoded_len()?;
        let replay_bytes = replay.encoded_len()?;
        let total = self.store.encoded_bytes - old.map_or(0, |s| s.bytes);
        let total = total
            .checked_add(session_bytes)
            .and_then(|b| b.checked_add(replay_bytes))
            .filter(|b| *b <= limits.max_encoded_bytes)
            .ok_or(SessionError::ByteCapacity)?;
        self.staged = Some((
            Stored {
                value: session,
                bytes: session_bytes,
            },
            Stored {
                value: replay,
                bytes: replay_bytes,
            },
            total,
        ));
        Ok(())
    }
}
impl SessionStore for MemorySessionStore {
    fn transaction<T>(
        &mut self,
        login_change: Option<&crate::portable::LoginRevocationChange>,
        operation: impl FnOnce(&mut dyn SessionTransaction) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        self.transaction_with_commit(login_change, operation, |_| Ok(()))
    }
}

impl MemorySessionStore {
    /// Reuse validated staging and indexes with a host-owned durable commit.
    /// Calls `commit` exactly once for a valid write, never for reads or refusals.
    /// A returned error preserves all in-memory records and accounting. The host
    /// MUST commit the supplied delta atomically or return an error without effects;
    /// no await/reentrancy or error after durable success. Success means the durable
    /// commit finished, never merely prepared.
    /// After callback success only the already-validated in-memory write remains.
    /// Maintenance, generation changes and clock/budget persistence remain host-owned.
    pub fn transaction_with_commit<T>(
        &mut self,
        login_change: Option<&crate::portable::LoginRevocationChange>,
        operation: impl FnOnce(&mut dyn SessionTransaction) -> Result<T, SessionError>,
        commit: impl FnOnce(SessionCommit<'_>) -> Result<(), SessionError>,
    ) -> Result<T, SessionError> {
        let mut tx = Transaction {
            store: self,
            staged: None,
            removed: None,
        };
        let result = operation(&mut tx)?;
        match (&tx.staged, tx.removed, login_change) {
            (None, None, None) => {}
            (Some((session, _, _)), None, Some(change)) if !change.revoked() => {
                session.value.check_login_change(change)?;
            }
            (None, Some(caller), Some(change)) if change.revoked() && change.caller() == caller => {
                if let Some(session) = self.sessions.get(&caller) {
                    session.value.check_login_change(change)?;
                }
            }
            _ => return Err(SessionError::InvalidPolicyOrRecord),
        }
        if let Some(login) = login_change {
            let old = self.sessions.get(&login.caller());
            let (admission, session_count, replay_count, encoded_bytes) =
                if let Some((session, replay, total)) = &tx.staged {
                    (
                        Some((&session.value, &replay.value)),
                        self.sessions.len() + usize::from(old.is_none()),
                        self.replays.len() + 1,
                        *total,
                    )
                } else {
                    (
                        None,
                        self.sessions.len() - usize::from(old.is_some()),
                        self.replays.len(),
                        self.encoded_bytes - old.map_or(0, |s| s.bytes),
                    )
                };
            commit(SessionCommit {
                login,
                admission,
                session_count,
                replay_count,
                encoded_bytes,
            })?;
        }
        let (staged, removed) = (tx.staged, tx.removed);
        if let Some(caller) = removed {
            self.clear(caller);
        }
        if let Some((session, replay, total)) = staged {
            let caller = session.value.caller();
            let fingerprint = replay.value.fingerprint();
            if let Some(old) = self.sessions.remove(&caller) {
                self.session_expiry
                    .remove(&(old.value.expires_at_ns(), caller));
            }
            self.session_expiry
                .insert((session.value.expires_at_ns(), caller));
            self.replay_expiry
                .insert((replay.value.remove_at_ns(), fingerprint));
            self.sessions.insert(caller, session);
            self.replays.insert(fingerprint, replay);
            *self.replay_counts.entry(caller).or_default() += 1;
            self.encoded_bytes = total;
        }
        Ok(result)
    }
}
