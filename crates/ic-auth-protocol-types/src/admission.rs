//! Bounded admission diagnostics, never evidence that a proof is authentic.
use serde::{Deserialize, Serialize};

/// A receiver refusal. Only `NotYetValid` identifies a future-start retry hint;
/// it grants no clock grace, new issuance, deadline extension or authorization.
/// The caller owns bounded retries of the same credential and login generation.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum AdmissionRefusal {
    /// The claimed validity window has not started; authenticity is not implied.
    NotYetValid,
    /// The credential or session validity window has ended.
    Expired,
    /// Encoding, identity binding, claims or cryptographic verification failed.
    InvalidCredential,
    /// Current protected authority is disabled, stale or unavailable.
    AuthorityUnavailable,
    /// The credential predates the receiver's protected reset boundary.
    BeforeReset,
    /// The requested scopes are not admitted for this session.
    ScopeRejected,
    /// Retained replay state conflicts with this admission.
    ReplayConflict,
    /// A retained revocation denies this login.
    Revoked,
    /// A bounded record or byte capacity cannot accommodate admission.
    Capacity,
    /// The receiver's verification work budget is exhausted.
    Throttled,
    /// Admission cannot complete because of local policy, clock or storage state.
    Unavailable,
}
/// Proof-free session-read refusal; detailed host diagnostics are not transported.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SessionReadRefusal {
    /// No usable session remains for the authenticated caller.
    MissingOrExpired,
    /// The session's authority is disabled or no longer current.
    AuthorityUnavailable,
    /// The session does not contain the required scope.
    ScopeRejected,
    /// Retained revocation state denies the login.
    Revoked,
    /// Local state prevents a reliable session decision.
    Unavailable,
}
