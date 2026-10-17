#[cfg(any(
    feature = "portable-signature-verification",
    feature = "certified-proof-verification"
))]
use super::PortableClaimsContext;
use crate::session_request::{SessionRequest, SessionRequestError};
use ic_auth_protocol_types::{AuthRole, PortableClaims};

/// Claims authenticated by one complete portable proof verifier at a host clock.
/// Read-only and not deserializable; decoding or editing claims cannot forge this
/// result. It is not a session, enrollment approval or resource authorization.
/// Consume it under the same host transaction/authority snapshot as verification;
/// matching time alone does not establish that authority is still current.
///
/// ```compile_fail
/// use ic_auth::portable::VerifiedPortableClaims;
/// use ic_auth_protocol_types::PortableClaims;
/// fn forge(claims: PortableClaims) -> VerifiedPortableClaims { claims.into() }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPortableClaims {
    claims: PortableClaims,
    fingerprint: [u8; 32],
    verified_at_ns: u64,
    authority_expires_at_ns: u64,
    enrollment_expires_at_ns: u64,
}

impl VerifiedPortableClaims {
    #[cfg(any(
        feature = "portable-signature-verification",
        feature = "certified-proof-verification"
    ))]
    pub(crate) fn new(
        claims: PortableClaims,
        fingerprint: [u8; 32],
        context: &PortableClaimsContext<'_>,
    ) -> Self {
        Self {
            claims,
            fingerprint,
            verified_at_ns: context.now_ns,
            authority_expires_at_ns: context.authority_expires_at_ns,
            enrollment_expires_at_ns: context.enrollment_expires_at_ns,
        }
    }

    /// Exclusive lease deadline for admission at this verification snapshot.
    /// Proof expiry limits new admission, not the resulting lease. The signed
    /// login deadline and protected key/enrollment deadlines cap the lease.
    /// Persist this result once; refreshing enrollment never extends stored leases.
    /// The role comes from protected receiver configuration, never the request.
    /// Requested scopes must narrow that role's grant; other roles cannot supply
    /// missing scopes. This does not check replay, revocation or current authority
    /// after verification, and does not create or authorize a session.
    pub fn session_expires_at_ns(
        &self,
        request: &SessionRequest,
        role: &AuthRole,
        default_ttl_secs: u64,
        max_ttl_secs: u64,
    ) -> Result<u64, SessionRequestError> {
        let grant = self
            .claims
            .grants
            .iter()
            .find(|grant| &grant.target == role)
            .ok_or(SessionRequestError::ScopeRejected)?;
        request.check_granted_scopes(&grant.scopes)?;
        let requested =
            request.expires_at_ns(self.verified_at_ns, default_ttl_secs, max_ttl_secs)?;
        Ok(requested
            .min(self.claims.login_deadline_ns)
            .min(self.authority_expires_at_ns)
            .min(self.enrollment_expires_at_ns))
    }

    pub fn claims(&self) -> &PortableClaims {
        &self.claims
    }

    /// Canonical claims digest, independent of alternate proof/signature bytes.
    pub fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }

    pub fn verified_at_ns(&self) -> u64 {
        self.verified_at_ns
    }
}
