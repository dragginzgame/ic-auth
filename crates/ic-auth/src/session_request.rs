//! Canonical unsigned request identity and host-selected lifetime bounds.
//!
//! Sorting here is specific to unsigned requests. Signed grant lists must retain
//! their separate strict-order admission; they must never be sorted into validity.

use crate::claims::{ClaimsError, validate_grants};
use ic_auth_protocol_types::{AuthRole, DelegatedRoleGrant, ScopeLabel, ScopeLabelError};
use sha2::{Digest, Sha256};
use std::{error::Error, fmt};

const MAX_SCOPES: usize = 16;
const MAX_SESSION_SECONDS: u64 = 1800;
const NANOS_PER_SECOND: u64 = 1_000_000_000;
const REQUEST_DOMAIN: &[u8] = b"canic-application-session-request-v1";

///
/// SessionRequestError
///
/// Shape or lifetime refusal. None of these results authenticate a request.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionRequestError {
    /// Scope count, uniqueness or nonzero TTL was violated.
    InvalidRequest,
    /// A label violates the canonical scope grammar.
    Scope(ScopeLabelError),
    /// Submitted grants failed bounded canonical shape/order admission.
    Grants(ClaimsError),
    /// The protected role is absent or does not contain every requested scope.
    ScopeRejected,
    /// Host policy, requested TTL or deadline arithmetic is invalid.
    InvalidLifetime,
}

impl fmt::Display for SessionRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest => f.write_str(
                "session request has invalid scope bounds, duplicate scopes or zero TTL",
            ),
            Self::Scope(error) => error.fmt(f),
            Self::Grants(error) => error.fmt(f),
            Self::ScopeRejected => f.write_str("requested scopes exceed the selected role"),
            Self::InvalidLifetime => {
                f.write_str("invalid session TTL policy, requested lifetime or expiry overflow")
            }
        }
    }
}

impl Error for SessionRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Scope(error) => Some(error),
            Self::Grants(error) => Some(error),
            _ => None,
        }
    }
}

///
/// SessionTtlPolicy
///
/// Host-supplied default and ceiling, with `1 <= default <= maximum <= 1800`.
/// Construction validates numeric bounds, not the provenance of host policy.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionTtlPolicy {
    default_secs: u64,
    maximum_secs: u64,
}

impl Default for SessionTtlPolicy {
    /// Fifteen-minute leases with a 30-minute request ceiling.
    fn default() -> Self {
        Self {
            default_secs: 900,
            maximum_secs: MAX_SESSION_SECONDS,
        }
    }
}

impl SessionTtlPolicy {
    /// Validate the host's default and maximum in seconds.
    pub fn new(default_secs: u64, maximum_secs: u64) -> Result<Self, SessionRequestError> {
        if default_secs == 0 || default_secs > maximum_secs || maximum_secs > MAX_SESSION_SECONDS {
            return Err(SessionRequestError::InvalidLifetime);
        }
        Ok(Self {
            default_secs,
            maximum_secs,
        })
    }

    /// Duration selected when the request omits a TTL.
    #[must_use]
    pub const fn default_secs(self) -> u64 {
        self.default_secs
    }

    /// Inclusive maximum duration accepted by this host policy.
    #[must_use]
    pub const fn maximum_secs(self) -> u64 {
        self.maximum_secs
    }
}

///
/// SessionRequest
///
/// An unsigned request with 1–16 unique sorted scopes and an optional nonzero TTL.
/// The scope bound implies at most 1024 label bytes. Constructing or hashing this
/// value does not grant scopes, authenticate a credential or admit its lifetime.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionRequest {
    scopes: Vec<ScopeLabel>,
    ttl_secs: Option<u64>,
}

impl SessionRequest {
    #[cfg(feature = "receiver-stable")]
    pub(crate) fn into_scopes(self) -> Vec<ScopeLabel> {
        self.scopes
    }

