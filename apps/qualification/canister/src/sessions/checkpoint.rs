//! Fixture-only upgrade checkpoint using the host-owned IC Memory allocation.
//! This is not a production stable backend or a Root enrollment implementation.
use super::*;
use ic_auth::{portable::LoginRevocationChange, session::Replay};
use ic_auth_qualification_contracts::{self as controls, Refusal};
use ic_wire::{Bytes, List};

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) enum Snapshot {
    // An explicit trusted checkpoint marker, never inferred from missing state.
    Unconfigured,
    Closed,
    Retained(Box<SavedReceiver>),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SavedReceiver {
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

pub(crate) fn capture() -> Snapshot {
    RECEIVER.with(|state| {
        let state = state.borrow();
        let Some(host) = state.as_ref() else {
            return if UNCONFIGURED.with(Cell::get) {
                Snapshot::Unconfigured
            } else {
                Snapshot::Closed
            };
        };
        if !host.lifecycle.checkpoint() {
            return Snapshot::Closed;
        }
        Snapshot::Retained(Box::new(SavedReceiver {
            config: host
                .inputs
                .config
                .clone()
                .try_into()
                .expect("bounded checkpoint config"),
            login_floor: host.inputs.login_floor,
            proof_floor: host.inputs.proof_floor,
            generation: host.store.generation(),
            enabled: host.enabled,
            checkpoint_time: ic_platform_runtime::cdk::api::time(),
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
        }))
    })
}

pub(crate) fn restore(snapshot: Snapshot) {
    // Only the explicit trusted never-configured marker permits first install.
    // A configured receiver's lost state remains closed across further upgrades.
    UNCONFIGURED.with(|unconfigured| unconfigured.set(matches!(&snapshot, Snapshot::Unconfigured)));
    let Snapshot::Retained(saved) = snapshot else {
        return;
    };
    let config: ReceiverConfig = saved.config.try_into().expect("checkpoint config");
    let now = ic_platform_runtime::cdk::api::time();
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
        .restore_records(
            (saved.session_count as usize, saved.replay_count as usize),
            saved.encoded_bytes as usize,
            saved
                .sessions
                .into_vec()
                .into_iter()
                .map(|bytes| Session::decode(bytes.as_slice()).expect("decode session")),
            saved
                .replays
                .into_vec()
                .into_iter()
                .map(|bytes| Replay::decode(bytes.as_slice()).expect("decode replay")),
        )
        .expect("restore complete session/replay store");
    let ledger = login_ledger(config.audience, now)
        .expect("ledger limits")
        .restore_records(
            saved.observed,
            (saved.admitted_count as usize, saved.unknown_count as usize),
            saved.logins.into_vec().into_iter().map(|bytes| {
                LoginRevocationChange::decode(bytes.as_slice()).expect("decode login")
            }),
        )
        .expect("restore complete login ledger");
    let host = Receiver::new(
        ReceiverInputs {
            config,
            login_floor: saved.login_floor,
            proof_floor: saved.proof_floor,
        },
        ReceiverLifecycle::AwaitingAuthority,
        saved.enabled,
        store,
        ledger,
        // Conservatively discard remaining credit; never refill on upgrade.
        now,
    )
    .expect("restored budgets");
    RECEIVER.with(|state| *state.borrow_mut() = Some(host));
}

#[ic_platform_runtime::cdk::update(
    guard = "crate::control::EnableCheckpoint::guard",
    decode_with = "crate::control::EnableCheckpoint::decode",
    encode_with = "crate::control::EnableCheckpoint::encode"
)]
fn enable_session_checkpoint() -> Result<(), Refusal> {
    super::super::controller()?;
    with_receiver(|host| {
        host.lifecycle = ReceiverLifecycle::CheckpointEnabled;
        Ok(())
    })
}

/// The fixture's controller attests current authority after this boot. Production
/// requires authenticated receiver-bound Root enrollment, not this test endpoint.
#[ic_platform_runtime::cdk::update(
    guard = "crate::control::ConfirmAuthority::guard",
    decode_with = "crate::control::ConfirmAuthority::decode",
    encode_with = "crate::control::ConfirmAuthority::encode"
)]
fn confirm_restored_session_authority(
    config: controls::model::ReceiverConfig,
) -> Result<(), Refusal> {
    super::super::controller()?;
    let config: ReceiverConfig = config
        .try_into()
        .map_err(|_: ic_wire::Error| Refusal::Codec)?;
    RECEIVER.with(|state| {
        let mut state = state.borrow_mut();
        let host = state
            .as_mut()
            .filter(|host| host.lifecycle == ReceiverLifecycle::AwaitingAuthority)
            .ok_or(Refusal::AwaitingAuthority)?;
        if config != host.inputs.config {
            return Err(Refusal::AuthorityMismatch);
        }
        let now = ic_platform_runtime::cdk::api::time();
        if config.valid_from_ns > now
            || config.valid_until_ns <= now
            || config.enrollment_until_ns <= now
        {
            return Err(Refusal::AuthorityExpired);
        }
        host.lifecycle = ReceiverLifecycle::CheckpointEnabled;
        Ok(())
    })
}
