use super::*;
use crate::portable::ValidatedGrants;
use sha2::{Digest, Sha256};

impl<M: Memory> IssuerStore<M> {
    /// Reserve the exact intent and worst-case readable completion before any
    /// random, authority or signing effect. Existing intent retries retain all
    /// original times, key selection and result bytes.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn prepare(
        &mut self,
        account: Principal,
        request: IssueRequest,
        policy: IssuerPolicy,
        authority: IssuerAuthority,
        now_ns: u64,
    ) -> Result<IssuerStatus, StoreError> {
        self.prepare_selected(account, request, policy, authority, None, now_ns)
    }

    pub(super) fn prepare_selected(
        &mut self,
        account: Principal,
        request: IssueRequest,
        policy: IssuerPolicy,
        authority: IssuerAuthority,
        renewal: Option<u64>,
        now_ns: u64,
    ) -> Result<IssuerStatus, StoreError> {
        self.observe(now_ns)?;
        validate(account, &request, policy)?;
        if let Some(row) = self.operation(account, request.epoch, &request.operation_id)? {
            if row.request != request || row.policy != policy || row.renewal != renewal {
                return Err(StoreError::Conflict);
            }
            if row.result.is_none() {
                self.check_pending(&row, authority, now_ns)?;
            }
            return self.status(&row);
        }
        // Preserve retained exact-result recovery, but reject oversized new
        // work before reserving capacity, nonce acquisition or signing.
        crate::portable::validate_credential_grants(&request.grants)
            .map_err(|_| StoreError::Invalid)?;
        if request.epoch != self.header.epoch {
            return Err(StoreError::EpochClosed);
        }
        if authority.generation == 0
            || authority.key_binding_id == [0; 32]
            || now_ns >= authority.expires_at_ns
        {
            return Err(StoreError::Denied);
        }
        let mut header = self.header.clone();
        if header.occupancy.operations >= header.config.max_operations {
            return Err(StoreError::Capacity);
        }
        let account_operations: u64 = self
            .records
            .get(&keys::account_operations(account))?
            .unwrap_or(0);
        if account_operations >= header.config.max_operations_per_account {
            return Err(StoreError::Capacity);
        }
        let id = LoginId {
            account,
            epoch: request.login_epoch,
            id: request.login_id,
        };
        let previous = self.login(&id)?;
        let creating = previous.is_none();
        let mut login = if let Some(login) = previous {
            if login.browser_public_key != request.browser_public_key
                || login.policy != policy
                || login.grants_hash != grants_hash(&request.grants)?
            {
                return Err(StoreError::Conflict);
            }
            if login.revoked {
                return Err(StoreError::Denied);
            }
            if now_ns >= login.deadline_ns {
                return Err(StoreError::Expired);
            }
            login
        } else {
            if request.login_epoch != header.epoch {
                return Err(StoreError::EpochClosed);
            }
            if header.occupancy.logins >= header.config.max_logins {
                return Err(StoreError::Capacity);
            }
            // One bounded account index: never examine another account's devices.
            if self
                .records
                .contains(&keys::browser(account, &request.browser_public_key))
            {
                return Err(StoreError::Conflict);
            }
            let slot = (0..header.config.max_devices_per_account)
                .find(|slot| !self.records.contains(&keys::slot(account, *slot)))
                .ok_or(StoreError::Capacity)?;
            header.generation = header
                .generation
                .checked_add(1)
                .ok_or(StoreError::Capacity)?;
            Login {
                id,
                browser_public_key: request.browser_public_key,
                slot,
                generation: header.generation,
                started_ns: now_ns,
                deadline_ns: now_ns
                    .checked_add(policy.login_ttl_ns)
                    .ok_or(StoreError::Invalid)?,
                grants_hash: grants_hash(&request.grants)?,
                policy,
                sequence: 0,
                completed: false,
                references: 0,
                revoked: false,
            }
        };
        if let Some(sequence) = renewal {
            if !login.completed || login.sequence.checked_add(1) != Some(sequence) {
                return Err(StoreError::Conflict);
            }
            login.sequence = sequence;
            login.completed = false;
        }
        let admission_ns = now_ns
            .checked_add(policy.proof_ttl_ns)
            .ok_or(StoreError::Invalid)?
            .min(login.deadline_ns)
            .min(authority.expires_at_ns);
        let receipt_ns = now_ns
            .checked_add(policy.receipt_timeout_ns)
            .ok_or(StoreError::Invalid)?;
        if admission_ns <= now_ns {
            return Err(StoreError::Expired);
        }
        let ticket = IssuerTicket {
            account,
            epoch: request.epoch,
            operation_id: request.operation_id,
            boot: header.boot,
            authority,
        };
        let mut row = Operation {
            ticket,
            request,
            policy,
            login: login.id.clone(),
            slot: login.slot,
            device_generation: login.generation,
            started_ns: login.started_ns,
            deadline_ns: login.deadline_ns,
            issued_ns: now_ns,
            admission_ns,
            receipt_ns,
            remove_ns: admission_ns
                .max(receipt_ns)
                .checked_add(policy.reconciliation_retention_ns)
                .ok_or(StoreError::Invalid)?,
            renewal,
            nonce: None,
            approval: None,
            result: None,
            reserved_bytes: u64::MAX,
        };
        // Size only this row's worst completion. No journal clone, serialization
        // or validation of unrelated pending/completed operations.
        let mut completion = row.clone();
        // Fixed arrays use integer elements in this record codec. Reserve their
        // longest encodings too, rather than assuming random bytes cost zero bytes.
        completion.nonce = Some([u8::MAX; 16]);
        completion.approval = Some([u8::MAX; 32]);
        completion.result = Some(IssuedCredential {
            claims: crate::portable::encode_claims(&Self::claims(&header.config, &completion)?)
                .map_err(|_| StoreError::Invalid)?,
            signature: Some(vec![0; 64]),
            admission_deadline_ns: admission_ns,
        });
        row.reserved_bytes = size(&completion)?;
        let reserved = row
            .reserved_bytes
            .checked_add(if creating {
                login_reservation(&login)?
            } else {
                0
            })
            .ok_or(StoreError::Capacity)?;
        header.occupancy.reserved_bytes = header
            .occupancy
            .reserved_bytes
            .checked_add(reserved)
            .filter(|bytes| *bytes <= header.config.max_reserved_bytes)
            .ok_or(StoreError::Capacity)?;
        header.occupancy.operations += 1;
        let mut writes = Writes::new();
        writes.put(keys::account_operations(account), &(account_operations + 1))?;
        if account_operations == 0 {
            header.operation_accounts += 1;
        }
        if creating {
            header.occupancy.logins += 1;
            writes.put(keys::slot(account, login.slot), &login.id)?;
            writes.put(keys::browser(account, &login.browser_public_key), &login.id)?;
        } else if login.references == 0 {
            header.idle_logins = header
                .idle_logins
                .checked_sub(1)
                .ok_or(StoreError::Corrupt)?;
            writes.delete(keys::login_expiry(&login));
        }
        login.references = login
            .references
            .checked_add(1)
            .ok_or(StoreError::Capacity)?;
        if let Some(sequence) = renewal {
            writes.put(keys::renewal(&login.id, sequence), &row.ticket)?;
            header.renewal_indexes += 1;
        }
        writes.put(keys::login(&login.id), &login)?;
        writes.put(
            keys::operation_expiry(row.remove_ns, &row.ticket),
            &row.ticket,
        )?;
        writes.put(keys::ticket(&row.ticket), &row)?;
        let status = IssuerStatus::Pending(row.ticket.clone());
        self.publish(header, writes)?;
        Ok(status)
    }
}

