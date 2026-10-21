#![cfg(feature = "sessions")]
use ic_auth::{
    portable::{LoginRevocationError, PortableClaimsError},
    session::SessionError,
};
use ic_auth_protocol_types::AdmissionRefusal as R;

#[test]
fn future_start_is_the_only_claims_retry_hint() {
    for (error, expected) in [
        (PortableClaimsError::NotYetValid, R::NotYetValid),
        (PortableClaimsError::Expired, R::Expired),
        (PortableClaimsError::BeforeReset, R::BeforeReset),
        (PortableClaimsError::StaleAuthority, R::AuthorityUnavailable),
        (PortableClaimsError::InvalidContext, R::Unavailable),
        (PortableClaimsError::CallerMismatch, R::InvalidCredential),
        (PortableClaimsError::KeyMismatch, R::InvalidCredential),
        (PortableClaimsError::GrantsExceeded, R::InvalidCredential),
    ] {
        assert_eq!(SessionError::Claims(error).admission_refusal(), expected);
    }
    for (error, expected) in [
        (SessionError::StorageFailure, R::Unavailable),
        (SessionError::AdmissionClockRollback, R::Unavailable),
        (SessionError::AdmissionThrottled, R::Throttled),
        (SessionError::StaleAuthority, R::AuthorityUnavailable),
        (SessionError::ScopeRejected, R::ScopeRejected),
        (SessionError::Capacity, R::Capacity),
        (SessionError::ReplayConflict, R::ReplayConflict),
        (
            SessionError::Revocation(LoginRevocationError::Revoked),
            R::Revoked,
        ),
    ] {
        assert_eq!(error.admission_refusal(), expected);
    }
}

#[cfg(feature = "portable-signature-verification")]
#[test]
fn signature_wrappers_preserve_categories_without_matching_messages() {
    use ic_auth::{
        admission_budget::AdmissionBudgetError, portable_credential::PortableCredentialError,
        portable_signature::PortableSignatureError as E,
    };
    for (error, expected) in [
        (E::Claims(PortableClaimsError::NotYetValid), R::NotYetValid),
        (E::Claims(PortableClaimsError::Expired), R::Expired),
        (E::InvalidSignature, R::InvalidCredential),
        (E::Budget(AdmissionBudgetError::Exhausted), R::Throttled),
        (
            E::Budget(AdmissionBudgetError::ClockRollback),
            R::Unavailable,
        ),
    ] {
        assert_eq!(SessionError::Signature(error).admission_refusal(), expected);
    }
    assert_eq!(
        SessionError::Credential(PortableCredentialError::Ed25519(E::Claims(
            PortableClaimsError::NotYetValid,
        )))
        .admission_refusal(),
        R::NotYetValid,
    );
}

#[cfg(feature = "certified-proof-verification")]
#[test]
fn certified_wrappers_preserve_categories_without_matching_messages() {
    use ic_auth::{
        portable_certified::PortableCertifiedError as E,
        portable_credential::PortableCredentialError,
    };
    assert_eq!(
        SessionError::Certified(E::Claims(PortableClaimsError::NotYetValid)).admission_refusal(),
        R::NotYetValid,
    );
    assert_eq!(
        SessionError::Credential(PortableCredentialError::Certified(E::Claims(
            PortableClaimsError::Expired,
        )))
        .admission_refusal(),
        R::Expired,
    );
    assert_eq!(
        SessionError::Certified(E::BindingMismatch).admission_refusal(),
        R::InvalidCredential
    );
}
