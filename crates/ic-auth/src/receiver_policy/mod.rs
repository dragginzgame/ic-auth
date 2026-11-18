//! Private decisions over bounded facts selected by the receiver.
//!
//! These views are not storage records, credentials or authorization tokens.
//! A host/backend must supply a complete isolated snapshot from trusted state,
//! admit persisted corruption before projection, retain clock observations on
//! normal refusals, and own all transactions. No function here creates a session,
//! verifies a proof, updates a budget or commits state. Never project request
//! fields into these facts. Only the receiver supplies them to these helpers;
//! they are not a public backend extension or testing API.

#[cfg(test)]
mod tests;

use crate::{
    claims::validate_grants,
    grant_policy::{for_role, scopes_subset},
    session_request::SessionRequest,
};
use ic_auth_protocol_types::{
    AudienceId, AuthRole, DelegatedRoleGrant, PortableMechanism, Principal, ScopeLabel,
};
use std::{error::Error, fmt};

mod login;
pub use login::LoginLimits;
pub(crate) use login::{LoginDecision, LoginSnapshot, assess_login};
mod capacity;
pub(crate) use capacity::{AdmissionFootprint, assess_capacity};
pub use capacity::{SessionCapacity, SessionUsage};

#[cfg(feature = "receiver-memory")]
mod logout;
#[cfg(feature = "receiver-memory")]
pub(crate) use logout::plan_logout;
#[cfg(feature = "receiver-memory")]
pub use logout::{LogoutError, LogoutReceipt, UnknownLogoutLimits};

/// Immutable login association retained independently of session lease expiry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoginBinding {
    /// Issuer qualifying the account.
    pub issuer: Principal,
    /// Issuer's account subject, not the browser caller.
    pub subject: Principal,
    /// Issuer-assigned device slot, 0 through 7.
    pub device_slot: u8,
    /// Nonzero slot generation.
    pub device_generation: u64,
    /// Original inclusive login start.
    pub started_at_ns: u64,
    /// Original exclusive absolute login deadline, never extended by renewal.
    pub deadline_ns: u64,
}

/// Borrowed facts from an already admitted session; construction authenticates nothing.
#[derive(Clone, Copy, Debug)]
pub struct SessionFacts<'a> {
    /// Stored browser caller.
    pub caller: Principal,
    /// Stored audience.
    pub audience: AudienceId,
    /// Retained immutable account/device/login association.
    pub login: LoginBinding,
    /// Proof mechanism captured at admission.
    pub mechanism: PortableMechanism,
    /// Enrolled key binding captured at admission.
    pub key_binding_id: [u8; 32],
    /// Protected role captured at admission.
    pub role: &'a AuthRole,
    /// Canonical narrowed scopes, at most 16.
    pub scopes: &'a [ScopeLabel],
    /// Receiver authority generation captured at admission.
    pub authority_generation: u64,
    /// Original host admission time.
    pub established_at_ns: u64,
    /// Original exclusive lease expiry.
    pub expires_at_ns: u64,
    /// SHA-256 of exact canonical claims bytes.
    pub fingerprint: [u8; 32],
    /// Canonical unsigned request hash, including TTL presence.
    pub request_hash: [u8; 32],
}

/// Selected caller's retained login reservation or denial.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LoginFacts {
    /// Lookup key; must match the authenticated caller.
    pub caller: Principal,
    /// Retained audience association.
    pub audience: AudienceId,
    /// Known login association; an unknown denial has none.
    pub binding: Option<LoginBinding>,
    /// Exclusive reservation or denial retention deadline.
    pub until_ns: u64,
    /// Whether this row denies the caller while its retention remains live.
    pub denied: bool,
}

/// Retained slot index/fence selected for the session's issuer-qualified account.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DeviceFacts {
    /// Issuer qualifying the slot key.
    pub issuer: Principal,
    /// Account subject qualifying the slot key.
    pub subject: Principal,
    /// Slot key.
    pub slot: u8,
    /// Current retained slot generation.
    pub generation: u64,
    /// Caller owning this generation.
    pub caller: Principal,
}

/// Bounded facts from one host transaction; not a backend trait or stored format.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ReceiverSnapshot<'a> {
    /// Receiver's protected audience.
    pub audience: AudienceId,
    /// Receiver's current authority generation.
    pub authority_generation: u64,
    /// Retained trusted-clock high-water mark before this operation.
    pub observed_at_ns: u64,
    /// Session selected by authenticated caller.
    pub session: Option<&'a SessionFacts<'a>>,
    /// Login row selected by authenticated caller, including retained denials.
    pub login: Option<&'a LoginFacts>,
    /// Slot association selected from the stored session, never submitted identity.
    pub device: Option<&'a DeviceFacts>,
}

