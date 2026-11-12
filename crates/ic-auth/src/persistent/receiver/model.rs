use crate::{
    admission_budget::SessionAdmissionBudget, persistent::StoreError, session::SessionLimits,
};
use ic_auth_protocol_types::AudienceId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoginLimits {
    pub admitted: usize,
    pub unknown: usize,
    pub maximum_ttl_ns: u64,
    pub devices_per_account: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BudgetPolicy {
    pub burst: u32,
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
}
impl ReceiverConfig {
    pub fn new(
        audience: AudienceId,
        sessions: SessionLimits,
        logins: LoginLimits,
        admission: BudgetPolicy,
        logout: BudgetPolicy,
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReceiverOccupancy {
    pub sessions: usize,
    pub replays: usize,
    pub admitted_logins: usize,
    pub unknown_denials: usize,
    pub encoded_bytes: usize,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReceiverPruned {
    pub sessions: u32,
    pub replays: u32,
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
