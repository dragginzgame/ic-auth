//! Browser binding and sequence admission; initial issue and renewal share all
//! operation reservation, charging, dispatch and completion rules.
use super::{
    IssueRequest, IssuerAuthority, IssuerError, IssuerTicket, Login, MemoryIssuer, Operation,
    RenewRequest, RenewalSelection, RetainedRenewal,
};
use crate::browser_identity::browser_principal;
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
    /// exact intent and nonce. Pending previous work cannot authorize a new sequence.
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
        let owner = self
            .accounts
            .get(&input.account)
            .ok_or(IssuerError::Denied)?;
        let identity = (input.login_epoch, input.login_id);
        let login = owner.logins.get(&identity).ok_or(IssuerError::Denied)?;
        if browser == Principal::anonymous()
            || browser != browser_principal(&login.browser)
            || input.grants != login.grants
        {
            return Err(IssuerError::Denied);
        }
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
            policy: login.policy,
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
#[derive(Serialize)]
struct RenewalIdentity<'a> {
    account: Principal,
    login_epoch: u64,
    login_id: &'a [u8; 32],
    sequence: u64,
    grants: Vec<Grant<'a>>,
}
#[derive(Serialize)]
struct Grant<'a> {
    target: &'a str,
    scopes: Vec<&'a str>,
}
pub(super) fn operation_id(input: &RenewRequest) -> Result<[u8; 32], IssuerError> {
    let identity = RenewalIdentity {
        account: input.account,
        login_epoch: input.login_epoch,
        login_id: &input.login_id,
        sequence: input.sequence,
        grants: input
            .grants
            .iter()
            .map(|g| Grant {
                target: g.target.as_str(),
                scopes: g.scopes.iter().map(|s| s.as_str()).collect(),
            })
            .collect(),
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
