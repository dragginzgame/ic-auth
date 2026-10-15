use super::{SessionContext, SessionError, SessionOccupancy};
#[cfg(feature = "session-restore")]
use crate::session_request::check_scopes;
use crate::{portable::VerifiedPortableClaims, session_request::SessionRequest};
use ic_auth_protocol_types::{AudienceId, AuthRole, PortableMechanism, Principal};
use serde::{Deserialize, Serialize};

/// Protected admission policy. Hosts must advance generation atomically with
/// authority changes, including disable/re-enable and issuer revocation.
#[derive(Clone, Copy, Debug)]
pub struct SessionPolicy {
    /// Current host-owned generation, equal to the transactional store's value.
    pub generation: u64,
    /// Protected capability switch; re-enabling requires a new generation.
    pub enabled: bool,
    /// Protected local subject eligibility, independent of resource ownership.
    pub subject_admissible: bool,
    /// Default request TTL and upper bound, in seconds; maximum 1800 seconds.
    pub default_ttl_secs: u64,
    pub max_ttl_secs: u64,
}
impl SessionPolicy {
    pub(super) fn validate(self) -> Result<(), SessionError> {
        crate::session_request::validate_ttl_policy(self.default_ttl_secs, self.max_ttl_secs)?;
        Ok(())
    }
}

/// Host storage bounds, never selected by submitted tokens. Physical records
/// and their encoded bytes count until pruning; bytes exclude allocator/index overhead.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionLimits {
    pub max_sessions: usize,
    pub max_replays: usize,
    pub max_replays_per_caller: usize,
    pub max_encoded_bytes: usize,
}
impl SessionLimits {
    pub(super) fn validate(self) -> Result<(), SessionError> {
        if self.max_sessions == 0
            || self.max_sessions > 2048
            || self.max_replays == 0
            || self.max_replays > 4096
            || self.max_replays_per_caller == 0
            || self.max_replays_per_caller > 256
            || self.max_encoded_bytes == 0
            || self.max_encoded_bytes > 8 * 1024 * 1024
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        Ok(())
    }

    pub(super) fn check_capacity(
        self,
        occupancy: SessionOccupancy,
        replacing: bool,
    ) -> Result<(), SessionError> {
        if (!replacing && occupancy.sessions >= self.max_sessions)
            || occupancy.replays >= self.max_replays
            || occupancy.replays_for_caller >= self.max_replays_per_caller
        {
            return Err(SessionError::Capacity);
        }
        Ok(())
    }
}

const RECORD_DOMAIN: &[u8] = b"IC-AUTH-PORTABLE-SESSION\0";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "session-restore", derive(Deserialize))]
#[serde(deny_unknown_fields)]
struct SessionData {
    caller: Principal,
    subject: Principal,
    issuer: Principal,
    audience: AudienceId,
    mechanism: PortableMechanism,
    key_binding_id: [u8; 32],
    role: AuthRole,
    scopes: Vec<String>,
    generation: u64,
    login_started_at_ns: u64,
    login_deadline_ns: u64,
    established_at_ns: u64,
    expires_at_ns: u64,
    fingerprint: [u8; 32],
    request_hash: [u8; 32],
}

/// A checked host-owned portable session, never an ingress credential.
/// Account identity is `(issuer(), subject())`; `caller()` identifies this login.
/// Restoration requires trusted storage and live authorization after decoding.
#[cfg_attr(
    not(feature = "session-restore"),
    doc = r#"
```compile_fail,E0599
use ic_auth::session::Session;
let _ = Session::decode(&[]);
```
"#
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session(SessionData);
impl Session {
    pub fn caller(&self) -> Principal {
        self.0.caller
    }
    pub fn subject(&self) -> Principal {
        self.0.subject
    }
    pub fn issuer(&self) -> Principal {
        self.0.issuer
    }
    pub fn scopes(&self) -> &[String] {
        &self.0.scopes
    }
    pub fn generation(&self) -> u64 {
        self.0.generation
    }
    pub fn established_at_ns(&self) -> u64 {
        self.0.established_at_ns
    }
    pub fn expires_at_ns(&self) -> u64 {
        self.0.expires_at_ns
    }
    pub fn fingerprint(&self) -> [u8; 32] {
        self.0.fingerprint
    }
    pub fn request_hash(&self) -> [u8; 32] {
        self.0.request_hash
    }

