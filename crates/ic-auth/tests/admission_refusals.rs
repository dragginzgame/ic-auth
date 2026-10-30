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
        assert_eq!(
            SessionError::Credential(PortableCredentialError::Ed25519(error)).admission_refusal(),
            expected,
        );
    }
}

#[cfg(feature = "certified-proof-verification")]
#[test]
fn certified_wrappers_preserve_categories_without_matching_messages() {
    use ic_auth::{
        portable_certified::PortableCertifiedError as E,
        portable_credential::PortableCredentialError,
    };
    for (error, expected) in [
        (E::Claims(PortableClaimsError::NotYetValid), R::NotYetValid),
        (E::Claims(PortableClaimsError::Expired), R::Expired),
        (E::BindingMismatch, R::InvalidCredential),
    ] {
        assert_eq!(
            SessionError::Credential(PortableCredentialError::Certified(error)).admission_refusal(),
            expected,
        );
    }
}

#[test]
fn read_refusals_keep_session_categories_and_redact_backend_details() {
    use ic_auth_protocol_types::SessionReadRefusal as Read;
    for (error, expected) in [
        (SessionError::MissingOrExpired, Read::MissingOrExpired),
        (SessionError::Disabled, Read::AuthorityUnavailable),
        (SessionError::StaleAuthority, Read::AuthorityUnavailable),
        (SessionError::ScopeRejected, Read::ScopeRejected),
        (
            SessionError::Revocation(LoginRevocationError::Revoked),
            Read::Revoked,
        ),
        (SessionError::StorageFailure, Read::Unavailable),
        (SessionError::InvalidPolicyOrRecord, Read::Unavailable),
        (
            SessionError::Revocation(LoginRevocationError::Clock),
            Read::Unavailable,
        ),
        (
            SessionError::Claims(PortableClaimsError::InvalidContext),
            Read::Unavailable,
        ),
    ] {
        assert_eq!(error.read_refusal(), expected);
    }
}