pub(super) fn login_reservation(login: &Login) -> Result<u64, StoreError> {
    let mut maximum = login.clone();
    maximum.references = u64::MAX;
    maximum.sequence = u64::MAX;
    size(&maximum)
}
fn validate(
    account: Principal,
    request: &IssueRequest,
    policy: IssuerPolicy,
) -> Result<(), StoreError> {
    if account == Principal::anonymous()
        || request.operation_id == [0; 32]
        || request.login_id == [0; 32]
        || request.browser_public_key == [0; 32]
        || request.epoch == 0
        || request.login_epoch == 0
        || policy.login_ttl_ns == 0
        || policy.login_ttl_ns > crate::limits::MAX_LOGIN_NS
        || policy.proof_ttl_ns == 0
        || policy.proof_ttl_ns > crate::limits::MAX_PROOF_NS
        || policy.receipt_timeout_ns == 0
        || policy.reconciliation_retention_ns == 0
    {
        return Err(StoreError::Invalid);
    }
    ValidatedGrants::new(request.grants.clone()).map_err(|_| StoreError::Invalid)?;
    Ok(())
}
pub(super) fn grants_hash(
    grants: &[ic_auth_protocol_types::DelegatedRoleGrant],
) -> Result<[u8; 32], StoreError> {
    ValidatedGrants::new(grants.to_vec()).map_err(|_| StoreError::Invalid)?;
    hash(&grants)
}
pub(super) fn hash<T: serde::Serialize>(value: &T) -> Result<[u8; 32], StoreError> {
    let bytes = ic_wire::native::encode_request(
        0x41555448495201,
        value,
        ic_wire::Limits {
            bytes: 73_728,
            allocation: 1_048_576,
            nodes: 16_384,
            depth: 16,
        },
    )
    .map_err(|_| StoreError::Invalid)?;
    Ok(Sha256::digest(bytes).into())
}
