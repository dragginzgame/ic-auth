//! Passive bounded traversal. Reference contract provenance is in reference.sha256.
use super::{Lifecycle, MemoryError, MemoryReceiver};
use crate::receiver_policy::{SessionFacts, SessionPolicyError};
use ic_auth_protocol_types::Principal;
use std::ops::Bound::{Excluded, Unbounded};

/// Largest permitted number of session projections in one page.
pub const MAX_INSPECTION_PAGE: u16 = 32;
/// Largest permitted caller-row scan, excluding one continuation lookahead.
pub const MAX_INSPECTION_SCAN: u16 = 128;

/// Borrowed retained facts for administrative inspection, never authorization.
/// Expired leases and stale generations remain visible until explicit removal.
#[derive(Debug)]
pub struct SessionPage<'a> {
    /// Host-supplied observation time; inspection does not retain this clock value.
    pub observed_at_ns: u64,
    /// Receiver generation at this call, for comparison with each retained session.
    pub generation: u64,
    /// Physical session records, including ones that no longer authorize anything.
    pub sessions: Vec<SessionFacts<'a>>,
    /// Exclusive caller cursor when more rows remain. Empty pages can continue.
    /// Mutations between calls are allowed; there is no cross-call snapshot.
    pub next_after: Option<Principal>,
    /// Caller rows examined for projection, including fences but excluding lookahead.
    pub scanned: u16,
}

impl MemoryReceiver {
    /// Inspect in Principal order with independently bounded results and visited
    /// rows. Host administrative permission is required outside this library.
    /// Neither enrollment freshness nor session liveness filters retained facts.
    ///
    /// Does not authorize, refill, observe time, prune or mutate state. Closed or
    /// poisoned instances refuse; a genuinely fresh instance is inspectable while
    /// boot enrollment is pending. Follow `next_after` even on an empty page, in a
    /// separate host request. A cursor remains valid if its caller row is deleted.
    pub fn inspect_sessions(
        &self,
        after: Option<Principal>,
        limit: u16,
        scan_budget: u16,
        now_ns: u64,
    ) -> Result<SessionPage<'_>, MemoryError> {
        if self.lifecycle == Lifecycle::Closed {
            return Err(MemoryError::Closed);
        }
        if limit == 0
            || limit > MAX_INSPECTION_PAGE
            || scan_budget < limit
            || scan_budget > MAX_INSPECTION_SCAN
        {
            return Err(SessionPolicyError::InvalidContext.into());
        }
        if now_ns < self.observed_at_ns {
            return Err(SessionPolicyError::ClockRollback.into());
        }
        let rows = self
            .callers
            .range((after.map_or(Unbounded, Excluded), Unbounded));
        Ok(collect(
            rows.map(|(caller, row)| (*caller, row.session_facts())),
            limit,
            scan_budget,
            now_ns,
            self.generation,
        ))
    }
}

fn collect<'a>(
    mut rows: impl Iterator<Item = (Principal, Option<SessionFacts<'a>>)>,
    limit: u16,
    scan_budget: u16,
    now_ns: u64,
    generation: u64,
) -> SessionPage<'a> {
    let mut page = SessionPage {
        observed_at_ns: now_ns,
        generation,
        sessions: Vec::with_capacity(usize::from(limit)),
        next_after: None,
        scanned: 0,
    };
    let mut last = None;
    while page.scanned < scan_budget && page.sessions.len() < usize::from(limit) {
        let Some((caller, session)) = rows.next() else {
            return page;
        };
        page.scanned += 1;
        last = Some(caller);
        if let Some(session) = session {
            page.sessions.push(session);
        }
    }
    if rows.next().is_some() {
        page.next_after = last;
    }
    page
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receiver_policy::LoginBinding;
    use ic_auth_protocol_types::{AudienceId, CanonicalId, PortableMechanism};
    use std::cell::Cell;

    #[test]
    fn inspection_consumes_only_the_bounded_rows_and_one_lookahead() {
        let audience = AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        };
        let mut callers: Vec<_> = (0u16..512)
            .map(|i| Principal::self_authenticating(i.to_be_bytes()))
            .collect();
        callers.sort();
        let visits = Cell::new(0);
        let rows = callers
            .iter()
            .map(|&caller| (caller, None))
            .inspect(|_| visits.set(visits.get() + 1));
        let page = collect(rows, 32, 128, 0, 7);
        assert_eq!(visits.get(), 129);
        assert_eq!(page.scanned, 128);
        assert!(page.sessions.is_empty());
        assert_eq!(page.next_after, Some(callers[127]));

        // Physical records are deliberately stale: the traversal must not filter
        // them or scan for a full page of currently authorizing sessions.
        let role = "user".parse().unwrap();
        let rows = callers.iter().map(|&caller| {
            (
                caller,
                Some(SessionFacts {
                    caller,
                    audience,
                    login: LoginBinding {
                        issuer: Principal::from_slice(&[1]),
                        subject: Principal::from_slice(&[2]),
                        device_slot: 0,
                        device_generation: 1,
                        started_at_ns: 1,
                        deadline_ns: 100,
                    },
                    role: &role,
                    scopes: &[],
                    mechanism: PortableMechanism::Ed25519,
                    key_binding_id: [3; 32],
                    authority_generation: 1,
                    established_at_ns: 2,
                    expires_at_ns: 50,
                    fingerprint: [4; 32],
                    request_hash: [5; 32],
                }),
            )
        });
        visits.set(0);
        let page = collect(
            rows.inspect(|_| visits.set(visits.get() + 1)),
            32,
            128,
            200,
            7,
        );
        assert_eq!(visits.get(), 33);
        assert_eq!(page.scanned, 32);
        assert_eq!(page.sessions.len(), 32);
        assert_eq!(page.next_after, Some(callers[31]));
        assert!(
            page.sessions
                .iter()
                .all(|s| s.expires_at_ns < 200 && s.authority_generation != 7)
        );
    }
}
