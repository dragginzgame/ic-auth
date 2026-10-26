//! Public refusal categories without exposing backend or cryptographic diagnostics.
use super::SessionError;
#[cfg(any(
    feature = "portable-signature-verification",
    feature = "certified-proof-verification"
))]
use crate::admission_budget::AdmissionBudgetError;
use crate::portable::PortableClaimsError;
use ic_auth_protocol_types::AdmissionRefusal;

impl SessionError {
    /// Project an admission failure without parsing its diagnostic text. A
    /// `NotYetValid` result is only a bounded retry hint, not proof validity.
    /// Hosts retain the original error for diagnostics and must not trap away
    /// failed-attempt charges. This does not decide retry or endpoint policy.
    pub fn admission_refusal(&self) -> AdmissionRefusal {
        use AdmissionRefusal as R;
        match self {
            Self::AdmissionThrottled => R::Throttled,
            Self::AdmissionClockRollback
            | Self::GenerationExhausted
            | Self::StorageFailure
            | Self::InvalidPolicyOrRecord => R::Unavailable,
            Self::Disabled | Self::StaleAuthority => R::AuthorityUnavailable,
            Self::MissingOrExpired => R::Expired,
            Self::ReplayConflict => R::ReplayConflict,
            Self::ScopeRejected => R::ScopeRejected,
            Self::Capacity | Self::ByteCapacity => R::Capacity,
            Self::Canonical(_) | Self::Encoding(_) => R::InvalidCredential,
            Self::Claims(error) => claims(error),
            Self::Revocation(error) => match error {
                crate::portable::LoginRevocationError::Revoked => R::Revoked,
                crate::portable::LoginRevocationError::Capacity => R::Capacity,
                crate::portable::LoginRevocationError::InvalidCaller => R::InvalidCredential,
                crate::portable::LoginRevocationError::InvalidPolicy
                | crate::portable::LoginRevocationError::Clock
                | crate::portable::LoginRevocationError::Storage => R::Unavailable,
            },
            #[cfg(any(
                feature = "portable-signature-verification",
                feature = "certified-proof-verification"
            ))]
            Self::Credential(error) => match error {
                crate::portable_credential::PortableCredentialError::MechanismMismatch => {
                    R::InvalidCredential
                }
                #[cfg(feature = "portable-signature-verification")]
                crate::portable_credential::PortableCredentialError::Ed25519(error) => {
                    signature(error)
                }
                #[cfg(feature = "certified-proof-verification")]
                crate::portable_credential::PortableCredentialError::Certified(error) => {
                    certified(error)
                }
            },
        }
    }
}

fn claims(error: &PortableClaimsError) -> AdmissionRefusal {
    use AdmissionRefusal as R;
    use PortableClaimsError as E;
    match error {
        E::NotYetValid => R::NotYetValid,
        E::Expired => R::Expired,
        E::BeforeReset => R::BeforeReset,
        E::StaleAuthority => R::AuthorityUnavailable,
        E::InvalidContext => R::Unavailable,
        E::AudienceMismatch
        | E::IssuerMismatch
        | E::MechanismMismatch
        | E::KeyMismatch
        | E::CallerMismatch
        | E::GrantsExceeded
        | E::InvalidWindow
        | E::Encoding(_) => R::InvalidCredential,
    }
}

#[cfg(any(
    feature = "portable-signature-verification",
    feature = "certified-proof-verification"
))]
fn budget(error: &AdmissionBudgetError) -> AdmissionRefusal {
    match error {
        AdmissionBudgetError::Exhausted => AdmissionRefusal::Throttled,
        AdmissionBudgetError::ClockRollback | AdmissionBudgetError::InvalidPolicy => {
            AdmissionRefusal::Unavailable
        }
    }
}

#[cfg(feature = "portable-signature-verification")]
fn signature(error: &crate::portable_signature::PortableSignatureError) -> AdmissionRefusal {
    use crate::portable_signature::PortableSignatureError as E;
    match error {
        E::Claims(error) => claims(error),
        E::Budget(error) => budget(error),
        E::InvalidPublicKey => AdmissionRefusal::AuthorityUnavailable,
        E::Encoding(_) | E::MalformedSignature | E::InvalidSignature => {
            AdmissionRefusal::InvalidCredential
        }
    }
}

#[cfg(feature = "certified-proof-verification")]
fn certified(error: &crate::portable_certified::PortableCertifiedError) -> AdmissionRefusal {
    use crate::portable_certified::PortableCertifiedError as E;
    match error {
        E::Claims(error) => claims(error),
        E::Budget(error) => budget(error),
        E::BindingMismatch
        | E::Encoding(_)
        | E::Canonical(_)
        | E::Delegation(_)
        | E::Signature(_) => AdmissionRefusal::InvalidCredential,
    }
}
