//! Browser binding and sequence admission; initial issue and renewal share all
//! operation reservation, charging, dispatch and completion rules.
use super::{
    IssueRequest, IssuerAuthority, IssuerError, IssuerTicket, Login, MemoryIssuer, Operation,
    RenewRequest, RenewalSelection, RetainedRenewal,
};
use ic_auth_protocol_types::Principal;
use serde::Serialize;
use sha2::{Digest, Sha256};

impl MemoryIssuer {
    /// Authenticate the ingress browser against retained state and select the
    /// original inputs for current application approval. No clock, quota, credit,
    /// nonce or sequence changes. This is not a signing/reservation token;
    /// `prepare_renewal` rechecks binding and current state after any await.
    pub fn preflight_renewal(
        &self,
        browser: Principal,
        input: &RenewRequest,
    ) -> Result<RenewalSelection, IssuerError> {
        let (login, retained) = self.select_renewal(browser, input)?;
        if login.revoked {
            return Err(IssuerError::Denied);
        }
        self.renewal_selection(login, retained, input)
    }

    /// Browser-authenticated lookup only, including expired/revoked/old-epoch
    /// work. Missing result evidence returns None and cannot recreate a sequence.
    /// The host supplies authenticated ingress; the request's account is only a hint.
    pub fn reconcile_renewal(
        &self,
        browser: Principal,
        input: &RenewRequest,
    ) -> Result<Option<RetainedRenewal<'_>>, IssuerError> {
        let (_, retained) = self.select_renewal(browser, input)?;
        retained
            .map(|op| {
                Ok(RetainedRenewal {
                    intent: self.intent(op.ticket)?,
                    status: op.status(),
                })
            })
            .transpose()
    }

    /// Reserve the next browser-authenticated sequence using the original login
    /// window and policy, or return its exact retained ticket. The caller must
    /// recheck current application permission; later approval still binds the
    /// exact intent and nonce. An in-flight renewal blocks its successor until
    /// bounded pruning removes the expired operation after reconciliation retention.
    /// Its sequence stays consumed even when its signing outcome is unknown.
    pub fn prepare_renewal(
        &mut self,
        browser: Principal,
        input: &RenewRequest,
        authority: IssuerAuthority,
        now_ns: u64,
    ) -> Result<IssuerTicket, IssuerError> {
        self.observe(now_ns)?;
        let (login, retained) = self.select_renewal(browser, input)?;
        if login.revoked {
            return Err(IssuerError::Denied);
        }
        if let Some(op) = retained {
            if !matches!(op.phase, super::Phase::Completed(..)) {
                self.check_pending(op, authority, now_ns)?;
            }
            return Ok(op.ticket);
        }
        if now_ns >= login.deadline {
            return Err(IssuerError::Expired);
        }
        let selected = self.renewal_selection(login, None, input)?;
        self.prepare(
            input.account,
            &selected.request,
            selected.policy,
            authority,
            Some(input.sequence),
            now_ns,
        )
    }

    fn select_renewal(
        &self,
        browser: Principal,
        input: &RenewRequest,
    ) -> Result<(&Login, Option<&Operation>), IssuerError> {
        self.check_open()?;
        if input.sequence == 0 {
            return Err(IssuerError::Invalid);
        }
        let identity = (input.login_epoch, input.login_id);
        let login = self.browser_login(browser, input.account, identity)?;
        if input.grants != login.grants {
            return Err(IssuerError::Denied);
        }
        let owner = &self.accounts[&input.account];
        let retained = owner
            .renewals
            .get(&(identity, input.sequence))
            .map(|key| &owner.operations[key]);
        Ok((login, retained))
    }

    fn renewal_selection(
        &self,
        login: &Login,
        retained: Option<&Operation>,
        input: &RenewRequest,
    ) -> Result<RenewalSelection, IssuerError> {
        let (epoch, operation_id) = if let Some(op) = retained {
            (op.ticket.epoch, op.ticket.operation_id)
        } else {
            login.renewal.check_next(input.sequence)?;
            (self.epoch, operation_id(input)?)
        };
        Ok(RenewalSelection {
            account: input.account,
            policy: super::IssuerPolicy {
                ext: retained.map_or(&login.policy.ext, |op| &op.ext).clone(),
                ..login.policy
            },
            request: IssueRequest {
                epoch,
                operation_id,
                login_epoch: input.login_epoch,
                login_id: input.login_id,
                browser_public_key: login.browser,
                grants: login.grants.clone(),
            },
        })
    }
}

// Preserve the reference RenewRequest field order, ordinary fixed u8 array and
// transparent string labels. This private encoding exists only for the existing
// operation identity, not as a stored-record codec or a new public Wire endpoint.

///
/// RenewalIdentity
///
/// Fields encoded to bind an exact renewal request.
///

#[derive(Serialize)]
struct RenewalIdentity<'a> {
    account: Principal,
    login_epoch: u64,
    login_id: &'a [u8; 32],
    sequence: u64,
    grants: &'a [ic_auth_protocol_types::DelegatedRoleGrant],
}

pub(super) fn operation_id(input: &RenewRequest) -> Result<[u8; 32], IssuerError> {
    let identity = RenewalIdentity {
        account: input.account,
        login_epoch: input.login_epoch,
        login_id: &input.login_id,
        sequence: input.sequence,
        grants: &input.grants,
    };
    let bytes = ic_wire::encode(
        &identity,
        ic_wire::Encoding::Frame {
            method: 0x41555448495201,
            direction: ic_wire::Direction::Request,
        },
        ic_wire::Limits {
            bytes: 73_728,
            allocation: 1_048_576,
            nodes: 16_384,
            depth: 16,
        },
    )
    .map_err(|_| IssuerError::Invalid)?;
    Ok(Sha256::digest(bytes).into())
}
