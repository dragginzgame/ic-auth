//! Explicit persistent fixture endpoints. Missing backing never falls back to
//! the volatile backend. Both use the same protected authority and verifier.
use super::{
    AdmissionRefusal, Backend, LogoutResult, LogoutUnavailable, Refusal, SessionInput,
    SessionOperation, SessionView, authorize_with, establish, with_receiver,
};

#[ic_platform_runtime::cdk::update(
    guard = "crate::control::PersistentAdmission::guard",
    decode_with = "crate::control::PersistentAdmission::decode",
    encode_with = "crate::control::PersistentAdmission::encode"
)]
fn persistent_admission(input: SessionInput) -> Result<SessionView, AdmissionRefusal> {
    establish(input, Backend::Persistent)
}

#[ic_platform_runtime::cdk::update(
    guard = "crate::control::PersistentRead::guard",
    decode_with = "crate::control::PersistentRead::decode",
    encode_with = "crate::control::PersistentRead::encode"
)]
fn persistent_read() -> Result<SessionView, ic_auth_wire::model::SessionReadRefusal> {
    authorize_with(SessionOperation::Read, Backend::Persistent).map_err(Into::into)
}

#[ic_platform_runtime::cdk::update(
    guard = "crate::control::PersistentLogout::guard",
    decode_with = "crate::control::PersistentLogout::decode",
    encode_with = "crate::control::PersistentLogout::encode"
)]
fn persistent_logout() -> ic_auth_wire::model::LogoutResult {
    let result: Result<_, Refusal> = with_receiver(|_| {
        crate::persistent::with_receiver(|store| {
            store.logout(
                ic_platform_runtime::cdk::api::msg_caller(),
                ic_platform_runtime::cdk::api::time(),
            )
        })
        .ok_or(Refusal::ReceiverUnavailable)
    });
    result
        .unwrap_or(LogoutResult::Unavailable(LogoutUnavailable::ResetPending))
        .into()
}
