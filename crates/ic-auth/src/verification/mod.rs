//! Fresh proof verification against explicit protected host inputs.
//!
//! Successful verification is not a session or application authorization. Hosts
//! still own current enrollment, replay/revocation, transactions and persistence.

mod budget;
#[cfg(feature = "certified-verification")]
mod certified;
mod context;
mod credential;
mod decoded;
#[cfg(feature = "ed25519-verification")]
mod ed25519;

pub(crate) use decoded::DecodedClaims;

#[cfg(feature = "receiver-stable")]
pub(crate) use {context::check as check_context, credential::verify_decoded};

pub use crate::browser_identity::browser_principal;
#[cfg(feature = "receiver-stable")]
pub(crate) use budget::RetainedBudget;
pub use budget::{BudgetError, VerificationBudget, VerificationCosts};
pub use context::{AuthorityGrants, ContextError, VerificationContext};
pub use credential::{CredentialError, VerificationProfile, verify_credential};

#[cfg(feature = "certified-verification")]
pub use certified::{
    CertifiedContext, CertifiedError, CertifiedLimits, CertifiedProof, verify_certified,
};
#[cfg(feature = "ed25519-verification")]
pub use ed25519::{Ed25519Key, VerificationError, verify_ed25519};

use crate::session_request::{SessionRequest, SessionRequestError, SessionTtlPolicy};
use ic_auth_protocol_types::{AuthRole, PortableClaims, Principal};

///
/// VerifiedClaims
///
/// Immutable proof result at one host snapshot, not a session or authorization.
/// Reuse requires the receiver's enrollment-generation, revocation/replay and
/// transaction rules; matching clock values alone do not establish freshness.
/// ```compile_fail
/// use ic_auth::verification::VerifiedClaims;
/// use ic_auth_protocol_types::PortableClaims;
/// fn forge(value: PortableClaims) -> VerifiedClaims { value.into() }
/// ```
/// ```compile_fail
/// use ic_auth::verification::VerifiedClaims;
/// fn change(value: &mut VerifiedClaims) { value.claims().device_generation = 9; }
/// ```
///

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedClaims {
    claims: PortableClaims,
    fingerprint: [u8; 32],
    caller: Principal,
    verified_at_ns: u64,
    lease_ceiling_ns: u64,
}

impl VerifiedClaims {
    // Only complete verifiers construct evidence, after successful cryptography.
    // Capture the lease ceiling now so later enrollment cannot lengthen it.
    fn from_verification(
        claims: PortableClaims,
        fingerprint: [u8; 32],
        caller: Principal,
        host: &VerificationContext<'_>,
    ) -> Self {
        let lease_ceiling_ns = claims.login_deadline_ns.min(host.authority_expires_at_ns);
        Self {
            claims,
            fingerprint,
            caller,
            verified_at_ns: host.now_ns,
            lease_ceiling_ns,
        }
    }

    /// Calculate a candidate session deadline at the original verification snapshot.
    ///
    /// Select scopes from exactly the host's protected role, enforce the requested
    /// TTL, then cap by the signed login deadline and captured authority expiry.
    /// Proof expiry limits fresh verification. Enrollment freshness is checked
    /// again when the receiver authorizes use of the lease.
    /// No current time or refreshed enrollment can extend this captured result.
    ///
    /// This creates no session or permission. The receiver must consume proof and
    /// request in the same uninterrupted host transaction, check current authority,
    /// replay, revocation, device fencing and capacity, and commit atomically.
    /// A matching timestamp does not establish that this snapshot is still current.
    pub fn session_deadline(
        &self,
        request: &SessionRequest,
        role: &AuthRole,
        policy: SessionTtlPolicy,
    ) -> Result<u64, SessionRequestError> {
        request
            .evaluate_grants(&self.claims.grants, role, self.verified_at_ns, policy)
            .map(|requested| requested.min(self.lease_ceiling_ns))
    }

    /// Borrow authenticated claims; current session authority remains separate.
    #[must_use]
    pub const fn claims(&self) -> &PortableClaims {
        &self.claims
    }

    /// SHA-256 of the exact received canonical claims, independent of proof bytes.
    #[must_use]
    pub const fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }

    /// Authenticated host caller matched to the signed browser key.
    #[must_use]
    pub const fn caller(&self) -> Principal {
        self.caller
    }

    /// Host time captured at verification, not a continuing freshness guarantee.
    #[must_use]
    pub const fn verified_at_ns(&self) -> u64 {
        self.verified_at_ns
    }
}
