//! Canonical session request identity, independent of proof verification or storage.
//!
//! Hashing a request does not authenticate a token, select host TTL policy or
//! grant scopes. Session admission and replay remain separate engine obligations.

use crate::canonical::CanonicalAuthError;
use ic_auth_protocol_types::{AuthRole, DelegatedRoleGrant, ScopeLabel};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Request shape, narrowing and lifetime rejection; no cryptographic verification.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum SessionRequestError {
    #[error("session request has invalid scope bounds, duplicate scopes or zero TTL")]
    InvalidRequest,
    #[error("invalid session TTL policy, requested lifetime or expiry overflow")]
    InvalidLifetime,
    #[error("requested scopes are not granted for the protected receiver role")]
    ScopeRejected,
    #[error(transparent)]
    Canonical(#[from] CanonicalAuthError),
}

/// Checked current maximum session lifetime, in seconds (1..=1800).
/// Authorization needs this bound, without an admission default.
/// Construction cannot bypass range admission:
/// ```compile_fail
/// use ic_auth::session_request::SessionTtlLimit;
/// let invalid = SessionTtlLimit(0);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionTtlLimit(u64);
impl SessionTtlLimit {
    pub fn new(seconds: u64) -> Result<Self, SessionRequestError> {
        if seconds == 0 || seconds > crate::limits::MAX_SESSION_SECS {
            return Err(SessionRequestError::InvalidLifetime);
        }
        Ok(Self(seconds))
    }
    pub const fn seconds(self) -> u64 {
        self.0
    }
}

/// Checked default and maximum TTL for fresh admission. Neither value is taken
/// from a submitted request. Live authorization uses only `maximum()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionTtlPolicy {
    default_secs: u64,
    maximum: SessionTtlLimit,
}
impl SessionTtlPolicy {
    pub fn new(default_secs: u64, maximum_secs: u64) -> Result<Self, SessionRequestError> {
        let maximum = SessionTtlLimit::new(maximum_secs)?;
        if default_secs == 0 || default_secs > maximum_secs {
            return Err(SessionRequestError::InvalidLifetime);
        }
        Ok(Self {
            default_secs,
            maximum,
        })
    }
    pub const fn default_secs(self) -> u64 {
        self.default_secs
    }
    pub const fn maximum(self) -> SessionTtlLimit {
        self.maximum
    }
}

/// Canonical narrowing request: 1..=16 unique scopes, at most 1024 scope bytes.
/// Input order is normalized before hashing; duplicates are rejected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionRequest {
    scopes: Vec<ScopeLabel>,
    ttl_secs: Option<u64>,
}
impl SessionRequest {
    pub fn new(scopes: Vec<String>, ttl_secs: Option<u64>) -> Result<Self, SessionRequestError> {
        check_scope_count(scopes.len())?;
        let mut scopes = ScopeLabel::from_strings(scopes).map_err(CanonicalAuthError::from)?;
        scopes.sort_unstable();
        if scopes.windows(2).any(|pair| pair[0] == pair[1]) || ttl_secs == Some(0) {
            return Err(SessionRequestError::InvalidRequest);
        }
        Ok(Self { scopes, ttl_secs })
    }
    /// Exact requested TTL in seconds; None leaves selection to protected host policy.
    pub const fn ttl_secs(&self) -> Option<u64> {
        self.ttl_secs
    }
    pub fn scopes(&self) -> &[ScopeLabel] {
        &self.scopes
    }
    #[cfg(all(
        feature = "sessions",
        any(
            feature = "portable-signature-verification",
            feature = "certified-proof-verification"
        )
    ))]
    pub(crate) fn into_scopes(self) -> Vec<String> {
        self.scopes
            .into_iter()
            .map(ScopeLabel::into_string)
            .collect()
    }
    /// Calculate a requested lease against protected TTL policy (maximum 1800s).
    /// This authenticates nothing and does not cap login or authority deadlines.
    /// The host supplies its clock and default/maximum policy, never the request.
    pub fn expires_at_ns(
        &self,
        now_ns: u64,
        policy: SessionTtlPolicy,
    ) -> Result<u64, SessionRequestError> {
        let ttl = self.ttl_secs.unwrap_or(policy.default_secs());
        if ttl > policy.maximum().seconds() {
            return Err(SessionRequestError::InvalidLifetime);
        }
        now_ns
            .checked_add(ttl * crate::limits::NS_PER_SECOND)
            .ok_or(SessionRequestError::InvalidLifetime)
    }
    fn check_granted_scopes(&self, granted: &[String]) -> Result<(), SessionRequestError> {
        if !crate::certificate_issuance::scopes_are_subset(&self.scopes, granted) {
            return Err(SessionRequestError::ScopeRejected);
        }
        Ok(())
    }
    /// Evaluate narrowing and requested TTL at this caller's protected snapshot.
    /// Admission and verified claims both use this policy; each retains its own
    /// authority checks and the verified lease additionally caps signed deadlines.
    pub(crate) fn evaluate(
        &self,
        grants: &[DelegatedRoleGrant],
        role: &AuthRole,
        now_ns: u64,
        policy: SessionTtlPolicy,
    ) -> Result<u64, SessionRequestError> {
        let grant = crate::certificate_issuance::grant_for_role(grants, role)
            .ok_or(SessionRequestError::ScopeRejected)?;
        self.check_granted_scopes(&grant.scopes)?;
        self.expires_at_ns(now_ns, policy)
    }
    /// Exact Canic V1 request hash; None and an explicit default TTL differ.
    pub fn hash(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"canic-application-session-request-v1");
        hash.update((self.scopes.len() as u64).to_be_bytes());
        for scope in &self.scopes {
            hash.update((scope.as_str().len() as u64).to_be_bytes());
            hash.update(scope.as_str().as_bytes());
        }
        match self.ttl_secs {
            None => hash.update([0]),
            Some(ttl) => {
                hash.update([1]);
                hash.update(ttl.to_be_bytes());
            }
        }
        hash.finalize().into()
    }
}

#[cfg(feature = "session-restore")]
pub(crate) fn check_scopes(scopes: &[String]) -> Result<(), SessionRequestError> {
    check_scope_count(scopes.len())?;
    // The count bound and canonical 64-byte label bound imply <=1024 bytes.
    for scope in scopes {
        crate::canonical::validate_scope_label(scope)?;
    }
    Ok(())
}

fn check_scope_count(count: usize) -> Result<(), SessionRequestError> {
    if count == 0 || count > crate::limits::MAX_SESSION_SCOPES {
        return Err(SessionRequestError::InvalidRequest);
    }
    Ok(())
}
