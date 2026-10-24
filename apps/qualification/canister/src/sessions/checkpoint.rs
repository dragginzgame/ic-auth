//! Fixture-only upgrade checkpoint using the host-owned IC Memory allocation.
//! This is not a production stable backend or a Root enrollment implementation.
use super::*;
use ic_auth::{portable::LoginRevocationChange, session::Replay};
use ic_auth_qualification_contracts as controls;
use ic_wire::{Bytes, List};

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Snapshot {
    config: controls::model::ReceiverConfig,
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
    sessions: List<Bytes<MAX_ENCODED_BYTES>, MAX_SESSIONS>,
    replays: List<Bytes<MAX_SMALL_RECORD_BYTES>, MAX_REPLAYS>,
    logins: List<Bytes<MAX_SMALL_RECORD_BYTES>, MAX_LOGIN_RECORDS>,
}

pub(crate) fn capture() -> Option<Snapshot> {
    RECEIVER.with(|state| {
        let state = state.borrow();
        let host = state.as_ref().filter(|host| host.checkpoint)?;
        Some(Snapshot {
            config: host
                .config
                .clone()
                .try_into()
                .expect("bounded checkpoint config"),
            login_floor: host.login_floor,
            proof_floor: host.proof_floor,
            generation: host.store.generation(),
            enabled: host.enabled,
            checkpoint_time: ic_platform::cdk::api::time(),
            observed: host.ledger.observed_at_ns(),
            session_count: host.store.session_records().len() as u64,
            replay_count: host.store.replay_records().len() as u64,
            encoded_bytes: host.store.encoded_bytes() as u64,
            admitted_count: host.ledger.occupancy().0 as u64,
            unknown_count: host.ledger.occupancy().1 as u64,
            sessions: List::new(
                host.store
                    .session_records()
                    .map(|s| {
                        Bytes::new(s.encode().expect("encode session")).expect("session bound")
                    })
                    .collect(),
            )
            .expect("checkpoint record count"),
            replays: List::new(
                host.store
                    .replay_records()
                    .map(|r| Bytes::new(r.encode().expect("encode replay")).expect("replay bound"))
                    .collect(),
            )
            .expect("checkpoint record count"),
            logins: List::new(
                host.ledger
                    .records()
                    .map(|r| Bytes::new(r.encode().expect("encode login")).expect("login bound"))
                    .collect(),
            )
            .expect("checkpoint record count"),
        })
    })
}

pub(crate) fn restore(snapshot: Option<Snapshot>) {
    let Some(saved) = snapshot else { return };
    let config: ReceiverConfig = saved.config.try_into().expect("checkpoint config");
    let now = ic_platform::cdk::api::time();
    assert!(now >= saved.checkpoint_time, "checkpoint clock rollback");
    assert!(
        saved.session_count <= MAX_SESSIONS as u64
            && saved.replay_count <= MAX_REPLAYS as u64
            && saved.admitted_count <= MAX_ADMITTED_LOGINS as u64
            && saved.unknown_count <= MAX_UNKNOWN_LOGINS as u64
            && saved.encoded_bytes <= MAX_ENCODED_BYTES as u64,
        "checkpoint bounds"
    );
    let store = MemorySessionStore::new(saved.generation, limits())
        .expect("store limits")
        .restore_records(
            (saved.session_count as usize, saved.replay_count as usize),
            saved.encoded_bytes as usize,
            saved
                .sessions
                .as_slice()
                .iter()
                .map(|bytes| Session::decode(bytes.as_slice()).expect("decode session")),
            saved
                .replays
                .as_slice()
                .iter()
                .map(|bytes| Replay::decode(bytes.as_slice()).expect("decode replay")),
        )
        .expect("restore complete session/replay store");
    let ledger =
        MemoryLoginRevocations::new(
            config.audience,
            MAX_ADMITTED_LOGINS,
            MAX_UNKNOWN_LOGINS,
            0,
            now,
        )
        .expect("ledger limits")
        .restore_records(
            saved.observed,
            (saved.admitted_count as usize, saved.unknown_count as usize),
            saved.logins.as_slice().iter().map(|bytes| {
                LoginRevocationChange::decode(bytes.as_slice()).expect("decode login")
            }),
        )
        .expect("restore complete login ledger");
    let admission = SessionAdmissionBudget::new(
        config.verification_burst,
        config.verification_refill_ns,
        now,
    )
    .expect("admission budget");
    let host = Receiver {
        checkpoint: true,
        ready: false,
        binding: config.authority.binding(),
        config,
        login_floor: saved.login_floor,
        proof_floor: saved.proof_floor,
        enabled: saved.enabled,
        store,
        ledger,
        // Conservatively discard remaining credit; never refill on upgrade.
        admission,
        unknown: SessionAdmissionBudget::new(16, 1, now).expect("logout budget"),
    };
    RECEIVER.with(|state| *state.borrow_mut() = Some(host));
}

#[ic_platform::cdk::update(
    guard = "crate::control::guard",
    decode_with = "crate::control::EnableCheckpoint::decode",
    encode_with = "crate::control::EnableCheckpoint::encode"
)]
fn enable_session_checkpoint() -> Result<(), String> {
    super::super::controller()?;
    with_receiver(|host| {
        host.checkpoint = true;
        Ok(())
    })
}

/// The fixture's controller attests current authority after this boot. Production
/// requires authenticated receiver-bound Root enrollment, not this test endpoint.
#[ic_platform::cdk::update(
    guard = "crate::control::guard",
    decode_with = "crate::control::ConfirmAuthority::decode",
    encode_with = "crate::control::ConfirmAuthority::encode"
)]
fn confirm_restored_session_authority(
    config: controls::model::ReceiverConfig,
) -> Result<(), String> {
    super::super::controller()?;
    let config: ReceiverConfig = config
        .try_into()
        .map_err(|e: ic_wire::Error| e.to_string())?;
    RECEIVER.with(|state| {
        let mut state = state.borrow_mut();
        let host = state
            .as_mut()
            .filter(|host| host.checkpoint && !host.ready)
            .ok_or("receiver is not awaiting authority")?;
        if config != host.config {
            return Err("checkpoint authority mismatch".into());
        }
        let now = ic_platform::cdk::api::time();
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
