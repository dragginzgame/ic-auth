//! Bounded volatile issuance, browser renewal and exact-result reconciliation.
//!
//! Account authentication, trusted time/authority, entropy, approval and signing
//! are host inputs. No endpoint, signing key, executor or persistence is installed.
//! The host owns certified leaf/root publication and witness assembly. Expired records remain
//! charged until explicit bounded pruning; capacity refusal never deletes them.
//!
//! A host must commit dispatch before an external effect, authenticate its response
//! against the exact ticket/input, and own transaction rollback. A native unwind
//! during multi-record publication closes the instance. Loss of this volatile
//! state requires disabling old bindings and fresh account authentication/browser
//! keys; constructing another instance is not recovery of unresolved effects.

mod certified;
mod renewal;
mod retention;
mod types;
use crate::{
    claims::{encode_claims, validate_grants},
    work_budget::{BudgetError, WorkBucket},
};
use ic_auth_protocol_types::{DelegatedRoleGrant, PortableClaims, Principal};
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
mod tests;
pub use certified::{CertifiedCompletionError, CertifiedPublication};
pub use types::{
    IssueRequest, IssuedCredential, IssuedProof, IssuerAuthority, IssuerConfig, IssuerError,
    IssuerIntent, IssuerPolicy, IssuerPruned, IssuerSigning, IssuerStatus, IssuerTicket,
    IssuerUsage, RenewRequest, RenewalSelection, RetainedRenewal, SigningInput,
};

type Identity = (u64, [u8; 32]);
// Match reference expiry ties: time, principal length/bytes, epoch, identity.
type Expiry = (u64, u8, Principal, Identity);
fn expiry(time: u64, account: Principal, identity: Identity) -> Expiry {
    (time, account.as_slice().len() as u8, account, identity)
}

#[derive(Default)]
struct Account {
    // At most eight logins: slot/browser selection scans only this bounded owner.
    logins: BTreeMap<Identity, Login>,
    operations: BTreeMap<Identity, Operation>,
    // Lookup projection only: results live in operations for both entry paths.
    renewals: BTreeMap<(Identity, u64), Identity>,
}
struct Login {
    browser: [u8; 32],
    grants: Vec<DelegatedRoleGrant>,
    policy: IssuerPolicy,
    slot: u8,
    generation: u64,
    started: u64,
    deadline: u64,
    revoked: bool,
    renewal: RenewalProgress,
    references: u32,
}
enum RenewalProgress {
    Initial,
    InFlight(u64),
    Completed(u64),
}
impl RenewalProgress {
    fn sequence(&self) -> u64 {
        match self {
            Self::Initial => 0,
            Self::InFlight(n) | Self::Completed(n) => *n,
        }
    }
    fn check_next(&self, next: u64) -> Result<(), IssuerError> {
        if next <= self.sequence() {
            return Err(IssuerError::EpochClosed);
        }
        if !matches!(self, Self::Completed(n) if n.checked_add(1) == Some(next)) {
            return Err(IssuerError::Conflict);
        }
        Ok(())
    }
}
struct Operation {
    ticket: IssuerTicket,
    login: Identity,
    issued: u64,
    admission: u64,
    receipt: u64,
    phase: Phase,
    renewal: Option<u64>,
    remove_at: u64,
    result_bytes: u64,
}
struct Approval {
    nonce: [u8; 16],
    commitment: [u8; 32],
    claims: Vec<u8>,
}
enum Phase {
    Reserved,
    Challenged([u8; 16]),
    Approved(Approval),
    Dispatched(Approval),
    Completed(Approval, Completion),
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum Completion {
    Ed25519([u8; 64]),
    Certified,
}
impl Phase {
    fn evidence(&self) -> (Option<[u8; 16]>, Option<[u8; 32]>) {
        match self {
            Self::Reserved => (None, None),
            Self::Challenged(nonce) => (Some(*nonce), None),
            Self::Approved(a) | Self::Dispatched(a) | Self::Completed(a, _) => {
                (Some(a.nonce), Some(a.commitment))
            }
        }
    }
}
impl Operation {
    fn status(&self) -> IssuerStatus<'_> {
        match &self.phase {
            Phase::Completed(a, completion) => IssuerStatus::Completed(IssuedCredential {
                claims: &a.claims,
                proof: match completion {
                    Completion::Ed25519(signature) => IssuedProof::Ed25519(signature),
                    Completion::Certified => {
                        let IssuerSigning::Certified { seed_hash } = self.ticket.authority.signing
                        else {
                            unreachable!("certified completion authority")
                        };
                        IssuedProof::Certified { seed_hash }
                    }
                },
                admission_deadline_ns: self.admission,
            }),
            Phase::Dispatched(_) => IssuerStatus::Unknown(self.ticket),
            _ => IssuerStatus::Pending(self.ticket),
        }
    }
}

