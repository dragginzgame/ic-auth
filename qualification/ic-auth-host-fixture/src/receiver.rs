//! Test-only receiver controls; enrollment uses an installation-bound IC call.
use serde::{Deserialize, Serialize};

#[cfg(target_family = "wasm")]
pub(crate) mod canister;

/// Fixed simulator epoch; independent signed vectors use the same public time origin.
pub const TIME_ORIGIN_NS: u64 = 2_000_000_000_000_000_000;

/// Receiver control identity, outside the product contracts.
pub const CONTROL: u64 = 0x4155_5448_5155_0003;
/// Passive receiver observation identity.
pub const OBSERVE: u64 = 0x4155_5448_5155_0004;
/// Bounded test credential input, with all four Wire allowances.
pub const REQUEST: ic_wire::Limits = ic_wire::Limits {
    bytes: 4096,
    allocation: 131_072,
    nodes: 8192,
    depth: 8,
};

/// Operations use actual IC caller/time, never caller/time fields from ingress.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    /// Fetch from the installed authority; never creates missing receiver state.
    Enroll(crate::enrollment::Fault),
    /// Verify an independent Ed25519 credential and optionally trap afterward.
    Admit {
        /// Original signed bytes.
        claims: Vec<u8>,
        /// Untrusted signature candidate.
        signature: Vec<u8>,
        /// Roll back the successful admission with the enclosing IC transaction.
        trap_after: bool,
    },
    /// Project an authorized application write, optionally trapping afterward.
    Authorize(bool),
    /// Actual caller's local logout, optionally trapping afterward.
    Logout(bool),
}

/// Fixed observations separate from the product Wire result contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Outcome {
    /// Fixture enrollment installed.
    Enrolled,
    /// Admission retained the original lease, with its exact-retry indicator.
    Admitted(bool, u64),
    /// Application write committed after authorization.
    Authorized,
    /// Known local login revoked through its original deadline.
    Revoked(u64),
    /// Unknown caller remains unconfirmed through a temporary denial.
    Unconfirmed(u64),
}

/// Test categories retain the distinction between closure and normal refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Refusal {
    /// Fresh or lost state is closed.
    Closed,
    /// Retained caller denial.
    Revoked,
    /// No usable local session.
    Missing,
    /// Cryptographic proof failed after charging work.
    Proof,
    /// Enrollment response or single-key entry validation refused.
    Enrollment,
    /// Authority call failed.
    Unavailable,
    /// Other ordinary refusal.
    Other,
}

/// Passive state and application evidence; observing does not refill work credit.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// False after an upgrade loses the heap; no empty receiver is manufactured.
    pub present: bool,
    /// Physical session, replay and logical encoded-byte occupancy.
    pub usage: (u64, u64, u64),
    /// Known login fences and unknown denials.
    pub logins: (u64, u64),
    /// Retained verification and unknown-logout credit.
    pub credit: (u32, u32),
    /// Last retained clock observation, including ordinary refusals.
    pub observed_at_ns: u64,
    /// Accepted revision, generation and freshness; absent until a real reply commits.
    pub enrollment: Option<(u64, u64, u64)>,
    /// Accepted host key revocation, committed atomically with metadata.
    pub revoked: Option<bool>,
    /// Committed protected application projections.
    pub application_writes: u32,
}
