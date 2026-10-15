//! Bounded volatile revocation bookkeeping, not an authentication endpoint.

use super::{VerifiedPortableClaims, browser_principal, validate_shape};
use crate::admission_budget::{AdmissionBudgetError, SessionAdmissionBudget};
use ic_auth_protocol_types::{AudienceId, PortableClaims, Principal};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const MAX_LOGIN_NS: u64 = 86_400_000_000_000;

/// A refusal leaves records unchanged; trusted clock observations are retained.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum LoginRevocationError {
    #[error("logout requires an authenticated self-authenticating caller")]
    InvalidCaller,
    #[error("invalid revocation limits or authenticated login deadline")]
    InvalidPolicy,
    #[error("login is locally revoked")]
    Revoked,
    #[error("revocation capacity is exhausted")]
    Capacity,
    #[error("revocation clock rolled back or retention arithmetic overflowed")]
    Clock,
    #[error("host session/replay transaction failed")]
    Storage,
}

#[derive(Clone, Copy)]
struct Record {
    until_ns: u64,
    login: Option<LoginBinding>,
    revoked: bool,
    admitted_pool: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct LoginBinding {
    issuer: Principal,
    subject: Principal,
    started_at_ns: u64,
    deadline_ns: u64,
}

/// Bounded reference ledger for one host-installed trust domain. Callers are
/// runtime principals, never request fields; this ledger cannot authenticate them.
/// One admitted slot survives lease expiry until the login deadline plus skew.
/// Unknown callers have a separate pool and cannot consume admitted capacity.
///
/// The host MUST serialize these operations with proof/session/replay state and
/// authority changes. Commit callbacks must atomically write that other state or
/// return an error without effects. No await/reentrancy may split the operation.
/// This ledger commits its staged record only after callback success, with no
/// further fallible policy/storage checks. Callbacks must not return success
/// before their commit, or commit and then return an error.
///
/// This is not portable admission or a logout endpoint. No proof verification,
/// startup/enrollment readiness or durable storage is supplied here. Hosts MUST
/// apply those gates and a qualified clock-skew bound separately. Fresh unknown
/// revocations consume an explicit host-retained budget before the commit callback.
/// Losing this ledger requires the protocol's fresh login/reset floors; restoring
/// sessions alone is unsafe. Never recreate it per request or share across domains.
pub struct MemoryLoginRevocations {
    audience: AudienceId,
    records: BTreeMap<Principal, Record>,
    expiry: BTreeSet<(u64, Principal)>,
    max_admitted: usize,
    max_unknown: usize,
    admitted: usize,
    unknown: usize,
    skew_ns: u64,
    observed_ns: u64,
}

impl MemoryLoginRevocations {
    /// Fixed record counts bound memory; no caller-supplied blobs are retained.
    /// `skew_ns` is deployment-qualified protected configuration, not a default.
    pub fn new(
        audience: AudienceId,
        max_admitted: usize,
        max_unknown: usize,
        skew_ns: u64,
        now_ns: u64,
    ) -> Result<Self, LoginRevocationError> {
        if max_admitted == 0 || max_unknown == 0 {
            return Err(LoginRevocationError::InvalidPolicy);
        }
        retention(now_ns, skew_ns)?;
        Ok(Self {
            audience,
            records: BTreeMap::new(),
            expiry: BTreeSet::new(),
            max_admitted,
            max_unknown,
            admitted: 0,
            unknown: 0,
            skew_ns,
            observed_ns: now_ns,
        })
    }

    #[cfg(feature = "sessions")]
    pub(crate) fn audience(&self) -> AudienceId {
        self.audience
    }

    /// A restored session requires its matching live reservation, not just absence
    /// of a tombstone. Losing the ledger must never revive a retained session.
    #[cfg(feature = "sessions")]
    pub(crate) fn check_session(
        &mut self,
        caller: Principal,
        issuer: Principal,
        subject: Principal,
        started_at_ns: u64,
        deadline_ns: u64,
        now_ns: u64,
    ) -> Result<(), LoginRevocationError> {
        self.observe(caller, now_ns)?;
        let record = self
            .records
            .get(&caller)
            .filter(|record| record.until_ns > now_ns)
            .ok_or(LoginRevocationError::InvalidPolicy)?;
        if record.revoked {
            return Err(LoginRevocationError::Revoked);
        }
        if record.login
            != Some(LoginBinding {
                issuer,
                subject,
                started_at_ns,
                deadline_ns,
            })
        {
            return Err(LoginRevocationError::InvalidPolicy);
        }
        Ok(())
    }

