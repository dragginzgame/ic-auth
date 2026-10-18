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
use ic_auth_wire::{SessionInput, SessionView, wire::Bytes};
use ic_platform::Principal;

use std::cell::RefCell;
const SECOND: u64 = 1_000_000_000;
fn limits() -> SessionLimits {
    SessionLimits {
        max_sessions: 8,
        max_replays: 32,
        max_replays_per_caller: 8,
        max_encoded_bytes: 65_536,
    }
}
struct Receiver {
    checkpoint: bool,
    ready: bool,
    config: ReceiverConfig,
    binding: [u8; 32],
    login_floor: u64,
    proof_floor: u64,
    generation: u64,
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
fn policy(host: &Receiver) -> SessionPolicy {
    SessionPolicy {
        generation: host.generation,
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
    hidden = true,
    guard = "crate::control::guard",
    decode_with = "crate::control::decode::<controls::InstallReceiver>",
    encode_with = "crate::control::encode_result::<controls::InstallReceiver, _>"
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
    let host = Receiver {
        checkpoint: false,
        ready: true,
        store: MemorySessionStore::new(0, limits()).map_err(|e| e.to_string())?,
        ledger: MemoryLoginRevocations::new(config.audience, 8, 8, 0, now)
            .map_err(|e| e.to_string())?,
        admission: SessionAdmissionBudget::new(
            config.verification_burst,
            config.verification_refill_ns,
            now,
        )
        .map_err(|e| e.to_string())?,
        unknown: SessionAdmissionBudget::new(16, 1, now).map_err(|e| e.to_string())?,
        config,
        binding,
        login_floor: floors.0,
        proof_floor: floors.1,
        generation: 0,
        enabled: true,
    };
    RECEIVER.with(|state| *state.borrow_mut() = Some(host));
    Ok(())
}
#[ic_platform::cdk::update(
    hidden = true,
    guard = "admission_guard",
    decode_with = "decode_admission",
    encode_with = "encode_admission"
)]
fn establish_portable_session(input: SessionInput) -> Result<SessionView, String> {
    let credential: PortableCredential = input
        .credential
        .try_into()
        .map_err(|e: ic_auth_wire::wire::Error| e.to_string())?;
    let scopes = input
        .scopes
        .into_vec()
        .into_iter()
        .map(|s| s.into_string())
        .collect();
    let request = SessionRequest::new(scopes, input.ttl_secs).map_err(|e| e.to_string())?;
    with_receiver(|host| {
        let policy = policy(host);
        let ctx = context(
            &host.config,
            host.binding,
            (host.login_floor, host.proof_floor),
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
        )
        .map_err(|e| e.to_string())?;
        Ok(view(match result {
            SessionEstablished::Created(s)
            | SessionEstablished::Replaced(s)
            | SessionEstablished::ExactRetry(s) => s,
        }))
    })
}
#[ic_platform::cdk::update(
    hidden = true,
    guard = "admission_guard",
    decode_with = "crate::control::decode::<controls::ProfileAdmission>",
    encode_with = "crate::control::encode::<controls::ProfileAdmission>"
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
    hidden = true,
    guard = "read_guard",
    decode_with = "decode_read",
    encode_with = "encode_read"
)]
fn portable_session_read() -> Result<SessionView, String> {
    with_receiver(|host| {
        let policy = policy(host);
        let required = ["read".into()];
        let mut ctx = context(
            &host.config,
            host.binding,
            (host.login_floor, host.proof_floor),
            ic_platform::cdk::api::msg_caller(),
            ic_platform::cdk::api::time(),
        );
        ctx.required_scopes = &required;
        authorize_session(&mut host.store, &mut host.ledger, &ctx, policy)
            .map(view)
            .map_err(|e| e.to_string())
    })
}
#[ic_platform::cdk::update(
    hidden = true,
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
    hidden = true,
    guard = "crate::control::guard",
    decode_with = "crate::control::decode::<controls::InvalidateAuthority>",
    encode_with = "crate::control::encode_result::<controls::InvalidateAuthority, _>"
)]
fn invalidate_session_authority(enabled: bool) -> Result<(), String> {
    super::controller()?;
    with_receiver(|host| {
        host.generation = host.store.advance_generation().map_err(|e| e.to_string())?;
        host.enabled = enabled;
        Ok(())
    })
}
#[ic_platform::cdk::query(
    hidden = true,
    guard = "crate::control::guard",
    decode_with = "crate::control::decode::<controls::ProfileSession>",
    encode_with = "crate::control::encode_result::<controls::ProfileSession, _>"
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
        let ctx = context(
            &host.config,
            host.binding,
            (host.login_floor, host.proof_floor),
            caller,
            ic_platform::cdk::api::time(),
        );
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
fn argument_guard<M: ic_auth_wire::wire::Method>() -> Result<(), String> {
    if ic_platform::ic0::msg_arg_data_size() > M::REQUEST_LIMITS.bytes {
        Err("wire argument too large".into())
    } else {
        Ok(())
    }
}
fn admission_guard() -> Result<(), String> {
    argument_guard::<ic_auth_wire::EstablishSession>()
}
fn read_guard() -> Result<(), String> {
    argument_guard::<ic_auth_wire::ReadSession>()
}
fn logout_guard() -> Result<(), String> {
    argument_guard::<ic_auth_wire::Logout>()
}
fn decode_admission(bytes: Vec<u8>) -> SessionInput {
    ic_auth_wire::wire::decode_request::<ic_auth_wire::EstablishSession>(&bytes)
        .expect("bounded admission request")
}

fn decode_read(bytes: Vec<u8>) {
    ic_auth_wire::wire::decode_request::<ic_auth_wire::ReadSession>(&bytes)
        .expect("bounded read request");
}
fn decode_logout(bytes: Vec<u8>) {
    ic_auth_wire::wire::decode_request::<ic_auth_wire::Logout>(&bytes)
        .expect("bounded logout request");
}
fn wire_result(
    result: Result<SessionView, String>,
) -> Result<ic_auth_wire::SessionView, ic_auth_wire::wire::Text<256>> {
    result.map_err(controls::refusal)
}
fn encode_admission(result: Result<SessionView, String>) -> Vec<u8> {
    ic_auth_wire::wire::encode_response::<ic_auth_wire::EstablishSession>(&wire_result(result))
        .expect("bounded admission reply")
        .into_bytes()
}
fn encode_read(result: Result<SessionView, String>) -> Vec<u8> {
    ic_auth_wire::wire::encode_response::<ic_auth_wire::ReadSession>(&wire_result(result))
        .expect("bounded read reply")
        .into_bytes()
}
fn encode_logout(result: LogoutResult) -> Vec<u8> {
    ic_auth_wire::wire::encode_response::<ic_auth_wire::Logout>(&result.into())
        .expect("bounded logout reply")
        .into_bytes()
}

#[cfg(test)]
mod wire_tests {
    #[test]
    fn long_refusals_remain_errors_without_trapping_reply_encoding() {
        for error in ["capacity".to_owned(), "🚫".repeat(100)] {
            let result = super::wire_result(Err(error.clone()));
            assert!(result.as_ref().unwrap_err().as_str().len() <= 256);
            let bytes =
                ic_auth_wire::wire::encode_response::<ic_auth_wire::EstablishSession>(&result)
                    .unwrap();
            let decoded = ic_auth_wire::wire::decode_response::<ic_auth_wire::EstablishSession>(
                bytes.as_bytes(),
            )
            .unwrap();
            assert!(error.starts_with(decoded.unwrap_err().as_str()));
        }
    }
}