    /// Admit count, then grammar in input order, then duplicates and nonzero TTL.
    ///
    /// Scope order is normalized. Positive TTLs, including those above the
    /// protocol ceiling, retain their exact identity until host-policy evaluation.
    pub fn new(scopes: Vec<String>, ttl_secs: Option<u64>) -> Result<Self, SessionRequestError> {
        if !(1..=MAX_SCOPES).contains(&scopes.len()) {
            return Err(SessionRequestError::InvalidRequest);
        }
        let mut scopes = scopes
            .into_iter()
            .map(ScopeLabel::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(SessionRequestError::Scope)?;
        scopes.sort_unstable();
        if scopes.windows(2).any(|pair| pair[0] == pair[1]) || ttl_secs == Some(0) {
            return Err(SessionRequestError::InvalidRequest);
        }
        Ok(Self { scopes, ttl_secs })
    }

    /// Borrow normalized scopes; their inclusion still needs a verified grant.
    #[must_use]
    pub fn scopes(&self) -> &[ScopeLabel] {
        &self.scopes
    }

    /// Exact submitted lifetime, preserving absence independently of host defaults.
    #[must_use]
    pub const fn ttl_secs(&self) -> Option<u64> {
        self.ttl_secs
    }

    /// Compute the existing V1 request identity without allocating encoded bytes.
    ///
    /// Domain bytes have no terminator. Counts, label lengths and the optional TTL
    /// use big-endian u64; the TTL presence tag is one byte. This is a request
    /// fingerprint, never a signed credential or authorization token.
    #[must_use]
    pub fn hash(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(REQUEST_DOMAIN);
        digest.update((self.scopes.len() as u64).to_be_bytes());
        for scope in &self.scopes {
            let bytes = scope.as_str().as_bytes();
            digest.update((bytes.len() as u64).to_be_bytes());
            digest.update(bytes);
        }
        digest.update([u8::from(self.ttl_secs.is_some())]);
        if let Some(seconds) = self.ttl_secs {
            digest.update(seconds.to_be_bytes());
        }
        digest.finalize().into()
    }

    /// Compute the requested deadline from a trusted host time and policy.
    ///
    /// Reject over-limit TTLs and overflow. The receiver must additionally cap
    /// this deadline by login and authority expiry. Current enrollment is
    /// independently required for using the session.
    pub fn expires_at_ns(
        &self,
        now_ns: u64,
        policy: SessionTtlPolicy,
    ) -> Result<u64, SessionRequestError> {
        let seconds = self.ttl_secs.unwrap_or(policy.default_secs);
        if seconds > policy.maximum_secs {
            return Err(SessionRequestError::InvalidLifetime);
        }
        // Policy admission bounds multiplication to 1800 seconds before it runs.
        now_ns
            .checked_add(seconds * NANOS_PER_SECOND)
            .ok_or(SessionRequestError::InvalidLifetime)
    }

    /// Check narrowing to one protected role, then calculate the requested deadline.
    ///
    /// This is cheap request preflight, not proof verification or session admission.
    /// Grants must be bounded, strictly ordered and unique; they are never sorted.
    /// Other roles cannot supply missing scopes. The host owns `role`, time and
    /// TTL policy. Signature, fresh-claims aggregate bounds, current authority,
    /// replay, revocation, enrollment freshness and the login and authority lease caps remain separate.
    pub fn evaluate_grants(
        &self,
        grants: &[DelegatedRoleGrant],
        role: &AuthRole,
        now_ns: u64,
        policy: SessionTtlPolicy,
    ) -> Result<u64, SessionRequestError> {
        validate_grants(grants).map_err(SessionRequestError::Grants)?;
        let selected = crate::grant_policy::for_role(grants, role)
            .ok_or(SessionRequestError::ScopeRejected)?;
        if !crate::grant_policy::scopes_subset(&self.scopes, &selected.scopes) {
            return Err(SessionRequestError::ScopeRejected);
        }
        self.expires_at_ns(now_ns, policy)
    }
}
