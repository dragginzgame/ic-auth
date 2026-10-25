//! Actual volatile receiver lifecycle for qualification, not a production issuer.
mod checkpoint;
pub(crate) use checkpoint::{Snapshot, capture, restore};
#[path = "../../sessions.rs"]
pub(crate) mod contract;
use contract::*;
use ic_auth::{
    admission_budget::SessionAdmissionBudget,
    certificate_issuance::validate_grants,
    portable::{MemoryLoginRevocations, PortableClaimsContext},
    portable_credential::PortableVerificationContext,
    session::{
        MemorySessionStore, Session, SessionContext, SessionEstablished, SessionLimits,
        SessionPolicy, SessionStore, authorize_session, establish_session, logout_session,
    },
    session_request::SessionRequest,
};
use ic_auth_protocol_types::{LogoutResult, LogoutUnavailable, PortableCredential};
use ic_auth_qualification_contracts as controls;
use ic_auth_wire::{SessionInput, SessionView, model::AdmissionRefusal, wire::Bytes};
use ic_platform::Principal;

use std::cell::RefCell;
const SECOND: u64 = 1_000_000_000;
const MAX_SESSIONS: usize = 8;
const MAX_REPLAYS: usize = 32;
const MAX_REPLAYS_PER_CALLER: usize = 8;
const MAX_ENCODED_BYTES: usize = 65_536;
const MAX_ADMITTED_LOGINS: usize = 8;
const MAX_UNKNOWN_LOGINS: usize = 8;
const MAX_LOGIN_RECORDS: usize = MAX_ADMITTED_LOGINS + MAX_UNKNOWN_LOGINS;
const MAX_SMALL_RECORD_BYTES: usize = 1024;
fn limits() -> SessionLimits {
    SessionLimits {
        max_sessions: MAX_SESSIONS,
        max_replays: MAX_REPLAYS,
        max_replays_per_caller: MAX_REPLAYS_PER_CALLER,
        max_encoded_bytes: MAX_ENCODED_BYTES,
    }
}
struct Receiver {
    checkpoint: bool,
    ready: bool,
    config: ReceiverConfig,
    binding: [u8; 32],
    login_floor: u64,
    proof_floor: u64,
    enabled: bool,
    store: MemorySessionStore,
    ledger: MemoryLoginRevocations,
    admission: SessionAdmissionBudget,
    unknown: SessionAdmissionBudget,
}
thread_local! { static RECEIVER: RefCell<Option<Receiver>> = const { RefCell::new(None) }; }
fn context(
    config: &ReceiverConfig,
    binding: [u8; 32],
    floors: (u64, u64),
    caller: Principal,
    now: u64,
) -> SessionContext<'_> {
    SessionContext {
        authority: PortableClaimsContext {
            audience: &config.audience,
            issuer: config.issuer,
            mechanism: config.authority.mechanism(),
            key_binding_id: binding,
            grants: &config.grants,
            caller,
            now_ns: now,
            authority_not_before_ns: config.valid_from_ns,
            authority_expires_at_ns: config.valid_until_ns,
            enrollment_expires_at_ns: config.enrollment_until_ns,
            min_login_started_at_ns: floors.0,
            min_proof_issued_at_ns: floors.1,
            max_login_ttl_ns: 86_400 * SECOND,
            max_proof_ttl_ns: 60 * SECOND,
        },
        role: &config.role,
        required_scopes: &[],
    }
}
// Expand field borrows separately so context can coexist with mutable store and
// ledger borrows. An &Receiver accessor would borrow the entire host.
macro_rules! receiver_context {
    ($host:ident, $caller:expr, $now:expr $(,)?) => {
        context(
            &$host.config,
            $host.binding,
            ($host.login_floor, $host.proof_floor),
            $caller,
            $now,
        )
    };
}

// Both installation and checkpoint restoration start with empty budgets.
fn budgets(
    config: &ReceiverConfig,
    now: u64,
) -> Result<(SessionAdmissionBudget, SessionAdmissionBudget), String> {
    Ok((
        SessionAdmissionBudget::new(
            config.verification_burst,
            config.verification_refill_ns,
            now,
        )
        .map_err(|e| e.to_string())?,
        SessionAdmissionBudget::new(16, 1, now).map_err(|e| e.to_string())?,
    ))
}

