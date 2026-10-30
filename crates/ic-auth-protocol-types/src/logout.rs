//! Caller-authenticated logout transport; no proof or target identity is accepted.
use serde::{Deserialize, Serialize};

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
