//! Caller-authenticated logout transport; no proof or target identity is accepted.
use serde::{Deserialize, Serialize};

/// The host authenticates the caller and selects its installed trust domain.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct LogoutRequest {}

/// Acknowledges only this destination's committed revocation, never global logout.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum LogoutResult {
    Revoked { deny_until_ns: u64 },
    InvalidCaller,
    Unavailable(LogoutUnavailable),
}

/// No successful revocation is implied; transport failures also remain unknown.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum LogoutUnavailable {
    Capacity,
    Storage,
    Clock,
    ResetPending,
}
