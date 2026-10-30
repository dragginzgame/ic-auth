//! Bounded admission diagnostics, never evidence that a proof is authentic.
use serde::{Deserialize, Serialize};

/// A receiver refusal. Only `NotYetValid` identifies a future-start retry hint;
/// it grants no clock grace, new issuance, deadline extension or authorization.
/// The caller owns bounded retries of the same credential and login generation.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum AdmissionRefusal {
    NotYetValid,
    Expired,
    InvalidCredential,
    AuthorityUnavailable,
    BeforeReset,
    ScopeRejected,
    ReplayConflict,
    Revoked,
    Capacity,
    Throttled,
    Unavailable,
}
/// Proof-free session-read refusal; detailed host diagnostics are not transported.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SessionReadRefusal {
    MissingOrExpired,
    AuthorityUnavailable,
    ScopeRejected,
    Revoked,
    Unavailable,
}