fn policy(host: &Receiver) -> SessionPolicy {
    SessionPolicy {
        generation: host.store.generation(),
        enabled: host.enabled,
        subject_admissible: true,
        default_ttl_secs: 30,
        max_ttl_secs: 60,
    }
}
fn view(session: Session) -> SessionView {
    SessionView {
        caller: Bytes::new(session.caller().as_slice().to_vec()).expect("principal bound"),
        issuer: Bytes::new(session.issuer().as_slice().to_vec()).expect("principal bound"),
        subject: Bytes::new(session.subject().as_slice().to_vec()).expect("principal bound"),
        expires_at_ns: session.expires_at_ns(),
    }
}
fn with_receiver<T>(
    operation: impl FnOnce(&mut Receiver) -> Result<T, String>,
) -> Result<T, String> {
    RECEIVER.with(|state| {
        let mut state = state.borrow_mut();
        let host = state
            .as_mut()
            .filter(|host| host.ready)
            .ok_or("receiver not ready")?;
        operation(host)
    })
}
#[ic_platform::cdk::update(
    guard = "crate::control::guard",
    decode_with = "crate::control::InstallReceiver::decode",
    encode_with = "crate::control::InstallReceiver::encode"
)]
fn install_session_receiver(config: controls::model::ReceiverConfig) -> Result<(), String> {
    super::controller()?;
    let config: ReceiverConfig = config
        .try_into()
        .map_err(|e: ic_wire::Error| e.to_string())?;
    validate_grants(&config.grants, 16, 32).map_err(|e| e.to_string())?;
    let now = ic_platform::cdk::api::time();
    if config.issuer == Principal::anonymous()
        || config.valid_from_ns > now
        || config.valid_until_ns <= now
        || config.enrollment_until_ns <= now
    {
        return Err("invalid receiver authority".into());
    }
    if let ReceiverAuthority::Certified {
        root, network_key, ..
    } = &config.authority
        && (network_key.len() != 96
            || root.root_canister_id == Principal::anonymous()
            || config.valid_until_ns > root.accept_until_ns)
    {
        return Err("invalid certified receiver authority".into());
    }
    let binding = config.authority.binding();
    let mut ctx = context(&config, binding, (0, 0), Principal::anonymous(), now);
    ctx.authority
        .advance_reset_floors(0)
        .map_err(|e| e.to_string())?;
    let floors = (
        ctx.authority.min_login_started_at_ns,
        ctx.authority.min_proof_issued_at_ns,
    );
    let (admission, unknown) = budgets(&config, now)?;
    let host = Receiver {
        checkpoint: false,
        ready: true,
        store: MemorySessionStore::new(0, limits()).map_err(|e| e.to_string())?,
        ledger: MemoryLoginRevocations::new(
            config.audience,
            MAX_ADMITTED_LOGINS,
            MAX_UNKNOWN_LOGINS,
            0,
            now,
        )
        .map_err(|e| e.to_string())?,
        admission,
        unknown,
        config,
        binding,
        login_floor: floors.0,
        proof_floor: floors.1,
        enabled: true,
    };
    RECEIVER.with(|state| *state.borrow_mut() = Some(host));
    Ok(())
}
#[ic_platform::cdk::update(
    guard = "admission_guard",
    decode_with = "decode_admission",
    encode_with = "encode_admission"
)]
fn establish_portable_session(input: SessionInput) -> Result<SessionView, AdmissionRefusal> {
    let credential: PortableCredential = input
        .credential
        .try_into()
        .map_err(|_: ic_auth_wire::wire::Error| AdmissionRefusal::InvalidCredential)?;
    let scopes = input
        .scopes
        .into_vec()
        .into_iter()
        .map(|s| s.into_string())
        .collect();
    let request = SessionRequest::new(scopes, input.ttl_secs)
        .map_err(|_| AdmissionRefusal::InvalidCredential)?;
    with_receiver(|host| {
        let policy = policy(host);
        let ctx = receiver_context!(
            host,
            ic_platform::cdk::api::msg_caller(),
            ic_platform::cdk::api::time(),
        );
        let result = establish_session(
            &mut host.store,
            &mut host.ledger,
            &mut host.admission,
            &credential.claims,
            &request,
            &ctx,
            policy,
            |budget, decoded| match &host.config.authority {
                ReceiverAuthority::Ed25519(key) => Ok(decoded.verify(
                    budget,
                    &credential.proof,
                    &PortableVerificationContext::Ed25519 {
                        claims: &ctx.authority,
                        public_key: key,
                    },
                )?),
                ReceiverAuthority::Certified {
                    root, network_key, ..
                } => {
                    let root = root.as_ref();
                    let verification = ic_auth::portable_certified::PortableCertifiedContext {
                        claims: ctx.authority,
                        root_key: root,
                        ic_root_public_key_raw: network_key,
                        limits: ic_auth::portable_certified::CertifiedProofLimits {
                            max_cert_ttl_ns: 120 * SECOND,
                            max_token_ttl_ns: 60 * SECOND,
                            max_future_skew_ns: 0,
                            max_certificate_age_ns: 10 * SECOND,
                            max_variable_bytes: 131_072,
                            max_issuer_signature_bytes: 131_072,
                            max_witness_steps: 6,
                        },
                    };
                    Ok(decoded.verify(
                        budget,
                        &credential.proof,
                        &PortableVerificationContext::Certified(&verification),
                    )?)
                }
            },
        );
        Ok(result
            .map(|result| {
                view(match result {
                    SessionEstablished::Created(s)
                    | SessionEstablished::Replaced(s)
                    | SessionEstablished::ExactRetry(s) => s,
                })
            })
            .map_err(|error| error.admission_refusal().into()))
    })
    .map_err(|_| AdmissionRefusal::Unavailable)?
}
#[ic_platform::cdk::update(
    guard = "admission_guard",
    decode_with = "crate::control::ProfileAdmission::decode",
    encode_with = "crate::control::ProfileAdmission::encode"
)]
fn profile_portable_admission(input: SessionInput) -> AdmissionProfile {
    let start = ic_platform::ic0::performance_counter(0);
    let result = establish_portable_session(input);
    let instructions = ic_platform::ic0::performance_counter(0) - start;
    AdmissionProfile {
        reply: ic_wire::Bytes::new(encode_admission(result)).expect("bounded admission diagnostic"),
        instructions,
    }
}

