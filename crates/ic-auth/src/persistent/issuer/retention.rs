use super::*;
use std::collections::BTreeMap;

impl<M: Memory> IssuerStore<M> {
    /// Revoke only this authenticated account's selected login. It cannot extend
    /// its window or evict another device. Already-issued receiver leases still
    /// obey receiver revocation/expiry; issuer revocation is not fleet broadcast.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn revoke(
        &mut self,
        account: Principal,
        login_epoch: u64,
        login_id: [u8; 32],
        now_ns: u64,
    ) -> Result<(), StoreError> {
        self.observe(now_ns)?;
        let mut login = self
            .login(&LoginId {
                account,
                epoch: login_epoch,
                id: login_id,
            })?
            .ok_or(StoreError::Denied)?;
        login.revoked = true;
        let mut writes = Writes::new();
        writes.put(keys::login(&login.id), &login)?;
        self.publish(self.header.clone(), writes)
    }

    /// Delete at most min(budget, 32) expired records, using expiry indexes and
    /// reference counts. Closing an epoch and forgetting its consumed IDs share
    /// one transaction. Pending work prevents early login deletion without scans.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn prune(&mut self, now_ns: u64, budget: u32) -> Result<IssuerPruned, StoreError> {
        self.observe(now_ns)?;
        let budget = budget.min(32) as usize;
        let operations = self
            .records
            .expired(keys::OPERATION_EXPIRY, now_ns, budget)?;
        let logins = self
            .records
            .expired(keys::LOGIN_EXPIRY, now_ns, budget - operations.len())?;
        let mut header = self.header.clone();
        let mut writes = Writes::new();
        let mut changed = BTreeMap::<Key, Login>::new();
        let mut accounts = BTreeMap::<Key, u64>::new();
        let mut closes_epoch = false;
        for expiry in &operations {
            let ticket: IssuerTicket = self.records.get(expiry)?.ok_or(StoreError::Corrupt)?;
            let row = self
                .operation(ticket.account, ticket.epoch, &ticket.operation_id)?
                .ok_or(StoreError::Corrupt)?;
            if &keys::operation_expiry(row.remove_ns, &ticket) != expiry || row.remove_ns > now_ns {
                return Err(StoreError::Corrupt);
            }
            let key = keys::login(&row.login);
            let login = if let Some(login) = changed.get_mut(&key) {
                login
            } else {
                changed
                    .entry(key)
                    .or_insert(self.login(&row.login)?.ok_or(StoreError::Corrupt)?)
            };
            login.references = login.references.checked_sub(1).ok_or(StoreError::Corrupt)?;
            header.occupancy.operations = header
                .occupancy
                .operations
                .checked_sub(1)
                .ok_or(StoreError::Corrupt)?;
            header.occupancy.reserved_bytes = header
                .occupancy
                .reserved_bytes
                .checked_sub(row.reserved_bytes)
                .ok_or(StoreError::Corrupt)?;
            writes.delete(keys::ticket(&ticket));
            writes.delete(expiry.clone());
            if let Some(sequence) = row.renewal {
                let indexed: Option<IssuerTicket> =
                    self.records.get(&keys::renewal(&row.login, sequence))?;
                if indexed.as_ref() != Some(&ticket) {
                    return Err(StoreError::Corrupt);
                }
                writes.delete(keys::renewal(&row.login, sequence));
                header.renewal_indexes = header
                    .renewal_indexes
                    .checked_sub(1)
                    .ok_or(StoreError::Corrupt)?;
            }
            closes_epoch |= ticket.epoch == header.epoch;
            let key = keys::account_operations(ticket.account);
            let count = if let Some(count) = accounts.get_mut(&key) {
                count
            } else {
                accounts
                    .entry(key.clone())
                    .or_insert(self.records.get(&key)?.ok_or(StoreError::Corrupt)?)
            };
            *count = count.checked_sub(1).ok_or(StoreError::Corrupt)?;
        }
        for login in changed.values() {
            writes.put(keys::login(&login.id), login)?;
            if login.references == 0 {
                writes.put(keys::login_expiry(login), &login.id)?;
                header.idle_logins += 1;
            }
        }
        for expiry in &logins {
            let id: LoginId = self.records.get(expiry)?.ok_or(StoreError::Corrupt)?;
            let login = self.login(&id)?.ok_or(StoreError::Corrupt)?;
            if &keys::login_expiry(&login) != expiry
                || login.references != 0
                || login.deadline_ns > now_ns
            {
                return Err(StoreError::Corrupt);
            }
            writes.delete(keys::login(&id));
            writes.delete(keys::slot(id.account, login.slot));
            writes.delete(keys::browser(id.account, &login.browser_public_key));
            writes.delete(expiry.clone());
            header.occupancy.logins = header
                .occupancy
                .logins
                .checked_sub(1)
                .ok_or(StoreError::Corrupt)?;
            header.idle_logins = header
                .idle_logins
                .checked_sub(1)
                .ok_or(StoreError::Corrupt)?;
            header.occupancy.reserved_bytes = header
                .occupancy
                .reserved_bytes
                .checked_sub(prepare::login_reservation(&login)?)
                .ok_or(StoreError::Corrupt)?;
            closes_epoch |= id.epoch == header.epoch;
        }
        if closes_epoch {
            header.epoch = header.epoch.checked_add(1).ok_or(StoreError::Capacity)?;
        }
        for (key, count) in accounts {
            if count == 0 {
                writes.delete(key);
                header.operation_accounts = header
                    .operation_accounts
                    .checked_sub(1)
                    .ok_or(StoreError::Corrupt)?;
            } else {
                writes.put(key, &count)?;
            }
        }
        if !operations.is_empty() || !logins.is_empty() {
            self.publish(header, writes)?;
        }
        Ok(IssuerPruned {
            operations: operations.len() as u32,
            logins: logins.len() as u32,
        })
    }
}
