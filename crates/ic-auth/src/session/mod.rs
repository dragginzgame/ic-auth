//! One portable session lifecycle for both complete proof mechanisms, with atomic replay
//! consumption. Hosts own protected context, generation changes and durable
//! transactions. Application resource policy and IC ingress delegation are separate.

mod memory;
mod model;

pub use memory::{MemorySessionStore, SessionPruned};
pub use model::{Replay, Session, SessionLimits, SessionPolicy};

use crate::{
    admission_budget::{AdmissionBudgetError, SessionAdmissionBudget},
    canonical::CanonicalAuthError,
    portable::{
        LoginRevocationError, MemoryLoginRevocations, PortableClaimsContext, PortableClaimsError,
        PortableEncodingError, VerifiedPortableClaims, claims_hash, decode_claims,
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
            || self.authority.issuer == Principal::anonymous()
            || self.authority.authority_not_before_ns >= self.authority.authority_expires_at_ns
            || crate::certificate_issuance::validate_grants(self.authority.grants, 16, 32).is_err()
            || self.required_scopes.len() > 16
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
    /// stale generation, consumed proofs and byte/capacity limits before writes.
    fn stage(&mut self, session: Session, replay: Replay) -> Result<(), SessionError>;
    /// Stage removal of this caller's session only; replay evidence is retained.
    fn remove(&mut self, caller: Principal) -> Result<(), SessionError>;
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
        feature = "token-verification"
    ))]
    #[error(transparent)]
    Credential(#[from] crate::portable_credential::PortableCredentialError),
    #[error(transparent)]
    Claims(#[from] PortableClaimsError),
    #[error(transparent)]
    Encoding(#[from] PortableEncodingError),
    #[error(transparent)]
    Revocation(#[from] LoginRevocationError),
    #[cfg(feature = "portable-signature-verification")]
    #[error(transparent)]
    Signature(#[from] crate::portable_signature::PortableSignatureError),
    #[cfg(feature = "token-verification")]
    #[error(transparent)]
    Certified(#[from] crate::portable_certified::PortableCertifiedError),
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
/// `verify` MUST use the same protected authority snapshot and provided budget;
/// it runs only for fresh admission, never exact retries or cheap refusals.
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
    verify: impl FnOnce(&mut SessionAdmissionBudget) -> Result<VerifiedPortableClaims, SessionError>,
) -> Result<SessionEstablished, SessionError> {
    context.validate(policy, ledger)?;
    let authority = &context.authority;
    let claims = decode_claims(encoded_claims)?;
    let fingerprint = claims_hash(&claims)?;
    if ledger.is_revoked(authority.caller, authority.now_ns)? {
        return Err(LoginRevocationError::Revoked.into());
    }
    let retry = store.transaction(|tx| {
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
        validate_claims_context(&claims, authority)?;
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
        ledger.check_reservation(authority.caller, &claims, authority.now_ns)?;
        Ok(None)
    })?;
    if let Some(session) = retry {
        return Ok(SessionEstablished::ExactRetry(session));
    }
    let verified = verify(budget)?;
    if verified.claims() != &claims || verified.verified_at_ns() != authority.now_ns {
        return Err(SessionError::InvalidPolicyOrRecord);
    }
    let session = Session::admit(&verified, request, context, policy)?;
    let replay = Replay::new(&session, claims.admission_expires_at_ns);
    let mut storage_error = None;
    let result = ledger.with_reservation(authority.caller, &verified, authority.now_ns, || {
        store
            .transaction(|tx| {
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
            .map_err(|error| {
                storage_error = Some(error);
                LoginRevocationError::Storage
            })
    });
    match storage_error {
        Some(error) => Err(error),
        None => result.map_err(SessionError::from),
    }
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
    store.transaction(|tx| {
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
    let mut storage_error = None;
    let result = ledger.with_revocation(unknown_budget, caller, now_ns, || {
        store.transaction(|tx| tx.remove(caller)).map_err(|error| {
            storage_error = Some(error);
            LoginRevocationError::Storage
        })
    });
    // Backend diagnostics stay host-side; never acknowledge a failed commit.
    if storage_error.is_some() {
        return LogoutResult::Unavailable(LogoutUnavailable::Storage);
    }
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
