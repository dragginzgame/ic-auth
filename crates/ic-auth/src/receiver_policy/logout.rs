//! Shared logout decisions; publication and work charging belong to the backend.

use super::{LoginFacts, login, login_caller};
use ic_auth_protocol_types::{AudienceId, Principal};
use std::{error::Error, fmt};

/// Protected unknown-login pool and operation budget, separate from known logins
/// and proof work. Denial retention uses the receiver's maximum login lifetime.
#[derive(Clone, Copy, Debug)]
pub struct UnknownLogoutLimits {
    /// Physical unknown denial rows, including expired rows not yet pruned.
    pub max_denials: usize,
    /// Positive operation bucket capacity; credit starts empty.
    pub capacity: u32,
    /// Positive nanoseconds per replenished operation credit.
    pub refill_interval_ns: u64,
}

/// Committed local logout facts; neither variant acknowledges other receivers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogoutReceipt {
    /// Known login denied through its authenticated deadline, retaining any
    /// longer denial. An expired known fence can return its original deadline.
    Revoked {
        /// Exclusive retained denial deadline in nanoseconds.
        deny_until_ns: u64,
    },
    /// Temporary denial without an authenticated login lifetime. This remains
    /// unconfirmed even on retry: unseen future credentials may outlive it.
    Unconfirmed {
        /// Exclusive temporary deadline; never proof of global revocation.
        deny_until_ns: u64,
    },
}
impl LogoutReceipt {
    /// Exact local denial deadline; inspect the variant before acknowledging logout.
    #[must_use]
    pub const fn deny_until_ns(self) -> u64 {
        match self {
            Self::Revoked { deny_until_ns } | Self::Unconfirmed { deny_until_ns } => deny_until_ns,
        }
    }
}

/// No logout acknowledgement may be inferred from a refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogoutError {
    /// Runtime caller is not a nonanonymous self-authenticating browser principal.
    InvalidCaller,
    /// Unknown-denial physical capacity or operation credit is exhausted.
    Capacity,
    /// Trusted clock moved backwards or retention arithmetic overflowed.
    Clock,
    /// Receiver is not ready; host recovery must establish its safety state.
    Closed,
    /// Selected retained facts are inconsistent; never replace them with absence.
    InvalidFacts,
}
impl fmt::Display for LogoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "logout refused: {self:?}")
    }
}
impl Error for LogoutError {}

pub(crate) struct LogoutPlan {
    pub(crate) receipt: LogoutReceipt,
    pub(crate) charge_unknown: bool,
}

pub(crate) fn plan_logout(
    caller: Principal,
    audience: AudienceId,
    previous: Option<&LoginFacts>,
    now_ns: u64,
    retention_ns: u64,
    unknown_pool: (usize, usize),
) -> Result<LogoutPlan, LogoutError> {
    if !login_caller(caller) {
        return Err(LogoutError::InvalidCaller);
    }
    if let Some(old) = previous {
        login::check_record(old, audience).map_err(|_| LogoutError::InvalidFacts)?;
        if old.caller != caller {
            return Err(LogoutError::InvalidFacts);
        }
        if old.binding.is_some() {
            return Ok(LogoutPlan {
                receipt: LogoutReceipt::Revoked {
                    deny_until_ns: old.until_ns,
                },
                charge_unknown: false,
            });
        }
        if now_ns < old.until_ns {
            return Ok(LogoutPlan {
                receipt: LogoutReceipt::Unconfirmed {
                    deny_until_ns: old.until_ns,
                },
                charge_unknown: false,
            });
        }
    }
    let deny_until_ns = now_ns.checked_add(retention_ns).ok_or(LogoutError::Clock)?;
    // An expired unknown row can reuse its own physical pool slot. Other expired
    // rows still count until explicit pruning; known capacity is independent.
    if previous.is_none() && unknown_pool.0 >= unknown_pool.1 {
        return Err(LogoutError::Capacity);
    }
    Ok(LogoutPlan {
        receipt: LogoutReceipt::Unconfirmed { deny_until_ns },
        charge_unknown: true,
    })
}
