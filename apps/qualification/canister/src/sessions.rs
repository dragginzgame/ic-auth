//! Actual volatile receiver lifecycle for qualification, not a production issuer.
mod checkpoint;
#[path = "../../session_metrics.rs"]
mod metrics;
mod profiling;
pub(crate) use checkpoint::{Snapshot, capture, restore};
#[path = "../../sessions.rs"]
pub(crate) mod contract;
use contract::*;
use ic_auth::{
    admission_budget::SessionAdmissionBudget,
    portable::{MemoryLoginRevocations, PortableClaimsContext},
    portable_credential::PortableVerificationContext,
    session::{
        MemorySessionStore, RequiredScopes, Session, SessionAdmissionContext, SessionContext,
        SessionLimits, SessionPolicy, SessionReceipt, SessionStore, authorize_session,
        establish_session, logout_session,
    },
    session_request::{SessionRequest, SessionTtlPolicy},
};
use ic_auth_protocol_types::{
    LogoutResult, LogoutUnavailable, PortableCredential, SessionReadRefusal,
};
use ic_auth_qualification_contracts as controls;
use ic_auth_wire::{SessionInput, SessionView, model::AdmissionRefusal};
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
const CERTIFIED_LIMITS: ic_auth::portable_certified::CertifiedProofLimits =
    ic_auth::portable_certified::CertifiedProofLimits {
        max_cert_ttl_ns: 120 * SECOND,
        max_token_ttl_ns: 60 * SECOND,
        max_certificate_age_ns: 10 * SECOND,
        max_variable_bytes: 131_072,
        max_issuer_signature_bytes: 131_072,
        max_witness_steps: 6,
    };
fn limits() -> SessionLimits {
    SessionLimits::new(
        MAX_SESSIONS,
        MAX_REPLAYS,
        MAX_REPLAYS_PER_CALLER,
        MAX_ENCODED_BYTES,
    )
    .expect("fixture limits")
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum ReceiverLifecycle {
    Active,
    CheckpointEnabled,
    AwaitingAuthority,
}
impl ReceiverLifecycle {
    fn ready(self) -> bool {
        self != Self::AwaitingAuthority
    }
    fn checkpoint(self) -> bool {
        self != Self::Active
    }
}
struct Receiver {
    lifecycle: ReceiverLifecycle,
    inputs: ReceiverInputs,
    enabled: bool,
    store: MemorySessionStore,
    ledger: MemoryLoginRevocations,
    admission: SessionAdmissionBudget,
    unknown: SessionAdmissionBudget,
    metrics: metrics::AdmissionMetrics,
    read_scopes: RequiredScopes,
    write_scopes: RequiredScopes,
}
thread_local! { static RECEIVER: RefCell<Option<Receiver>> = const { RefCell::new(None) }; }
struct ReceiverInputs {
    config: ReceiverConfig,
    login_floor: u64,
    proof_floor: u64,
}
fn ttl() -> SessionTtlPolicy {
    SessionTtlPolicy::new(30, 60).expect("fixture TTL bounds")
}
impl ReceiverInputs {
    fn authority(&self, caller: Principal, now: u64) -> ic_auth::portable::PortableAuthority<'_> {
        ic_auth::portable::PortableAuthority {
            audience: &self.config.audience,
            issuer: self.config.issuer,
            mechanism: self.config.authority.mechanism(),
            key_binding_id: self.config.authority.binding(),
            grants: &self.config.grants,
            caller,
            now_ns: now,
            authority_not_before_ns: self.config.valid_from_ns,
            authority_expires_at_ns: self.config.valid_until_ns,
            enrollment_expires_at_ns: self.config.enrollment_until_ns,
            min_login_started_at_ns: self.login_floor,
        }
    }
    fn claims_context(&self, caller: Principal, now: u64) -> PortableClaimsContext<'_> {
        PortableClaimsContext {
            authority: self.authority(caller, now),
            min_proof_issued_at_ns: self.proof_floor,
            max_login_ttl_ns: 86_400 * SECOND,
            max_proof_ttl_ns: 60 * SECOND,
        }
    }
    fn context(&self, caller: Principal, now: u64) -> SessionContext<'_> {
        SessionContext {
            authority: self.authority(caller, now),
            max_ttl: ttl().maximum(),
            role: &self.config.role,
            required_scopes: RequiredScopes::NONE,
        }
    }
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

