//! Canonical session request identity, independent of proof verification or storage.
//!
//! Hashing a request does not authenticate a token, select host TTL policy or
//! grant scopes. Session admission and replay remain separate engine obligations.

use crate::canonical::{CanonicalAuthError, validate_scope_label};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Request shape rejection, without cryptographic or session-policy causes.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum SessionRequestError {
    #[error("session request has invalid scope bounds, duplicate scopes or zero TTL")]
    InvalidRequest,
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

pub(crate) fn check_scopes(scopes: &[String]) -> Result<(), SessionRequestError> {
    if scopes.is_empty() || scopes.len() > 16 {
        return Err(SessionRequestError::InvalidRequest);
    }
    let mut bytes = 0usize;
    for scope in scopes {
        validate_scope_label(scope)?;
        bytes += scope.len(); // Each validated scope is <=64 bytes, count <=16.
    }
    if bytes > 1024 {
        return Err(SessionRequestError::InvalidRequest);
    }
    Ok(())
}
