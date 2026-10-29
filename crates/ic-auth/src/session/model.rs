use super::{SessionContext, SessionError, SessionOccupancy};
use crate::portable::{LoginBinding, LoginRecordState, is_login_caller};
#[cfg(feature = "session-restore")]
use crate::session_request::check_scopes;
#[cfg(any(
    feature = "portable-signature-verification",
    feature = "certified-proof-verification"
))]
use crate::{portable::VerifiedPortableClaims, session_request::SessionRequest};
use ic_auth_protocol_types::{AudienceId, AuthRole, PortableMechanism, Principal};
#[cfg(feature = "session-restore")]
use serde::Deserialize;
use serde::Serialize;

/// Current host eligibility and generation for admission and authorization.
/// Hosts must advance generation atomically with
/// authority changes, including disable/re-enable and issuer revocation.
#[derive(Clone, Copy, Debug)]
pub struct SessionPolicy {
    /// Current host-owned generation, equal to the transactional store's value.
    pub generation: u64,
    /// Protected capability switch; re-enabling requires a new generation.
    pub enabled: bool,
    /// Protected local subject eligibility, independent of resource ownership.
    pub subject_admissible: bool,
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

const MAX_SESSION_RECORD_BYTES: usize = 2048;
const MAX_REPLAY_RECORD_BYTES: usize = 512;

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
    /// Check a proposed host transaction's login record against this session.
    /// Custom stores call this before persistence instead of decoding private
    /// record fields. Both operations require the same caller and audience.
    /// Admission additionally requires the retained issuer, account subject,
    /// login window and sufficient reservation retention to match.
    ///
    /// A revoked change is valid only for session removal: its login binding may
    /// differ or be absent, allowing logout to clear an older retained session.
    /// Stores MUST require `Reserved` for admission and `Revoked` for logout.
    /// The operation's login change selects the removal caller.
    /// This checks record association, not proof validity, current authority,
    /// replay, capacity, storage provenance or transaction atomicity.
    pub fn check_login_change(
        &self,
        change: &crate::portable::LoginRevocationChange,
    ) -> Result<(), SessionError> {
        if change.caller() != self.0.caller
            || change.audience() != self.0.audience
            || matches!(change.state(), LoginRecordState::Reserved(binding)
            if change.until_ns() < self.0.login_deadline_ns || binding != self.login_binding())
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        Ok(())
    }
    pub fn caller(&self) -> Principal {
        self.0.caller
    }
    pub fn subject(&self) -> Principal {
        self.0.subject
    }
    pub fn issuer(&self) -> Principal {
        self.0.issuer
    }
    /// Proof mechanism retained at admission, for selecting protected enrollment.
    /// This selector does not establish freshness or authorize session use.
    pub fn mechanism(&self) -> PortableMechanism {
        self.0.mechanism
    }
    /// Exact issuer key binding retained at admission. Resolve it in current
    /// protected enrollment and still call `authorize_session` before use.
    pub fn key_binding_id(&self) -> [u8; 32] {
        self.0.key_binding_id
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
        crate::storage_codec::encode(
            &self.0,
            crate::storage_codec::SESSION,
            MAX_SESSION_RECORD_BYTES,
            SessionError::InvalidPolicyOrRecord,
            SessionError::ByteCapacity,
        )
    }
    pub(super) fn encoded_len(&self) -> Result<usize, SessionError> {
        crate::storage_codec::encoded_len(
            &self.0,
            MAX_SESSION_RECORD_BYTES,
            SessionError::InvalidPolicyOrRecord,
            SessionError::ByteCapacity,
        )
    }
    /// Trusted-storage restoration only. Old token-session bytes are rejected.
    #[cfg(feature = "session-restore")]
    pub fn decode(bytes: &[u8]) -> Result<Self, SessionError> {
        let data: SessionData = crate::storage_codec::decode(
            bytes,
            crate::storage_codec::SESSION,
            MAX_SESSION_RECORD_BYTES,
            SessionError::InvalidPolicyOrRecord,
            SessionError::ByteCapacity,
        )?;
        check_scopes(&data.scopes)?;
        if !is_login_caller(data.caller)
            || data.subject == Principal::anonymous()
            || data.issuer == Principal::anonymous()
            || data.login_started_at_ns > data.established_at_ns
            || data.login_started_at_ns >= data.login_deadline_ns
            || data.login_deadline_ns - data.login_started_at_ns > crate::limits::MAX_LOGIN_NS
            || data.established_at_ns >= data.expires_at_ns
            || data.expires_at_ns > data.login_deadline_ns
            || data.expires_at_ns - data.established_at_ns > crate::limits::MAX_SESSION_NS
            || data.scopes.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        Ok(Self(data))
    }
    #[cfg(any(
        feature = "portable-signature-verification",
        feature = "certified-proof-verification"
    ))]
    pub(super) fn admit(
        verified: &VerifiedPortableClaims,
        request: &SessionRequest,
        ctx: &SessionContext<'_>,
        policy: SessionPolicy,
        requested_expiry: u64,
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
            expires_at_ns: verified.clamp_session_expiry(requested_expiry),
            fingerprint: verified.fingerprint(),
            request_hash: request.hash(),
        });
        session.check_live(ctx, policy)?;
        Ok(session)
    }
    pub(super) fn check_revocation(&self, login: Option<LoginBinding>) -> Result<(), SessionError> {
        if login != Some(self.login_binding()) {
            return Err(crate::portable::LoginRevocationError::InvalidPolicy.into());
        }
        Ok(())
    }

    fn login_binding(&self) -> LoginBinding {
        LoginBinding {
            issuer: self.0.issuer,
            subject: self.0.subject,
            started_at_ns: self.0.login_started_at_ns,
            deadline_ns: self.0.login_deadline_ns,
        }
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
            || self.0.expires_at_ns - self.0.established_at_ns
                > ctx.max_ttl.seconds() * crate::limits::NS_PER_SECOND
        {
            return Err(SessionError::StaleAuthority);
        }
        let grant =
            crate::certificate_issuance::grant_for_role(authority.grants.as_slice(), ctx.role)
                .ok_or(SessionError::ScopeRejected)?;
        if !crate::certificate_issuance::scopes_are_subset(&self.0.scopes, &grant.scopes) {
            return Err(SessionError::ScopeRejected);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "session-restore", derive(Deserialize))]
