//! Bounded login reservation decisions, with no state publication.

use super::{
    DeviceFacts, LoginBinding, LoginFacts, SessionPolicyError as E, login_caller, valid_binding,
};
use ic_auth_protocol_types::{AudienceId, Principal};

///
/// LoginLimits
///
/// Protected device-slot capacity and login lifetime bounds.
///

#[derive(Clone, Copy, Debug)]
#[cfg_attr(
    feature = "receiver-stable",
    derive(serde::Serialize, serde::Deserialize, Eq, PartialEq)
)]
pub struct LoginLimits {
    /// Allowed issuer-assigned slots per account, 1 through 8.
    pub max_device_slots: u8,
    /// Maximum fresh login duration in nanoseconds, positive and at most 90 days.
    pub max_login_ttl_ns: u64,
    /// Maximum retained (issuer, account, device slot) rows across the receiver.
    pub max_retained_device_slots: usize,
}

impl LoginLimits {
    pub(crate) fn validate(self) -> Result<(), E> {
        if !(1..=8).contains(&self.max_device_slots)
            || self.max_login_ttl_ns == 0
            || self.max_login_ttl_ns > 90 * 86_400_000_000_000
            || self.max_retained_device_slots == 0
        {
            return Err(E::InvalidContext);
        }
        Ok(())
    }
}

///
/// LoginSnapshot
///
/// Two selected login rows and their slot association from one isolated snapshot.
/// The caller row is selected by authenticated ingress. The slot index and its
/// owner row are selected by the proposed issuer-qualified account and slot after
/// bounded canonical decoding and browser/caller binding checks. Absence must be
/// established by the backend; do not substitute missing/corrupt reads with None.
///

#[derive(Clone, Copy, Debug)]
pub(crate) struct LoginSnapshot<'a> {
    /// Protected receiver audience, already matched to claims by the operation owner.
    pub audience: AudienceId,
    /// Actual authenticated browser caller.
    pub caller: Principal,
    /// Retained clock high-water mark before this operation.
    pub observed_at_ns: u64,
    /// Existing caller reservation/denial, including expired known fences.
    pub caller_login: Option<&'a LoginFacts>,
    /// Selected slot index entry; must be present exactly when `slot_owner` is.
    pub device: Option<&'a DeviceFacts>,
    /// Selected slot owner's known login row, possibly the caller row itself.
    pub slot_owner: Option<&'a LoginFacts>,
    /// Physical known-login occupancy, including expired and denied rows.
    pub admitted_logins: usize,
}

///
/// LoginDecision
///
/// Pure preflight classification, never a reservation, verified proof or write token.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoginDecision {
    /// No retained slot owner; a new known-login row needs physical capacity.
    New,
    /// Same browser and exact immutable binding; retain any longer retention.
    Renewal,
    /// A different browser proposes a strictly higher generation for this slot.
    /// Only complete proof verification can permit atomic displacement afterward.
    Replacement {
        /// Previous caller whose session/login association would be displaced.
        displaced_caller: Principal,
    },
}

/// Assess the immutable login binding and slot fence without reserving anything.
///
/// Caller/time/audience and retained rows are protected host inputs. The proposed
/// binding remains untrusted preflight material; success permits more checks only.
/// This does not check proof freshness/signatures, current issuer authority,
/// grants, replay or session/encoded-byte capacity. After complete verification,
/// the receiver must recheck the same facts within its original transaction and
/// commit session/replay/login/index/accounting changes atomically. No decision
/// can survive an await, changed authority or changed storage snapshot.
///
/// Known caller denials and device generations survive session expiry until safely pruned.
/// Unknown caller denial expires exclusively. A same-browser renewal cannot change
/// account, slot, generation or either end of its original login window. Replacing
/// a retained slot requires a different browser and a strictly greater generation;
/// the function compares generations without incrementing or wrapping them.
/// Clock observation and work charging remain with the operation owner.
pub(crate) fn assess_login(
    snapshot: &LoginSnapshot<'_>,
    proposed: LoginBinding,
    now_ns: u64,
    limits: LoginLimits,
) -> Result<LoginDecision, E> {
    limits.validate()?;
    if !login_caller(snapshot.caller) {
        return Err(E::InvalidContext);
    }
    if now_ns < snapshot.observed_at_ns {
        return Err(E::ClockRollback);
    }
    if !valid_binding(proposed)
        || proposed.device_slot >= limits.max_device_slots
        || proposed.deadline_ns - proposed.started_at_ns > limits.max_login_ttl_ns
    {
        return Err(E::BindingConflict);
    }
    if let Some(old) = snapshot.caller_login {
        check_record(old, snapshot.audience)?;
        if old.caller != snapshot.caller {
            return Err(E::InvalidFacts);
        }
        // Pruning owns removal of known fences after their protection deadline.
        if old.binding.is_some() || now_ns < old.until_ns {
            if old.denied {
                return Err(E::Revoked);
            }
            if old.binding != Some(proposed) {
                return Err(E::BindingConflict);
            }
        }
    }
    let owner = match (snapshot.device, snapshot.slot_owner) {
        (None, None) => None,
        (Some(device), Some(owner)) => {
            check_record(owner, snapshot.audience)?;
            let old = owner.binding.ok_or(E::InvalidFacts)?;
            if device.issuer != proposed.issuer
                || device.subject != proposed.subject
                || device.slot != proposed.device_slot
                || device.caller != owner.caller
                || device.generation != old.device_generation
                || device.issuer != old.issuer
                || device.subject != old.subject
                || device.slot != old.device_slot
                || snapshot.admitted_logins == 0
            {
                return Err(E::InvalidFacts);
            }
            // Two views of the same caller row must agree; the backend may borrow
            // that row twice rather than constructing a second logical owner.
            if owner.caller == snapshot.caller && snapshot.caller_login != Some(owner) {
                return Err(E::InvalidFacts);
            }
            if owner.caller != snapshot.caller
                && snapshot
                    .caller_login
                    .is_some_and(|row| row.binding.is_some())
            {
                return Err(E::InvalidFacts);
            }
            Some((owner, old))
        }
        _ => return Err(E::InvalidFacts),
    };
    if let Some((owner, old)) = owner {
        if proposed.device_generation < old.device_generation {
            return Err(E::Revoked);
        }
        if proposed.device_generation == old.device_generation {
            if owner.caller != snapshot.caller || old != proposed || owner.denied {
                return Err(E::Revoked);
            }
            return Ok(LoginDecision::Renewal);
        }
        // The caller-binding check above already prevents upgrading one's own
        // generation; this also refuses inconsistent same-caller selected facts.
        if owner.caller == snapshot.caller {
            return Err(E::BindingConflict);
        }
        return Ok(LoginDecision::Replacement {
            displaced_caller: owner.caller,
        });
    }
    if snapshot
        .caller_login
        .is_some_and(|row| row.binding.is_some())
    {
        return Err(E::InvalidFacts); // A known caller cannot lose its slot index.
    }
    if snapshot.admitted_logins >= limits.max_retained_device_slots {
        return Err(E::Capacity);
    }
    Ok(LoginDecision::New)
}

pub(crate) fn check_record(record: &LoginFacts, audience: AudienceId) -> Result<(), E> {
    if record.audience != audience || !login_caller(record.caller) || record.until_ns == 0 {
        return Err(E::InvalidFacts);
    }
    match record.binding {
        Some(binding) if valid_binding(binding) && record.until_ns >= binding.deadline_ns => Ok(()),
        None if record.denied => Ok(()),
        _ => Err(E::InvalidFacts),
    }
}
