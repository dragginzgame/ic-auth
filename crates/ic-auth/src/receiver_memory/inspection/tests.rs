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
                ext: None,
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
