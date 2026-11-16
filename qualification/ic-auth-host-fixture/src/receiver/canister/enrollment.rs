//! Installation-bound call and atomic callback publication for a single test key.
use super::{Outcome, Refusal, SECOND, STATE, audience};
use crate::enrollment::{Fault, KEY, LIMITS, METHOD, Snapshot};
use ic_auth::receiver_memory::EnrollmentMetadata;
use ic_platform::canister_sig_creation::hash_bytes;
use ic_platform_runtime::{cdk::api, cdk::call::Call};

pub(super) async fn refresh(fault: Fault) -> Result<Outcome, Refusal> {
    let (authority, challenge) = STATE.with_borrow_mut(|s| {
        let receiver = s.receiver.as_mut().ok_or(Refusal::Closed)?;
        let authority = s.authority.ok_or(Refusal::Closed)?;
        s.sequence = s.sequence.checked_add(1).ok_or(Refusal::Closed)?;
        // Deterministic fixture uniqueness only, NOT production challenge entropy.
        // Missing heap never constructs another receiver or dispatches a request.
        let mut challenge = [0; 32];
        challenge[..8].copy_from_slice(&s.sequence.to_be_bytes());
        let now = api::time();
        receiver
            .request_enrollment(now, challenge, now + 10 * SECOND)
            .map_err(|_| Refusal::Enrollment)?;
        Ok((authority, challenge))
    })?;
    let bytes = ic_wire::encode(
        &challenge,
        ic_wire::Encoding::Frame {
            method: METHOD,
            direction: ic_wire::Direction::Request,
        },
        LIMITS,
    )
    .unwrap();
    // The runtime owns callback provenance. Never use ingress caller or a reply
    // field as authenticated_authority. No state borrow survives this await.
    let response = Call::bounded_wait(authority, "authority_fixture_snapshot")
        .with_raw_args(&bytes)
        .await
        .map_err(|_| Refusal::Unavailable)?;
    let snapshot: Snapshot = ic_wire::decode(
        response.as_ref(),
        ic_wire::Encoding::Frame {
            method: METHOD,
            direction: ic_wire::Direction::Response,
        },
        LIMITS,
    )
    .map_err(|_| Refusal::Enrollment)?;
    let snapshot_digest =
        hash_bytes(ic_wire::encode(&snapshot, ic_wire::Encoding::Payload, LIMITS).unwrap());
    let authority_digest = hash_bytes(
        ic_wire::encode(
            &(snapshot.key, snapshot.revoked),
            ic_wire::Encoding::Payload,
            LIMITS,
        )
        .unwrap(),
    );
    STATE.with_borrow_mut(|s| {
        let receiver = s.receiver.as_mut().ok_or(Refusal::Closed)?;
        if s.authority != Some(authority)
            || snapshot.key != KEY
            || s.entry
                .as_ref()
                .is_some_and(|old| old.revoked && !snapshot.revoked)
        {
            return Err(Refusal::Enrollment);
        }
        let receiver_id = ic_auth_protocol_types::Principal::try_from_slice(&snapshot.receiver)
            .map_err(|_| Refusal::Enrollment)?;
        receiver
            .install_enrollment(
                api::time(),
                authority,
                EnrollmentMetadata {
                    receiver: receiver_id,
                    audience: audience(),
                    challenge: snapshot.challenge,
                    revision: snapshot.revision,
                    generation: snapshot.generation,
                    snapshot_digest,
                    authority_digest,
                    issued_at_ns: snapshot.issued_at_ns,
                    expires_at_ns: snapshot.expires_at_ns,
                },
            )
            .map_err(|_| Refusal::Enrollment)?;
        if matches!(fault, Fault::Metadata) {
            api::trap("after enrollment metadata");
        }
        s.entry = Some(snapshot);
        if matches!(fault, Fault::Entry) {
            api::trap("after enrollment entry");
        }
        Ok(Outcome::Enrolled)
    })
}
