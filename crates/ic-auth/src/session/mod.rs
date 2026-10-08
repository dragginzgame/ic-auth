//! Local sessions admitted from complete token verification, with atomic replay
//! consumption. Hosts own protected context, generation changes and durable
//! transactions. Application resource policy and IC ingress delegation are separate.

mod memory;
mod model;

pub use memory::{MemorySessionStore, SessionPruned};
pub use model::{Replay, Session, SessionLimits, SessionPolicy, SessionRequest};

use crate::{
    canonical::{CanonicalAuthError, claims_hash},
    token::{TokenVerificationContext, TokenVerificationError, verify_token},
};
use ic_auth_protocol_types::{DelegatedToken, Principal};
use thiserror::Error;

/// Physical occupancy: expired records count until explicit bounded pruning.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SessionOccupancy {
    pub sessions: usize,
    pub replays: usize,
    pub replays_for_subject: usize,
}

/// One isolated transaction. Reads, capacity checks and the staged pair observe
/// the same state. On error nothing is committed, including indexes and counters.
/// All record reads/updates are bounded; implementations must not clone a store.
pub trait SessionTransaction {
    fn generation(&self) -> u64;
    fn limits(&self) -> SessionLimits;
    fn session(&self, caller: Principal) -> Result<Option<Session>, SessionError>;
    fn replay_exists(&self, fingerprint: [u8; 32]) -> Result<bool, SessionError>;
    fn occupancy(&self, subject: Principal) -> Result<SessionOccupancy, SessionError>;
    /// Stage exactly one session/replay pair; reject mismatched identities,
    /// stale generation, consumed proofs and byte/capacity limits before writes.
    fn stage(&mut self, session: Session, replay: Replay) -> Result<(), SessionError>;
}

/// Host-owned transaction boundary. No await, reentrancy or concurrent authority
/// update may split the operation. Commit errors must preserve all prior state.
pub trait SessionStore {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn SessionTransaction) -> Result<T, SessionError>,
    ) -> Result<T, SessionError>;
}

/// Session creation, replacement, or the unchanged result of an exact retry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionEstablished {
    Created(Session),
    Replaced(Session),
    ExactRetry(Session),
}

/// Session/replay admission and local authorization failures.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum SessionError {
    #[error("session capability is disabled or subject is inadmissible")]
    Disabled,
    #[error("session policy or stored record is invalid")]
    InvalidPolicyOrRecord,
    #[error("session authority is no longer current")]
    StaleAuthority,
    #[error("session is missing or expired")]
    MissingOrExpired,
    #[error("proof is not currently eligible for session admission")]
    ProofIneligible,
    #[error("proof was already consumed or retry request differs")]
    ReplayConflict,
    #[error("requested or required scope is not granted")]
    ScopeRejected,
    #[error("session/replay capacity is exhausted")]
    Capacity,
    #[error("encoded record budget is exhausted")]
    ByteCapacity,
    #[error("authority generation is exhausted")]
    GenerationExhausted,
    #[error("host transaction could not commit")]
    StorageFailure,
    #[error(transparent)]
    Canonical(#[from] CanonicalAuthError),
    #[error(transparent)]
    Token(#[from] TokenVerificationError),
}

/// Admit one token, atomically consuming its canonical claims fingerprint. Exact
/// retry returns only the caller's still-authorized session, even after proof
/// expiry; it neither re-verifies altered proof transport nor extends authority.
pub fn establish_session(
    store: &mut impl SessionStore,
    token: &DelegatedToken,
    request: &SessionRequest,
    context: &TokenVerificationContext<'_>,
    policy: SessionPolicy,
) -> Result<SessionEstablished, SessionError> {
    policy.validate()?;
    if !policy.enabled || !policy.subject_admissible {
        return Err(SessionError::Disabled);
    }
    // Bound input before hashing even on the inexpensive retry path.
    crate::token::rules::check_size(token, context.limits)?;
    let fingerprint = claims_hash(&token.claims)?;
    let request_hash = request.hash();
    store.transaction(|tx| {
        if tx.generation() != policy.generation {
            return Err(SessionError::StaleAuthority);
        }
        let previous = tx.session(context.caller)?;
        if let Some(session) = &previous
            && session.fingerprint() == fingerprint
        {
            if session.request_hash() != request_hash {
                return Err(SessionError::ReplayConflict);
            }
            session.check_live(context, policy)?;
            return Ok(SessionEstablished::ExactRetry(session.clone()));
        }
        if tx.replay_exists(fingerprint)? {
            return Err(SessionError::ReplayConflict);
        }
        let verified = verify_token(token, context)?;
        if context.now_ns < verified.claims().issued_at_ns
            || context.now_ns < token.proof.cert.not_before_ns
            || verified.claims().expires_at_ns - verified.claims().issued_at_ns
                > policy.max_proof_ttl_ns
        {
            return Err(SessionError::ProofIneligible);
        }
        if request
            .scopes()
            .iter()
            .any(|s| !verified.scopes().contains(s))
        {
            return Err(SessionError::ScopeRejected);
        }
        let limits = tx.limits();
        limits.validate()?;
        let occupancy = tx.occupancy(context.caller)?;
        if (previous.is_none() && occupancy.sessions >= limits.max_sessions)
            || occupancy.replays >= limits.max_replays
            || occupancy.replays_for_subject >= limits.max_replays_per_subject
        {
            return Err(SessionError::Capacity);
        }
        let session = Session::admit(&verified, token, request, context, policy)?;
        let replay = Replay::new(&session, token.claims.expires_at_ns);
        tx.stage(session.clone(), replay)?;
        Ok(if previous.is_some() {
            SessionEstablished::Replaced(session)
        } else {
            SessionEstablished::Created(session)
        })
    })
}

/// Authorize current session scopes with actual caller and protected policy.
/// A successful session check does not authorize application-owned resources.
pub fn authorize_session(
    store: &mut impl SessionStore,
    context: &TokenVerificationContext<'_>,
    policy: SessionPolicy,
) -> Result<Session, SessionError> {
    policy.validate()?;
    store.transaction(|tx| {
        if tx.generation() != policy.generation {
            return Err(SessionError::StaleAuthority);
        }
        let session = tx
            .session(context.caller)?
            .ok_or(SessionError::MissingOrExpired)?;
        session.check_live(context, policy)?;
        for scope in context.required_scopes {
            crate::canonical::validate_scope_label(scope)?;
            if !session.scopes().contains(scope) {
                return Err(SessionError::ScopeRejected);
            }
        }
        Ok(session)
    })
}