/// One bounded owner of account bindings, issuer operations and fresh-intent credit.
/// Do not clone a live instance or reconstruct it to recover lost state.
pub struct MemoryIssuer {
    config: IssuerConfig,
    accounts: BTreeMap<Principal, Account>,
    operation_expiry: BTreeSet<Expiry>,
    idle_logins: BTreeSet<Expiry>,
    usage: IssuerUsage,
    budget: WorkBucket,
    observed: u64,
    epoch: u64,
    boot: u64,
    generation: u64,
    open: bool,
}
impl MemoryIssuer {
    /// Create genuinely new state with full initial fresh-intent credit.
    /// This cannot restore old login, nonce, result or external-effect evidence.
    pub fn new_fresh(config: IssuerConfig, now_ns: u64) -> Result<Self, IssuerError> {
        if config.issuer == Principal::anonymous()
            || config.max_logins == 0
            || config.max_operations == 0
            || config.max_operations_per_account == 0
            || config.max_operations_per_account > config.max_operations
            || !(1..=8).contains(&config.max_devices_per_account)
            || config.max_result_bytes == 0
        {
            return Err(IssuerError::Invalid);
        }
        Ok(Self {
            budget: WorkBucket::new_full(
                config.issuance_burst,
                config.issuance_refill_interval_ns,
                now_ns,
            )
            .map_err(|_| IssuerError::Invalid)?,
            config,
            accounts: BTreeMap::new(),
            operation_expiry: BTreeSet::new(),
            idle_logins: BTreeSet::new(),
            usage: IssuerUsage::default(),
            observed: now_ns,
            epoch: 1,
            boot: 1,
            generation: 0,
            open: true,
        })
    }

    /// Current epoch for new work. Pruning closes consumed IDs; restart also fences
    /// callbacks. Neither resets retained state, generation counters or credit.
    #[must_use]
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Physical counts; expired rows remain charged until explicit pruning.
    #[must_use]
    pub const fn usage(&self) -> IssuerUsage {
        self.usage
    }
    /// Retained credit, without lazy refill or a clock observation.
    #[must_use]
    pub const fn available_credit(&self) -> u32 {
        self.budget.available()
    }
    /// Most recently observed trusted operation time, including ordinary refusals.
    #[must_use]
    pub const fn observed_at_ns(&self) -> u64 {
        self.observed
    }
    /// Terminal containment after lost state or host failure. It cannot be reopened.
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Reserve an exact account-authenticated intent and its completed-result bytes
    /// before entropy, approval or signing. `account` must come from host authentication.
    /// Exact retries are free and retain original deadlines. Later ordinary refusals
    /// after a fresh-attempt charge do not refund credit.
    pub fn prepare_issue(
        &mut self,
        account: Principal,
        request: &IssueRequest,
        policy: IssuerPolicy,
        authority: IssuerAuthority,
        now_ns: u64,
    ) -> Result<IssuerTicket, IssuerError> {
        self.prepare(account, request, policy, authority, None, now_ns)
    }