    /// Physical pool occupancy, including expired entries until bounded pruning.
    pub fn occupancy(&self) -> (usize, usize) {
        (self.admitted, self.unknown)
    }

    /// Cheap refusal before proof verification. This does not authenticate claims,
    /// reserve capacity or permit a session commit; success is only permission to
    /// continue checking. Untrusted claims can cause refusal, never admission.
    /// Records and pool occupancy stay unchanged; trusted clock observations remain.
    /// The host must serialize this check, verification and `with_reservation`
    /// with authority/session/replay changes. The final reservation rechecks state.
    pub fn check_reservation(
        &mut self,
        caller: Principal,
        claims: &PortableClaims,
        now_ns: u64,
    ) -> Result<(), LoginRevocationError> {
        self.reservation(caller, claims, now_ns).map(|_| ())
    }

    /// Reserve before committing a fully verified admission's session/replay pair.
    /// The issuer/account and original login window stay unchanged on renewal.
    /// Verified claims must match this ledger domain, caller and current clock.
    /// The host still enforces current authority and replay under the same lock.
    /// A live tombstone rejects even an exact admission retry before its callback.
    /// No reservation is installed if session/replay storage in `commit` fails.
    pub fn with_reservation<T>(
        &mut self,
        caller: Principal,
        verified: &VerifiedPortableClaims,
        now_ns: u64,
        commit: impl FnOnce() -> Result<T, LoginRevocationError>,
    ) -> Result<T, LoginRevocationError> {
        let record = self.reservation(caller, verified.claims(), now_ns)?;
        if verified.verified_at_ns() != now_ns {
            return Err(LoginRevocationError::InvalidPolicy);
        }
        let value = commit()?;
        self.install(caller, record);
        Ok(value)
    }

    /// Stage local denial, then atomically remove this caller's host sessions in
    /// `commit`. Return the denial deadline only after that callback commits.
    /// An existing live tombstone keeps its deadline; the callback still runs so
    /// retry never falsely acknowledges a failed host operation. No proof needed.
    /// An admitted reservation converts in its own pool even if unknowns fill theirs.
    /// A caller without a live record consumes one unknown-logout budget credit
    /// before the callback; exhaustion returns Capacity, never false success.
    /// Keep that receiver-wide budget separate from fresh proof verification so
    /// unknown logout traffic cannot spend admission credit. Failure retains the
    /// charge; hosts must return normal errors rather than trap/roll back credit.
    /// Live reservations and tombstone retries do not compete for this budget.
    pub fn with_revocation(
        &mut self,
        unknown_budget: &mut SessionAdmissionBudget,
        caller: Principal,
        now_ns: u64,
        commit: impl FnOnce() -> Result<(), LoginRevocationError>,
    ) -> Result<u64, LoginRevocationError> {
        self.observe(caller, now_ns)?;
        let old = self
            .records
            .get(&caller)
            .copied()
            .filter(|r| r.until_ns > now_ns);
        let record = if let Some(old) = old.filter(|r| r.revoked) {
            old
        } else {
            let until_ns = retention(now_ns, self.skew_ns)?;
            if old.is_some_and(|r| r.until_ns > until_ns) {
                return Err(LoginRevocationError::Clock);
            }
            Record {
                until_ns,
                login: old.and_then(|r| r.login),
                revoked: true,
                admitted_pool: old.is_some_and(|r| r.admitted_pool),
            }
        };
        self.capacity(caller, record)?;
        if old.is_none() {
            unknown_budget
                .consume(now_ns)
                .map_err(|error| match error {
                    AdmissionBudgetError::Exhausted => LoginRevocationError::Capacity,
                    AdmissionBudgetError::ClockRollback => LoginRevocationError::Clock,
                    AdmissionBudgetError::InvalidPolicy => LoginRevocationError::InvalidPolicy,
                })?;
        }
        commit()?;
        self.install(caller, record);
        Ok(record.until_ns)
    }

