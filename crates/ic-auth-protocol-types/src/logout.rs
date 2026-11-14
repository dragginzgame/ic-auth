//! Caller-authenticated logout transport; no proof or target identity is accepted.
use serde::{Deserialize, Serialize};

/// Acknowledges only this destination's committed revocation, never global logout.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum LogoutResult {
    /// Committed denial through this known login's authenticated deadline.
    Revoked {
        /// Exclusive end of the committed local denial, in nanoseconds.
        deny_until_ns: u64,
    },
    /// Local denial committed without an authenticated login lifetime. This is
    /// not confirmation that every outstanding credential is revoked.
    Unconfirmed {
        /// Exclusive local denial deadline, not proof of the login's true lifetime.
        deny_until_ns: u64,
    },
    /// The authenticated ingress caller is not admissible for logout.
    InvalidCaller,
    /// No successful revocation can be acknowledged for this attempt.
    Unavailable(LogoutUnavailable),
}

/// No successful revocation is implied; transport failures also remain unknown.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum LogoutUnavailable {
    /// The bounded revocation store cannot retain the required denial.
    Capacity,
    /// The backing store could not commit or read the required state.
    Storage,
    /// Trusted time is unavailable or violates the store's clock fence.
    Clock,
    /// A protected reset must complete before logout can be acknowledged.
    ResetPending,
}
