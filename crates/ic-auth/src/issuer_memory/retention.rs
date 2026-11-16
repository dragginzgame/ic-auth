//! Indexed, operation-first maintenance. No account/history scan or state clone.
use super::{IssuerError, IssuerPruned, MemoryIssuer, expiry};

impl MemoryIssuer {
    /// Remove at most `min(budget, 32)` expired records. Operation expiry is the
    /// captured later admission/receipt deadline plus reconciliation retention.
    /// Pending, unknown and completed results use the same retention obligation.
    ///
    /// Select operations first, then already-idle expired logins from the remaining
    /// budget. Newly idle logins wait for another call. Referenced logins and their
    /// sequence watermarks survive. Closing an epoch and forgetting its IDs share
    /// one publication; unrelated live callbacks retain their restart fence.
    /// No attempt credit is charged or refilled. Clock observations survive refusals.
    ///
    /// # Panics
    /// Internal invariant failure or native publication interruption closes this
    /// instance; the host must discard it. This is containment, not host rollback.
    pub fn prune(&mut self, now_ns: u64, budget: u32) -> Result<IssuerPruned, IssuerError> {
        self.observe(now_ns)?;
        let budget = budget.min(32) as usize;
        // At most budget keys from each ordered prefix, with only the first
        // nonexpired key inspected. No work proportional to unrelated live rows.
        let operations: Vec<_> = self
            .operation_expiry
            .iter()
            .take(budget)
            .take_while(|key| key.0 <= now_ns)
            .copied()
            .collect();
        let logins: Vec<_> = self
            .idle_logins
            .iter()
            .take(budget - operations.len())
            .take_while(|key| key.0 <= now_ns)
            .copied()
            .collect();
        let result = IssuerPruned {
            operations: operations.len() as u32,
            logins: logins.len() as u32,
        };
        if operations.is_empty() && logins.is_empty() {
            return Ok(result);
        }
        let closes_epoch = operations
            .iter()
            .chain(&logins)
            .any(|key| key.3.0 == self.epoch);
        let next_epoch = if closes_epoch {
            self.epoch.checked_add(1).ok_or(IssuerError::Capacity)?
        } else {
            self.epoch
        };

        // All ordinary refusals precede publication. Index/record relationships are
        // maintained by this memory owner, not admitted from a persisted format.
        self.open = false;
        self.epoch = next_epoch;
        #[cfg(test)]
        super::tests::after_write();
        for key in operations {
            self.operation_expiry.remove(&key);
            #[cfg(test)]
            super::tests::after_write();
            let owner = self
                .accounts
                .get_mut(&key.2)
                .expect("indexed operation account");
            let op = owner.operations.remove(&key.3).expect("indexed operation");
            #[cfg(test)]
            super::tests::after_write();
            if let Some(sequence) = op.renewal {
                owner.renewals.remove(&(op.login, sequence));
                #[cfg(test)]
                super::tests::after_write();
            }
            let login = owner.logins.get_mut(&op.login).expect("referenced login");
            login.references -= 1;
            #[cfg(test)]
            super::tests::after_write();
            if login.references == 0 {
                self.idle_logins
                    .insert(expiry(login.deadline, key.2, op.login));
                #[cfg(test)]
                super::tests::after_write();
            }
            self.usage.operations -= 1;
            self.usage.result_bytes -= op.result_bytes;
            #[cfg(test)]
            super::tests::after_write();
        }
        for key in logins {
            self.idle_logins.remove(&key);
            #[cfg(test)]
            super::tests::after_write();
            let owner = self
                .accounts
                .get_mut(&key.2)
                .expect("indexed login account");
            owner.logins.remove(&key.3).expect("indexed idle login");
            #[cfg(test)]
            super::tests::after_write();
            self.usage.logins -= 1;
            #[cfg(test)]
            super::tests::after_write();
            if owner.logins.is_empty() {
                self.accounts.remove(&key.2);
                #[cfg(test)]
                super::tests::after_write();
            }
        }
        self.open = true;
        Ok(result)
    }
}