    fn prepare(
        &mut self,
        account: Principal,
        request: &IssueRequest,
        policy: IssuerPolicy,
        authority: IssuerAuthority,
        renewal: Option<u64>,
        now_ns: u64,
    ) -> Result<IssuerTicket, IssuerError> {
        self.observe(now_ns)?;
        validate_request(account, request, policy)?;
        let operation_key = (request.epoch, request.operation_id);
        let login_key = (request.login_epoch, request.login_id);
        let owner = self.accounts.get(&account);
        if let Some(op) = owner.and_then(|a| a.operations.get(&operation_key)) {
            let login = &self.accounts[&account].logins[&op.login];
            if op.login != login_key || op.renewal != renewal || !login.matches(request, policy) {
                return Err(IssuerError::Conflict);
            }
            if !matches!(op.phase, Phase::Completed(..)) {
                self.check_pending(op, authority, now_ns)?;
            }
            return Ok(op.ticket);
        }
        if request.epoch != self.epoch {
            return Err(IssuerError::EpochClosed);
        }
        if authority.generation == 0
            || authority.key_binding_id == [0; 32]
            || now_ns >= authority.expires_at_ns
        {
            return Err(IssuerError::Denied);
        }
        self.budget.charge(now_ns, 1).map_err(|e| match e {
            BudgetError::Exhausted => IssuerError::Throttled,
            BudgetError::ClockRollback => IssuerError::Clock,
            BudgetError::InvalidPolicy => IssuerError::Invalid,
        })?;
        if self.usage.operations >= self.config.max_operations
            || owner.is_some_and(|a| {
                a.operations.len() >= self.config.max_operations_per_account as usize
            })
        {
            return Err(IssuerError::Capacity);
        }
        let previous = owner.and_then(|a| a.logins.get(&login_key));
        let new_login;
        let login = if let Some(login) = previous {
            if !login.matches(request, policy) {
                return Err(IssuerError::Conflict);
            }
            if login.revoked {
                return Err(IssuerError::Denied);
            }
            if now_ns >= login.deadline {
                return Err(IssuerError::Expired);
            }
            new_login = None;
            login
        } else {
            if request.login_epoch != self.epoch {
                return Err(IssuerError::EpochClosed);
            }
            if self.usage.logins >= self.config.max_logins {
                return Err(IssuerError::Capacity);
            }
            if owner.is_some_and(|a| {
                a.logins
                    .values()
                    .any(|l| l.browser == request.browser_public_key)
            }) {
                return Err(IssuerError::Conflict);
            }
            let slot = (0..self.config.max_devices_per_account)
                .find(|slot| owner.is_none_or(|a| a.logins.values().all(|l| l.slot != *slot)))
                .ok_or(IssuerError::Capacity)?;
            new_login = Some(Login {
                browser: request.browser_public_key,
                grants: request.grants.clone(),
                policy,
                slot,
                generation: self
                    .generation
                    .checked_add(1)
                    .ok_or(IssuerError::Capacity)?,
                started: now_ns,
                deadline: now_ns
                    .checked_add(policy.login_ttl_ns)
                    .ok_or(IssuerError::Invalid)?,
                revoked: false,
                renewal: RenewalProgress::Initial,
                references: 0,
            });
            new_login.as_ref().ok_or(IssuerError::Invalid)?
        };
        if let Some(sequence) = renewal {
            login.renewal.check_next(sequence)?;
        }
        let ticket = IssuerTicket {
            account,
            epoch: request.epoch,
            operation_id: request.operation_id,
            authority,
            boot: self.boot,
        };
        let mut op = Operation {
            ticket,
            login: login_key,
            issued: now_ns,
            admission: now_ns
                .checked_add(policy.proof_ttl_ns)
                .ok_or(IssuerError::Invalid)?
                .min(login.deadline)
                .min(authority.expires_at_ns),
            receipt: now_ns
                .checked_add(policy.receipt_timeout_ns)
                .ok_or(IssuerError::Invalid)?,
            phase: Phase::Reserved,
            renewal,
            remove_at: 0,
            result_bytes: 0,
        };
        op.remove_at = op
            .admission
            .max(op.receipt)
            .checked_add(policy.reconciliation_retention_ns)
            .ok_or(IssuerError::Invalid)?;
        // Nonce contents do not change encoded length. Reserve the exact largest
        // result before publishing; no retained record is encoded for accounting.
        let bytes = self.claims(&op, login, [0; 16])?.len() as u64
            + if authority.signing == IssuerSigning::Ed25519 {
                64
            } else {
                0
            };
        op.result_bytes = bytes;
        let references = login
            .references
            .checked_add(1)
            .ok_or(IssuerError::Capacity)?;
        let idle_key = expiry(login.deadline, account, login_key);
        let operation_expiry = expiry(op.remove_at, account, operation_key);
        let result_bytes = self
            .usage
            .result_bytes
            .checked_add(bytes)
            .filter(|n| *n <= self.config.max_result_bytes)
            .ok_or(IssuerError::Capacity)?;
        // No fallible tail. A native unwind between these writes leaves us closed;
        // an IC host still owns rollback of the enclosing transaction.
        self.open = false;
        let owner = self.accounts.entry(account).or_default();
        #[cfg(test)]
        tests::after_write();
        if let Some(login) = new_login {
            self.generation = login.generation;
            owner.logins.insert(login_key, login);
            self.usage.logins += 1;
            #[cfg(test)]
            tests::after_write();
        }
        self.idle_logins.remove(&idle_key);
        #[cfg(test)]
        tests::after_write();
        owner
            .logins
            .get_mut(&login_key)
            .expect("selected operation login")
            .references = references;
        #[cfg(test)]
        tests::after_write();
        if let Some(sequence) = renewal {
            // Existing login was selected above; advance its watermark in the
            // same publication as the one shared operation and lookup index.
            owner
                .logins
                .get_mut(&login_key)
                .expect("selected renewal login")
                .renewal = RenewalProgress::InFlight(sequence);
            #[cfg(test)]
            tests::after_write();
            owner.renewals.insert((login_key, sequence), operation_key);
            #[cfg(test)]
            tests::after_write();
        }
        self.operation_expiry.insert(operation_expiry);
        #[cfg(test)]
        tests::after_write();
        owner.operations.insert(operation_key, op);
        #[cfg(test)]
        tests::after_write();
        self.usage.operations += 1;
        self.usage.result_bytes = result_bytes;
        #[cfg(test)]
        tests::after_write();
        self.open = true;
        Ok(ticket)
    }

