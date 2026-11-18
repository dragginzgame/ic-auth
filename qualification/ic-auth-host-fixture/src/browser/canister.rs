//! Fixed test account/browser, local test signer and installation-owned enrollment.
//! No wallet service, external signing/recovery or production Canic mounting is modeled.
use super::{CREDENTIAL, OBSERVE, Observation, WRITE};
use crate::{REQUEST, RESPONSE, canister::hash_bytes};
use ic_auth::{
    issuer_memory::{
        IssueRequest, IssuedProof, IssuerAuthority, IssuerConfig, IssuerError, IssuerPolicy,
        IssuerSigning, IssuerStatus, MemoryIssuer,
    },
    receiver_memory::{
        AdmissionHost, EnrollmentBinding, EnrollmentMetadata, LoginLimits, LogoutReceipt,
        MemoryConfig, MemoryError, MemoryReceiver, SessionAuthority, SessionCapacity,
        SessionPolicyError, UnknownLogoutLimits,
    },
    session_request::{SessionRequest, SessionTtlPolicy},
    verification::{
        AuthorityGrants, Ed25519Key, VerificationContext, VerificationCosts, VerificationProfile,
    },
};
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableMechanism, Principal,
};
use ic_auth_wire::{
    CredentialEnvelope, EstablishSession, Logout, SessionInput, SessionView,
    model::{AdmissionRefusal, LogoutResult, LogoutUnavailable, PortableCredential, PortableProof},
};
use ic_platform::ed25519::{Signer, SigningKey};
use ic_platform_runtime::{self as runtime, system as api};
use std::cell::RefCell;

const SECOND: u64 = 1_000_000_000;
const OP: [u8; 32] = [1; 32];
struct State {
    issuer: MemoryIssuer,
    receiver: MemoryReceiver,
    expires: u64,
    signatures: u32,
    writes: u32,
}

thread_local! { static STATE: RefCell<Option<State>> = const { RefCell::new(None) }; }

fn audience() -> AudienceId {
    AudienceId {
        canonical_network_id: CanonicalId::from_bytes([0x11; 32]),
        fleet_id: CanonicalId::from_bytes([0x22; 32]),
    }
}

// Public deterministic test seeds; production key custody is expressly outside this fixture.
fn signer() -> SigningKey {
    SigningKey::from_bytes(&std::array::from_fn(|i| i as u8))
}

fn browser_key() -> [u8; 32] {
    SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes()
}

fn principal(seed: u8) -> Principal {
    let key = SigningKey::from_bytes(&[seed; 32])
        .verifying_key()
        .to_bytes();
    ic_auth::verification::browser_principal(&key)
}

fn grants() -> Vec<DelegatedRoleGrant> {
    vec![DelegatedRoleGrant {
        target: "user".parse().unwrap(),
        scopes: vec!["write".parse().unwrap()],
    }]
}

fn authority(expires: u64) -> IssuerAuthority {
    IssuerAuthority {
        signing: IssuerSigning::Ed25519,
        generation: 1,
        key_binding_id: hash_bytes(signer().verifying_key().to_bytes()),
        expires_at_ns: expires,
    }
}

