use crate::{
    admission_budget::SessionAdmissionBudget, persistent::StoreError, session::SessionLimits,
};
use ic_auth_protocol_types::AudienceId;
use serde::{Deserialize, Serialize};

/// Bounds on retained login fences and device slots, separate from session rows.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoginLimits {
    /// Maximum authenticated login records retained by this receiver.
    pub admitted: usize,
    /// Maximum denial records whose login lifetime is not authenticated.
    pub unknown: usize,
    /// Maximum accepted absolute login lifetime in nanoseconds.
    pub maximum_ttl_ns: u64,
    /// Maximum device slots per issuer-qualified account.
    pub devices_per_account: u8,
}
/// Protected token-bucket policy; the persistent store retains its current credit.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BudgetPolicy {
    /// Maximum accumulated work units, not necessarily a count of requests.
    pub burst: u32,
    /// Trusted nanoseconds required to replenish one work unit.
    pub refill_interval_ns: u64,
}

/// Immutable protected receiver configuration. Limits are admitted on creation
/// and compared exactly on open; no reset, reinterpretation or silent shrinking.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReceiverConfig {
    #[serde(with = "crate::storage_codec::audience")]
    pub(super) audience: AudienceId,
    pub(super) sessions: [usize; 4],
    pub(super) logins: LoginLimits,
    pub(super) admission: BudgetPolicy,
    pub(super) logout: BudgetPolicy,
    pub(super) verification_costs: crate::admission_budget::VerificationCosts,
}
impl ReceiverConfig {
    /// Validate session/login limits and explicit verification/logout budgets.
    /// Returns `StoreError::Invalid` for zero, inconsistent or unsupported limits.
    pub fn new(
        audience: AudienceId,
        sessions: SessionLimits,
        logins: LoginLimits,
        admission: BudgetPolicy,
        logout: BudgetPolicy,
        verification_costs: crate::admission_budget::VerificationCosts,
    ) -> Result<Self, StoreError> {
        let config = Self {
            audience,
            sessions: [
                sessions.max_sessions(),
                sessions.max_replays(),
                sessions.max_replays_per_account(),
                sessions.max_encoded_bytes(),
            ],
            logins,
            admission,
            logout,
            verification_costs,
        };
        config.validate()?;
        Ok(config)
    }
    pub(super) fn limits(&self) -> Result<SessionLimits, StoreError> {
        SessionLimits::new(
            self.sessions[0],
            self.sessions[1],
            self.sessions[2],
            self.sessions[3],
        )
        .map_err(|_| StoreError::Invalid)
    }
    pub(super) fn validate(&self) -> Result<(), StoreError> {
        SessionAdmissionBudget::for_verification(
            self.admission.burst,
            self.admission.refill_interval_ns,
            self.verification_costs,
            0,
        )
        .map_err(|_| StoreError::Invalid)?;
        self.limits()?;
        let login = self.logins;
        if login.admitted == 0
            || login.unknown == 0
            || login.maximum_ttl_ns == 0
            || login.maximum_ttl_ns > crate::limits::MAX_LOGIN_NS
            || !(1..=ic_auth_protocol_types::limits::MAX_DEVICE_SLOTS)
                .contains(&login.devices_per_account)
            || self.admission.burst == 0
            || self.admission.refill_interval_ns == 0
            || self.logout.burst == 0
            || self.logout.refill_interval_ns == 0
        {
            return Err(StoreError::Invalid);
        }
        Ok(())
    }
}

/// Retained receiver counts and encoded payload accounting, excluding index overhead.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReceiverOccupancy {
    /// Active retained session rows, including any not yet pruned after expiry.
    pub sessions: usize,
    /// Retained credential replay records.
    pub replays: usize,
    /// Retained authenticated login bindings and fences.
    pub admitted_logins: usize,
    /// Retained logout denials without an authenticated login lifetime.
    pub unknown_denials: usize,
    /// Encoded session/replay bytes charged against configured capacity.
    pub encoded_bytes: usize,
}
/// Counts removed by one bounded receiver cleanup operation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReceiverPruned {
    /// Expired session rows removed, preserving required login fences.
    pub sessions: u32,
    /// Expired replay records removed.
    pub replays: u32,
    /// Expired unknown-lifetime denial records removed.
    pub unknown_denials: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Header {
    pub schema: u64,
    pub config: ReceiverConfig,
    pub generation: u64,
    pub observed_ns: u64,
    pub counts: ReceiverOccupancy,
    pub replay_accounts: usize,
    pub admission: SessionAdmissionBudget,
    pub logout: SessionAdmissionBudget,
}

// An active session already carries its reserved binding. Materialize the full
// fence only when the session is removed; never persist two copies of that binding.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) enum CallerRecord {
    Session(#[serde(with = "serde_bytes")] Vec<u8>),
    Fence(#[serde(with = "serde_bytes")] Vec<u8>),
}
