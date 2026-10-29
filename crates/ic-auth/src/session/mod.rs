//! One portable session lifecycle for both complete proof mechanisms, with atomic replay
//! consumption. Hosts own protected context, generation changes and durable
//! transactions. Application resource policy and IC ingress delegation are separate.

mod memory;
mod model;
mod refusal;
mod scopes;

pub use memory::{MemorySessionStore, SessionCommit, SessionPrune, SessionPruned, SessionWrite};
pub use model::{Replay, Session, SessionLimits, SessionPolicy};
pub use scopes::RequiredScopes;

use crate::{
    admission_budget::{AdmissionBudgetError, SessionAdmissionBudget},
    canonical::CanonicalAuthError,
    portable::{
        LoginRevocationChange, LoginRevocationError, MemoryLoginRevocations, PortableAuthority,
        PortableClaimsError, PortableEncodingError,
    },
    session_request::{SessionRequestError, SessionTtlLimit},
};
use ic_auth_protocol_types::{AuthRole, LogoutResult, LogoutUnavailable, Principal};
use std::borrow::Cow;
#[cfg(any(
    feature = "portable-signature-verification",
    feature = "certified-proof-verification"
))]
use std::ops::ControlFlow;
use thiserror::Error;

/// Protected current authority, operation scopes and live session lifetime bound.
/// Proof admission limits and default TTL are absent from ordinary authorization.
/// Enrollment, caller and clock are host-authenticated inputs. Advance generation
/// atomically with authority changes; never derive them from submitted claims.
pub struct SessionContext<'a> {
    pub authority: PortableAuthority<'a>,
    pub max_ttl: SessionTtlLimit,
    pub role: &'a AuthRole,
    pub required_scopes: &'a RequiredScopes,
}
impl SessionContext<'_> {
    fn validate(
        &self,
        policy: SessionPolicy,
        ledger: &MemoryLoginRevocations,
    ) -> Result<(), SessionError> {
        if !policy.enabled || !policy.subject_admissible {
            return Err(SessionError::Disabled);
        }
        if ledger.audience() != *self.authority.audience || !self.authority.authority_shape_valid()
        {
            return Err(SessionError::InvalidPolicyOrRecord);
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

/// One host-isolated read snapshot. No writes or whole-store clones are permitted.
/// A reader cannot publish a write:
/// ```compile_fail
/// use ic_auth::session::{SessionReader, Session, Replay};
/// use ic_auth::portable::LoginRevocationChange;
/// fn write(reader: &dyn SessionReader, session: &Session, replay: &Replay, login: &LoginRevocationChange) {
///     reader.admit(session, replay, login);
/// }
/// ```
pub trait SessionReader {
    fn generation(&self) -> u64;
    fn limits(&self) -> SessionLimits;
    /// Borrow in-memory records; storage adapters may return an owned decode.
    /// The value cannot escape the host's read snapshot without copying it.
    fn session(&self, caller: Principal) -> Result<Option<Cow<'_, Session>>, SessionError>;
    fn replay_exists(&self, fingerprint: [u8; 32]) -> Result<bool, SessionError>;
    fn occupancy(&self, caller: Principal) -> Result<SessionOccupancy, SessionError>;
}

/// Host-owned atomic storage boundary. No await/reentrancy may split an engine
/// operation or change authority. Refusals and commit errors preserve prior state.
/// Admission validates generation, pair association, replay, login reservation,
/// record sizes and capacity before atomically persisting the pair and login.
/// Logout validates a revocation, removes only its caller's session and retains
/// replay history, atomically with the login tombstone.
pub trait SessionStore {
    fn read<T>(
        &mut self,
        operation: impl FnOnce(&dyn SessionReader) -> Result<T, SessionError>,
    ) -> Result<T, SessionError>;
    /// Consumes the validated pair so the backend can install it without copying.
    /// Returns true when replacing an existing session; false when creating one.
    fn admit(
        &mut self,
        session: Session,
        replay: Replay,
        login: &LoginRevocationChange,
    ) -> Result<bool, SessionError>;
    fn logout(&mut self, login: &LoginRevocationChange) -> Result<(), SessionError>;
}

/// Compact result of admission or an exact retry, never authorization evidence.
/// Full records remain in the host store. Authorize every later operation using
/// current protected authority, clock and generation; this receipt grants nothing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionReceipt {
    caller: Principal,
    issuer: Principal,
    subject: Principal,
    expires_at_ns: u64,
}
impl From<&Session> for SessionReceipt {
    fn from(session: &Session) -> Self {
        Self {
            caller: session.caller(),
            issuer: session.issuer(),
            subject: session.subject(),
            expires_at_ns: session.expires_at_ns(),
        }
    }
}
impl SessionReceipt {
    pub fn caller(&self) -> Principal {
        self.caller
    }
    pub fn issuer(&self) -> Principal {
        self.issuer
    }
    pub fn subject(&self) -> Principal {
        self.subject
    }
    pub fn expires_at_ns(&self) -> u64 {
        self.expires_at_ns
    }
}

/// Session creation, replacement, or the unchanged receipt of an exact retry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionEstablished {
    Created(SessionReceipt),
    Replaced(SessionReceipt),
    ExactRetry(SessionReceipt),
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

/// Fresh admission inputs. The selected verifier is the single source of current
/// authority and proof constraints. Authorization needs only SessionContext.
#[cfg(any(
    feature = "portable-signature-verification",
    feature = "certified-proof-verification"
))]
pub struct SessionAdmissionContext<'a> {
    pub verification: crate::portable_credential::PortableVerificationContext<'a>,
    pub role: &'a AuthRole,
    pub ttl: crate::session_request::SessionTtlPolicy,
}

