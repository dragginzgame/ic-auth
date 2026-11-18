//! Standalone fixture adapter, not Canic application endpoint integration.
use super::{
    CONTROL, Command, Fault, OBSERVE, Observation, Progress, REQUEST, RESPONSE, Refusal, SEED,
};
use ic_auth::issuer_memory::{
    CertifiedCompletionError, IssueRequest, IssuerAuthority, IssuerConfig, IssuerError,
    IssuerPolicy, IssuerSigning, IssuerStatus, IssuerTicket, MemoryIssuer, RenewRequest,
};
use ic_auth_protocol_types::{AudienceId, CanonicalId, DelegatedRoleGrant};
use ic_platform::certificate::construction::{
    HashTree, empty, encode_canister_signature, labeled, leaf,
};
use ic_platform_runtime::{self as runtime, system as api};
use std::cell::RefCell;

#[derive(Default)]
struct State {
    issuer: Option<MemoryIssuer>,
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

fn decode_install(bytes: &[u8]) -> Result<crate::Installation, ic_wire::Error> {
    if bytes.is_empty() {
        return Ok(crate::Installation {
            authority: Vec::new(),
            certified_network_key: Vec::new(),
            receiver_capacity: None,
        });
    }
    ic_wire::decode(bytes, ic_wire::Encoding::Payload, crate::enrollment::LIMITS)
}

/// Initialize the issuer and receiver fixtures on a fresh installation.
#[runtime::init(
    runtime = ic_platform_runtime,
    guard = bounded_request,
    decode = decode_install,
    max_bytes = 256,
    on_error = crate::canister::endpoint_error,
)]
fn init(installation: crate::Installation) {
    let authority = if installation.authority.is_empty() {
        api::canister_self()
    } else {
        ic_auth_protocol_types::Principal::try_from_slice(&installation.authority).unwrap()
    };
    super::certified::canister::init(installation.certified_network_key);
    super::receiver::canister::init(
        authority,
        installation.receiver_capacity.unwrap_or_default(),
    );
    super::browser::canister::init();
    // Only a fresh install creates state. Post-upgrade never calls this function.
    let issuer = MemoryIssuer::new_fresh(
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
        },
        api::time(),
    )
    .unwrap();
    STATE.with_borrow_mut(|s| s.issuer = Some(issuer));
    api::set_certified_data(&witness(None).digest());
}

/// Clear certification after volatile state loss.
#[runtime::post_upgrade(
    runtime = ic_platform_runtime,
    guard = bounded_request,
    decode = decode_upgrade,
    max_bytes = 0,
    on_error = crate::canister::endpoint_error,
)]
fn lost_volatile_state((): ()) {
    // No stable adapter: heap loss is terminal. Clear the certified contribution,
    // leave the issuer absent, and provide no timestamp/generation reset escape.
    api::set_certified_data(&witness(None).digest());
}

/// Reject oversized ingress before copying or decoding the request.
fn bounded_request() -> Result<(), String> {
    if api::argument_size() > REQUEST.bytes {
        Err("oversized fixture frame".into())
    } else {
        Ok(())
    }
}

fn decode_command(bytes: &[u8]) -> Result<Command, ic_wire::Error> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: CONTROL,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
}

fn encode_result(result: Result<(), Refusal>) -> Result<Vec<u8>, ic_wire::Error> {
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

/// Exercise an issuer operation with the actual ingress caller and time.
#[runtime::update(
    runtime = ic_platform_runtime,
    name = "auth_fixture_control",
    guard = bounded_request,
    decode = decode_command,
    encode = encode_result,
    max_bytes = 256,
    on_error = crate::canister::endpoint_error,
)]
fn control(command: Command) -> Result<(), Refusal> {
    let caller = api::caller();
    let now = api::time();
    STATE.with_borrow_mut(|state| {
        let State {
            issuer,
            ticket,
            leaf,
        } = state;
        let issuer = issuer.as_mut().ok_or(Refusal::Closed)?;
        if caller == ic_auth_protocol_types::Principal::anonymous()
            || (!matches!(command, Command::Renew) && ticket.is_some_and(|t| t.account() != caller))
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
                let account = ticket.ok_or(Refusal::Auth)?.account();
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
                *ticket = Some(t);
            }
            Command::Revoke => issuer.revoke(caller, 1, [2; 32], now).map_err(refusal)?,
        }
        Ok(())
    })
}

fn decode_unit(bytes: &[u8]) -> Result<(), ic_wire::Error> {
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
fn encode_observation(value: Observation) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: OBSERVE,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
}

/// Read retained issuer state and its certification witness.
#[runtime::query(
    runtime = ic_platform_runtime,
    name = "auth_fixture_observe",
    guard = bounded_request,
    decode = decode_unit,
    encode = encode_observation,
    max_bytes = 256,
    on_error = crate::canister::endpoint_error,
)]
fn observe((): ()) -> Observation {
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
            view.progress = match s.ticket {
                None => Progress::Empty,
                Some(t) => match issuer.reconcile(t.account(), t.epoch(), t.operation_id()) {
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

fn decode_upgrade(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        Ok(())
    } else {
        Err("unexpected upgrade input".into())
    }
}

fn decode_measurement(bytes: &[u8]) -> Result<Command, ic_wire::Error> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: super::MEASURE,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
}

fn encode_measurement(value: super::Measurement) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: super::MEASURE,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
}

/// Measure one issuer control operation inside an update.
#[runtime::update(
    runtime = ic_platform_runtime,
    name = "auth_fixture_measure",
    guard = bounded_request,
    decode = decode_measurement,
    encode = encode_measurement,
    max_bytes = 256,
    on_error = crate::canister::endpoint_error,
)]
fn measure(command: Command) -> super::Measurement {
    let before_heap = std::arch::wasm32::memory_size::<0>() as u64;
    let before = runtime::raw::instruction_counter();
    let result = control(command);
    let instructions = runtime::raw::instruction_counter() - before;
    let after_heap = std::arch::wasm32::memory_size::<0>() as u64;
    let response_bytes = encode_result(result).unwrap().len() as u64;
    super::Measurement {
        result,
        instructions,
        heap_pages: (before_heap, after_heap),
        response_bytes,
    }
}

fn decode_witness_measurement(bytes: &[u8]) -> Result<(), ic_wire::Error> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: super::WITNESS_MEASURE,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform owns the handler result"
)]
fn encode_witness_measurement(value: super::WitnessMeasurement) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: super::WITNESS_MEASURE,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
}

/// Measure witness construction in a query.
#[runtime::query(
    runtime = ic_platform_runtime,
    name = "auth_fixture_witness_measure",
    guard = bounded_request,
    decode = decode_witness_measurement,
    encode = encode_witness_measurement,
    max_bytes = 256,
    on_error = crate::canister::endpoint_error,
)]
fn measure_witness((): ()) -> super::WitnessMeasurement {
    let before_heap = std::arch::wasm32::memory_size::<0>() as u64;
    let before = runtime::raw::instruction_counter();
    let observation = observe(());
    let instructions = runtime::raw::instruction_counter() - before;
    let after_heap = std::arch::wasm32::memory_size::<0>() as u64;
    let response_bytes = encode_observation(observation.clone()).unwrap().len() as u64;
    super::WitnessMeasurement {
        observation,
        instructions,
        heap_pages: (before_heap, after_heap),
        response_bytes,
    }
}

pub(crate) fn hash_bytes(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes.as_ref()).into()
}

pub(crate) fn endpoint_error<E: std::fmt::Debug>(error: E) -> ! {
    ic_platform_runtime::system::trap(&format!("fixture endpoint refused: {error:?}"))
}
