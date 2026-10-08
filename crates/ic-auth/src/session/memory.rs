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

impl MemorySessionStore {
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
        let next = self
            .generation
            .checked_add(1)
            .ok_or(SessionError::GenerationExhausted)?;
        self.generation = next;
        Ok(next)
    }
    pub fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }
    /// Logout removes only this caller's session, preserving consumed proofs.
    pub fn clear(&mut self, caller: Principal) -> bool {
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
        let mut result = SessionPruned::default();
        for _ in 0..budget.min(128) {
            let session = self
                .session_expiry
                .first()
                .copied()
                .filter(|(time, _)| *time <= now_ns);
            let replay = self
                .replay_expiry
                .first()
                .copied()
                .filter(|(time, _)| *time <= now_ns);
            if let Some((time, caller)) = session
                && replay.is_none_or(|(other, _)| time <= other)
            {
                self.clear(caller);
                result.sessions += 1;
            } else if let Some((time, fingerprint)) = replay {
                self.replay_expiry.remove(&(time, fingerprint));
                let old = self
                    .replays
                    .remove(&fingerprint)
                    .expect("owned expiry index");
                self.encoded_bytes -= old.bytes;
                let subject = old.value.subject();
                let count = self
                    .replay_counts
                    .get_mut(&subject)
                    .expect("owned replay count");
                *count -= 1;
                if *count == 0 {
                    self.replay_counts.remove(&subject);
                }
                result.replays += 1;
            } else {
                break;
            }
        }
        result
    }
}

struct Transaction<'a> {
    store: &'a MemorySessionStore,
    staged: Option<(Stored<Session>, Stored<Replay>, usize)>,
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
    fn occupancy(&self, subject: Principal) -> Result<SessionOccupancy, SessionError> {
        Ok(SessionOccupancy {
            sessions: self.store.sessions.len(),
            replays: self.store.replays.len(),
            replays_for_subject: self.store.replay_counts.get(&subject).copied().unwrap_or(0),
        })
    }
    fn stage(&mut self, session: Session, replay: Replay) -> Result<(), SessionError> {
        if self.staged.is_some()
            || session.subject() != replay.subject()
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
        let count = self.occupancy(session.subject())?;
        if (old.is_none() && count.sessions >= limits.max_sessions)
            || count.replays >= limits.max_replays
            || count.replays_for_subject >= limits.max_replays_per_subject
        {
            return Err(SessionError::Capacity);
        }
        let session_bytes = session.encode()?.len();
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
        operation: impl FnOnce(&mut dyn SessionTransaction) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        let mut tx = Transaction {
            store: self,
            staged: None,
        };
        let result = operation(&mut tx)?;
        if let Some((session, replay, total)) = tx.staged {
            let caller = session.value.caller();
            let subject = session.value.subject();
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
            *self.replay_counts.entry(subject).or_default() += 1;
            self.encoded_bytes = total;
        }
        Ok(result)
    }
}
