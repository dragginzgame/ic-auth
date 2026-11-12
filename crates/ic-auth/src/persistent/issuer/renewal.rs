use super::*;
use crate::portable::browser_principal;
use prepare::{grants_hash, hash};

impl<M: Memory> IssuerStore<M> {
    /// Authenticate the actual browser and project the original request/policy
    /// before application authorization. This reads only selected point indexes:
    /// no reservation, clock observation, nonce, signing or deadline extension.
    /// Expiry/authority and post-callback state are rechecked by `renew`.
    pub fn preflight_renewal(
        &self,
        browser: Principal,
        input: &RenewRequest,
    ) -> Result<RenewalSelection, StoreError> {
        let (login, retained) = self.select_renewal(browser, input)?;
        if login.revoked {
            return Err(StoreError::Denied);
        }
        self.renewal_selection(&login, retained.as_ref(), input)
    }

    /// Browser-authenticated lookup only. Original pending identity or completed
    /// bytes remain readable after expiry, restart or revocation while retained.
    /// Missing/pruned results return `None`; they never recreate consumed work.
    /// Missing login authority or wrong browser/grants refuses. No clock is read
    /// or advanced. The host supplies the authenticated ingress browser principal.
    pub fn retrieve_renewal(
        &self,
        browser: Principal,
        input: &RenewRequest,
    ) -> Result<Option<RetainedRenewal>, StoreError> {
        let (_, retained) = self.select_renewal(browser, input)?;
        retained
            .map(|row| {
                Ok(RetainedRenewal {
                    status: self.status(&row)?,
                    intent: Self::project_intent(row),
                })
            })
            .transpose()
    }

    /// Reserve the next browser-bound renewal, or return the exact retained work.
    /// Rechecks live state after preflight; never extends the original window.
    /// # Panics
    /// Storage faults require host rollback; native hosts must discard the runtime.
    pub fn renew(
        &mut self,
        browser: Principal,
        input: RenewRequest,
        authority: IssuerAuthority,
        now_ns: u64,
    ) -> Result<IssuerStatus, StoreError> {
        self.observe(now_ns)?;
        let (login, retained) = self.select_renewal(browser, &input)?;
        if login.revoked {
            return Err(StoreError::Denied);
        }
        if let Some(row) = retained {
            if row.result.is_none() {
                self.check_pending(&row, authority, now_ns)?;
            }
            return self.status(&row);
        }
        if now_ns >= login.deadline_ns {
            return Err(StoreError::Expired);
        }
        let selected = self.renewal_selection(&login, None, &input)?;
        self.prepare_selected(
            selected.account,
            selected.request,
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
    ) -> Result<(Login, Option<Operation>), StoreError> {
        if input.sequence == 0 {
            return Err(StoreError::Invalid);
        }
        let id = LoginId {
            account: input.account,
            epoch: input.login_epoch,
            id: input.login_id,
        };
        let login = self.login(&id)?.ok_or(StoreError::Denied)?;
        if browser == Principal::anonymous()
            || browser_principal(&login.browser_public_key) != browser
            || grants_hash(&input.grants)? != login.grants_hash
        {
            return Err(StoreError::Denied);
        }
        let ticket: Option<IssuerTicket> = self.records.get(&keys::renewal(&id, input.sequence))?;
        let retained = ticket
            .map(|ticket| {
                let row = self
                    .operation(ticket.account, ticket.epoch, &ticket.operation_id)?
                    .ok_or(StoreError::Corrupt)?;
                if row.ticket != ticket
                    || row.login != id
                    || row.renewal != Some(input.sequence)
                    || row.request.grants != input.grants
                    || row.request.operation_id != hash(input)?
                    || row.request.browser_public_key != login.browser_public_key
                    || row.policy != login.policy
                    || row.slot != login.slot
                    || row.device_generation != login.generation
                    || row.started_ns != login.started_ns
                    || row.deadline_ns != login.deadline_ns
                {
                    return Err(StoreError::Corrupt);
                }
                Ok(row)
            })
            .transpose()?;
        Ok((login, retained))
    }

    fn renewal_selection(
        &self,
        login: &Login,
        retained: Option<&Operation>,
        input: &RenewRequest,
    ) -> Result<RenewalSelection, StoreError> {
        let request = if let Some(row) = retained {
            row.request.clone()
        } else {
            if input.sequence <= login.sequence {
                return Err(StoreError::EpochClosed);
            }
            if !login.completed || login.sequence.checked_add(1) != Some(input.sequence) {
                return Err(StoreError::Conflict);
            }
            IssueRequest {
                epoch: self.header.epoch,
                operation_id: hash(input)?,
                login_epoch: login.id.epoch,
                login_id: login.id.id,
                browser_public_key: login.browser_public_key,
                grants: input.grants.clone(),
            }
        };
        Ok(RenewalSelection {
            account: login.id.account,
            request,
            policy: login.policy,
        })
    }
}