    /// Lookup only, using the authenticated account. Missing evidence never creates
    /// an intent or authorizes another effect. Expiry/revocation do not erase results.
    pub fn reconcile(
        &self,
        account: Principal,
        epoch: u64,
        operation_id: [u8; 32],
    ) -> Result<Option<IssuerStatus<'_>>, IssuerError> {
        self.check_open()?;
        Ok(self
            .accounts
            .get(&account)
            .and_then(|a| a.operations.get(&(epoch, operation_id)))
            .map(Operation::status))
    }

    /// Borrow exact inputs for host approval. A ticket alone does not authenticate
    /// an external caller; the host must restrict access to this projection.
    pub fn intent(&self, ticket: IssuerTicket) -> Result<IssuerIntent<'_>, IssuerError> {
        let op = self.operation(ticket)?;
        let login = &self.accounts[&ticket.account].logins[&op.login];
        let (nonce, approval) = op.phase.evidence();
        Ok(IssuerIntent {
            ticket,
            browser_public_key: login.browser,
            grants: &login.grants,
            nonce,
            approval,
            admission_deadline_ns: op.admission,
            receipt_deadline_ns: op.receipt,
        })
    }

    /// Retain the first host-supplied entropy result. Repeats must match exactly.
    pub fn record_nonce(
        &mut self,
        ticket: IssuerTicket,
        authority: IssuerAuthority,
        nonce: [u8; 16],
        now_ns: u64,
    ) -> Result<(), IssuerError> {
        self.observe(now_ns)?;
        let op = self.operation(ticket)?;
        self.check_pending(op, authority, now_ns)?;
        if let Some(old) = op.phase.evidence().0 {
            return if nonce == old {
                Ok(())
            } else {
                Err(IssuerError::Conflict)
            };
        }
        self.operation_mut(ticket)?.phase = Phase::Challenged(nonce);
        Ok(())
    }

    /// Record a host-authenticated approval of the exact intent and nonce, after
    /// rechecking application permission. An untrusted response hash is insufficient.
    pub fn record_approval(
        &mut self,
        ticket: IssuerTicket,
        authority: IssuerAuthority,
        commitment: [u8; 32],
        now_ns: u64,
    ) -> Result<(), IssuerError> {
        self.observe(now_ns)?;
        let op = self.operation(ticket)?;
        self.check_pending(op, authority, now_ns)?;
        let (nonce, old) = op.phase.evidence();
        if let Some(old) = old {
            return if old == commitment {
                Ok(())
            } else {
                Err(IssuerError::Conflict)
            };
        }
        let nonce = nonce.ok_or(IssuerError::Pending)?;
        let claims = self.claims(op, &self.accounts[&ticket.account].logins[&op.login], nonce)?;
        self.operation_mut(ticket)?.phase = Phase::Approved(Approval {
            nonce,
            commitment,
            claims,
        });
        Ok(())
    }

    /// Record Ed25519 dispatch and return canonical input exactly once. The host commits
    /// this transition before signing. A lost response is lookup-only; no retry
    /// of this method yields a second signing permission.
    pub fn dispatch(
        &mut self,
        ticket: IssuerTicket,
        authority: IssuerAuthority,
        now_ns: u64,
    ) -> Result<SigningInput, IssuerError> {
        self.observe(now_ns)?;
        let op = self.operation(ticket)?;
        self.check_pending(op, authority, now_ns)?;
        if ticket.authority.signing != IssuerSigning::Ed25519 {
            return Err(IssuerError::Invalid);
        }
        let input = match &op.phase {
            Phase::Approved(a) => SigningInput {
                ticket,
                claims: a.claims.clone(),
            },
            Phase::Dispatched(_) | Phase::Completed(..) => return Err(IssuerError::Dispatched),
            _ => return Err(IssuerError::Pending),
        };
        let op = self.operation_mut(ticket)?;
        if let Phase::Approved(a) = std::mem::replace(&mut op.phase, Phase::Reserved) {
            op.phase = Phase::Dispatched(a);
        }
        Ok(input)
    }

    /// Retain a host-authenticated Ed25519 signing response for the dispatched
    /// ticket/input. This does not verify a signature or authenticate a remote
    /// callback: the host must do that before calling. Exact completed repeats
    /// remain readable after expiry/revocation/restart, without granting authority.
    pub fn complete(
        &mut self,
        ticket: IssuerTicket,
        authority: IssuerAuthority,
        signature: [u8; 64],
        now_ns: u64,
    ) -> Result<IssuedCredential<'_>, IssuerError> {
        self.observe(now_ns)?;
        let op = self.operation(ticket)?;
        if ticket.authority.signing != IssuerSigning::Ed25519 {
            return Err(IssuerError::Invalid);
        }
        if let Phase::Completed(_, old) = &op.phase {
            if *old != Completion::Ed25519(signature) {
                return Err(IssuerError::Conflict);
            }
        } else {
            self.check_pending(op, authority, now_ns)?;
            if !matches!(op.phase, Phase::Dispatched(_)) {
                return Err(IssuerError::Pending);
            }
            self.finish(ticket, Completion::Ed25519(signature));
        }
        self.completed(ticket)
    }

    fn completed(&self, ticket: IssuerTicket) -> Result<IssuedCredential<'_>, IssuerError> {
        match self.operation(ticket)?.status() {
            IssuerStatus::Completed(result) => Ok(result),
            _ => Err(IssuerError::Pending),
        }
    }

    // Both entry paths preflight their phase and current authority. No ordinary
    // refusal after an external publication; interrupted writes stay closed.
    fn finish(&mut self, ticket: IssuerTicket, completion: Completion) {
        self.open = false;
        let owner = self
            .accounts
            .get_mut(&ticket.account)
            .expect("selected account");
        let op = owner
            .operations
            .get_mut(&(ticket.epoch, ticket.operation_id))
            .expect("selected operation");
        let login = owner.logins.get_mut(&op.login).expect("selected login");
        let approval = match std::mem::replace(&mut op.phase, Phase::Reserved) {
            Phase::Approved(a) | Phase::Dispatched(a) => a,
            _ => unreachable!("preflighted completion phase"),
        };
        op.phase = Phase::Completed(approval, completion);
        #[cfg(test)]
        tests::after_write();
        match (op.renewal, &login.renewal) {
            (None, RenewalProgress::Initial) => login.renewal = RenewalProgress::Completed(0),
            (Some(n), RenewalProgress::InFlight(current)) if n == *current => {
                login.renewal = RenewalProgress::Completed(n)
            }
            _ => {}
        }
        #[cfg(test)]
        tests::after_write();
        self.open = true;
    }

    /// Account-authenticated revocation, retaining all fences and result evidence.
    /// The host must authenticate `account`; browser callers use `revoke_browser`.
    pub fn revoke(
        &mut self,
        account: Principal,
        login_epoch: u64,
        login_id: [u8; 32],
        now_ns: u64,
    ) -> Result<(), IssuerError> {
        self.observe(now_ns)?;
        self.revoke_login(account, (login_epoch, login_id))
            .map(|_| ())
    }

    /// Revoke one retained login after authenticating its browser binding.
    /// The host supplies the actual caller and time; account/epoch/id are only
    /// selectors. This synchronous operation fences pending completion and future
    /// renewal without deleting results, uncertainty or other account logins.
    /// Repeats, expired logins and old epochs retain the original login deadline,
    /// returned as metadata, not proof of logout at any separate receiver.
    /// Missing/pruned bindings refuse; volatile state loss cannot confirm logout.
    ///
    /// # Errors
    /// Returns `Denied` for an absent or mismatched browser/login binding, or the
    /// ordinary closed-owner/clock refusal. The host owns transaction rollback.
    pub fn revoke_browser(
        &mut self,
        browser: Principal,
        account: Principal,
        login_epoch: u64,
        login_id: [u8; 32],
        now_ns: u64,
    ) -> Result<u64, IssuerError> {
        self.observe(now_ns)?;
        let identity = (login_epoch, login_id);
        self.browser_login(browser, account, identity)?;
        self.revoke_login(account, identity)
    }

    // Called only after the public operation has observed time and authenticated
    // either the account or its retained browser. No await separates these steps.
    fn revoke_login(&mut self, account: Principal, identity: Identity) -> Result<u64, IssuerError> {
        let login = self
            .accounts
            .get_mut(&account)
            .and_then(|a| a.logins.get_mut(&identity))
            .ok_or(IssuerError::Denied)?;
        login.revoked = true;
        Ok(login.deadline)
    }

    fn browser_login(
        &self,
        browser: Principal,
        account: Principal,
        identity: Identity,
    ) -> Result<&Login, IssuerError> {
        let login = self
            .accounts
            .get(&account)
            .and_then(|a| a.logins.get(&identity))
            .ok_or(IssuerError::Denied)?;
        if browser == Principal::anonymous()
            || browser != crate::browser_identity::browser_principal(&login.browser)
        {
            return Err(IssuerError::Denied);
        }
        Ok(login)
    }

    /// Fence all old callbacks/new operation IDs without clearing logins, results
    /// or credit. This retains memory; it is not restoration after state loss.
    pub fn restart(&mut self, now_ns: u64) -> Result<(), IssuerError> {
        self.observe(now_ns)?;
        let epoch = self.epoch.checked_add(1).ok_or(IssuerError::Capacity)?;
        let boot = self.boot.checked_add(1).ok_or(IssuerError::Capacity)?;
        self.open = false;
        self.epoch = epoch;
        #[cfg(test)]
        tests::after_write();
        self.boot = boot;
        #[cfg(test)]
        tests::after_write();
        self.open = true;
        Ok(())
    }

    fn check_open(&self) -> Result<(), IssuerError> {
        if self.open {
            Ok(())
        } else {
            Err(IssuerError::Closed)
        }
    }
    fn observe(&mut self, now: u64) -> Result<(), IssuerError> {
        self.check_open()?;
        if now < self.observed || now < self.budget.observed_at_ns() {
            return Err(IssuerError::Clock);
        }
        self.observed = now;
        Ok(())
    }
    fn operation(&self, ticket: IssuerTicket) -> Result<&Operation, IssuerError> {
        self.check_open()?;
        self.accounts
            .get(&ticket.account)
            .and_then(|a| a.operations.get(&(ticket.epoch, ticket.operation_id)))
            .filter(|op| op.ticket == ticket)
            .ok_or(IssuerError::Denied)
    }
    fn operation_mut(&mut self, ticket: IssuerTicket) -> Result<&mut Operation, IssuerError> {
        self.accounts
            .get_mut(&ticket.account)
            .and_then(|a| a.operations.get_mut(&(ticket.epoch, ticket.operation_id)))
            .filter(|op| op.ticket == ticket)
            .ok_or(IssuerError::Denied)
    }
    fn check_pending(
        &self,
        op: &Operation,
        authority: IssuerAuthority,
        now: u64,
    ) -> Result<(), IssuerError> {
        if op.ticket.boot != self.boot || op.ticket.authority != authority {
            return Err(IssuerError::Denied);
        }
        if now >= op.receipt.min(op.admission).min(authority.expires_at_ns) {
            return Err(IssuerError::Expired);
        }
        if self.accounts[&op.ticket.account].logins[&op.login].revoked {
            return Err(IssuerError::Denied);
        }
        Ok(())
    }
    fn claims(
        &self,
        op: &Operation,
        login: &Login,
        nonce: [u8; 16],
    ) -> Result<Vec<u8>, IssuerError> {
        encode_claims(&PortableClaims {
            mechanism: op.ticket.authority.signing.mechanism(),
            audience: self.config.audience,
            issuer: self.config.issuer,
            key_binding_id: op.ticket.authority.key_binding_id,
            browser_public_key: login.browser,
            account_subject: op.ticket.account,
            device_slot: login.slot,
            device_generation: login.generation,
            login_started_at_ns: login.started,
            login_deadline_ns: login.deadline,
            issued_at_ns: op.issued,
            admission_expires_at_ns: op.admission,
            nonce,
            grants: login.grants.clone(),
        })
        .map_err(|_| IssuerError::Invalid)
    }
}
impl Login {
    fn matches(&self, request: &IssueRequest, policy: IssuerPolicy) -> bool {
        self.browser == request.browser_public_key
            && self.grants == request.grants
            && self.policy == policy
    }
}
fn validate_request(
    account: Principal,
    r: &IssueRequest,
    p: IssuerPolicy,
) -> Result<(), IssuerError> {
    if account == Principal::anonymous()
        || r.operation_id == [0; 32]
        || r.login_id == [0; 32]
        || r.browser_public_key == [0; 32]
        || r.epoch == 0
        || r.login_epoch == 0
        || p.login_ttl_ns == 0
        || p.login_ttl_ns > 90 * 86_400_000_000_000
        || p.proof_ttl_ns == 0
        || p.proof_ttl_ns > 60_000_000_000
        || p.receipt_timeout_ns == 0
        || p.reconciliation_retention_ns == 0
    {
        return Err(IssuerError::Invalid);
    }
    validate_grants(&r.grants).map_err(|_| IssuerError::Invalid)
}
