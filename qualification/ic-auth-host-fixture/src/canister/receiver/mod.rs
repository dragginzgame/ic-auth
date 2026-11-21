//! Real IC receiver transactions with fixed protected test authority.
mod enrollment;
mod upgrade;
use upgrade::Backend;
pub(super) use upgrade::{decode_upgrade_fault, reopen, seal, upgrade_guard};

use crate::{
    RESPONSE,
    enrollment::Snapshot,
    hash_bytes,
    receiver::{
        CONTROL, Capacity, Command, MEASURE, Measurement, OBSERVE, Observation, Outcome, REQUEST,
        Refusal, TIME_ORIGIN_NS, audience,
    },
};
use ic_auth::{
    receiver_memory::{
        AdmissionHost, EnrollmentBinding, LoginLimits, LogoutError, LogoutReceipt, MemoryConfig,
        MemoryError, MemoryReceiver, SessionAuthority, SessionCapacity, SessionPolicyError,
        UnknownLogoutLimits,
    },
    session_request::{SessionRequest, SessionTtlPolicy},
    verification::{
        AuthorityGrants, Ed25519Key, VerificationContext, VerificationCosts, VerificationProfile,
    },
};
use ic_auth_protocol_types::{
    Credential, CredentialProof, DelegatedRoleGrant, PortableMechanism, Principal,
};
use ic_platform_runtime::{self as runtime, system as api};
use std::cell::RefCell;

const SECOND: u64 = 1_000_000_000;

///
/// State
///
/// Owns the installed receiver and host-side enrollment configuration.
///