/// Current protected authority and application policy, independent of proof freshness.
#[derive(Clone, Copy, Debug)]
pub struct SessionAuthority<'a> {
    /// Actual authenticated runtime caller.
    pub caller: Principal,
    /// Trusted current time.
    pub now_ns: u64,
    /// Current receiver audience.
    pub audience: AudienceId,
    /// Enrolled issuer.
    pub issuer: Principal,
    /// Current enrolled mechanism.
    pub mechanism: PortableMechanism,
    /// Current enrolled key binding.
    pub key_binding_id: [u8; 32],
    /// Protected receiver role.
    pub role: &'a AuthRole,
    /// Current bounded canonical grants; unrelated roles cannot supply scopes.
    pub grants: &'a [DelegatedRoleGrant],
    /// Current protected authority generation, changed atomically with authority.
    pub generation: u64,
    /// Inclusive current key/authority start.
    pub not_before_ns: u64,
    /// Exclusive current key/authority end.
    pub expires_at_ns: u64,
    /// Exclusive current enrollment freshness end.
    pub enrollment_expires_at_ns: u64,
    /// Current inclusive login reset floor.
    pub min_login_started_at_ns: u64,
    /// Maximum original session lease duration, in seconds, 1 through 1800.
    pub max_session_ttl_secs: u64,
    /// Protected receiver enablement.
    pub enabled: bool,
    /// Application eligibility for the selected stored account.
    pub subject_admissible: bool,
}

/// Policy refusal; no variant implies a state mutation or completed host observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionPolicyError {
    /// Receiver disabled or account ineligible.
    Disabled,
    /// Invalid protected inputs or mismatched receiver domain.
    InvalidContext,
    /// Current time predates the retained clock observation.
    ClockRollback,
    /// Selected records are malformed, missing their reservation/fence or inconsistent.
    InvalidFacts,
    /// Caller has a live retained denial.
    Revoked,
    /// No session, not yet established, or exclusive lease deadline reached.
    MissingOrExpired,
    /// Changed authority, generation, key, login floor or lease ceiling.
    StaleAuthority,
    /// Retained or required scopes exceed the applicable single-role grant.
    ScopeRejected,
    /// Consumed claims or changed narrowing request cannot be admitted again.
    ReplayConflict,
    /// An existing browser's immutable account/device/login association differs.
    BindingConflict,
    /// A new known login would exceed its protected physical pool capacity.
    Capacity,
    /// Encoded session/replay bytes exceed the protected ceiling or arithmetic range.
    ByteCapacity,
}
impl fmt::Display for SessionPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "receiver policy refused: {self:?}")
    }
}
impl Error for SessionPolicyError {}

/// Retry classification only; neither variant is a transferable permission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RetryDecision {
    /// No exact live retry exists. Every fresh admission check still remains.
    FreshProofRequired,
    /// The retained session passed current checks; return its original receipt.
    /// The deadline is unchanged and no proof or budget operation is required.
    ExactRetry {
        /// Original exclusive lease deadline; not a newly calculated lease.
        expires_at_ns: u64,
    },
}

/// Assess a proof-free read against current state and protected operation scopes.
///
/// Requirements may be empty, duplicated or unordered, but contain at most 16
/// validated labels. Snapshot provenance, backend corruption admission, trusted
/// clock observation and application effects remain the host's responsibility.
/// Recheck after any await; this result cannot authorize a later transaction.
pub(crate) fn assess_session(
    snapshot: &ReceiverSnapshot<'_>,
    authority: &SessionAuthority<'_>,
    required_scopes: &[ScopeLabel],
) -> Result<(), SessionPolicyError> {
    if required_scopes.len() > 16 {
        return Err(SessionPolicyError::InvalidContext);
    }
    check_snapshot(snapshot, authority)?;
    let session = snapshot
        .session
        .ok_or(SessionPolicyError::MissingOrExpired)?;
    check_live(session, snapshot, authority)?;
    if !scopes_subset(required_scopes, session.scopes) {
        return Err(SessionPolicyError::ScopeRejected);
    }
    Ok(())
}

/// Assess an exact retry before fresh-proof work or capacity checks.
///
/// The caller first bounds/decodes canonical claims and checks browser/caller
/// binding; `fingerprint` is their exact digest, not a request-supplied assertion.
/// `replay_consumed` comes from the matching protected replay lookup. A matching
/// stored fingerprint/request still needs all live checks. Proof bytes and expiry
/// are intentionally absent; an exact retry may outlive its admission proof.
/// A fresh result allows further checking only, never a write or authorization.
pub(crate) fn assess_retry(
    snapshot: &ReceiverSnapshot<'_>,
    authority: &SessionAuthority<'_>,
    fingerprint: [u8; 32],
    request: &SessionRequest,
    replay_consumed: bool,
) -> Result<RetryDecision, SessionPolicyError> {
    check_snapshot(snapshot, authority)?;
    if let Some(session) = snapshot.session
        && session.fingerprint == fingerprint
    {
        if session.request_hash != request.hash() {
            return Err(SessionPolicyError::ReplayConflict);
        }
        check_live(session, snapshot, authority)?;
        return Ok(RetryDecision::ExactRetry {
            expires_at_ns: session.expires_at_ns,
        });
    }
    if replay_consumed {
        return Err(SessionPolicyError::ReplayConflict);
    }
    Ok(RetryDecision::FreshProofRequired)
}

