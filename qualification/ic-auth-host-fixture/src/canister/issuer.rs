//! Issuer operations and certification for the installed qualification fixture.

use super::{browser, certified, receiver};
use crate::{
    CONTROL, Command, Fault, OBSERVE, Observation, Progress, REQUEST, RESPONSE, Refusal, SEED,
    hash_bytes,
};
use ic_auth::issuer_memory::{
    CertifiedCompletionError, IssueRequest, IssuerAuthority, IssuerConfig, IssuerError,
    IssuerPolicy, IssuerSigning, IssuerStatus, IssuerTicket, MemoryIssuer, RenewRequest,
    StableIssuer,
};
use ic_auth_protocol_types::{AudienceId, CanonicalId, DelegatedRoleGrant, Principal};
use ic_platform::certificate::construction::{
    HashTree, empty, encode_canister_signature, labeled, leaf,
};
use ic_platform_runtime::{self as runtime, system as api};
use ic_storage::{DefaultMemoryImpl, RuntimeMemory};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

///
/// Backend
///
/// Selects the volatile or stable Auth state used by the installed fixture.
///

enum Backend {
    Volatile(MemoryIssuer),
    Stable(StableIssuer),
}

impl Deref for Backend {
    type Target = MemoryIssuer;
    fn deref(&self) -> &MemoryIssuer {
        match self {
            Self::Volatile(s) => s,
            Self::Stable(s) => s.as_ref(),
        }
    }
}

impl DerefMut for Backend {
    fn deref_mut(&mut self) -> &mut MemoryIssuer {
        match self {
            Self::Volatile(s) => s,
            Self::Stable(s) => s.state(),
        }
    }
}

///
/// HostState
///
/// Host selection and certified leaf saved alongside the issuer checkpoint.
///

#[derive(Serialize, Deserialize)]
pub(super) struct HostState {
    selected: Option<(Principal, u64, [u8; 32])>,
    leaf: Option<([u8; 32], [u8; 32])>,
}

///
/// State
///
/// Holds the installed issuer, selected operation, ticket and certified leaf.
///

#[derive(Default)]
struct State {
    issuer: Option<Backend>,
    selected: Option<(Principal, u64, [u8; 32])>,
    ticket: Option<IssuerTicket>,
    leaf: Option<([u8; 32], [u8; 32])>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn witness(key: Option<([u8; 32], [u8; 32])>) -> HashTree {
    labeled(
        b"sig".to_vec(),
        match key {
            Some((seed, message)) => labeled(seed, labeled(message, leaf(Vec::new()))),
            None => empty(),
        },
    )
}

fn authority() -> IssuerAuthority {
    IssuerAuthority {
        signing: IssuerSigning::Certified {
            seed_hash: hash_bytes(SEED),
        },
        generation: 1,
        key_binding_id: [7; 32], // Protected fixture authority, no root enrollment claim.
        expires_at_ns: u64::MAX,
    }
}

pub(super) fn decode_install(bytes: &[u8]) -> Result<crate::Installation, ic_wire::Error> {
    if bytes.is_empty() {
        return Ok(crate::Installation {
            authority: Vec::new(),
            certified_network_key: Vec::new(),
            stable_auth: false,
            receiver_capacity: None,
        });
    }
    ic_wire::decode(bytes, ic_wire::Encoding::Payload, crate::enrollment::LIMITS)
}

pub(super) fn init(installation: crate::Installation) {
    let authority = if installation.authority.is_empty() {
        api::canister_self()
    } else {
        ic_auth_protocol_types::Principal::try_from_slice(&installation.authority).unwrap()
    };
    certified::init(installation.certified_network_key);
    receiver::init(
        authority,
        installation.receiver_capacity.unwrap_or_default(),
        installation.stable_auth,
    );
    browser::init();
    // Only a fresh install creates state. Post-upgrade never calls this function.
    if !installation.stable_auth {
        let issuer = MemoryIssuer::new_fresh(config(), api::time()).unwrap();
        STATE.with_borrow_mut(|s| s.issuer = Some(Backend::Volatile(issuer)));
    }
    api::set_certified_data(&witness(None).digest());
}

fn config() -> IssuerConfig {
    IssuerConfig {
        audience: AudienceId {
            canonical_network_id: CanonicalId::from_bytes([1; 32]),
            fleet_id: CanonicalId::from_bytes([2; 32]),
        },
        issuer: api::canister_self(),
        max_logins: 1,
        max_operations: 2,
        max_operations_per_account: 2,
        max_devices_per_account: 1,
        max_result_bytes: 4096,
        issuance_burst: 4,
        issuance_refill_interval_ns: 1_000_000_000,
    }
}

pub(super) fn create_stable(lease: RuntimeMemory<DefaultMemoryImpl>) {
    let issuer = StableIssuer::new_fresh(lease, config(), api::time()).unwrap();
    STATE.with_borrow_mut(|s| s.issuer = Some(Backend::Stable(issuer)));
}

pub(super) fn seal() -> HostState {
    STATE.with_borrow_mut(|s| {
        let Some(Backend::Stable(owner)) = &mut s.issuer else {
            api::trap("missing stable issuer")
        };
        owner.seal().unwrap();
        HostState {
            selected: s.selected,
            leaf: s.leaf,
        }
    })
}

pub(super) fn reopen(lease: RuntimeMemory<DefaultMemoryImpl>, host: &HostState) {
    let issuer = StableIssuer::reopen(lease, config(), api::time()).unwrap();
    let ticket = host.selected.and_then(|(account, epoch, id)| {
        match issuer
            .as_ref()
            .reconcile(account, epoch, id)
            .unwrap()
            .unwrap()
        {
            IssuerStatus::Pending(t) | IssuerStatus::Unknown(t) => Some(t),
            IssuerStatus::Completed(_) => None,
        }
    });
    api::set_certified_data(&witness(host.leaf).digest());
    STATE.with_borrow_mut(|s| {
        *s = State {
            issuer: Some(Backend::Stable(issuer)),
            ticket,
            selected: host.selected,
            leaf: host.leaf,
        }
    });
}

pub(super) fn lost_volatile_state() {
    // On a volatile installation, heap loss is terminal. Clear the certified contribution,
    // leave the issuer absent, and provide no timestamp/generation reset escape.
    api::set_certified_data(&witness(None).digest());
}

/// Reject oversized ingress before copying or decoding the request.
pub(super) fn bounded_request() -> Result<(), String> {
    if api::argument_size() > REQUEST.bytes {
        Err("oversized fixture frame".into())
    } else {
        Ok(())
    }
}

pub(super) fn decode_command(bytes: &[u8]) -> Result<Command, ic_wire::Error> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: CONTROL,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
}

