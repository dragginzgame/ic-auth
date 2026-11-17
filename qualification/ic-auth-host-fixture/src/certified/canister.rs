//! Isolated installed certified receiver; no application endpoint mounting.
use super::{Command, Measurement, SECOND, at, audience, bytes, credential, grants, hash};
use crate::receiver::{Outcome, Refusal};
use ic_auth::{
    receiver_memory::{
        AdmissionHost, EnrollmentBinding, EnrollmentMetadata, LoginLimits, LogoutReceipt,
        MemoryConfig, MemoryError, MemoryReceiver, SessionAuthority, SessionCapacity,
        SessionPolicyError, UnknownLogoutLimits,
    },
    root_delegation::{DelegationLimits, RootKeyPolicy},
    session_request::{SessionRequest, SessionTtlPolicy},
    verification::{
        AuthorityGrants, CertifiedContext, CertifiedLimits, VerificationContext, VerificationCosts,
        VerificationProfile,
    },
};
use ic_auth_protocol_types::{PortableMechanism, Principal};
use ic_platform_runtime::{self as runtime, system as api};
use std::cell::RefCell;
struct State {
    receiver: MemoryReceiver,
    network: Vec<u8>,
}
thread_local! { static STATE: RefCell<Option<State>> = const { RefCell::new(None) }; }
pub(crate) fn init(network: Vec<u8>) {
    if network.is_empty() {
        return;
    }
    assert_eq!(network.len(), 96);
    let now = api::time();
    let id = api::canister_self();
    let mut receiver = MemoryReceiver::new_fresh(
        MemoryConfig {
            audience: audience(),
            enrollment: EnrollmentBinding {
                receiver: id,
                authority: id,
            },
            login: LoginLimits {
                max_device_slots: 8,
                max_login_ttl_ns: 100 * SECOND,
                max_admitted_logins: 1,
            },
            capacity: SessionCapacity {
                max_sessions: 1,
                max_replays: 2,
                max_replays_per_account: 2,
                max_encoded_bytes: 4096,
            },
            verification_capacity: 10,
            refill_interval_ns: 10 * SECOND,
            verification_costs: VerificationCosts {
                ed25519: 2,
                certified: 3,
            },
            unknown_logout: UnknownLogoutLimits {
                max_denials: 1,
                capacity: 2,
                refill_interval_ns: SECOND,
            },
        },
        now,
    )
    .unwrap();
    receiver.request_enrollment(now, [1; 32], at(100)).unwrap();
    receiver
        .install_enrollment(
            now,
            id,
            EnrollmentMetadata {
                receiver: id,
                audience: audience(),
                challenge: [1; 32],
                revision: 1,
                generation: 9,
                snapshot_digest: [2; 32],
                authority_digest: [3; 32],
                issued_at_ns: now,
                expires_at_ns: at(210),
            },
        )
        .unwrap();
    STATE.with_borrow_mut(|s| *s = Some(State { receiver, network }));
}
fn refusal(error: MemoryError) -> Refusal {
    match error {
        MemoryError::Proof(_) => Refusal::Proof,
        MemoryError::Policy(SessionPolicyError::Revoked) => Refusal::Revoked,
        _ => Refusal::Other,
    }
}
fn control(s: &mut State, command: Command) -> Result<Outcome, Refusal> {
    let now = api::time();
    let caller = api::caller();
    let audience = audience();
    let grants = AuthorityGrants::new(grants()).unwrap();
    let role = "app".parse().unwrap();
    let context = VerificationContext {
        audience: &audience,
        grants: &grants,
        issuer: Principal::from_slice(&[2; 10]),
        mechanism: PortableMechanism::Certified,
        key_binding_id: hash("certificate_hash"),
        caller,
        now_ns: now,
        authority_not_before_ns: at(100),
        authority_expires_at_ns: at(200),
        enrollment_expires_at_ns: at(210),
        min_login_started_at_ns: at(100),
        min_proof_issued_at_ns: at(100),
        max_login_ttl_ns: 100 * SECOND,
        max_proof_ttl_ns: 60 * SECOND,
    };
    match command {
        Command::Admit(signature) => {
            let key = bytes("public_key");
            let certified = CertifiedContext {
                claims: context,
                root_key: RootKeyPolicy {
                    root: Principal::from_slice(&[1; 10]),
                    key_name: "test_key_1",
                    derivation_path_hash: hash("path_hash"),
                    public_key: &key,
                    key_version: 4,
                    min_key_version: 4,
                    min_proof_epoch: 2,
                    min_registry_epoch: 3,
                    valid_from_ns: at(90),
                    accept_until_ns: at(210),
                    max_revocation_latency_ns: 100 * SECOND,
                },
                ic_root_public_key_raw: &s.network,
                limits: CertifiedLimits {
                    delegation: DelegationLimits {
                        max_certificate_ttl_ns: 100 * SECOND,
                        max_token_ttl_ns: 60 * SECOND,
                        max_variable_bytes: 131072,
                        max_witness_steps: 64,
                    },
                    max_issuer_signature_bytes: 16384,
                    max_certificate_age_ns: 20 * SECOND,
                },
            };
            let der = ic_platform::certificate::construction::canister_key_der(
                context.issuer,
                super::SEED,
            )
            .unwrap();
            let proof = credential(signature, der);
            let result = s
                .receiver
                .admit(
                    &proof,
                    SessionRequest::new(vec!["read".into()], None).unwrap(),
                    AdmissionHost {
                        verifier: VerificationProfile::Certified(&certified),
                        role: &role,
                        ttl: SessionTtlPolicy::new(60, 1800).unwrap(),
                        generation: 9,
                        enabled: true,
                        subject_admissible: true,
                    },
                )
                .map_err(refusal)?;
            Ok(Outcome::Admitted(
                result.exact_retry,
                result.receipt.expires_at_ns,
            ))
        }
        Command::Authorize => {
            let authority = SessionAuthority {
                caller,
                now_ns: now,
                audience,
                issuer: context.issuer,
                mechanism: context.mechanism,
                key_binding_id: context.key_binding_id,
                role: &role,
                grants: grants.as_slice(),
                generation: 9,
                not_before_ns: at(100),
                expires_at_ns: at(200),
                enrollment_expires_at_ns: at(210),
                min_login_started_at_ns: at(100),
                max_session_ttl_secs: 1800,
                enabled: true,
                subject_admissible: true,
            };
            s.receiver
                .authorize(&authority, &["read".parse().unwrap()], |_| ())
                .map_err(refusal)?;
            Ok(Outcome::Authorized)
        }
        Command::Logout => match s.receiver.logout(caller, now).map_err(|_| Refusal::Other)? {
            LogoutReceipt::Revoked { deny_until_ns } => Ok(Outcome::Revoked(deny_until_ns)),
            LogoutReceipt::Unconfirmed { deny_until_ns } => Ok(Outcome::Unconfirmed(deny_until_ns)),
        },
    }
}
fn decode(bytes: &[u8]) -> Result<Command, ic_wire::Error> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: super::MEASURE,
            direction: ic_wire::Direction::Request,
        },
        crate::RESPONSE,
    )
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform owns the handler result"
)]
fn encode(value: Measurement) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: super::MEASURE,
            direction: ic_wire::Direction::Response,
        },
        crate::RESPONSE,
    )
}
fn guard() -> Result<(), String> {
    if api::argument_size() <= crate::RESPONSE.bytes {
        Ok(())
    } else {
        Err("oversized certified fixture input".into())
    }
}
#[runtime::update(runtime = ic_platform_runtime, name = "certified_fixture_measure", guard = guard, decode = decode, encode = encode, max_bytes = 32768, on_error = crate::endpoint_error)]
fn measure(command: Command) -> Measurement {
    STATE.with_borrow_mut(|state| {
        let s = state
            .as_mut()
            .expect("missing installed certified trust/state");
        let before_heap = core::arch::wasm32::memory_size::<0>() as u64;
        let before = runtime::raw::instruction_counter();
        let result = control(s, command);
        let instructions = runtime::raw::instruction_counter() - before;
        let after_heap = core::arch::wasm32::memory_size::<0>() as u64;
        let usage = s.receiver.usage((
            Principal::from_slice(&[2; 10]),
            Principal::from_slice(&[5; 10]),
        ));
        let response_bytes = ic_wire::encode(
            &result,
            ic_wire::Encoding::Frame {
                method: super::MEASURE,
                direction: ic_wire::Direction::Response,
            },
            crate::RESPONSE,
        )
        .unwrap()
        .len() as u64;
        Measurement {
            result,
            instructions,
            heap_pages: (before_heap, after_heap),
            usage: (usage.sessions as u64, usage.replays as u64),
            credit: s.receiver.verification_budget().available(),
            response_bytes,
        }
    })
}
