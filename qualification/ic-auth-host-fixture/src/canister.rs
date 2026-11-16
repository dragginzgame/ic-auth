//! Standalone fixture adapter, not Canic application endpoint integration.
use super::{
    CONTROL, Command, Fault, OBSERVE, Observation, Progress, REQUEST, RESPONSE, Refusal, SEED,
};
use ic_auth::issuer_memory::{
    CertifiedCompletionError, IssueRequest, IssuerAuthority, IssuerConfig, IssuerError,
    IssuerPolicy, IssuerSigning, IssuerStatus, IssuerTicket, MemoryIssuer,
};
use ic_auth_protocol_types::{AudienceId, CanonicalId, DelegatedRoleGrant};
use ic_platform::{
    canister_sig_creation::hash_bytes,
    certification::{HashTree, cbor::encode_canister_signature, empty, labeled, leaf},
};
use ic_platform_runtime::{cdk, cdk::api, ic0};
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

#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform owns the installation buffer"
)]
fn decode_install(bytes: Vec<u8>) -> ic_auth_protocol_types::Principal {
    if bytes.is_empty() {
        return api::canister_self();
    }
    let authority: Vec<u8> = ic_wire::decode(
        &bytes,
        ic_wire::Encoding::Payload,
        crate::enrollment::LIMITS,
    )
    .unwrap();
    ic_auth_protocol_types::Principal::try_from_slice(&authority).unwrap()
}
#[cdk::init(decode_with = "decode_install")]
fn init(authority: ic_auth_protocol_types::Principal) {
    super::receiver::canister::init(authority);
    // Only a fresh install creates state. Post-upgrade never calls this function.
    let issuer = MemoryIssuer::new_fresh(
        IssuerConfig {
            audience: AudienceId {
                canonical_network_id: CanonicalId::from_bytes([1; 32]),
                fleet_id: CanonicalId::from_bytes([2; 32]),
            },
            issuer: api::canister_self(),
            max_logins: 1,
            max_operations: 1,
            max_operations_per_account: 1,
            max_devices_per_account: 1,
            max_result_bytes: 4096,
            issuance_burst: 4,
            issuance_refill_interval_ns: 1_000_000_000,
        },
        api::time(),
    )
    .unwrap();
    STATE.with_borrow_mut(|s| s.issuer = Some(issuer));
    api::certified_data_set(witness(None).digest());
}

#[cdk::post_upgrade]
fn lost_volatile_state() {
    // No stable adapter: heap loss is terminal. Clear the certified contribution,
    // leave the issuer absent, and provide no timestamp/generation reset escape.
    api::certified_data_set(witness(None).digest());
}

fn bounded_request() -> Result<(), String> {
    if ic0::msg_arg_data_size() > REQUEST.bytes {
        Err("oversized fixture frame".into())
    } else {
        Ok(())
    }
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform transfers the admitted argument buffer to its codec hook"
)]
fn decode_command(bytes: Vec<u8>) -> Command {
    ic_wire::decode(
        &bytes,
        ic_wire::Encoding::Frame {
            method: CONTROL,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
    .unwrap()
}
fn encode_result(result: Result<(), Refusal>) -> Vec<u8> {
    ic_wire::encode(
        &result,
        ic_wire::Encoding::Frame {
            method: CONTROL,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
    .unwrap()
}
fn refusal(error: IssuerError) -> Refusal {
    match error {
        IssuerError::Closed => Refusal::Closed,
        IssuerError::Denied => Refusal::Denied,
        IssuerError::Expired => Refusal::Expired,
        _ => Refusal::Auth,
    }
}

#[cdk::update(
    name = "auth_fixture_control",
    guard = "bounded_request",
    decode_with = "decode_command",
    encode_with = "encode_result"
)]
fn control(command: Command) -> Result<(), Refusal> {
    let caller = api::msg_caller();
    let now = api::time();
    STATE.with_borrow_mut(|state| {
        let State {
            issuer,
            ticket,
            leaf,
        } = state;
        let issuer = issuer.as_mut().ok_or(Refusal::Closed)?;
        if caller == ic_auth_protocol_types::Principal::anonymous()
            || ticket.is_some_and(|t| t.account() != caller)
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
                    api::certified_data_set(witness(*leaf).digest());
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
            Command::Revoke => issuer.revoke(caller, 1, [2; 32], now).map_err(refusal)?,
        }
        Ok(())
    })
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform transfers the admitted argument buffer to its codec hook"
)]
fn decode_unit(bytes: Vec<u8>) {
    ic_wire::decode::<()>(
        &bytes,
        ic_wire::Encoding::Frame {
            method: OBSERVE,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
    .unwrap();
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform transfers the handler result to its encoding hook"
)]
fn encode_observation(value: Observation) -> Vec<u8> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: OBSERVE,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
    .unwrap()
}
#[cdk::query(
    name = "auth_fixture_observe",
    guard = "bounded_request",
    decode_with = "decode_unit",
    encode_with = "encode_observation"
)]
fn observe() -> Observation {
    STATE.with_borrow(|s| {
        let certificate = api::data_certificate().unwrap();
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
