//! Fresh proof verification against explicit protected host inputs.
//!
//! Successful verification is not a session or application authorization. Hosts
//! still own current enrollment, replay/revocation, transactions and persistence.

mod budget;
mod context;

pub use budget::{BudgetError, VerificationBudget, VerificationCosts};
pub use context::{AuthorityGrants, ContextError, VerificationContext, browser_principal};

#[cfg(feature = "ed25519-verification")]
mod ed25519;
#[cfg(feature = "ed25519-verification")]
pub use ed25519::{Ed25519Key, VerificationError, verify_ed25519};
#[cfg(feature = "certified-verification")]
mod certified;
#[cfg(feature = "certified-verification")]
pub use certified::{
    CertifiedContext, CertifiedError, CertifiedLimits, CertifiedProof, verify_certified,
};

use ic_auth_protocol_types::{PortableClaims, Principal};

/// Immutable proof result at one host snapshot, not a session or authorization.
///
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
#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedClaims {
    claims: PortableClaims,
    fingerprint: [u8; 32],
    caller: Principal,
    verified_at_ns: u64,
}

impl VerifiedClaims {
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