#[serde(deny_unknown_fields)]
struct ReplayData {
    fingerprint: [u8; 32],
    caller: Principal,
    generation: u64,
    remove_at_ns: u64,
}

/// Replay tombstone retained independently of a session and of logout.
/// Use the bounded domain-separated codec, never raw serde storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replay(ReplayData);
impl Replay {
    /// Bounded, domain-separated storage record; not an admission credential.
    pub fn encode(&self) -> Result<Vec<u8>, SessionError> {
        self.validate()?;
        crate::storage_codec::encode(
            &self.0,
            crate::storage_codec::REPLAY,
            MAX_REPLAY_RECORD_BYTES,
            SessionError::InvalidPolicyOrRecord,
            SessionError::ByteCapacity,
        )
    }

    fn validate(&self) -> Result<(), SessionError> {
        if !is_login_caller(self.0.caller) || self.0.remove_at_ns == 0 {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        Ok(())
    }

    /// Trusted-storage restoration only; provenance and rollback are host-owned.
    #[cfg(feature = "session-restore")]
    pub fn decode(bytes: &[u8]) -> Result<Self, SessionError> {
        let record = Self(crate::storage_codec::decode(
            bytes,
            crate::storage_codec::REPLAY,
            MAX_REPLAY_RECORD_BYTES,
            SessionError::InvalidPolicyOrRecord,
            SessionError::ByteCapacity,
        )?);
        record.validate()?;
        Ok(record)
    }

    #[cfg(any(
        feature = "portable-signature-verification",
        feature = "certified-proof-verification"
    ))]
    pub(super) fn new(session: &Session, remove_at_ns: u64) -> Self {
        Self(ReplayData {
            fingerprint: session.fingerprint(),
            caller: session.caller(),
            generation: session.generation(),
            remove_at_ns,
        })
    }
    pub fn fingerprint(&self) -> [u8; 32] {
        self.0.fingerprint
    }
    pub fn caller(&self) -> Principal {
        self.0.caller
    }
    pub fn generation(&self) -> u64 {
        self.0.generation
    }
    pub fn remove_at_ns(&self) -> u64 {
        self.0.remove_at_ns
    }
    pub(super) fn encoded_len(&self) -> Result<usize, SessionError> {
        self.validate()?;
        crate::storage_codec::encoded_len(
            &self.0,
            MAX_REPLAY_RECORD_BYTES,
            SessionError::InvalidPolicyOrRecord,
            SessionError::ByteCapacity,
        )
    }
}