#[derive(Default)]
struct State {
    receiver: Option<Backend>,
    capacity: Capacity,
    authority: Option<Principal>,
    sequence: u64,
    entry: Option<Snapshot>,
    application_writes: u32,
    wide_scopes: bool,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn config(authority: Principal, capacity: Capacity) -> MemoryConfig {
    MemoryConfig {
        audience: audience(),
        enrollment: EnrollmentBinding {
            receiver: api::canister_self(),
            authority,
        },
        login: LoginLimits {
            max_device_slots: 8,
            max_login_ttl_ns: 100 * SECOND,
            max_admitted_logins: capacity.logins as usize,
        },
        capacity: SessionCapacity {
            max_sessions: capacity.sessions as usize,
            max_replays: capacity.replays as usize,
            max_replays_per_account: capacity.account_replays as usize,
            max_encoded_bytes: capacity.bytes as usize,
        },
        verification_capacity: capacity.verification,
        refill_interval_ns: capacity.refill_ns,
        verification_costs: VerificationCosts {
            ed25519: 2,
            certified: 3,
        },
        unknown_logout: UnknownLogoutLimits {
            max_denials: 1,
            capacity: 2,
            refill_interval_ns: SECOND,
        },
    }
}
pub(super) fn init(authority: Principal, capacity: Capacity, stable: bool) {
    let config = config(authority, capacity);
    let receiver = if stable {
        upgrade::create(config)
    } else {
        Backend::Volatile(MemoryReceiver::new_fresh(config, api::time()).unwrap())
    };
    STATE.with_borrow_mut(|s| {
        s.capacity = capacity;
        s.receiver = Some(receiver);
        s.authority = Some(authority);
        s.wide_scopes = capacity.wide_scopes;
    });
}

fn refusal(e: MemoryError) -> Refusal {
    match e {
        MemoryError::Closed => Refusal::Closed,
        MemoryError::Policy(SessionPolicyError::Revoked) => Refusal::Revoked,
        MemoryError::Policy(SessionPolicyError::MissingOrExpired) => Refusal::Missing,
        MemoryError::Proof(_) => Refusal::Proof,
        _ => Refusal::Other,
    }
}

/// Reject oversized ingress before copying or decoding the request.
pub(super) fn bounded_request() -> Result<(), String> {
    if api::argument_size() > REQUEST.bytes {
        Err("oversized receiver fixture frame".into())
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

pub(super) fn encode_result(result: Result<Outcome, Refusal>) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &result,
        ic_wire::Encoding::Frame {
            method: CONTROL,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
}

pub(super) async fn control(command: Command) -> Result<Outcome, Refusal> {
    if let Command::Enroll(fault) = command {
        return enrollment::refresh(fault).await;
    }
    let caller = api::caller();
    let now = api::time();
    STATE.with_borrow_mut(|state| {
        let wide = state.wide_scopes;
        let State {
            receiver,
            application_writes,
            entry,
            ..
        } = state;
        let receiver = receiver.as_mut().ok_or(Refusal::Closed)?;
        if let Command::Prune(limit) = command {
            let report = receiver.prune(now, limit as usize).map_err(refusal)?;
            return Ok(Outcome::Pruned(
                report.sessions as u32,
                report.replays as u32,
                report.unknown_denials as u32,
            ));
        }
        // Logout depends on intact safety state, not current authority freshness.
        if let Command::Logout(trap_after) = command {
            let outcome = match receiver.logout(caller, now).map_err(|e| match e {
                LogoutError::Closed => Refusal::Closed,
                _ => Refusal::Other,
            })? {
                LogoutReceipt::Revoked { deny_until_ns } => Outcome::Revoked(deny_until_ns),
                LogoutReceipt::Unconfirmed { deny_until_ns } => Outcome::Unconfirmed(deny_until_ns),
            };
            if trap_after {
                api::trap("after receiver logout");
            }
            return Ok(outcome);
        }
        let entry = entry.as_ref().ok_or(Refusal::Closed)?;
        let audience = audience();
        let role: ic_auth_protocol_types::AuthRole =
            if wide { "r".repeat(64) } else { "user".into() }
                .parse()
                .unwrap();
        let scopes: Vec<ic_auth_protocol_types::ScopeLabel> = if wide {
            (0..32)
                .map(|i| format!("s{i:02}{}", "x".repeat(61)).parse().unwrap())
                .collect()
        } else {
            vec!["read".parse().unwrap()]
        };
        let requested = &scopes[..scopes.len().min(16)];
        let grants = AuthorityGrants::new(vec![DelegatedRoleGrant {
            target: role.clone(),
            scopes: scopes.clone(),
        }])
        .unwrap();
        let context = VerificationContext {
            audience: &audience,
            grants: &grants,
            issuer: Principal::from_slice(&[1]),
            mechanism: PortableMechanism::Ed25519,
            key_binding_id: hash_bytes(entry.key),
            caller,
            now_ns: now,
            authority_not_before_ns: 0,
            authority_expires_at_ns: TIME_ORIGIN_NS + 200 * SECOND,
            enrollment_expires_at_ns: entry.expires_at_ns,
            min_login_started_at_ns: 0,
            min_proof_issued_at_ns: 0,
            max_login_ttl_ns: 100 * SECOND,
            max_proof_ttl_ns: 60 * SECOND,
        };
        let (outcome, trap_after) = match command {
            Command::Admit {
                claims,
                signature,
                trap_after,
            } => {
                let credential = Credential {
                    claims,
                    proof: CredentialProof::Ed25519(signature),
                };
                let key = Ed25519Key::new(entry.key).unwrap();
                let admitted = receiver
                    .admit(
                        &credential,
                        SessionRequest::new(
                            requested.iter().map(|s| s.as_str().to_owned()).collect(),
                            None,
                        )
                        .unwrap(),
                        AdmissionHost {
                            verifier: VerificationProfile::Ed25519 {
                                context: &context,
                                key: &key,
                            },
                            role: &role,
                            ttl: SessionTtlPolicy::new(60, 1800).unwrap(),
                            generation: entry.generation,
                            enabled: !entry.revoked,
                            subject_admissible: true,
                        },
                    )
                    .map_err(refusal)?;
                (
                    Outcome::Admitted(admitted.exact_retry, admitted.receipt.expires_at_ns),
                    trap_after,
                )
            }
            Command::Authorize(_) | Command::AuthorizeUnrequested => {
                let trap_after = matches!(command, Command::Authorize(true));
                let required = if matches!(command, Command::AuthorizeUnrequested) {
                    std::slice::from_ref(scopes.get(16).ok_or(Refusal::Other)?)
                } else {
                    requested
                };

                let authority = SessionAuthority {
                    caller,
                    now_ns: now,
                    audience,
                    issuer: context.issuer,
                    mechanism: context.mechanism,
                    key_binding_id: context.key_binding_id,
                    role: &role,
                    grants: grants.as_slice(),
                    generation: entry.generation,
                    not_before_ns: 0,
                    expires_at_ns: context.authority_expires_at_ns,
                    enrollment_expires_at_ns: context.enrollment_expires_at_ns,
                    min_login_started_at_ns: 0,
                    max_session_ttl_secs: 1800,
                    enabled: !entry.revoked,
                    subject_admissible: true,
                };
                receiver
                    .authorize(&authority, required, |_| {
                        *application_writes += 1;
                    })
                    .map_err(refusal)?;
                (Outcome::Authorized, trap_after)
            }
            Command::Logout(_) | Command::Enroll(_) | Command::Prune(_) => unreachable!(),
        };
        if trap_after {
            api::trap("after receiver operation and application projection");
        }
        Ok(outcome)
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
    reason = "Platform owns the handler result"
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
        let mut result = Observation {
            present: s.receiver.is_some(),
            usage: (0, 0, 0),
            logins: (0, 0),
            account_replays: (0, 0),
            credit: (0, 0),
            observed_at_ns: 0,
            application_writes: s.application_writes,
            enrollment: s
                .receiver
                .as_ref()
                .and_then(|r| r.enrollment())
                .map(|m| (m.revision, m.generation, m.expires_at_ns)),
            revoked: s.entry.as_ref().map(|e| e.revoked),
        };
        if let Some(receiver) = &s.receiver {
            let usage = receiver.usage((Principal::from_slice(&[1]), Principal::from_slice(&[2])));
            result.account_replays = (
                usage.replays_for_account as u64,
                receiver
                    .usage((Principal::from_slice(&[1]), Principal::from_slice(&[3])))
                    .replays_for_account as u64,
            );
            result.usage = (
                usage.sessions as u64,
                usage.replays as u64,
                usage.encoded_bytes as u64,
            );
            result.logins = (
                receiver.known_logins() as u64,
                receiver.unknown_logins() as u64,
            );
            result.credit = (
                receiver.verification_budget().available(),
                receiver.unknown_logout_credit(),
            );
            result.observed_at_ns = receiver.observed_at_ns();
        }
        result
    })
}

pub(super) fn decode_measurement(bytes: &[u8]) -> Result<Command, ic_wire::Error> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: MEASURE,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
}

pub(super) fn encode_measurement(value: Measurement) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: MEASURE,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
}

pub(super) async fn measure(command: Command) -> Measurement {
    // Counter zero is message-local: never measure across an inter-canister await.
    if matches!(command, Command::Enroll(_)) {
        api::trap("enrollment is outside synchronous measurement");
    }
    let before_heap = std::arch::wasm32::memory_size::<0>() as u64;
    let before = runtime::raw::instruction_counter();
    let result = control(command).await;
    let instructions = runtime::raw::instruction_counter() - before;
    let after_heap = std::arch::wasm32::memory_size::<0>() as u64;
    let response_bytes = encode_result(result).unwrap().len() as u64;
    Measurement {
        result,
        instructions,
        heap_pages: (before_heap, after_heap),
        response_bytes,
    }
}
