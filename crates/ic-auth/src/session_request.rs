//! Canonical session request identity, independent of proof verification or storage.
//!
//! Hashing a request does not authenticate a token, select host TTL policy or
//! grant scopes. Session admission and replay remain separate engine obligations.

use crate::canonical::{CanonicalAuthError, validate_scope_label};
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

/// Canonical narrowing request: 1..=16 unique scopes, at most 1024 scope bytes.
/// Input order is normalized before hashing; duplicates are rejected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionRequest {
    scopes: Vec<String>,
    ttl_secs: Option<u64>,
}
impl SessionRequest {
    pub fn new(
        mut scopes: Vec<String>,
        ttl_secs: Option<u64>,
    ) -> Result<Self, SessionRequestError> {
        check_scopes(&scopes)?;
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
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }
    /// Calculate a requested lease against protected TTL policy (maximum 1800s).
    /// This authenticates nothing and does not cap login or authority deadlines.
    /// The host supplies its clock and default/maximum policy, never the request.
    pub fn expires_at_ns(
        &self,
        now_ns: u64,
        default_ttl_secs: u64,
        max_ttl_secs: u64,
    ) -> Result<u64, SessionRequestError> {
        validate_ttl_policy(default_ttl_secs, max_ttl_secs)?;
        let ttl = self.ttl_secs.unwrap_or(default_ttl_secs);
        if ttl == 0 || ttl > max_ttl_secs {
            return Err(SessionRequestError::InvalidLifetime);
        }
        now_ns
            .checked_add(ttl * 1_000_000_000)
            .ok_or(SessionRequestError::InvalidLifetime)
    }
    pub(crate) fn check_granted_scopes(
        &self,
        granted: &[String],
    ) -> Result<(), SessionRequestError> {
        if self.scopes.iter().any(|scope| !granted.contains(scope)) {
            return Err(SessionRequestError::ScopeRejected);
        }
        Ok(())
    }
    /// Exact Canic V1 request hash; None and an explicit default TTL differ.
    pub fn hash(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"canic-application-session-request-v1");
        hash.update((self.scopes.len() as u64).to_be_bytes());
        for scope in &self.scopes {
            hash.update((scope.len() as u64).to_be_bytes());
            hash.update(scope.as_bytes());
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

pub(crate) fn validate_ttl_policy(default: u64, maximum: u64) -> Result<(), SessionRequestError> {
    if default == 0 || default > maximum || maximum > 1800 {
        return Err(SessionRequestError::InvalidLifetime);
    }
    Ok(())
}

pub(crate) fn check_scopes(scopes: &[String]) -> Result<(), SessionRequestError> {
    if scopes.is_empty() || scopes.len() > 16 {
        return Err(SessionRequestError::InvalidRequest);
    }
    // The count bound and canonical 64-byte label bound imply <=1024 bytes.
    for scope in scopes {
        validate_scope_label(scope)?;
    }
    Ok(())
}