pub(super) fn encode_result(result: Result<(), Refusal>) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &result,
        ic_wire::Encoding::Frame {
            method: CONTROL,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
}

fn refusal(error: IssuerError) -> Refusal {
    match error {
        IssuerError::Closed => Refusal::Closed,
        IssuerError::Denied => Refusal::Denied,
        IssuerError::Expired => Refusal::Expired,
        _ => Refusal::Auth,
    }
}

pub(super) fn control(command: Command) -> Result<(), Refusal> {
    let caller = api::caller();
    let now = api::time();
    STATE.with_borrow_mut(|state| {
        let State {
            issuer,
            ticket,
            selected,
            leaf,
        } = state;
        let issuer = issuer.as_mut().ok_or(Refusal::Closed)?;
        if caller == ic_auth_protocol_types::Principal::anonymous()
            || (!matches!(command, Command::Renew)
                && selected.is_some_and(|(account, _, _)| account != caller))
        {
            return Err(Refusal::Denied);
        }
        match command {
            Command::Prepare => {
                let request = IssueRequest {
                    epoch: 1,
                    operation_id: [1; 32],
                    login_epoch: 1,
                    login_id: [2; 32],
                    browser_public_key: [0x44; 32],
                    grants: vec![DelegatedRoleGrant {
                        target: "app".parse().unwrap(),
                        scopes: vec!["read".parse().unwrap()],
                    }],
                };
                let t = issuer
                    .prepare_issue(
                        caller,
                        &request,
                        IssuerPolicy {
                            login_ttl_ns: 90_000_000_000,
                            proof_ttl_ns: 10_000_000_000,
                            receipt_timeout_ns: 20_000_000_000,
                            reconciliation_retention_ns: 20_000_000_000,
                        },
                        authority(),
                        now,
                    )
                    .map_err(refusal)?;
                // Deterministic entropy and approval exist only in this fixture.
                issuer
                    .record_nonce(t, authority(), [5; 16], now)
                    .map_err(refusal)?;
                issuer
                    .record_approval(t, authority(), [6; 32], now)
                    .map_err(refusal)?;
                *selected = Some((t.account(), t.epoch(), t.operation_id()));
                *ticket = Some(t);
            }
            Command::Complete(fault) => {
                let t = ticket.ok_or(Refusal::Auth)?;
                let result = issuer.complete_certified(t, authority(), now, |p| {
                    *leaf = Some((p.seed_hash, p.message_hash));
                    if matches!(fault, Fault::Leaf) {
                        api::trap("after leaf mutation");
                    }
                    api::set_certified_data(&witness(*leaf).digest());
                    if matches!(fault, Fault::Root) {
                        api::trap("after certified root publication");
                    }
                    if matches!(fault, Fault::HostError) {
                        return Err("host failed after publication");
                    }
                    Ok(())
                });
                match result {
                    Ok(_) => {}
                    Err(CertifiedCompletionError::Issuer(e)) => return Err(refusal(e)),
                    // Returning an ordinary Err here would COMMIT the host's leaf
                    // and partial state. Trap across the enclosing IC transaction.
                    Err(CertifiedCompletionError::Publication(e)) => api::trap(e),
                }
                if matches!(fault, Fault::Completion) {
                    api::trap("after retained completion");
                }
            }
            Command::Renew => {
                let account = selected.ok_or(Refusal::Auth)?.0;
                let t = issuer
                    .prepare_renewal(
                        caller,
                        &RenewRequest {
                            account,
                            login_epoch: 1,
                            login_id: [2; 32],
                            sequence: 1,
                            grants: vec![DelegatedRoleGrant {
                                target: "app".parse().unwrap(),
                                scopes: vec!["read".parse().unwrap()],
                            }],
                        },
                        authority(),
                        now,
                    )
                    .map_err(refusal)?;
                issuer
                    .record_nonce(t, authority(), [8; 16], now)
                    .map_err(refusal)?;
                issuer
                    .record_approval(t, authority(), [9; 32], now)
                    .map_err(refusal)?;
                *selected = Some((t.account(), t.epoch(), t.operation_id()));
                *ticket = Some(t);
            }
            Command::Revoke => issuer.revoke(caller, 1, [2; 32], now).map_err(refusal)?,
        }
        Ok(())
    })
}

