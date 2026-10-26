//! One portable session lifecycle for both complete proof mechanisms, with atomic replay
//! consumption. Hosts own protected context, generation changes and durable
//! transactions. Application resource policy and IC ingress delegation are separate.

mod memory;
mod model;
mod refusal;

pub use memory::{MemorySessionStore, SessionCommit, SessionPrune, SessionPruned};
pub use model::{Replay, Session, SessionLimits, SessionPolicy};

use crate::{
    admission_budget::{AdmissionBudgetError, SessionAdmissionBudget},
    canonical::CanonicalAuthError,
    portable::{
        DecodedPortableClaims, LoginRevocationChange, LoginRevocationError, MemoryLoginRevocations,
        PortableClaimsContext, PortableClaimsError, PortableEncodingError, VerifiedPortableClaims,
        validate_claims_context,
    },
    session_request::{SessionRequest, SessionRequestError},
};
use ic_auth_protocol_types::{AuthRole, LogoutResult, LogoutUnavailable, Principal};
use thiserror::Error;

/// Protected receiver context, shared by both proof mechanisms and session checks.
/// Enrollment, caller and clock are host-authenticated inputs. Advance generation
/// atomically with authority changes; never derive them from submitted claims.
pub struct SessionContext<'a> {
    pub authority: PortableClaimsContext<'a>,
    pub role: &'a AuthRole,
    pub required_scopes: &'a [String],
}
impl SessionContext<'_> {
    fn validate(
        &self,
        policy: SessionPolicy,
        ledger: &MemoryLoginRevocations,
    ) -> Result<(), SessionError> {
        policy.validate()?;
        if !policy.enabled || !policy.subject_admissible {
            return Err(SessionError::Disabled);
        }
        if ledger.audience() != *self.authority.audience
            || !self.authority.authority_shape_valid()
            || self.required_scopes.len() > crate::limits::MAX_SESSION_SCOPES
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        for scope in self.required_scopes {
            crate::canonical::validate_scope_label(scope)?;
        }
        Ok(())
    }
}

/// Physical occupancy: expired records count until explicit bounded pruning.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SessionOccupancy {
    pub sessions: usize,
    pub replays: usize,
    pub replays_for_caller: usize,
}

/// One isolated transaction. Reads, capacity checks and the staged pair observe
/// the same state. On error nothing is committed, including indexes and counters.
/// All record reads/updates are bounded; implementations must not clone a store.
pub trait SessionTransaction {
    fn generation(&self) -> u64;
    fn limits(&self) -> SessionLimits;
    fn session(&self, caller: Principal) -> Result<Option<Session>, SessionError>;
    fn replay_exists(&self, fingerprint: [u8; 32]) -> Result<bool, SessionError>;
    fn occupancy(&self, caller: Principal) -> Result<SessionOccupancy, SessionError>;
    /// Stage exactly one session/replay pair; reject mismatched identities,
    /// stale generation, consumed proofs, per-record encoding ceilings and total
    /// byte/capacity limits before writes.
    fn stage(&mut self, session: Session, replay: Replay) -> Result<(), SessionError>;
    /// Stage removal of this caller's session only; replay evidence is retained.
    fn remove(&mut self, caller: Principal) -> Result<(), SessionError>;
}