pub(crate) fn init() {
    let now = api::time();
    let expires = now + 300 * SECOND;
    let id = api::canister_self();
    let mut receiver = MemoryReceiver::new_fresh(
        MemoryConfig {
            audience: audience(),
            enrollment: EnrollmentBinding {
                receiver: id,
                authority: id,
            },
            login: LoginLimits {
                max_device_slots: 1,
                max_login_ttl_ns: 120 * SECOND,
                max_admitted_logins: 1,
            },
            capacity: SessionCapacity {
                max_sessions: 1,
                max_replays: 2,
                max_replays_per_account: 2,
                max_encoded_bytes: 4096,
            },
            verification_capacity: 10,
            refill_interval_ns: 1_000_000,
            verification_costs: VerificationCosts {
                ed25519: 1,
                certified: 1,
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
    // Protected installation-local authority, not ingress fields or a simulated IC reply.
    receiver.request_enrollment(now, OP, now + SECOND).unwrap();
    receiver
        .install_enrollment(
            now,
            id,
            EnrollmentMetadata {
                receiver: id,
                audience: audience(),
                challenge: OP,
                revision: 1,
                generation: 1,
                snapshot_digest: [2; 32],
                authority_digest: [3; 32],
                issued_at_ns: now,
                expires_at_ns: expires,
            },
        )
        .unwrap();
    let issuer = MemoryIssuer::new_fresh(
        IssuerConfig {
            audience: audience(),
            issuer: id,
            max_logins: 1,
            max_operations: 1,
            max_operations_per_account: 1,
            max_devices_per_account: 1,
            max_result_bytes: 4096,
            issuance_burst: 2,
            issuance_refill_interval_ns: SECOND,
        },
        now,
    )
    .unwrap();
    STATE.with_borrow_mut(|s| {
        *s = Some(State {
            issuer,
            receiver,
            expires,
            signatures: 0,
            writes: 0,
        })
    });
}

/// Reject oversized ingress before copying or decoding the request.
fn bounded<const N: usize>() -> Result<(), String> {
    if api::argument_size() <= N {
        Ok(())
    } else {
        Err("oversized browser fixture frame".into())
    }
}

fn decode_lookup(bytes: &[u8]) -> Result<bool, ic_wire::Error> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: CREDENTIAL,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform owns the reply value"
)]
fn encode_credential(value: PortableCredential) -> Result<Vec<u8>, ic_wire::Error> {
    // Return exact retained credential REQUEST bytes, the browser journal's artifact format.
    CredentialEnvelope::encode_request(&value)
}

/// Issue or look up a browser-bound credential.
#[runtime::update(
    runtime = ic_platform_runtime,
    name = "browser_fixture_credential",
    guard = bounded::<256>,
    decode = decode_lookup,
    encode = encode_credential,
    max_bytes = 256,
    on_error = crate::canister::endpoint_error,
)]
fn credential(lookup: bool) -> PortableCredential {
    let caller = api::caller();
    assert_eq!(caller, principal(8), "fixed fixture account required");
    let now = api::time();
    STATE.with_borrow_mut(|state| {
        let s = state.as_mut().expect("lost browser fixture state");
        if !lookup && s.issuer.reconcile(caller, 1, OP).unwrap().is_none() {
            let auth = authority(s.expires);
            let ticket = s
                .issuer
                .prepare_issue(
                    caller,
                    &IssueRequest {
                        epoch: 1,
                        operation_id: OP,
                        login_epoch: 1,
                        login_id: OP,
                        browser_public_key: browser_key(),
                        grants: grants(),
                    },
                    IssuerPolicy {
                        login_ttl_ns: 120 * SECOND,
                        proof_ttl_ns: 60 * SECOND,
                        receipt_timeout_ns: 60 * SECOND,
                        reconciliation_retention_ns: 120 * SECOND,
                    },
                    auth,
                    now,
                )
                .unwrap();
            s.issuer.record_nonce(ticket, auth, [4; 16], now).unwrap();
            s.issuer
                .record_approval(ticket, auth, [5; 32], now)
                .unwrap();
            let input = s.issuer.dispatch(ticket, auth, now).unwrap();
            // Local deterministic signing has no external effect before the enclosing message commits.
            let signature = signer().sign(&hash_bytes(input.claims())).to_bytes();
            s.issuer.complete(ticket, auth, signature, now).unwrap();
            s.signatures += 1;
        }
        let Some(IssuerStatus::Completed(value)) = s.issuer.reconcile(caller, 1, OP).unwrap()
        else {
            api::trap("missing fixture result; lookup never issues");
        };
        let IssuedProof::Ed25519(signature) = value.proof else {
            unreachable!()
        };
        PortableCredential {
            claims: ic_wire::Bytes::new(value.claims.to_vec()).unwrap(),
            proof: PortableProof::Ed25519(ic_wire::Bytes::new(signature.to_vec()).unwrap()),
        }
    })
}

