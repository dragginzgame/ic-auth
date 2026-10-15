//! Actual volatile receiver lifecycle for qualification, not a production issuer.
mod checkpoint;
pub(crate) use checkpoint::{Snapshot, capture, restore};
#[path = "../../sessions.rs"]
pub(crate) mod contract;
use candid::Principal;
use contract::*;
use ic_auth::{
    admission_budget::SessionAdmissionBudget,
    certificate_issuance::validate_grants,
    portable::{MemoryLoginRevocations, PortableClaimsContext},
    portable_credential::{PortableVerificationContext, verify_credential},
    session::{
        MemorySessionStore, Session, SessionContext, SessionEstablished, SessionLimits,
        SessionPolicy, SessionStore, authorize_session, establish_session, logout_session,
    },
    session_request::SessionRequest,
};
use ic_auth_protocol_types::{LogoutRequest, LogoutResult, LogoutUnavailable, PortableMechanism};
use sha2::{Digest, Sha256};
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
            mechanism: PortableMechanism::Ed25519,
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
        caller: session.caller(),
        issuer: session.issuer(),
        subject: session.subject(),
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
#[ic_cdk::update]
fn install_session_receiver(config: ReceiverConfig) -> Result<(), String> {
    super::controller()?;
    validate_grants(&config.grants, 16, 32).map_err(|e| e.to_string())?;
    let now = ic_cdk::api::time();
    if config.issuer == Principal::anonymous()
        || config.valid_from_ns > now
        || config.valid_until_ns <= now
        || config.enrollment_until_ns <= now
    {
        return Err("invalid receiver authority".into());
    }
    let binding = Sha256::digest(config.public_key).into();
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
        admission: SessionAdmissionBudget::new(16, 1, now).map_err(|e| e.to_string())?,
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
#[ic_cdk::update]
fn establish_portable_session(input: SessionInput) -> Result<SessionView, String> {
    let request = SessionRequest::new(input.scopes, input.ttl_secs).map_err(|e| e.to_string())?;
    with_receiver(|host| {
        let policy = policy(host);
        let ctx = context(
            &host.config,
            host.binding,
            (host.login_floor, host.proof_floor),
            ic_cdk::api::msg_caller(),
            ic_cdk::api::time(),
        );
        let result = establish_session(
            &mut host.store,
            &mut host.ledger,
            &mut host.admission,
            &input.credential.claims,
            &request,
            &ctx,
            policy,
            |budget| {
                Ok(verify_credential(
                    budget,
                    &input.credential,
                    &PortableVerificationContext::Ed25519 {
                        claims: &ctx.authority,
                        public_key: &host.config.public_key,
                    },
                )?)
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
#[ic_cdk::update]
fn portable_session_read() -> Result<SessionView, String> {
    with_receiver(|host| {
        let policy = policy(host);
        let required = ["read".into()];
        let mut ctx = context(
            &host.config,
            host.binding,
            (host.login_floor, host.proof_floor),
            ic_cdk::api::msg_caller(),
            ic_cdk::api::time(),
        );
        ctx.required_scopes = &required;
        authorize_session(&mut host.store, &mut host.ledger, &ctx, policy)
            .map(view)
            .map_err(|e| e.to_string())
    })
}
#[ic_cdk::update]
fn logout_portable_session(_request: LogoutRequest) -> LogoutResult {
    RECEIVER.with(|state| {
        let mut state = state.borrow_mut();
        let Some(host) = state.as_mut().filter(|host| host.ready) else {
            return LogoutResult::Unavailable(LogoutUnavailable::ResetPending);
        };
        logout_session(
            &mut host.store,
            &mut host.ledger,
            &mut host.unknown,
            ic_cdk::api::msg_caller(),
            ic_cdk::api::time(),
        )
    })
}
#[ic_cdk::update]
fn invalidate_session_authority(enabled: bool) -> Result<(), String> {
    super::controller()?;
    with_receiver(|host| {
        host.generation = host.store.advance_generation().map_err(|e| e.to_string())?;
        host.enabled = enabled;
        Ok(())
    })
}
#[ic_cdk::query]
fn profile_portable_session() -> Result<SessionProfile, String> {
    with_receiver(|host| {
        let caller = ic_cdk::api::msg_caller();
        let session = host
            .store
            .transaction(None, |tx| tx.session(caller))
            .map_err(|e| e.to_string())?
            .ok_or("no session")?;
        let encoded = session.encode().map_err(|e| e.to_string())?;
        let start = ic_cdk::api::performance_counter(0);
        let decoded = Session::decode(&encoded).map_err(|e| e.to_string())?;
        let after_decode = ic_cdk::api::performance_counter(0);
        if decoded != session {
            return Err("restoration differs".into());
        }
        let ctx = context(
            &host.config,
            host.binding,
            (host.login_floor, host.proof_floor),
            caller,
            ic_cdk::api::time(),
        );
        let policy = policy(host);
        let start_authorization = ic_cdk::api::performance_counter(0);
        authorize_session(&mut host.store, &mut host.ledger, &ctx, policy)
            .map_err(|e| e.to_string())?;
        Ok(SessionProfile {
            record_bytes: encoded.len() as u64,
            decode_instructions: after_decode - start,
            authorization_instructions: ic_cdk::api::performance_counter(0) - start_authorization,
        })
    })
}
