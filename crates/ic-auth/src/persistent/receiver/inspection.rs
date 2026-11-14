use super::{CallerRecord, Memory, Principal, ReceiverStore, Session, StoreError, keys};

/// Maximum returned sessions per inspection call.
pub const MAX_INSPECTION_PAGE: u16 = 32;
/// Maximum visited caller rows, plus one lookahead row, per inspection call.
pub const MAX_INSPECTION_SCAN: u16 = 128;

/// Passive stored metadata, never an authentication/authorization result.
/// Expired and stale-generation sessions remain visible. Derive observations
/// from each session's expiry/generation and this page's supplied time/generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionPage {
    /// Durable clock observation accompanying this page.
    pub observed_at_ns: u64,
    /// Protected receiver generation observed with this inspection page.
    pub generation: u64,
    /// Bounded physical session rows selected by the cursor; not authorization results.
    pub sessions: Vec<Session>,
    /// Exclusive cursor over visited caller rows, including fences. A page can
    /// be empty and still have a continuation. No snapshot across calls is held.
    pub next_after: Option<Principal>,
    /// Rows visited for projection, excluding the optional single lookahead.
    pub scanned: u16,
}

impl<M: Memory> ReceiverStore<M> {
    /// Inspect stored sessions in canonical Principal order. Endpoint permission
    /// belongs to the host. Reads at most `scan_budget + 1` caller rows, returns
    /// at most `limit` sessions, and never writes, authenticates, prunes or advances
    /// the clock. Fences consume the scan budget and advance the cursor. Callers
    /// must follow `next_after` even on empty/short pages; do not loop to fill a
    /// page inside one host request. All per-record decoding remains bounded.
    pub fn inspect_sessions(
        &self,
        after: Option<Principal>,
        limit: u16,
        scan_budget: u16,
        now_ns: u64,
    ) -> Result<SessionPage, StoreError> {
        if limit == 0
            || limit > MAX_INSPECTION_PAGE
            || scan_budget < limit
            || scan_budget > MAX_INSPECTION_SCAN
        {
            return Err(StoreError::Invalid);
        }
        if now_ns < self.header.observed_ns {
            return Err(StoreError::Clock);
        }
        let mut rows = self
            .records
            .range::<CallerRecord>(keys::LOGIN, after.map(keys::login));
        let mut page = SessionPage {
            observed_at_ns: now_ns,
            generation: self.header.generation,
            sessions: Vec::with_capacity(usize::from(limit)),
            next_after: None,
            scanned: 0,
        };
        let mut last = None;
        for row in rows.by_ref().take(usize::from(scan_budget)) {
            let (key, row) = row?;
            let caller = key.principal_value(keys::LOGIN)?;
            let (session, _) = self.decode_caller(caller, &row)?;
            page.scanned += 1;
            last = Some(caller);
            if let Some(session) = session {
                page.sessions.push(session);
                if page.sessions.len() == usize::from(limit) {
                    break;
                }
            }
        }
        if rows.next().transpose()?.is_some() {
            page.next_after = last;
        }
        Ok(page)
    }
}