    /// Consult local denial before any session use, including exact retries.
    /// Absence is not authentication or authorization; other gates still apply.
    pub fn is_revoked(
        &mut self,
        caller: Principal,
        now_ns: u64,
    ) -> Result<bool, LoginRevocationError> {
        self.observe(caller, now_ns)?;
        Ok(self
            .records
            .get(&caller)
            .is_some_and(|r| r.revoked && r.until_ns > now_ns))
    }

    /// Explicit maintenance, at most 128 expired entries; never evict live state.
    /// The host clock must be monotonic, including across failed operations.
    pub fn prune(&mut self, now_ns: u64, budget: usize) -> Result<usize, LoginRevocationError> {
        self.clock(now_ns)?;
        let mut removed = 0;
        for _ in 0..budget.min(128) {
            let Some((until, caller)) = self.expiry.first().copied() else {
                break;
            };
            if until > now_ns {
                break;
            }
            self.remove(caller);
            removed += 1;
        }
        Ok(removed)
    }

    fn reservation(
        &mut self,
        caller: Principal,
        claims: &PortableClaims,
        now_ns: u64,
    ) -> Result<Record, LoginRevocationError> {
        self.observe(caller, now_ns)?;
        validate_shape(claims).map_err(|_| LoginRevocationError::InvalidPolicy)?;
        let login_deadline_ns = claims.login_deadline_ns;
        if claims.audience != self.audience
            || browser_principal(&claims.browser_public_key) != caller
        {
            return Err(LoginRevocationError::InvalidPolicy);
        }
        let until_ns = login_deadline_ns
            .checked_add(self.skew_ns)
            .ok_or(LoginRevocationError::Clock)?;
        let login = LoginBinding {
            issuer: claims.issuer,
            subject: claims.account_subject,
            started_at_ns: claims.login_started_at_ns,
            deadline_ns: login_deadline_ns,
        };
        if let Some(old) = self.records.get(&caller).filter(|r| r.until_ns > now_ns) {
            if old.revoked {
                return Err(LoginRevocationError::Revoked);
            }
            if old.login != Some(login) {
                return Err(LoginRevocationError::InvalidPolicy);
            }
        }
        let record = Record {
            until_ns,
            login: Some(login),
            revoked: false,
            admitted_pool: true,
        };
        self.capacity(caller, record)?;
        Ok(record)
    }

    fn clock(&mut self, now_ns: u64) -> Result<(), LoginRevocationError> {
        if now_ns < self.observed_ns {
            return Err(LoginRevocationError::Clock);
        }
        self.observed_ns = now_ns;
        Ok(())
    }

    fn observe(&mut self, caller: Principal, now_ns: u64) -> Result<(), LoginRevocationError> {
        let bytes = caller.as_slice();
        if bytes.len() != 29 || bytes.last() != Some(&2) {
            return Err(LoginRevocationError::InvalidCaller);
        }
        self.clock(now_ns)
    }

    fn capacity(&self, caller: Principal, record: Record) -> Result<(), LoginRevocationError> {
        let reuses_pool = self
            .records
            .get(&caller)
            .is_some_and(|old| old.admitted_pool == record.admitted_pool);
        let (count, limit) = if record.admitted_pool {
            (self.admitted, self.max_admitted)
        } else {
            (self.unknown, self.max_unknown)
        };
        if !reuses_pool && count >= limit {
            return Err(LoginRevocationError::Capacity);
        }
        Ok(())
    }

    fn remove(&mut self, caller: Principal) {
        if let Some(old) = self.records.remove(&caller) {
            self.expiry.remove(&(old.until_ns, caller));
            if old.admitted_pool {
                self.admitted -= 1;
            } else {
                self.unknown -= 1;
            }
        }
    }

    fn install(&mut self, caller: Principal, record: Record) {
        self.remove(caller);
        if record.admitted_pool {
            self.admitted += 1;
        } else {
            self.unknown += 1;
        }
        self.expiry.insert((record.until_ns, caller));
        self.records.insert(caller, record);
    }
}

fn retention(now_ns: u64, skew_ns: u64) -> Result<u64, LoginRevocationError> {
    skew_ns
        .checked_mul(2)
        .and_then(|skew| MAX_LOGIN_NS.checked_add(skew))
        .and_then(|ttl| now_ns.checked_add(ttl))
        .ok_or(LoginRevocationError::Clock)
}