    /// Bounded current record with a domain distinct from the retired token record.
    pub fn encode(&self) -> Result<Vec<u8>, SessionError> {
        let mut bytes = RECORD_DOMAIN.to_vec();
        bytes.extend(serde_cbor::to_vec(&self.0).map_err(|_| SessionError::InvalidPolicyOrRecord)?);
        if bytes.len() > 2048 {
            return Err(SessionError::ByteCapacity);
        }
        Ok(bytes)
    }
    /// Trusted-storage restoration only. Old token-session bytes are rejected.
    #[cfg(feature = "session-restore")]
    pub fn decode(bytes: &[u8]) -> Result<Self, SessionError> {
        if bytes.len() > 2048 {
            return Err(SessionError::ByteCapacity);
        }
        let bytes = bytes
            .strip_prefix(RECORD_DOMAIN)
            .ok_or(SessionError::InvalidPolicyOrRecord)?;
        let data: SessionData =
            serde_cbor::from_slice(bytes).map_err(|_| SessionError::InvalidPolicyOrRecord)?;
        check_scopes(&data.scopes)?;
        if data.caller.as_slice().len() != 29
            || data.caller.as_slice().last() != Some(&2)
            || data.subject == Principal::anonymous()
            || data.issuer == Principal::anonymous()
            || data.login_started_at_ns > data.established_at_ns
            || data.login_started_at_ns >= data.login_deadline_ns
            || data.login_deadline_ns - data.login_started_at_ns > 86_400_000_000_000
            || data.established_at_ns >= data.expires_at_ns
            || data.expires_at_ns > data.login_deadline_ns
            || data.expires_at_ns - data.established_at_ns > 1_800_000_000_000
            || data.scopes.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        Ok(Self(data))
    }
    pub(super) fn admit(
        verified: &VerifiedPortableClaims,
        request: &SessionRequest,
        ctx: &SessionContext<'_>,
        policy: SessionPolicy,
    ) -> Result<Self, SessionError> {
        let claims = verified.claims();
        let session = Self(SessionData {
            caller: ctx.authority.caller,
            subject: claims.account_subject,
            issuer: claims.issuer,
            audience: claims.audience,
            mechanism: claims.mechanism,
            key_binding_id: claims.key_binding_id,
            role: ctx.role.clone(),
            scopes: request.scopes().to_vec(),
            generation: policy.generation,
            login_started_at_ns: claims.login_started_at_ns,
            login_deadline_ns: claims.login_deadline_ns,
            established_at_ns: ctx.authority.now_ns,
            expires_at_ns: verified.session_expires_at_ns(
                request,
                ctx.role,
                policy.default_ttl_secs,
                policy.max_ttl_secs,
            )?,
            fingerprint: verified.fingerprint(),
            request_hash: request.hash(),
        });
        session.check_live(ctx, policy)?;
        session.encode()?;
        Ok(session)
    }
    pub(super) fn check_revocation(
        &self,
        ledger: &mut crate::portable::MemoryLoginRevocations,
        now_ns: u64,
    ) -> Result<(), SessionError> {
        ledger.check_session(
            self.0.caller,
            self.0.issuer,
            self.0.subject,
            self.0.login_started_at_ns,
            self.0.login_deadline_ns,
            now_ns,
        )?;
        Ok(())
    }
    pub(super) fn check_live(
        &self,
        ctx: &SessionContext<'_>,
        policy: SessionPolicy,
    ) -> Result<(), SessionError> {
        if !policy.enabled || !policy.subject_admissible {
            return Err(SessionError::Disabled);
        }
        let authority = &ctx.authority;
        if authority.now_ns < self.0.established_at_ns || authority.now_ns >= self.0.expires_at_ns {
            return Err(SessionError::MissingOrExpired);
        }
        if self.0.caller != authority.caller
            || self.0.issuer != authority.issuer
            || self.0.audience != *authority.audience
            || self.0.mechanism != authority.mechanism
            || self.0.key_binding_id != authority.key_binding_id
            || &self.0.role != ctx.role
            || self.0.generation != policy.generation
            || self.0.login_started_at_ns < authority.min_login_started_at_ns
            || authority.now_ns < authority.authority_not_before_ns
            || authority.now_ns >= authority.authority_expires_at_ns
            || authority.now_ns >= authority.enrollment_expires_at_ns
            || self.0.expires_at_ns > authority.authority_expires_at_ns
            || self.0.expires_at_ns > authority.enrollment_expires_at_ns
            || self.0.expires_at_ns - self.0.established_at_ns > policy.max_ttl_secs * 1_000_000_000
        {
            return Err(SessionError::StaleAuthority);
        }
        let grant = authority
            .grants
            .iter()
            .find(|grant| &grant.target == ctx.role)
            .ok_or(SessionError::ScopeRejected)?;
        if self
            .0
            .scopes
            .iter()
            .any(|scope| !grant.scopes.contains(scope))
        {
            return Err(SessionError::ScopeRejected);
        }
        Ok(())
    }
}

/// Replay tombstone retained independently of a session and of logout.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Replay {
    fingerprint: [u8; 32],
    caller: Principal,
    generation: u64,
    remove_at_ns: u64,
}
impl Replay {
    pub(super) fn new(session: &Session, remove_at_ns: u64) -> Self {
        Self {
            fingerprint: session.fingerprint(),
            caller: session.caller(),
            generation: session.generation(),
            remove_at_ns,
        }
    }
    pub fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }
    pub fn caller(&self) -> Principal {
        self.caller
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn remove_at_ns(&self) -> u64 {
        self.remove_at_ns
    }
    pub(super) fn encoded_len(&self) -> Result<usize, SessionError> {
        serde_cbor::to_vec(self)
            .map(|b| b.len())
            .map_err(|_| SessionError::InvalidPolicyOrRecord)
    }
}