pub(super) fn decode_unit(bytes: &[u8]) -> Result<(), ic_wire::Error> {
    ic_wire::decode::<()>(
        bytes,
        ic_wire::Encoding::Frame {
            method: OBSERVE,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform transfers the handler result to its encoding hook"
)]
pub(super) fn encode_observation(value: Observation) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: OBSERVE,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
}

pub(super) fn observe() -> Observation {
    STATE.with_borrow(|s| {
        let certificate = api::data_certificate(RESPONSE.bytes).unwrap().unwrap();
        let tree = witness(s.leaf);
        let mut view = Observation {
            progress: Progress::Closed,
            usage: (0, 0, 0),
            credit: 0,
            claims: Vec::new(),
            root: tree.digest(),
            certificate,
            signature: Vec::new(),
            now_ns: api::time(),
        };
        if let Some(issuer) = &s.issuer {
            let u = issuer.usage();
            view.usage = (u.logins, u.operations, u.result_bytes);
            view.credit = issuer.available_credit();
            view.progress = match s.selected {
                None => Progress::Empty,
                Some((account, epoch, id)) => match issuer.reconcile(account, epoch, id) {
                    Ok(Some(IssuerStatus::Completed(result))) => {
                        view.claims = result.claims.to_vec();
                        Progress::Completed
                    }
                    Ok(Some(_)) => Progress::Pending,
                    _ => Progress::Closed,
                },
            };
        }
        if s.leaf.is_some() && view.progress == Progress::Completed {
            view.signature =
                encode_canister_signature(&view.certificate, &tree, RESPONSE.bytes).unwrap();
        }
        view
    })
}

pub(super) fn decode_upgrade(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        Ok(())
    } else {
        Err("unexpected upgrade input".into())
    }
}

pub(super) fn decode_measurement(bytes: &[u8]) -> Result<Command, ic_wire::Error> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: crate::MEASURE,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
}

pub(super) fn encode_measurement(value: crate::Measurement) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: crate::MEASURE,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
}

pub(super) fn measure(command: Command) -> crate::Measurement {
    let before_heap = std::arch::wasm32::memory_size::<0>() as u64;
    let before = runtime::raw::instruction_counter();
    let result = control(command);
    let instructions = runtime::raw::instruction_counter() - before;
    let after_heap = std::arch::wasm32::memory_size::<0>() as u64;
    let response_bytes = encode_result(result).unwrap().len() as u64;
    crate::Measurement {
        result,
        instructions,
        heap_pages: (before_heap, after_heap),
        response_bytes,
    }
}

pub(super) fn decode_witness_measurement(bytes: &[u8]) -> Result<(), ic_wire::Error> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: crate::WITNESS_MEASURE,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform owns the handler result"
)]
pub(super) fn encode_witness_measurement(
    value: crate::WitnessMeasurement,
) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: crate::WITNESS_MEASURE,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
}

pub(super) fn measure_witness() -> crate::WitnessMeasurement {
    let before_heap = std::arch::wasm32::memory_size::<0>() as u64;
    let before = runtime::raw::instruction_counter();
    let observation = observe();
    let instructions = runtime::raw::instruction_counter() - before;
    let after_heap = std::arch::wasm32::memory_size::<0>() as u64;
    let response_bytes = encode_observation(observation.clone()).unwrap().len() as u64;
    crate::WitnessMeasurement {
        observation,
        instructions,
        heap_pages: (before_heap, after_heap),
        response_bytes,
    }
}

pub(super) fn endpoint_error<E: std::fmt::Debug>(error: E) -> ! {
    ic_platform_runtime::system::trap(&format!("fixture endpoint refused: {error:?}"))
}
