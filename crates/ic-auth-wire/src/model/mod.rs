//! Transport records and discriminants, kept distinct from engine decisions.

mod credential;

use ic_auth_protocol_types::Principal;
use serde::{Deserialize, Serialize};

pub use credential::{
    AudienceId, CertifiedProof, ChainKeyAlgorithm, ChainKeyBatchHeaderV1,
    ChainKeyBatchWitnessStepV1, ChainKeyBatchWitnessV1, ChainKeyDelegationCertV1, ChainKeyKeyId,
    ChainKeyRootSignatureV1, DelegatedRoleGrant, DelegationAudience, DelegationCert,
    DelegationProof, IcCanisterSignatureProofV1, IcChainKeyBatchSignatureProofV1,
    IssuerProofAlgorithm, IssuerProofBinding, PortableCredential, PortableProof, RootProof,
};

/// Session-admission refusals in their fixed transport order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub enum AdmissionRefusal {
    /// Credential is not yet valid at trusted time.
    NotYetValid,
    /// Credential has expired for fresh admission.
    Expired,
    /// Credential verification failed.
    InvalidCredential,
    /// Protected authority is not ready.
    AuthorityUnavailable,
    /// Credential predates the protected reset boundary.
    BeforeReset,
    /// Requested scopes are not admitted.
    ScopeRejected,
    /// A consumed credential conflicts with this request.
    ReplayConflict,
    /// Retained revocation prevents admission.
    Revoked,
    /// Required bounded state has no capacity.
    Capacity,
    /// The applicable attempt budget is exhausted.
    Throttled,
    /// Admission could not be completed.
    Unavailable,
}

/// A local session receipt; receiving or constructing it grants no authority.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub struct SessionView {
    /// Host-authenticated browser principal bound to the session.
    pub caller: Principal,
    /// Issuer qualifying the subject.
    pub issuer: Principal,
    /// Issuer-qualified account subject.
    pub subject: Principal,
    /// Exclusive local lease deadline in nanoseconds.
    pub expires_at_ns: u64,
}

/// Existing read-refusal discriminants, in their fixed Wire order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub enum SessionReadRefusal {
    /// No live local session.
    MissingOrExpired,
    /// Protected authority is not ready.
    AuthorityUnavailable,
    /// The required scope is not granted.
    ScopeRejected,
    /// Revocation prevents use.
    Revoked,
    /// The read could not be completed.
    Unavailable,
}

/// Existing unavailable-logout discriminants, in their fixed Wire order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub enum LogoutUnavailable {
    /// No capacity to retain the required denial.
    Capacity,
    /// Storage could not commit the denial.
    Storage,
    /// Trusted time could not support the operation.
    Clock,
    /// A reset is pending.
    ResetPending,
}

/// Local logout result; no variant acknowledges logout at another receiver.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
pub enum LogoutResult {
    /// Committed denial through this nanosecond deadline.
    Revoked(u64),
    /// Local denial deadline without confirmation of outstanding issuance.
    Unconfirmed(u64),
    /// The host-authenticated caller is not a valid logout identity.
    InvalidCaller,
    /// No revocation acknowledgement; issuance uncertainty must be retained.
    Unavailable(LogoutUnavailable),
}