impl Receiver {
    // Callers admit installation or restoration independently. This assembly
    // supplies common operations and fresh, empty budgets, never authority.
    fn new(
        inputs: ReceiverInputs,
        lifecycle: ReceiverLifecycle,
        enabled: bool,
        store: MemorySessionStore,
        ledger: MemoryLoginRevocations,
        now: u64,
    ) -> Result<Self, String> {
        let (admission, unknown) = budgets(&inputs.config, now)?;
        Ok(Self {
            inputs,
            lifecycle,
            enabled,
            store,
            ledger,
            admission,
            unknown,
            metrics: Default::default(),
            read_scopes: RequiredScopes::new(vec!["read".into()]).expect("read operation"),
            write_scopes: RequiredScopes::new(vec!["write".into()]).expect("write operation"),
        })
    }
}

fn policy(host: &Receiver) -> SessionPolicy {
    SessionPolicy {
        generation: host.store.generation(),
        enabled: host.enabled,
        subject_admissible: true,
    }
}
fn view(receipt: SessionReceipt) -> SessionView {
    SessionView {
        caller: receipt.caller(),
        issuer: receipt.issuer(),
        subject: receipt.subject(),
        expires_at_ns: receipt.expires_at_ns(),
    }
}
struct ReceiverUnavailable;
impl From<ReceiverUnavailable> for String {
    fn from(_: ReceiverUnavailable) -> Self {
        "receiver not ready".into()
    }
}
impl From<ReceiverUnavailable> for AdmissionRefusal {
    fn from(_: ReceiverUnavailable) -> Self {
        Self::Unavailable
    }
}
impl From<ReceiverUnavailable> for SessionReadRefusal {
    fn from(_: ReceiverUnavailable) -> Self {
        Self::Unavailable
    }
}
fn with_receiver<T, E: From<ReceiverUnavailable>>(
    operation: impl FnOnce(&mut Receiver) -> Result<T, E>,
) -> Result<T, E> {
    RECEIVER.with(|state| {
        let mut state = state.borrow_mut();
        let host = state
            .as_mut()
            .filter(|host| host.lifecycle.ready())
            .ok_or(ReceiverUnavailable)?;
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
    let mut inputs = ReceiverInputs {
        config,
        login_floor: 0,
        proof_floor: 0,
    };
    let mut ctx = inputs.claims_context(Principal::anonymous(), now);
    ctx.advance_reset_floors(0).map_err(|e| e.to_string())?;
    let floors = (
        ctx.authority.min_login_started_at_ns,
        ctx.min_proof_issued_at_ns,
    );
    inputs.login_floor = floors.0;
    inputs.proof_floor = floors.1;
    let ledger = MemoryLoginRevocations::new(
        inputs.config.audience,
        MAX_ADMITTED_LOGINS,
        MAX_UNKNOWN_LOGINS,
        0,
        now,
    )
    .map_err(|e| e.to_string())?;
    let host = Receiver::new(
        inputs,
        ReceiverLifecycle::Active,
        true,
        MemorySessionStore::new(0, limits()),
        ledger,
        now,
    )?;
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
        let ctx = host.inputs.claims_context(
            ic_platform::cdk::api::msg_caller(),
            ic_platform::cdk::api::time(),
        );
        let certified;
        let verification = match &host.inputs.config.authority {
            ReceiverAuthority::Ed25519(key) => PortableVerificationContext::Ed25519 {
                claims: &ctx,
                public_key: key,
            },
            ReceiverAuthority::Certified {
                root, network_key, ..
            } => {
                certified = ic_auth::portable_certified::PortableCertifiedContext {
                    claims: ctx,
                    root_key: root,
                    ic_root_public_key_raw: network_key,
                    limits: CERTIFIED_LIMITS,
                };
                PortableVerificationContext::Certified(&certified)
            }
        };
        let result = establish_session(
            &mut host.store,
            &mut host.ledger,
            &mut host.admission,
            &credential,
            request,
            &SessionAdmissionContext {
                verification,
                role: &host.inputs.config.role,
                ttl: ttl(),
            },
            policy,
        );
        host.metrics.observe(&result);
        result
            .map(|result| view(result.receipt()))
            .map_err(|error| error.admission_refusal().into())
    })
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
fn portable_session_read() -> Result<SessionView, SessionReadRefusal> {
    authorize(SessionOperation::Read)
}

enum SessionOperation {
    Read,
    Write,
}

// Runtime caller, clock, authority and required scopes are host inputs. Ordinary
// operations reuse the retained session; they decode no credential or proof.
fn authorize(operation: SessionOperation) -> Result<SessionView, SessionReadRefusal> {
    with_receiver(|host| {
        let policy = policy(host);
        let mut ctx = host.inputs.context(
            ic_platform::cdk::api::msg_caller(),
            ic_platform::cdk::api::time(),
        );
        ctx.required_scopes = match operation {
            SessionOperation::Read => &host.read_scopes,
            SessionOperation::Write => &host.write_scopes,
        };
        authorize_session(&mut host.store, &mut host.ledger, &ctx, policy, |session| {
            view(session.into())
        })
        .map_err(|e| e.read_refusal())
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
    authorize(SessionOperation::Write).map_err(|e| format!("{e:?}"))?;
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
        let Some(host) = state.as_mut().filter(|host| host.lifecycle.ready()) else {
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
            .read(|tx| {
                tx.session(caller)
                    .map(|session| session.map(std::borrow::Cow::into_owned))
            })
            .map_err(|e| e.to_string())?
            .ok_or("no session")?;
        let encoded = session.encode().map_err(|e| e.to_string())?;
        let start = ic_platform::ic0::performance_counter(0);
        let decoded = Session::decode(&encoded).map_err(|e| e.to_string())?;
        let after_decode = ic_platform::ic0::performance_counter(0);
        if decoded != session {
            return Err("restoration differs".into());
        }
        let ctx = host.inputs.context(caller, ic_platform::cdk::api::time());
        let policy = policy(host);
        let start_authorization = ic_platform::ic0::performance_counter(0);
        authorize_session(&mut host.store, &mut host.ledger, &ctx, policy, |session| {
            view(session.into())
        })
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
fn encode_admission(result: Result<SessionView, AdmissionRefusal>) -> Vec<u8> {
    ic_auth_wire::EstablishSession::encode_response(&result).expect("bounded admission reply")
}
fn encode_read(result: Result<SessionView, SessionReadRefusal>) -> Vec<u8> {
    ic_auth_wire::ReadSession::encode_response(&result.map_err(Into::into))
        .expect("bounded read reply")
}
fn encode_logout(result: LogoutResult) -> Vec<u8> {
    ic_auth_wire::Logout::encode_response(&result.into()).expect("bounded logout reply")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_receiver_refuses_before_any_operation_with_owned_categories() {
        RECEIVER.with(|state| *state.borrow_mut() = None);
        let refusal: Result<(), AdmissionRefusal> = with_receiver(|_| panic!("admission ran"));
        assert_eq!(refusal, Err(AdmissionRefusal::Unavailable));
        let refusal: Result<(), SessionReadRefusal> = with_receiver(|_| panic!("read ran"));
        assert_eq!(refusal, Err(SessionReadRefusal::Unavailable));
        let refusal: Result<(), String> = with_receiver(|_| panic!("control ran"));
        assert_eq!(refusal, Err("receiver not ready".into()));
    }
}
