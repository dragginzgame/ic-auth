use super::*;
use std::collections::BTreeMap;

impl<M: Memory> ReceiverStore<M> {
    /// At most 32 selected expired records per transaction. Known device fences
    /// and replay evidence survive logout; only expired replay rows and unknown
    /// denials are eligible here. No full-table snapshot or compaction is used.
    /// # Panics
    /// Collection/IO faults require host transaction rollback; native hosts must
    /// discard the runtime after a mutation panic.
    pub fn prune(&mut self, now_ns: u64, budget: u32) -> Result<ReceiverPruned, StoreError> {
        self.observe(now_ns)?;
        let budget = budget.min(32) as usize;
        let mut selected = Vec::new();
        for kind in [
            keys::SESSION_EXPIRY,
            keys::REPLAY_EXPIRY,
            keys::LOGIN_EXPIRY,
        ] {
            selected.extend(self.records.expired(kind, now_ns, budget)?);
        }
        selected.sort_by_key(|key| key.time().expect("expiry keys admitted by selection"));
        selected.truncate(budget);
        let mut header = self.header.clone();
        let mut writes = Writes::new();
        let mut accounts = BTreeMap::<Key, usize>::new();
        let mut pruned = ReceiverPruned::default();
        for expiry in selected {
            match expiry.0[0] {
                keys::SESSION_EXPIRY => {
                    let caller = self.records.get(&expiry)?.ok_or(StoreError::Corrupt)?;
                    let session = self.read_session(caller)?.ok_or(StoreError::Corrupt)?;
                    if keys::session_expiry(&session) != expiry || session.expires_at_ns() > now_ns
                    {
                        return Err(StoreError::Corrupt);
                    }
                    header.counts.sessions = header
                        .counts
                        .sessions
                        .checked_sub(1)
                        .ok_or(StoreError::Corrupt)?;
                    header.counts.encoded_bytes = header
                        .counts
                        .encoded_bytes
                        .checked_sub(session.encode().map_err(|_| StoreError::Corrupt)?.len())
                        .ok_or(StoreError::Corrupt)?;
                    self.delete_session(&session, &mut writes)?;
                    pruned.sessions += 1;
                }
                keys::REPLAY_EXPIRY => {
                    let fingerprint: [u8; 32] =
                        self.records.get(&expiry)?.ok_or(StoreError::Corrupt)?;
                    let bytes: serde_bytes::ByteBuf = self
                        .records
                        .get(&keys::replay(&fingerprint))?
                        .ok_or(StoreError::Corrupt)?;
                    let replay = Replay::decode(&bytes).map_err(|_| StoreError::Corrupt)?;
                    if replay.fingerprint() != fingerprint
                        || keys::replay_expiry(&replay) != expiry
                        || replay.remove_at_ns() > now_ns
                    {
                        return Err(StoreError::Corrupt);
                    }
                    let key = keys::replay_count(replay.account());
                    let count = if let Some(count) = accounts.get_mut(&key) {
                        count
                    } else {
                        accounts
                            .entry(key.clone())
                            .or_insert(self.records.get(&key)?.ok_or(StoreError::Corrupt)?)
                    };
                    *count = count.checked_sub(1).ok_or(StoreError::Corrupt)?;
                    header.counts.replays = header
                        .counts
                        .replays
                        .checked_sub(1)
                        .ok_or(StoreError::Corrupt)?;
                    header.counts.encoded_bytes = header
                        .counts
                        .encoded_bytes
                        .checked_sub(bytes.len())
                        .ok_or(StoreError::Corrupt)?;
                    writes.delete(keys::replay(&fingerprint));
                    writes.delete(expiry);
                    pruned.replays += 1;
                }
                keys::LOGIN_EXPIRY => {
                    let caller = self.records.get(&expiry)?.ok_or(StoreError::Corrupt)?;
                    let login = self.login(caller)?.ok_or(StoreError::Corrupt)?;
                    if binding(&login).is_some()
                        || keys::login_expiry(&login) != expiry
                        || login.until_ns() > now_ns
                    {
                        return Err(StoreError::Corrupt);
                    }
                    header.counts.unknown_denials = header
                        .counts
                        .unknown_denials
                        .checked_sub(1)
                        .ok_or(StoreError::Corrupt)?;
                    writes.delete(keys::login(caller));
                    writes.delete(expiry);
                    pruned.unknown_denials += 1;
                }
                _ => return Err(StoreError::Corrupt),
            }
        }
        for (key, count) in accounts {
            if count == 0 {
                writes.delete(key);
                header.replay_accounts = header
                    .replay_accounts
                    .checked_sub(1)
                    .ok_or(StoreError::Corrupt)?;
            } else {
                writes.put(key, &count)?;
            }
        }
        if pruned != ReceiverPruned::default() {
            self.publish(header, writes)?;
        }
        Ok(pruned)
    }
}