fn context<'a>(
    audience: &'a AudienceId,
    grants: &'a AuthorityGrants,
    expires: u64,
) -> VerificationContext<'a> {
    VerificationContext {
        audience,
        grants,
        issuer: api::canister_self(),
        mechanism: PortableMechanism::Ed25519,
        key_binding_id: authority(expires).key_binding_id,
        caller: api::caller(),
        now_ns: api::time(),
        authority_not_before_ns: 0,
        authority_expires_at_ns: expires,
        enrollment_expires_at_ns: expires,
        min_login_started_at_ns: 0,
        min_proof_issued_at_ns: 0,
        max_login_ttl_ns: 120 * SECOND,
        max_proof_ttl_ns: 60 * SECOND,
    }
}

fn decode_admit(bytes: &[u8]) -> Result<SessionInput, ic_wire::Error> {
    EstablishSession::decode_request(bytes)
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform owns the reply value"
)]
fn encode_admit(value: Result<SessionView, AdmissionRefusal>) -> Result<Vec<u8>, ic_wire::Error> {
    EstablishSession::encode_response(&value)
}

/// Admit the browser credential and return its session view.
#[runtime::update(
    runtime = ic_platform_runtime,
    name = "establish_portable_session",
    guard = bounded::<524288>,
    decode = decode_admit,
    encode = encode_admit,
    max_bytes = 524288,
    on_error = crate::canister::endpoint_error,
)]
fn admit(input: SessionInput) -> Result<SessionView, AdmissionRefusal> {
    let proof = input
        .credential
        .try_into()
        .map_err(|_| AdmissionRefusal::InvalidCredential)?;
    let request = SessionRequest::new(
        input
            .scopes
            .into_vec()
            .into_iter()
            .map(ic_wire::Text::into_string)
            .collect(),
        input.ttl_secs,
    )
    .map_err(|_| AdmissionRefusal::ScopeRejected)?;
    STATE.with_borrow_mut(|state| {
        let s = state.as_mut().ok_or(AdmissionRefusal::Unavailable)?;
        let audience = audience();
        let grants = AuthorityGrants::new(grants()).unwrap();
        let context = context(&audience, &grants, s.expires);
        let role = "user".parse().unwrap();
        let key = Ed25519Key::new(signer().verifying_key().to_bytes()).unwrap();
        let value = s
            .receiver
            .admit(
                &proof,
                request,
                AdmissionHost {
                    verifier: VerificationProfile::Ed25519 {
                        context: &context,
                        key: &key,
                    },
                    role: &role,
                    ttl: SessionTtlPolicy::new(60, 1800).unwrap(),
                    generation: 1,
                    enabled: true,
                    subject_admissible: true,
                },
            )
            .map_err(|e| match e {
                MemoryError::Policy(SessionPolicyError::Revoked) => AdmissionRefusal::Revoked,
                MemoryError::Request(_) => AdmissionRefusal::ScopeRejected,
                MemoryError::Claims(_) | MemoryError::Context(_) | MemoryError::Proof(_) => {
                    AdmissionRefusal::InvalidCredential
                }
                _ => AdmissionRefusal::Unavailable,
            })?
            .receipt;
        Ok(SessionView {
            caller: value.caller,
            issuer: value.issuer,
            subject: value.subject,
            expires_at_ns: value.expires_at_ns,
        })
    })
}

fn decode_write(bytes: &[u8]) -> Result<(), ic_wire::Error> {
    ic_wire::decode::<()>(
        bytes,
        ic_wire::Encoding::Frame {
            method: WRITE,
            direction: ic_wire::Direction::Request,
        },
        REQUEST,
    )
}

fn encode_write(value: Result<u32, ()>) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: WRITE,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
}

