//! Test-only receiver controls; enrollment uses an installation-bound IC call.

use ic_auth_protocol_types::{AudienceId, CanonicalId};
use serde::{Deserialize, Serialize};

/// Receiver audience used by the signed login and browser fixtures.
#[must_use]
pub fn audience() -> AudienceId {
    AudienceId {
        canonical_network_id: CanonicalId::from_bytes([0x11; 32]),
        fleet_id: CanonicalId::from_bytes([0x22; 32]),
    }
}

/// Fixed simulator epoch; independent signed vectors use the same public time origin.
pub const TIME_ORIGIN_NS: u64 = 2_000_000_000_000_000_000;

/// Receiver control identity, outside the product contracts.
pub const CONTROL: u64 = 0x4155_5448_5155_0003;
/// Passive receiver observation identity.
pub const OBSERVE: u64 = 0x4155_5448_5155_0004;
/// Bounded test credential input, with all four Wire allowances.
pub const REQUEST: ic_wire::Limits = ic_wire::Limits {
    bytes: 4096,
    allocation: 524_288,
    nodes: 32768,
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
    /// Remove a bounded number of expired rows using actual host time.
    Prune(u32),
    /// Require an authority-granted scope outside the selected wide session.
    AuthorizeUnrequested,
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
    /// Removed sessions, replays and unknown denials, respectively.
    Pruned(u32, u32, u32),
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
    /// Retained replay rows for fixture subjects two and three, respectively.
    pub account_replays: (u64, u64),
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

/// Qualification-only synchronous cost observation, outside product Wire contracts.
pub const MEASURE: u64 = 0x4155_5448_5155_000a;

/// Fixture control execution costs; excludes outer request decoding/reply encoding.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Measurement {
    /// Actual control result, checked independently by the host test.
    pub result: Result<Outcome, Refusal>,
    /// IC counter-zero delta around synchronous control execution.
    pub instructions: u64,
    /// Wasm linear-memory pages before and after control execution (64 KiB each).
    pub heap_pages: (u64, u64),
    /// Encoded ordinary control response size, excluding this measurement envelope.
    pub response_bytes: u64,
}

/// Installation-owned capacity overrides for independent saturation tests.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Capacity {
    /// Physical session ceiling.
    pub sessions: u32,
    /// Physical global replay ceiling.
    pub replays: u32,
    /// Physical per-account replay ceiling.
    pub account_replays: u32,
    /// Logical session/replay encoded-byte ceiling.
    pub bytes: u32,
    /// Known-login ceiling, including retained fences.
    pub logins: u32,
    /// Verification work-credit ceiling.
    pub verification: u32,
    /// Nanoseconds per replenished work credit.
    pub refill_ns: u64,
    /// Select fixed maximum-width role/grants and a sixteen-scope session.
    pub wide_scopes: bool,
}
impl Default for Capacity {
    fn default() -> Self {
        Self {
            sessions: 1,
            replays: 2,
            account_replays: 2,
            bytes: 4096,
            logins: 1,
            verification: 10,
            refill_ns: 1_000_000_000,
            wide_scopes: false,
        }
    }
}