#[ic_platform::cdk::update(
    guard = "read_guard",
    decode_with = "decode_read",
    encode_with = "encode_read"
)]
fn portable_session_read() -> Result<SessionView, String> {
    authorize(&["read".into()]).map(view)
}

// Runtime caller, clock, authority and required scopes are host inputs. Ordinary
// operations reuse the retained session; they decode no credential or proof.
fn authorize(required_scopes: &[String]) -> Result<Session, String> {
    with_receiver(|host| {
        let policy = policy(host);
        let mut ctx = receiver_context!(
            host,
            ic_platform::cdk::api::msg_caller(),
            ic_platform::cdk::api::time(),
        );
        ctx.required_scopes = required_scopes;
        authorize_session(&mut host.store, &mut host.ledger, &ctx, policy)
            .map_err(|e| e.to_string())
    })
}

// One fixture application asset is writable by the protected "write" scope.
// Real application resource/account policy remains the consuming host's job.
#[ic_platform::cdk::update(
    guard = "crate::control::guard",
    decode_with = "crate::control::SetSessionAsset::decode",
    encode_with = "crate::control::SetSessionAsset::encode"
)]
fn session_set_asset(version: u64) -> Result<(), String> {
    authorize(&["write".into()])?;
    // No await separates the session decision from the application effect.
    super::with_state(|state| {
        state.metadata.asset_version = version;
        state.publish();
    });
    Ok(())
}
#[ic_platform::cdk::update(
    guard = "logout_guard",
    decode_with = "decode_logout",
    encode_with = "encode_logout"
)]
fn logout_portable_session() -> LogoutResult {
    RECEIVER.with(|state| {
        let mut state = state.borrow_mut();
        let Some(host) = state.as_mut().filter(|host| host.ready) else {
            return LogoutResult::Unavailable(LogoutUnavailable::ResetPending);
        };
        logout_session(
            &mut host.store,
            &mut host.ledger,
            &mut host.unknown,
            ic_platform::cdk::api::msg_caller(),
            ic_platform::cdk::api::time(),
        )
    })
}
#[ic_platform::cdk::update(
    guard = "crate::control::guard",
    decode_with = "crate::control::InvalidateAuthority::decode",
    encode_with = "crate::control::InvalidateAuthority::encode"
)]
fn invalidate_session_authority(enabled: bool) -> Result<(), String> {
    super::controller()?;
    with_receiver(|host| {
        host.store.advance_generation().map_err(|e| e.to_string())?;
        host.enabled = enabled;
        Ok(())
    })
}
#[ic_platform::cdk::query(
    guard = "crate::control::guard",
    decode_with = "crate::control::ProfileSession::decode",
    encode_with = "crate::control::ProfileSession::encode"
)]
fn profile_portable_session() -> Result<SessionProfile, String> {
    with_receiver(|host| {
        let caller = ic_platform::cdk::api::msg_caller();
        let session = host
            .store
            .transaction(None, |tx| tx.session(caller))
            .map_err(|e| e.to_string())?
            .ok_or("no session")?;
        let encoded = session.encode().map_err(|e| e.to_string())?;
        let start = ic_platform::ic0::performance_counter(0);
        let decoded = Session::decode(&encoded).map_err(|e| e.to_string())?;
        let after_decode = ic_platform::ic0::performance_counter(0);
        if decoded != session {
            return Err("restoration differs".into());
        }
        let ctx = receiver_context!(host, caller, ic_platform::cdk::api::time(),);
        let policy = policy(host);
        let start_authorization = ic_platform::ic0::performance_counter(0);
        authorize_session(&mut host.store, &mut host.ledger, &ctx, policy)
            .map_err(|e| e.to_string())?;
        Ok(SessionProfile {
            record_bytes: encoded.len() as u64,
            decode_instructions: after_decode - start,
            authorization_instructions: ic_platform::ic0::performance_counter(0)
                - start_authorization,
        })
    })
}

