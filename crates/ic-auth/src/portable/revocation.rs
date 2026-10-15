//! Bounded volatile revocation bookkeeping, not an authentication endpoint.

use super::{VerifiedPortableClaims, browser_principal, validate_shape};
use crate::admission_budget::{AdmissionBudgetError, SessionAdmissionBudget};
use ic_auth_protocol_types::{AudienceId, PortableClaims, Principal};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const MAX_LOGIN_NS: u64 = 86_400_000_000_000;
#[cfg(feature = "sessions")]
const RECORD_DOMAIN: &[u8] = b"IC-AUTH-LOGIN-RECORD\0";

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
#[cfg_attr(feature = "sessions", derive(serde::Serialize))]
#[cfg_attr(feature = "session-restore", derive(serde::Deserialize))]
#[cfg_attr(feature = "sessions", serde(deny_unknown_fields))]
struct Record {
    until_ns: u64,
    login: Option<LoginBinding>,
    revoked: bool,
    admitted_pool: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "sessions", derive(serde::Serialize))]
#[cfg_attr(feature = "session-restore", derive(serde::Deserialize))]
#[cfg_attr(feature = "sessions", serde(deny_unknown_fields))]
struct LoginBinding {
    issuer: Principal,
    subject: Principal,
    started_at_ns: u64,
    deadline_ns: u64,
}

/// Proposed ledger record from a serialized commit callback or trusted storage.
/// A durable host must persist this with its session/replay changes before
/// returning success. This is not a credential or reusable authorization result.
pub struct LoginRevocationChange {
    audience: AudienceId,
    caller: Principal,
    record: Record,
}

impl LoginRevocationChange {
    /// Encode the proposed record for the host's atomic durable commit.
    #[cfg(feature = "sessions")]
    pub fn encode(&self) -> Result<Vec<u8>, LoginRevocationError> {
        crate::storage_codec::encode(
            &(self.audience, self.caller, self.record),
            RECORD_DOMAIN,
            1024,
            LoginRevocationError::Storage,
            LoginRevocationError::InvalidPolicy,
        )
    }

    /// Decode trusted storage only; never accept these bytes from a client.
    /// This checks shape, not provenance, completeness, rollback or live authority.
    #[cfg(feature = "session-restore")]
    pub fn decode(bytes: &[u8]) -> Result<Self, LoginRevocationError> {
        let (audience, caller, record) = crate::storage_codec::decode(
            bytes,
            RECORD_DOMAIN,
            1024,
            || LoginRevocationError::InvalidPolicy,
            LoginRevocationError::InvalidPolicy,
        )?;
        let change = Self {
            audience,
            caller,
            record,
        };
        let caller = change.caller.as_slice();
        if caller.len() != 29 || caller.last() != Some(&2) {
            return Err(LoginRevocationError::InvalidCaller);
        }
        match change.record.login {
            Some(login)
                if change.record.admitted_pool
                    && login.issuer != Principal::anonymous()
                    && login.subject != Principal::anonymous()
                    && login.started_at_ns < login.deadline_ns
                    && login.deadline_ns - login.started_at_ns <= MAX_LOGIN_NS => {}
            None if !change.record.admitted_pool && change.record.revoked => {}
            _ => return Err(LoginRevocationError::InvalidPolicy),
        }
        Ok(change)
    }

    /// Protected trust domain owning the record.
    pub fn audience(&self) -> AudienceId {
        self.audience
    }
    /// Authenticated runtime caller whose record changes.
    pub fn caller(&self) -> Principal {
        self.caller
    }
    /// Exclusive retention deadline, including the configured clock margin.
    pub fn until_ns(&self) -> u64 {
        self.record.until_ns
    }
    /// Whether this change denies further use of the login.
    pub fn revoked(&self) -> bool {
        self.record.revoked
    }
    /// Whether this record occupies reserved admitted-login capacity.
    pub fn admitted_pool(&self) -> bool {
        self.record.admitted_pool
    }
    /// Original issuer, account subject, login start and absolute deadline.
    /// Unknown-login revocation has no authenticated account binding.
    pub fn binding(&self) -> Option<(Principal, Principal, u64, u64)> {
        self.record.login.map(|login| {
            (
                login.issuer,
                login.subject,
                login.started_at_ns,
                login.deadline_ns,
            )
        })
    }
}

