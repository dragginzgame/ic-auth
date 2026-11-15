//! Bounded volatile receiver with synchronous admission and retained safety state.
//!
//! Host inputs must come from one isolated transaction. This module neither
//! authenticates enrollment nor installs endpoints. Fresh activation is only for
//! a genuinely new receiver; losing this state requires host-controlled recovery,
//! never constructing an empty replacement and resuming the old authority.

mod records;
use crate::{
    claims::ClaimsError,
    receiver_policy::{
        self as policy, AdmissionFootprint, LoginBinding, LoginDecision, LoginLimits,
        LoginSnapshot, LogoutError, LogoutReceipt, RetryDecision, SessionAuthority,
        SessionCapacity, SessionPolicyError, SessionUsage, UnknownLogoutLimits,
    },
    session_request::{SessionRequest, SessionRequestError, SessionTtlPolicy},
    verification::{
        self, BudgetError, ContextError, CredentialError, DecodedClaims, VerificationBudget,
        VerificationCosts, VerificationProfile,
    },
    work_budget::WorkBucket,
};
use ic_auth_protocol_types::{AudienceId, AuthRole, Credential, Principal, ScopeLabel};
use records::{CallerRecord, Replay, Session};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

type Account = (Principal, Principal);
type Slot = (Principal, Principal, u8);

/// Protected bounds; construction allocates no backing proportional to limits.
#[derive(Clone, Copy, Debug)]
pub struct MemoryConfig {
    /// Receiver domain, fixed for the lifetime of the instance.
    pub audience: AudienceId,
    /// Physical known-login pool and original login lifetime bounds.
    pub login: LoginLimits,
    /// Physical session/replay counts and logical encoded-byte limits.
    pub capacity: SessionCapacity,
    /// Shared proof work bucket capacity.
    pub verification_capacity: u32,
    /// Nanoseconds per replenished work unit.
    pub refill_interval_ns: u64,
    /// Relative costs of each complete verification mechanism.
    pub verification_costs: VerificationCosts,
    /// Separate unknown-login denial pool and operation budget.
    pub unknown_logout: UnknownLogoutLimits,
}

/// Current protected admission inputs, supplied synchronously by the host.
#[derive(Clone, Copy, Debug)]
pub struct AdmissionHost<'a> {
    /// Independently selected enrolled verifier, caller and time.
    pub verifier: VerificationProfile<'a>,
    /// Protected role for narrowing.
    pub role: &'a AuthRole,
    /// Protected default and maximum session duration.
    pub ttl: SessionTtlPolicy,
    /// Current authority generation, changed with enrollment/enablement policy.
    pub generation: u64,
    /// Whether this receiver accepts authentication operations.
    pub enabled: bool,
    /// Application eligibility for the proposed issuer-qualified subject.
    pub subject_admissible: bool,
}
impl AdmissionHost<'_> {
    fn authority(&self) -> SessionAuthority<'_> {
        let c = self.verifier.context();
        SessionAuthority {
            caller: c.caller,
            now_ns: c.now_ns,
            audience: *c.audience,
            issuer: c.issuer,
            mechanism: c.mechanism,
            key_binding_id: c.key_binding_id,
            role: self.role,
            grants: c.grants.as_slice(),
            generation: self.generation,
            not_before_ns: c.authority_not_before_ns,
            expires_at_ns: c.authority_expires_at_ns,
            enrollment_expires_at_ns: c.enrollment_expires_at_ns,
            min_login_started_at_ns: c.min_login_started_at_ns,
            max_session_ttl_secs: self.ttl.maximum_secs(),
            enabled: self.enabled,
            subject_admissible: self.subject_admissible,
        }
    }
}

/// Account facts at a successful operation, never authority after an await.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionReceipt {
    /// Authenticated browser caller.
    pub caller: Principal,
    /// Account issuer.
    pub issuer: Principal,
    /// Issuer-qualified subject.
    pub subject: Principal,
    /// Original exclusive session lease deadline.
    pub expires_at_ns: u64,
}

/// Successful admission; exact retries retain the original receipt and work credit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmissionOutcome {
    /// Current session facts.
    pub receipt: SessionReceipt,
    /// True only when no new session or replay was published.
    pub exact_retry: bool,
}