// Fixture exports bind the shared contract explicitly. Admit raw length before
// CDK copying; malformed input traps before the stateful handler. Failed reply
// encoding traps as well, rolling back local session/revocation mutations.
fn argument_guard(maximum: usize) -> Result<(), String> {
    if ic_platform::ic0::msg_arg_data_size() > maximum {
        Err("wire argument too large".into())
    } else {
        Ok(())
    }
}
fn admission_guard() -> Result<(), String> {
    argument_guard(ic_auth_wire::EstablishSession::REQUEST_LIMITS.bytes)
}
fn read_guard() -> Result<(), String> {
    argument_guard(ic_auth_wire::ReadSession::REQUEST_LIMITS.bytes)
}
fn logout_guard() -> Result<(), String> {
    argument_guard(ic_auth_wire::Logout::REQUEST_LIMITS.bytes)
}
fn decode_admission(bytes: Vec<u8>) -> SessionInput {
    ic_auth_wire::EstablishSession::decode_request(&bytes).expect("bounded admission request")
}

fn decode_read(bytes: Vec<u8>) {
    ic_auth_wire::ReadSession::decode_request(&bytes).expect("bounded read request");
}
fn decode_logout(bytes: Vec<u8>) {
    ic_auth_wire::Logout::decode_request(&bytes).expect("bounded logout request");
}
fn wire_result(
    result: Result<SessionView, String>,
) -> Result<ic_auth_wire::SessionView, ic_auth_wire::wire::Text<256>> {
    result.map_err(controls::refusal)
}
fn encode_admission(result: Result<SessionView, AdmissionRefusal>) -> Vec<u8> {
    ic_auth_wire::EstablishSession::encode_response(&result).expect("bounded admission reply")
}
fn encode_read(result: Result<SessionView, String>) -> Vec<u8> {
    ic_auth_wire::ReadSession::encode_response(&wire_result(result)).expect("bounded read reply")
}
fn encode_logout(result: LogoutResult) -> Vec<u8> {
    ic_auth_wire::Logout::encode_response(&result.into()).expect("bounded logout reply")
}

#[cfg(test)]
mod wire_tests {
    #[test]
    fn long_refusals_remain_errors_without_trapping_reply_encoding() {
        for error in ["capacity".to_owned(), "🚫".repeat(100)] {
            let result = super::wire_result(Err(error.clone()));
            assert!(result.as_ref().unwrap_err().as_str().len() <= 256);
            let bytes = ic_auth_wire::ReadSession::encode_response(&result).unwrap();
            let decoded = ic_auth_wire::ReadSession::decode_response(bytes.as_slice()).unwrap();
            assert!(error.starts_with(decoded.unwrap_err().as_str()));
        }
    }
}