/// Admit the exact credential with the protected verifier and shared budget.
/// The engine owns decoding and verification; no caller-supplied verification
/// callback can substitute claims or a different authority snapshot. Exact retries
/// and cheap refusals do not verify proofs or consume verification credit.
/// Hosts serialize the entire call across store, ledger, authority and budget.
/// No await/reentrancy is permitted, and charges survive ordinary storage errors.
#[cfg(any(
    feature = "portable-signature-verification",
    feature = "certified-proof-verification"
))]
pub fn establish_session(
    store: &mut impl SessionStore,
    ledger: &mut MemoryLoginRevocations,
    budget: &mut SessionAdmissionBudget,
    credential: &ic_auth_protocol_types::PortableCredential,
    request: &crate::session_request::SessionRequest,
    admission: &SessionAdmissionContext<'_>,
    policy: SessionPolicy,
) -> Result<SessionEstablished, SessionError> {
    let claims_context = admission.verification.claims();
    let context = SessionContext {
        authority: claims_context.authority,
        max_ttl: admission.ttl.maximum(),
        role: admission.role,
        required_scopes: RequiredScopes::NONE,
    };
    context.validate(policy, ledger)?;
    let authority = &context.authority;
    let decoded = crate::portable::DecodedPortableClaims::decode(&credential.claims)?;
    let claims = &decoded.claims;
    let fingerprint = decoded.fingerprint;
    let login = ledger.observe_login(authority.caller, authority.now_ns)?;
    let admission_expires_at_ns = claims.admission_expires_at_ns;
    let checked = store.read(|tx| {
        if tx.generation() != policy.generation {
            return Err(SessionError::StaleAuthority);
        }
        let previous = tx.session(authority.caller)?;
        let replacing = previous.is_some();
        if let Some(session) = previous
            && session.fingerprint() == fingerprint
        {
            if session.request_hash() != request.hash() {
                return Err(SessionError::ReplayConflict);
            }
            session.check_live(&context, policy)?;
            session.check_revocation(login)?;
            return Ok(ControlFlow::Break(SessionReceipt::from(session.as_ref())));
        }
        if tx.replay_exists(fingerprint)? {
            return Err(SessionError::ReplayConflict);
        }
        let prepared = admission.verification.prepare(decoded)?;
        let requested_expiry = request.evaluate(
            &prepared.decoded().claims.grants,
            admission.role,
            authority.now_ns,
            admission.ttl,
        )?;
        let limits = tx.limits();
        limits.check_capacity(tx.occupancy(authority.caller)?, replacing)?;
        ledger.check_prepared_reservation(
            authority.caller,
            prepared.contextual(),
            authority.now_ns,
        )?;
        Ok(ControlFlow::Continue((prepared, requested_expiry)))
    })?;
    let (prepared, requested_expiry) = match checked {
        ControlFlow::Break(session) => return Ok(SessionEstablished::ExactRetry(session)),
        ControlFlow::Continue(fresh) => fresh,
    };
    let verified = prepared.verify(budget, &credential.proof)?;
    let session = Session::admit(&verified, request, &context, policy, requested_expiry)?;
    let replay = Replay::new(&session, admission_expires_at_ns);
    let receipt = SessionReceipt::from(&session);
    ledger.with_reservation(authority.caller, &verified, authority.now_ns, |login| {
        let replacing = store.admit(session, replay, login)?;
        Ok(if replacing {
            SessionEstablished::Replaced(receipt)
        } else {
            SessionEstablished::Created(receipt)
        })
    })
}

/// Check the caller's local session against current enrollment, generation,
/// revocation and required scopes. No signature verification or lease extension.
/// Project the required result inside the host read snapshot without cloning a
/// full session. The callback runs only after every authorization check passes.
/// Applications separately enforce resource ownership and effects after awaits.
pub fn authorize_session<T>(
    store: &mut impl SessionStore,
    ledger: &mut MemoryLoginRevocations,
    context: &SessionContext<'_>,
    policy: SessionPolicy,
    operation: impl FnOnce(&Session) -> T,
) -> Result<T, SessionError> {
    context.validate(policy, ledger)?;
    let login = ledger.observe_login(context.authority.caller, context.authority.now_ns)?;
    store.read(|tx| {
        if tx.generation() != policy.generation {
            return Err(SessionError::StaleAuthority);
        }
        let session = tx
            .session(context.authority.caller)?
            .ok_or(SessionError::MissingOrExpired)?;
        session.check_live(context, policy)?;
        session.check_revocation(login)?;
        if !crate::certificate_issuance::scopes_are_subset(
            context.required_scopes.as_slice(),
            session.scopes(),
        ) {
            return Err(SessionError::ScopeRejected);
        }
        Ok(operation(&session))
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
            .logout(change)
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