/// Host-owned transaction boundary. No await, reentrancy or concurrent authority
/// update may split the operation. Commit errors must preserve all prior state.
/// Writes require the proposed login change; a durable implementation must commit
/// it atomically with session/replay/index changes. `None` permits reads only.
pub trait SessionStore {
    fn transaction<T>(
        &mut self,
        login_change: Option<&LoginRevocationChange>,
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
    #[error("fresh session verification budget is exhausted")]
    AdmissionThrottled,
    #[error("session admission clock moved backwards")]
    AdmissionClockRollback,
    #[error("session capability is disabled or subject is inadmissible")]
    Disabled,
    #[error("session policy or stored record is invalid")]
    InvalidPolicyOrRecord,
    #[error("session authority is no longer current")]
    StaleAuthority,
    #[error("session is missing or expired")]
    MissingOrExpired,
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
    #[cfg(any(
        feature = "portable-signature-verification",
        feature = "certified-proof-verification"
    ))]
    #[error(transparent)]
    Credential(#[from] crate::portable_credential::PortableCredentialError),
    #[error(transparent)]
    Claims(#[from] PortableClaimsError),
    #[error(transparent)]
    Encoding(#[from] PortableEncodingError),
    #[error(transparent)]
    Revocation(#[from] LoginRevocationError),
}

impl From<AdmissionBudgetError> for SessionError {
    fn from(error: AdmissionBudgetError) -> Self {
        match error {
            AdmissionBudgetError::InvalidPolicy => Self::InvalidPolicyOrRecord,
            AdmissionBudgetError::Exhausted => Self::AdmissionThrottled,
            AdmissionBudgetError::ClockRollback => Self::AdmissionClockRollback,
        }
    }
}

// Stored record checks retain the engine's existing rejection surface.
impl From<SessionRequestError> for SessionError {
    fn from(error: SessionRequestError) -> Self {
        match error {
            SessionRequestError::InvalidRequest | SessionRequestError::InvalidLifetime => {
                Self::InvalidPolicyOrRecord
            }
            SessionRequestError::ScopeRejected => Self::ScopeRejected,
            SessionRequestError::Canonical(error) => Self::Canonical(error),
        }
    }
}

/// Admit bounded canonical portable claims through either complete verifier.
/// `verify` MUST consume its DecodedPortableClaims with `verify`, using the same
/// protected authority snapshot and provided budget. This shares one decode and
/// digest; the callback runs only for fresh admission, never exact retries or cheap
/// refusals.
/// The sealed verified result must match the submitted claims and current time.
/// Host serialization covers the entire call, including both store transactions,
/// the revocation ledger and authority. No await/reentrancy is permitted. Budget
/// charges survive ordinary errors; the host must not trap and roll them back.
#[allow(
    clippy::too_many_arguments,
    reason = "Keep independently host-owned state and protected inputs explicit at the admission boundary"
)]
pub fn establish_session(
    store: &mut impl SessionStore,
    ledger: &mut MemoryLoginRevocations,
    budget: &mut SessionAdmissionBudget,
    encoded_claims: &[u8],
    request: &SessionRequest,
    context: &SessionContext<'_>,
    policy: SessionPolicy,
    verify: impl FnOnce(
        &mut SessionAdmissionBudget,
        DecodedPortableClaims,
    ) -> Result<VerifiedPortableClaims, SessionError>,
) -> Result<SessionEstablished, SessionError> {
    context.validate(policy, ledger)?;
    let authority = &context.authority;
    let decoded = DecodedPortableClaims::decode(encoded_claims)?;
    let claims = &decoded.claims;
    let fingerprint = decoded.fingerprint;
    if ledger.is_revoked(authority.caller, authority.now_ns)? {
        return Err(LoginRevocationError::Revoked.into());
    }
    let retry = store.transaction(None, |tx| {
        if tx.generation() != policy.generation {
            return Err(SessionError::StaleAuthority);
        }
        let previous = tx.session(authority.caller)?;
        if let Some(session) = &previous
            && session.fingerprint() == fingerprint
        {
            if session.request_hash() != request.hash() {
                return Err(SessionError::ReplayConflict);
            }
            session.check_live(context, policy)?;
            session.check_revocation(ledger, authority.now_ns)?;
            return Ok(Some(session.clone()));
        }
        if tx.replay_exists(fingerprint)? {
            return Err(SessionError::ReplayConflict);
        }
        validate_claims_context(claims, authority)?;
        let grant = claims
            .grants
            .iter()
            .find(|grant| &grant.target == context.role)
            .ok_or(SessionError::ScopeRejected)?;
        request.check_granted_scopes(&grant.scopes)?;
        request.expires_at_ns(
            authority.now_ns,
            policy.default_ttl_secs,
            policy.max_ttl_secs,
        )?;
        let limits = tx.limits();
        limits.validate()?;
        limits.check_capacity(tx.occupancy(authority.caller)?, previous.is_some())?;
        ledger.check_reservation(authority.caller, claims, authority.now_ns)?;
        Ok(None)
    })?;
    if let Some(session) = retry {
        return Ok(SessionEstablished::ExactRetry(session));
    }
    let admission_expires_at_ns = claims.admission_expires_at_ns;
    let verified = verify(budget, decoded)?;
    if verified.fingerprint() != fingerprint || verified.verified_at_ns() != authority.now_ns {
        return Err(SessionError::InvalidPolicyOrRecord);
    }
    let session = Session::admit(&verified, request, context, policy)?;
    let replay = Replay::new(&session, admission_expires_at_ns);
    ledger.with_reservation(authority.caller, &verified, authority.now_ns, |change| {
        store.transaction(Some(change), |tx| {
            if tx.generation() != policy.generation {
                return Err(SessionError::StaleAuthority);
            }
            let replacing = tx.session(authority.caller)?.is_some();
            tx.stage(session.clone(), replay)?;
            Ok(if replacing {
                SessionEstablished::Replaced(session)
            } else {
                SessionEstablished::Created(session)
            })
        })
    })
}

/// Check the caller's local session against current enrollment, generation,
/// revocation and required scopes. No signature verification or lease extension.
/// Applications separately enforce resource ownership and effects after awaits.
pub fn authorize_session(
    store: &mut impl SessionStore,
    ledger: &mut MemoryLoginRevocations,
    context: &SessionContext<'_>,
    policy: SessionPolicy,
) -> Result<Session, SessionError> {
    context.validate(policy, ledger)?;
    if ledger.is_revoked(context.authority.caller, context.authority.now_ns)? {
        return Err(LoginRevocationError::Revoked.into());
    }
    store.transaction(None, |tx| {
        if tx.generation() != policy.generation {
            return Err(SessionError::StaleAuthority);
        }
        let session = tx
            .session(context.authority.caller)?
            .ok_or(SessionError::MissingOrExpired)?;
        session.check_live(context, policy)?;
        session.check_revocation(ledger, context.authority.now_ns)?;
        if context
            .required_scopes
            .iter()
            .any(|scope| !session.scopes().contains(scope))
        {
            return Err(SessionError::ScopeRejected);
        }
        Ok(session)
    })
}

/// Authenticate logout with the runtime caller even if proof/enrollment expired
/// or admission is disabled. Commit session removal before acknowledging denial;
/// replay records remain. Host startup readiness and serialization still apply.
pub fn logout_session(
    store: &mut impl SessionStore,
    ledger: &mut MemoryLoginRevocations,
    unknown_budget: &mut SessionAdmissionBudget,
    caller: Principal,
    now_ns: u64,
) -> LogoutResult {
    let result = ledger.with_revocation(unknown_budget, caller, now_ns, |change| {
        store
            .transaction(Some(change), |tx| tx.remove(caller))
            .map_err(|_| LoginRevocationError::Storage)
    });
    // Backend diagnostics stay host-side; never acknowledge a failed commit.
    match result {
        Ok(deny_until_ns) => LogoutResult::Revoked { deny_until_ns },
        Err(LoginRevocationError::InvalidCaller) => LogoutResult::InvalidCaller,
        Err(LoginRevocationError::Capacity) => {
            LogoutResult::Unavailable(LogoutUnavailable::Capacity)
        }
        Err(LoginRevocationError::Clock) => LogoutResult::Unavailable(LogoutUnavailable::Clock),
        Err(
            LoginRevocationError::InvalidPolicy
            | LoginRevocationError::Revoked
            | LoginRevocationError::Storage,
        ) => LogoutResult::Unavailable(LogoutUnavailable::Storage),
    }
}