/// Refusal leaves authorization records unchanged; clocks/charges may advance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryError {
    /// Fresh receiver has not been activated, or was permanently closed.
    Closed,
    /// Shared receiver policy refused.
    Policy(SessionPolicyError),
    /// Canonical claims could not be decoded.
    Claims(ClaimsError),
    /// Fresh claims failed protected context checks.
    Context(ContextError),
    /// Requested grants or lifetime refused.
    Request(SessionRequestError),
    /// Complete proof refused, retaining any attempted-work charge.
    Proof(CredentialError),
    /// Invalid work configuration.
    Budget(BudgetError),
}
impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "memory receiver refused: {self:?}")
    }
}
impl Error for MemoryError {}
impl From<SessionPolicyError> for MemoryError {
    fn from(e: SessionPolicyError) -> Self {
        Self::Policy(e)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Lifecycle {
    Fresh,
    Ready,
    Closed,
}

/// Owned bounded state. No clone, reset, unchecked restore or mutable budget API.
///
/// All ordinary errors occur before authorization-record publication. Allocation
/// panic during publication permanently closes this instance; a host must abort
/// its transaction or discard it and recover safely, never resume partial state.
/// IC traps remain the host's whole-message rollback boundary.
pub struct MemoryReceiver {
    config: MemoryConfig,
    lifecycle: Lifecycle,
    generation: u64,
    observed_at_ns: u64,
    budget: VerificationBudget,
    unknown_budget: WorkBucket,
    unknown: usize,
    unknown_expiry: BTreeSet<(u64, Principal)>,
    callers: BTreeMap<Principal, CallerRecord>,
    slots: BTreeMap<Slot, Principal>,
    replays: BTreeMap<[u8; 32], Replay>,
    accounts: BTreeMap<Account, usize>,
    session_expiry: BTreeSet<(u64, Principal)>,
    replay_expiry: BTreeSet<(u64, [u8; 32])>,
    sessions: usize,
    encoded_bytes: usize,
}

impl MemoryReceiver {
    /// Create closed, empty state for a genuinely new receiver; work starts empty.
    /// This is not a recovery API for a receiver whose replay/fence state was lost.
    pub fn new_fresh(config: MemoryConfig, now_ns: u64) -> Result<Self, MemoryError> {
        config.login.validate()?;
        config.capacity.validate()?;
        if config.unknown_logout.max_denials == 0 {
            return Err(SessionPolicyError::InvalidContext.into());
        }
        now_ns
            .checked_add(config.login.max_login_ttl_ns)
            .ok_or(SessionPolicyError::InvalidContext)?;
        Ok(Self {
            config,
            lifecycle: Lifecycle::Fresh,
            generation: 0,
            observed_at_ns: now_ns,
            budget: VerificationBudget::new(
                config.verification_capacity,
                config.refill_interval_ns,
                config.verification_costs,
                now_ns,
            )
            .map_err(MemoryError::Budget)?,
            unknown_budget: WorkBucket::new(
                config.unknown_logout.capacity,
                config.unknown_logout.refill_interval_ns,
                now_ns,
            )
            .map_err(MemoryError::Budget)?,
            unknown: 0,
            unknown_expiry: BTreeSet::new(),
            callers: BTreeMap::new(),
            slots: BTreeMap::new(),
            replays: BTreeMap::new(),
            accounts: BTreeMap::new(),
            session_expiry: BTreeSet::new(),
            replay_expiry: BTreeSet::new(),
            sessions: 0,
            encoded_bytes: 0,
        })
    }

    /// Host assertion that fresh authenticated boot enrollment is ready for this
    /// new receiver. Allowed once; it cannot reopen a closed or lost receiver.
    pub fn activate_fresh(&mut self, generation: u64) -> Result<(), MemoryError> {
        if self.lifecycle != Lifecycle::Fresh {
            return Err(MemoryError::Closed);
        }
        self.generation = generation;
        self.lifecycle = Lifecycle::Ready;
        Ok(())
    }

    /// Permanently stop admission and authorization without discarding safety state.
    pub fn close(&mut self) {
        self.lifecycle = Lifecycle::Closed;
    }

    /// Host administrative invalidation, atomically paired with authority changes.
    /// Retains replay, device fences and work credit; overflow refuses.
    pub fn advance_generation(&mut self, now_ns: u64) -> Result<u64, MemoryError> {
        self.ready()?;
        let next = self
            .generation
            .checked_add(1)
            .ok_or(SessionPolicyError::StaleAuthority)?;
        self.observe(now_ns)?;
        self.generation = next;
        Ok(next)
    }

    /// Physical occupancy including expired records awaiting bounded pruning.
    #[must_use]
    pub fn usage(&self, account: Account) -> SessionUsage {
        SessionUsage {
            account,
            sessions: self.sessions,
            replays: self.replays.len(),
            replays_for_account: self.accounts.get(&account).copied().unwrap_or(0),
            encoded_bytes: self.encoded_bytes,
        }
    }

    /// Physical retained known logins, including denied/expired generation fences.
    #[must_use]
    pub fn known_logins(&self) -> usize {
        self.callers.len() - self.unknown
    }

    /// Physical unknown denials, including expired rows awaiting explicit pruning.
    #[must_use]
    pub const fn unknown_logins(&self) -> usize {
        self.unknown
    }

    /// Retained unknown-logout credit; this getter does not observe time or refill.
    #[must_use]
    pub const fn unknown_logout_credit(&self) -> u32 {
        self.unknown_budget.available()
    }

    /// Read-only work accounting; callers cannot reset or refill it.
    #[must_use]
    pub fn verification_budget(&self) -> &VerificationBudget {
        &self.budget
    }

    /// Latest retained trusted time, including normal operation refusals.
    #[must_use]
    pub const fn observed_at_ns(&self) -> u64 {
        self.observed_at_ns
    }

    /// Admit a complete credential in one synchronous host transaction.
    /// Canonical decoding and caller binding precede lookup. Exact retry precedes
    /// proof freshness/shape/charging; fresh preflight cannot reserve state.
    pub fn admit(
        &mut self,
        credential: &Credential,
        request: SessionRequest,
        host: AdmissionHost<'_>,
    ) -> Result<AdmissionOutcome, MemoryError> {
        self.ready()?;
        let authority = host.authority();
        policy::check_authority(&authority, self.config.audience)?;
        let decoded = DecodedClaims::new(&credential.claims).map_err(MemoryError::Claims)?;
        let claims = &decoded.claims;
        if verification::browser_principal(&claims.browser_public_key) != authority.caller {
            return Err(MemoryError::Context(ContextError::CallerMismatch));
        }
        self.observe(authority.now_ns)?;
        let fingerprint = decoded.fingerprint();
        let selected = self.selected(authority.caller);
        if let RetryDecision::ExactRetry { .. } = policy::assess_retry(
            &selected.snapshot(self),
            &authority,
            fingerprint,
            &request,
            self.replays.contains_key(&fingerprint),
        )? {
            return Ok(AdmissionOutcome {
                receipt: selected.receipt().ok_or(SessionPolicyError::InvalidFacts)?,
                exact_retry: true,
            });
        }
        verification::check_context(claims, host.verifier.context())
            .map_err(MemoryError::Context)?;
        let expires_at_ns = request
            .evaluate_grants(&claims.grants, host.role, authority.now_ns, host.ttl)
            .map_err(MemoryError::Request)?
            .min(claims.login_deadline_ns)
            .min(authority.expires_at_ns)
            .min(authority.enrollment_expires_at_ns);
        let binding = LoginBinding {
            issuer: claims.issuer,
            subject: claims.account_subject,
            device_slot: claims.device_slot,
            device_generation: claims.device_generation,
            started_at_ns: claims.login_started_at_ns,
            deadline_ns: claims.login_deadline_ns,
        };
        let slot = (binding.issuer, binding.subject, binding.device_slot);
        let owner = self
            .slots
            .get(&slot)
            .and_then(|caller| self.callers.get(caller));
        let device = owner.and_then(CallerRecord::device);
        let login = policy::assess_login(
            &LoginSnapshot {
                audience: self.config.audience,
                caller: authority.caller,
                observed_at_ns: self.observed_at_ns,
                caller_login: selected.login.as_ref(),
                device: device.as_ref(),
                slot_owner: owner.map(|row| &row.login),
                admitted_logins: self.known_logins(),
            },
            binding,
            authority.now_ns,
            self.config.login,
        )?;
        let displaced = match login {
            LoginDecision::Replacement { displaced_caller } => displaced_caller,
            _ => authority.caller,
        };
        let old = self
            .callers
            .get(&displaced)
            .and_then(|r| r.session.as_ref());
        let session = Session::prepare(claims, &request, &authority, expires_at_ns, fingerprint)?;
        let replay = Replay::prepare(
            fingerprint,
            authority.caller,
            binding,
            self.generation,
            claims.admission_expires_at_ns,
        )?;
        let usage = policy::assess_capacity(
            (binding.issuer, binding.subject),
            self.usage((binding.issuer, binding.subject)),
            AdmissionFootprint {
                session_bytes: session.bytes,
                replay_bytes: replay.bytes,
                replaced_session_bytes: old.map(|s| s.bytes),
            },
            self.config.capacity,
        )?;
        let until_ns = selected.login.map_or(binding.deadline_ns, |row| {
            row.until_ns.max(binding.deadline_ns)
        });
        // The private verifier consumes this exact canonical decode. All ordinary
        // failures precede publication; attempted crypto charges remain in budget.
        verification::verify_decoded(decoded, &credential.proof, host.verifier, &mut self.budget)
            .map_err(MemoryError::Proof)?;
        let receipt = SessionReceipt {
            caller: authority.caller,
            issuer: binding.issuer,
            subject: binding.subject,
            expires_at_ns,
        };
        let record = CallerRecord::new(
            authority.caller,
            self.config.audience,
            binding,
            until_ns,
            session,
            request.into_scopes(),
        );
        // No fallible return after this point. Poison before any map allocation,
        // so even a caught panic cannot expose a partially published receiver.
        self.lifecycle = Lifecycle::Closed;
        self.remove_session(displaced);
        if displaced != authority.caller {
            self.callers.remove(&displaced);
        }
        self.remove_unknown(authority.caller);
        self.callers.insert(authority.caller, record);
        self.slots.insert(slot, authority.caller);
        self.session_expiry
            .insert((expires_at_ns, authority.caller));
        self.replay_expiry
            .insert((replay.remove_at_ns, fingerprint));
        self.replays.insert(fingerprint, replay);
        self.accounts
            .insert(usage.account, usage.replays_for_account);
        self.sessions = usage.sessions;
        self.encoded_bytes = usage.encoded_bytes;
        self.lifecycle = Lifecycle::Ready;
        Ok(AdmissionOutcome {
            receipt,
            exact_retry: false,
        })
    }

    /// Proof-free read under current enrollment and application policy. Recheck
    /// after every await before a privileged operation; receipts are only facts.
    pub fn read(
        &mut self,
        authority: &SessionAuthority<'_>,
        required_scopes: &[ScopeLabel],
    ) -> Result<SessionReceipt, MemoryError> {
        self.ready()?;
        policy::check_authority(authority, self.config.audience)?;
        self.observe(authority.now_ns)?;
        let selected = self.selected(authority.caller);
        policy::assess_session(&selected.snapshot(self), authority, required_scopes)?;
        selected
            .receipt()
            .ok_or_else(|| SessionPolicyError::InvalidFacts.into())
    }

    /// Commit local denial for the authenticated caller without requiring current
    /// enrollment or accepting a caller-selected target/deadline. Known login
    /// deadlines and live unknown retries never slide. Unknown denials remain
    /// unconfirmed, use a separate pool/bucket and cannot consume proof credit.
    pub fn logout(&mut self, caller: Principal, now_ns: u64) -> Result<LogoutReceipt, LogoutError> {
        self.ready().map_err(|_| LogoutError::Closed)?;
        if !policy::login_caller(caller) {
            return Err(LogoutError::InvalidCaller);
        }
        self.observe(now_ns).map_err(|_| LogoutError::Clock)?;
        let plan = policy::plan_logout(
            caller,
            self.config.audience,
            self.callers.get(&caller).map(|row| &row.login),
            now_ns,
            self.config.login.max_login_ttl_ns,
            (self.unknown, self.config.unknown_logout.max_denials),
        )?;
        if plan.charge_unknown {
            self.unknown_budget.charge(now_ns, 1).map_err(|e| match e {
                BudgetError::Exhausted => LogoutError::Capacity,
                BudgetError::ClockRollback => LogoutError::Clock,
                BudgetError::InvalidPolicy => LogoutError::InvalidFacts,
            })?;
        }
        match plan.receipt {
            LogoutReceipt::Revoked { .. } => {
                self.lifecycle = Lifecycle::Closed;
                self.remove_session(caller);
                if let Some(row) = self.callers.get_mut(&caller) {
                    row.login.denied = true;
                }
            }
            LogoutReceipt::Unconfirmed { deny_until_ns } if plan.charge_unknown => {
                let row = CallerRecord::unknown(caller, self.config.audience, deny_until_ns);
                self.lifecycle = Lifecycle::Closed;
                self.remove_unknown(caller);
                self.callers.insert(caller, row);
                self.unknown_expiry.insert((deny_until_ns, caller));
                self.unknown += 1;
            }
            LogoutReceipt::Unconfirmed { .. } => {} // already committed; deadline and credit unchanged
        }
        self.lifecycle = Lifecycle::Ready;
        Ok(plan.receipt)
    }

    /// Remove at most `limit` expired session/replay/unknown-denial rows in total (1–128).
    /// Visits indexed deadlines only; known login/device fences are never pruned.
    pub fn prune(&mut self, now_ns: u64, limit: usize) -> Result<PruneReport, MemoryError> {
        self.ready()?;
        if !(1..=128).contains(&limit) {
            return Err(SessionPolicyError::InvalidContext.into());
        }
        self.observe(now_ns)?;
        let mut report = PruneReport::default();
        self.lifecycle = Lifecycle::Closed;
        while report.sessions + report.replays + report.unknown_denials < limit {
            if let Some(&(deadline, caller)) = self.session_expiry.first()
                && deadline <= now_ns
            {
                self.remove_session(caller);
                report.sessions += 1;
            } else if let Some(&(deadline, fingerprint)) = self.replay_expiry.first()
                && deadline <= now_ns
            {
                self.replay_expiry.remove(&(deadline, fingerprint));
                if let Some(replay) = self.replays.remove(&fingerprint) {
                    self.encoded_bytes -= replay.bytes;
                    if let Some(count) = self.accounts.get_mut(&replay.account) {
                        *count -= 1;
                        if *count == 0 {
                            self.accounts.remove(&replay.account);
                        }
                    }
                }
                report.replays += 1;
            } else if let Some(&(deadline, caller)) = self.unknown_expiry.first()
                && deadline <= now_ns
            {
                self.remove_unknown(caller);
                report.unknown_denials += 1;
            } else {
                break;
            }
        }
        self.lifecycle = Lifecycle::Ready;
        Ok(report)
    }

    fn ready(&self) -> Result<(), MemoryError> {
        if self.lifecycle == Lifecycle::Ready {
            Ok(())
        } else {
            Err(MemoryError::Closed)
        }
    }
    fn observe(&mut self, now_ns: u64) -> Result<(), MemoryError> {
        if now_ns < self.observed_at_ns {
            return Err(SessionPolicyError::ClockRollback.into());
        }
        self.observed_at_ns = now_ns;
        Ok(())
    }
    fn remove_session(&mut self, caller: Principal) {
        if let Some(row) = self.callers.get_mut(&caller)
            && let Some(session) = row.session.take()
        {
            self.session_expiry.remove(&(session.expires_at_ns, caller));
            self.sessions -= 1;
            self.encoded_bytes -= session.bytes;
        }
    }
    fn remove_unknown(&mut self, caller: Principal) {
        if let Some(row) = self.callers.get(&caller)
            && row.login.binding.is_none()
        {
            self.unknown_expiry.remove(&(row.login.until_ns, caller));
            self.callers.remove(&caller);
            self.unknown -= 1;
        }
    }
    fn selected(&self, caller: Principal) -> records::Selected<'_> {
        let row = self.callers.get(&caller);
        let session = row.and_then(CallerRecord::session_facts);
        let device = row
            .and_then(CallerRecord::slot)
            .and_then(|slot| self.slots.get(&slot))
            .and_then(|owner| self.callers.get(owner))
            .and_then(CallerRecord::device);
        records::Selected {
            session,
            login: row.map(|r| r.login),
            device,
        }
    }
}

/// Number of physical rows removed; known fences remain unchanged.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PruneReport {
    /// Expired session rows removed.
    pub sessions: usize,
    /// Expired replay rows removed.
    pub replays: usize,
    /// Expired unknown denials removed; known login fences remain retained.
    pub unknown_denials: usize,
}
