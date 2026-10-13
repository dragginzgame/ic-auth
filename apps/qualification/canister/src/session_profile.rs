//! Read-only SDK cost probe. Controller-installed context and trusted records;
//! no stable backend, issuance or application resource authorization.
use super::{controller, session_profile_contract::*};
use ic_auth::{
    session::{
        Session, SessionError, SessionLimits, SessionOccupancy, SessionPolicy, SessionStore,
        SessionTransaction, authorize_session,
    },
    session_request::SessionRequest,
    token::{RootKeyPolicy, TokenVerificationContext, TokenVerificationLimits},
};
use ic_auth_protocol_types::{ChainKeyAlgorithm, ChainKeyKeyId, Principal};
use std::cell::RefCell;

thread_local! {
    static PROFILE: RefCell<Option<SessionProfileInput>> = const { RefCell::new(None) };
}

#[ic_cdk::update]
fn install_session_profile(input: SessionProfileInput) -> Result<(), String> {
    controller()?;
    if input
        .record
        .as_ref()
        .is_some_and(|bytes| bytes.len() > 2048)
        || input.public_key.len() != 33
        || input.root == Principal::anonymous()
        || input.valid_from_ns >= input.accept_until_ns
    {
        return Err("invalid bounded profile inputs".into());
    }
    let scopes = SessionRequest::new(input.allowed_scopes.clone(), None)
        .map_err(|error| error.to_string())?;
    if scopes.scopes() != input.allowed_scopes {
        return Err("profile ceiling must be canonical".into());
    }
    SessionRequest::new(vec![input.required_scope.clone()], None)
        .map_err(|error| error.to_string())?;
    PROFILE.with(|profile| *profile.borrow_mut() = Some(input));
    Ok(())
}

struct ReadOnlyStore {
    generation: u64,
    session: Option<Session>,
}
impl SessionStore for ReadOnlyStore {
    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut dyn SessionTransaction) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        operation(self)
    }
}
impl SessionTransaction for ReadOnlyStore {
    fn generation(&self) -> u64 {
        self.generation
    }
    fn session(&self, caller: Principal) -> Result<Option<Session>, SessionError> {
        Ok(self
            .session
            .as_ref()
            .filter(|session| session.caller() == caller)
            .cloned())
    }
    fn limits(&self) -> SessionLimits {
        panic!("authorization must not read admission limits")
    }
    fn replay_exists(&self, _: [u8; 32]) -> Result<bool, SessionError> {
        panic!("authorization must not read replay admission")
    }
    fn occupancy(&self, _: Principal) -> Result<SessionOccupancy, SessionError> {
        panic!("authorization must not read physical occupancy")
    }
    fn stage(&mut self, _: Session, _: ic_auth::session::Replay) -> Result<(), SessionError> {
        panic!("authorization must not stage writes")
    }
}

#[ic_cdk::query]
fn profile_session() -> Result<SessionProfileResult, String> {
    PROFILE.with(|profile| {
        let profile = profile.borrow();
        let input = profile.as_ref().ok_or("profile not installed")?;
        // Fixed fixture enrollment is installed separately from restored bytes.
        let root = RootKeyPolicy {
            root_canister_id: input.root,
            algorithm: ChainKeyAlgorithm::EcdsaSecp256k1,
            key_id: ChainKeyKeyId {
                name: "test_key_1".into(),
            },
            derivation_path_hash: input.path_hash,
            public_key: input.public_key.clone(),
            key_version: 4,
            min_accepted_key_version: 4,
            min_accepted_proof_epoch: 2,
            min_accepted_registry_epoch: 3,
            valid_from_ns: input.valid_from_ns,
            accept_until_ns: input.accept_until_ns,
            max_revocation_latency_ns: 120_000_000_000,
        };
        let required = [input.required_scope.clone()];
        let context = TokenVerificationContext {
            caller: ic_cdk::api::msg_caller(),
            audience: input.audience,
            role: &input.role,
            allowed_scopes: &input.allowed_scopes,
            required_scopes: &required,
            root_key: &root,
            ic_root_public_key_raw: &[],
            now_ns: ic_cdk::api::time(),
            limits: TokenVerificationLimits {
                max_cert_ttl_ns: 120_000_000_000,
                max_token_ttl_ns: 60_000_000_000,
                max_future_skew_ns: 0,
                max_certificate_age_ns: 10_000_000_000,
                max_variable_bytes: 256 * 1024,
                max_issuer_signature_bytes: 128 * 1024,
                max_witness_steps: 6,
            },
        };
        let start = ic_cdk::api::performance_counter(0);
        let session = input
            .record
            .as_deref()
            .map(Session::decode)
            .transpose()
            .map_err(|error| error.to_string())?;
        let decoded = ic_cdk::api::performance_counter(0);
        let mut store = ReadOnlyStore {
            generation: input.generation,
            session,
        };
        let policy = SessionPolicy {
            generation: input.generation,
            enabled: input.enabled,
            subject_admissible: true,
            default_ttl_secs: 60,
            max_ttl_secs: 60,
            max_proof_ttl_ns: 60_000_000_000,
        };
        let authorization_start = ic_cdk::api::performance_counter(0);
        let decision = authorize_session(&mut store, &context, policy);
        let end = ic_cdk::api::performance_counter(0);
        let decision = match decision {
            Ok(_) => "allow",
            Err(SessionError::StaleAuthority) => "stale_authority",
            Err(SessionError::MissingOrExpired) => "missing_or_expired",
            Err(SessionError::ScopeRejected) => "scope_rejected",
            Err(SessionError::Disabled) => "disabled",
            Err(_) => "other_error",
        };
        Ok(SessionProfileResult {
            record_bytes: input.record.as_ref().map_or(0, |bytes| bytes.len()) as u64,
            decode_instructions: decoded - start,
            authorization_instructions: end - authorization_start,
            total_instructions: end - start,
            decision: decision.into(),
        })
    })
}
