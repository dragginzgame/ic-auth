//! Fixture-only upgrade checkpoint using the host-owned IC Memory allocation.
//! This is not a production stable backend or a Root enrollment implementation.
use super::*;
use candid::CandidType;
use ic_auth::{portable::LoginRevocationChange, session::Replay};
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub(crate) struct Snapshot {
    config: ReceiverConfig,
    login_floor: u64,
    proof_floor: u64,
    generation: u64,
    enabled: bool,
    checkpoint_time: u64,
    observed: u64,
    session_count: u64,
    replay_count: u64,
    encoded_bytes: u64,
    admitted_count: u64,
    unknown_count: u64,
    sessions: Vec<Vec<u8>>,
    replays: Vec<Vec<u8>>,
    logins: Vec<Vec<u8>>,
}

pub(crate) fn capture() -> Option<Snapshot> {
    RECEIVER.with(|state| {
        let state = state.borrow();
        let host = state.as_ref().filter(|host| host.checkpoint)?;
        Some(Snapshot {
            config: host.config.clone(),
            login_floor: host.login_floor,
            proof_floor: host.proof_floor,
            generation: host.generation,
            enabled: host.enabled,
            checkpoint_time: ic_cdk::api::time(),
            observed: host.ledger.observed_at_ns(),
            session_count: host.store.session_records().len() as u64,
            replay_count: host.store.replay_records().len() as u64,
            encoded_bytes: host.store.encoded_bytes() as u64,
            admitted_count: host.ledger.occupancy().0 as u64,
            unknown_count: host.ledger.occupancy().1 as u64,
            sessions: host
                .store
                .session_records()
                .map(|s| s.encode().expect("encode session"))
                .collect(),
            replays: host
                .store
                .replay_records()
                .map(|r| r.encode().expect("encode replay"))
                .collect(),
            logins: host
                .ledger
                .records()
                .map(|r| r.encode().expect("encode login"))
                .collect(),
        })
    })
}

pub(crate) fn restore(snapshot: Option<Snapshot>) {
    let Some(saved) = snapshot else { return };
    let now = ic_cdk::api::time();
    assert!(now >= saved.checkpoint_time, "checkpoint clock rollback");
    assert!(
        saved.session_count <= 8
            && saved.replay_count <= 32
            && saved.admitted_count <= 8
            && saved.unknown_count <= 8
            && saved.encoded_bytes <= 65_536,
        "checkpoint bounds"
    );
    let store = MemorySessionStore::new(saved.generation, limits())
        .expect("store limits")
        .restore_records(
            (saved.session_count as usize, saved.replay_count as usize),
            saved.encoded_bytes as usize,
            saved
                .sessions
                .iter()
                .map(|bytes| Session::decode(bytes).expect("decode session")),
            saved
                .replays
                .iter()
                .map(|bytes| Replay::decode(bytes).expect("decode replay")),
        )
        .expect("restore complete session/replay store");
    let ledger = MemoryLoginRevocations::new(saved.config.audience.0, 8, 8, 0, now)
        .expect("ledger limits")
        .restore_records(
            saved.observed,
            (saved.admitted_count as usize, saved.unknown_count as usize),
            saved
                .logins
                .iter()
                .map(|bytes| LoginRevocationChange::decode(bytes).expect("decode login")),
        )
        .expect("restore complete login ledger");
    let admission = SessionAdmissionBudget::new(
        saved.config.verification_burst,
        saved.config.verification_refill_ns,
        now,
    )
    .expect("admission budget");
    let host = Receiver {
        checkpoint: true,
        ready: false,
        binding: saved.config.authority.binding(),
        config: saved.config,
        login_floor: saved.login_floor,
        proof_floor: saved.proof_floor,
        generation: saved.generation,
        enabled: saved.enabled,
        store,
        ledger,
        // Conservatively discard remaining credit; never refill on upgrade.
        admission,
        unknown: SessionAdmissionBudget::new(16, 1, now).expect("logout budget"),
    };
    RECEIVER.with(|state| *state.borrow_mut() = Some(host));
}

#[ic_cdk::update]
fn enable_session_checkpoint() -> Result<(), String> {
    super::super::controller()?;
    with_receiver(|host| {
        host.checkpoint = true;
        Ok(())
    })
}

/// The fixture's controller attests current authority after this boot. Production
/// requires authenticated receiver-bound Root enrollment, not this test endpoint.
#[ic_cdk::update]
fn confirm_restored_session_authority(config: ReceiverConfig) -> Result<(), String> {
    super::super::controller()?;
    RECEIVER.with(|state| {
        let mut state = state.borrow_mut();
        let host = state
            .as_mut()
            .filter(|host| host.checkpoint && !host.ready)
            .ok_or("receiver is not awaiting authority")?;
        if candid::encode_one(&config).map_err(|e| e.to_string())?
            != candid::encode_one(&host.config).map_err(|e| e.to_string())?
        {
            return Err("checkpoint authority mismatch".into());
        }
        let now = ic_cdk::api::time();
        if config.valid_from_ns > now
            || config.valid_until_ns <= now
            || config.enrollment_until_ns <= now
        {
            return Err("checkpoint authority expired".into());
        }
        host.ready = true;
        Ok(())
    })
}