/// Apply a fixture write under the current caller’s session.
#[runtime::update(
    runtime = ic_platform_runtime,
    name = "browser_fixture_write",
    guard = bounded::<256>,
    decode = decode_write,
    encode = encode_write,
    max_bytes = 256,
    on_error = crate::canister::endpoint_error,
)]
fn write((): ()) -> Result<u32, ()> {
    STATE.with_borrow_mut(|state| {
        let s = state.as_mut().ok_or(())?;
        let audience = audience();
        let grants = AuthorityGrants::new(grants()).unwrap();
        let context = context(&audience, &grants, s.expires);
        let role = "user".parse().unwrap();
        let host = SessionAuthority {
            caller: context.caller,
            now_ns: context.now_ns,
            audience,
            issuer: context.issuer,
            mechanism: context.mechanism,
            key_binding_id: context.key_binding_id,
            role: &role,
            grants: grants.as_slice(),
            generation: 1,
            not_before_ns: 0,
            expires_at_ns: s.expires,
            enrollment_expires_at_ns: s.expires,
            min_login_started_at_ns: 0,
            max_session_ttl_secs: 1800,
            enabled: true,
            subject_admissible: true,
        };
        s.receiver
            .authorize(&host, &["write".parse().unwrap()], |_| {
                s.writes += 1;
                s.writes
            })
            .map_err(|_| ())
    })
}

fn decode_logout(bytes: &[u8]) -> Result<(), ic_wire::Error> {
    Logout::decode_request(bytes)
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "Platform owns the reply value"
)]
fn encode_logout(value: LogoutResult) -> Result<Vec<u8>, ic_wire::Error> {
    Logout::encode_response(&value)
}

/// Revoke the caller’s login and report the resulting logout receipt.
#[runtime::update(
    runtime = ic_platform_runtime,
    name = "logout_portable_session",
    guard = bounded::<11>,
    decode = decode_logout,
    encode = encode_logout,
    max_bytes = 11,
    on_error = crate::canister::endpoint_error,
)]
fn logout((): ()) -> LogoutResult {
    STATE.with_borrow_mut(|state| {
        let Some(s) = state.as_mut() else {
            return LogoutResult::Unavailable(LogoutUnavailable::ResetPending);
        };
        let caller = api::caller();
        // The installation selects the account/login; Auth authenticates the
        // actual browser against its retained binding without account impersonation.
        match s
            .issuer
            .revoke_browser(caller, principal(8), 1, OP, api::time())
        {
            Ok(_) => {}
            Err(IssuerError::Denied) => return LogoutResult::InvalidCaller,
            Err(IssuerError::Clock) => return LogoutResult::Unavailable(LogoutUnavailable::Clock),
            Err(IssuerError::Closed) => {
                return LogoutResult::Unavailable(LogoutUnavailable::ResetPending);
            }
            Err(_) => api::trap("unexpected issuer logout refusal"),
        }
        match s.receiver.logout(caller, api::time()).unwrap() {
            LogoutReceipt::Revoked { deny_until_ns } => LogoutResult::Revoked(deny_until_ns),
            LogoutReceipt::Unconfirmed { deny_until_ns } => {
                LogoutResult::Unconfirmed(deny_until_ns)
            }
        }
    })
}

fn decode_observe(bytes: &[u8]) -> Result<(), ic_wire::Error> {
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
    reason = "Platform owns the reply value"
)]
fn encode_observe(value: Observation) -> Result<Vec<u8>, ic_wire::Error> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: OBSERVE,
            direction: ic_wire::Direction::Response,
        },
        RESPONSE,
    )
}

/// Read the browser fixture’s observable state.
#[runtime::query(
    runtime = ic_platform_runtime,
    name = "browser_fixture_observe",
    guard = bounded::<256>,
    decode = decode_observe,
    encode = encode_observe,
    max_bytes = 256,
    on_error = crate::canister::endpoint_error,
)]
fn observe((): ()) -> Observation {
    STATE.with_borrow(|state| {
        let s = state.as_ref().expect("lost browser fixture state");
        Observation {
            signatures: s.signatures,
            writes: s.writes,
            sessions: s
                .receiver
                .usage((api::canister_self(), principal(8)))
                .sessions as u32,
            logins: s.receiver.known_logins() as u32,
            operations: s.issuer.usage().operations,
        }
    })
}