/// Bounded reference ledger for one host-installed trust domain. Callers are
/// runtime principals, never request fields; this ledger cannot authenticate them.
/// One admitted slot survives lease expiry until the login deadline plus skew.
/// Unknown callers have a separate pool and cannot consume admitted capacity.
///
/// The host MUST serialize these operations with proof/session/replay state and
/// authority changes. Commit callbacks receive the exact proposed login record
/// and must atomically write it with that other state in durable backends, or
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
    /// Stream the complete physical ledger under a serialized host checkpoint.
    /// Persist these records with occupancy, clock and session/replay/authority
    /// state atomically; expired entries count until explicit pruning.
    #[cfg(feature = "session-restore")]
    pub fn records(&self) -> impl ExactSizeIterator<Item = LoginRevocationChange> + '_ {
        self.records
            .iter()
            .map(|(&caller, &record)| LoginRevocationChange {
                audience: self.audience,
                caller,
                record,
            })
    }

    /// Rebuild a fresh ledger from a complete, trusted atomic storage generation.
    /// `self` must be newly constructed at current trusted time. The host supplies
    /// that generation's persisted clock high-water mark and physical pool counts,
    /// including expired records, from the same transaction as the records.
    /// Failure consumes the partial ledger; no usable partial state is returned.
    /// Counts detect missing entries, not malicious rollback: hosts MUST protect
    /// snapshot provenance/generation and restore reset/authority/session/replay
    /// state together, keeping admission closed until fresh enrollment is checked.
    #[cfg(feature = "session-restore")]
    pub fn restore_records(
        mut self,
        observed_ns: u64,
        expected_counts: (usize, usize),
        records: impl IntoIterator<Item = LoginRevocationChange>,
    ) -> Result<Self, LoginRevocationError> {
        if !self.records.is_empty()
            || observed_ns > self.observed_ns
            || expected_counts.0 > self.max_admitted
            || expected_counts.1 > self.max_unknown
        {
            return Err(LoginRevocationError::InvalidPolicy);
        }
        let max_until = retention(observed_ns, self.skew_ns)?;
        for change in records {
            if change.audience != self.audience || self.records.contains_key(&change.caller) {
                return Err(LoginRevocationError::InvalidPolicy);
            }
            let record = change.record;
            if record.until_ns == 0 || record.until_ns > max_until {
                return Err(LoginRevocationError::InvalidPolicy);
            }
            if let Some(login) = record.login {
                let minimum = login
                    .deadline_ns
                    .checked_add(self.skew_ns)
                    .ok_or(LoginRevocationError::Clock)?;
                if login.started_at_ns > observed_ns
                    || record.until_ns < minimum
                    || (!record.revoked && record.until_ns != minimum)
                {
                    return Err(LoginRevocationError::InvalidPolicy);
                }
            }
            // Stop at the persisted counts, even for an unbounded input iterator.
            let (used, expected) = if record.admitted_pool {
                (self.admitted, expected_counts.0)
            } else {
                (self.unknown, expected_counts.1)
            };
            if used >= expected {
                return Err(LoginRevocationError::Capacity);
            }
            self.install(change.caller, record);
        }
        if self.occupancy() != expected_counts {
            return Err(LoginRevocationError::InvalidPolicy);
        }
        Ok(self)
    }

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

    /// Clock high-water mark for an atomic host checkpoint, including refusals.
    /// Durable hosts retain this with pool counts and the complete record set.
    pub fn observed_at_ns(&self) -> u64 {
        self.observed_ns
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
    /// The callback receives the proposed record for that same atomic commit.
    pub fn with_reservation<T>(
        &mut self,
        caller: Principal,
        verified: &VerifiedPortableClaims,
        now_ns: u64,
        commit: impl FnOnce(&LoginRevocationChange) -> Result<T, LoginRevocationError>,
    ) -> Result<T, LoginRevocationError> {
        let record = self.reservation(caller, verified.claims(), now_ns)?;
        if verified.verified_at_ns() != now_ns {
            return Err(LoginRevocationError::InvalidPolicy);
        }
        let value = commit(&LoginRevocationChange {
            audience: self.audience,
            caller,
            record,
        })?;
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
    /// The callback receives the proposed tombstone, including retained binding.
    pub fn with_revocation(
        &mut self,
        unknown_budget: &mut SessionAdmissionBudget,
        caller: Principal,
        now_ns: u64,
        commit: impl FnOnce(&LoginRevocationChange) -> Result<(), LoginRevocationError>,
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
        commit(&LoginRevocationChange {
            audience: self.audience,
            caller,
            record,
        })?;
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
        self.prune_with_commit(now_ns, budget, |_, _| Ok(()))
    }
    /// Commit bounded expired-login deletion and resulting admitted/unknown counts
    /// before heap changes. Empty work skips the callback. Failure preserves all
    /// records but retains the trusted clock observation, as other refusals do.
    /// Hosts persist clock/authority metadata separately even when no rows expire.
    pub fn prune_with_commit(
        &mut self,
        now_ns: u64,
        budget: usize,
        commit: impl FnOnce(&[Principal], (usize, usize)) -> Result<(), LoginRevocationError>,
    ) -> Result<usize, LoginRevocationError> {
        self.clock(now_ns)?;
        let callers: Vec<_> = self
            .expiry
            .iter()
            .take_while(|(until, _)| *until <= now_ns)
            .take(budget.min(128))
            .map(|(_, caller)| *caller)
            .collect();
        if callers.is_empty() {
            return Ok(0);
        }
        let mut counts = self.occupancy();
        for caller in &callers {
            if self.records[caller].admitted_pool {
                counts.0 -= 1;
            } else {
                counts.1 -= 1;
            }
        }
        commit(&callers, counts)?;
        for caller in &callers {
            self.remove(*caller);
        }
        Ok(callers.len())
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