fn check_snapshot(
    s: &ReceiverSnapshot<'_>,
    a: &SessionAuthority<'_>,
) -> Result<(), SessionPolicyError> {
    check_authority(a, s.audience)?;
    use SessionPolicyError as E;
    if a.now_ns < s.observed_at_ns {
        return Err(E::ClockRollback);
    }
    if let Some(login) = s.login {
        if login.caller != a.caller || login.audience != s.audience || login.until_ns == 0 {
            return Err(E::InvalidFacts);
        }
        if login.denied && a.now_ns < login.until_ns {
            return Err(E::Revoked);
        }
    }
    if s.authority_generation != a.generation {
        return Err(E::StaleAuthority);
    }
    Ok(())
}

pub(crate) fn check_authority(
    a: &SessionAuthority<'_>,
    audience: AudienceId,
) -> Result<(), SessionPolicyError> {
    use SessionPolicyError as E;
    if !a.enabled || !a.subject_admissible {
        return Err(E::Disabled);
    }
    if !login_caller(a.caller)
        || a.issuer == Principal::anonymous()
        || a.audience != audience
        || a.not_before_ns >= a.expires_at_ns
        || a.enrollment_expires_at_ns == 0
        || !(1..=1800).contains(&a.max_session_ttl_secs)
        || validate_grants(a.grants).is_err()
    {
        return Err(E::InvalidContext);
    }
    Ok(())
}

fn check_live(
    s: &SessionFacts<'_>,
    facts: &ReceiverSnapshot<'_>,
    a: &SessionAuthority<'_>,
) -> Result<(), SessionPolicyError> {
    use SessionPolicyError as E;
    let b = s.login;
    if !(1..=16).contains(&s.scopes.len())
        || s.scopes.windows(2).any(|pair| pair[0] >= pair[1])
        || !valid_binding(b)
        || b.started_at_ns > s.established_at_ns
        || s.established_at_ns >= s.expires_at_ns
        || s.expires_at_ns > b.deadline_ns
        || s.expires_at_ns - s.established_at_ns > 1_800_000_000_000
    {
        return Err(E::InvalidFacts);
    }
    if a.now_ns < s.established_at_ns || a.now_ns >= s.expires_at_ns {
        return Err(E::MissingOrExpired);
    }
    if s.caller != a.caller
        || s.audience != a.audience
        || b.issuer != a.issuer
        || s.mechanism != a.mechanism
        || s.key_binding_id != a.key_binding_id
        || s.role != a.role
        || s.authority_generation != a.generation
        || b.started_at_ns < a.min_login_started_at_ns
        || a.now_ns < a.not_before_ns
        || a.now_ns >= a.expires_at_ns
        || a.now_ns >= a.enrollment_expires_at_ns
        || s.expires_at_ns > a.expires_at_ns
        || s.expires_at_ns > a.enrollment_expires_at_ns
        || s.expires_at_ns - s.established_at_ns > a.max_session_ttl_secs * 1_000_000_000
    {
        return Err(E::StaleAuthority);
    }
    let grant = for_role(a.grants, a.role).ok_or(E::ScopeRejected)?;
    if !scopes_subset(s.scopes, &grant.scopes) {
        return Err(E::ScopeRejected);
    }
    let login = facts.login.ok_or(E::InvalidFacts)?;
    let device = facts.device.ok_or(E::InvalidFacts)?;
    if login.denied
        || login.until_ns < b.deadline_ns
        || login.until_ns <= a.now_ns
        || login.binding != Some(b)
        || device.issuer != b.issuer
        || device.subject != b.subject
        || device.slot != b.device_slot
        || device.generation != b.device_generation
        || device.caller != s.caller
    {
        return Err(E::InvalidFacts);
    }
    Ok(())
}

pub(crate) fn login_caller(caller: Principal) -> bool {
    let bytes = caller.as_slice();
    bytes.len() == 29 && bytes[28] == 2
}

fn valid_binding(binding: LoginBinding) -> bool {
    binding.issuer != Principal::anonymous()
        && binding.subject != Principal::anonymous()
        && binding.device_slot < 8
        && binding.device_generation != 0
        && binding
            .deadline_ns
            .checked_sub(binding.started_at_ns)
            .is_some_and(|ttl| ttl > 0 && ttl <= 90 * 86_400_000_000_000)
}
