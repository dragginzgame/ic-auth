//! Fixture-only CDK codec hooks over the canonical native Wire contracts.
use ic_auth_qualification_contracts::{self as controls, Refusal};
use ic_wire::Bytes;

fn argument_guard(maximum: usize) -> Result<(), String> {
    if ic_platform_runtime::ic0::msg_arg_data_size() > maximum {
        Err("wire argument too large".into())
    } else {
        Ok(())
    }
}

// CDK requires concrete codec hooks; constants and codecs stay with contracts.
macro_rules! bind {
    ($name:ident, $contract:path, $request:ty $(, $response:ty)?) => {
        #[expect(non_snake_case, reason = "Mirror the declared Wire control names.")]
        pub(crate) mod $name {
            use super::*;
            use $contract as contract;
            pub(crate) fn decode(bytes: Vec<u8>) -> $request {
                contract::decode_request(&bytes).expect("bounded fixture Wire request")
            }
            $(pub(crate) fn guard() -> Result<(), String> {
                argument_guard(contract::REQUEST_LIMITS.bytes)
            }
            pub(crate) fn encode(value: $response) -> Vec<u8> {
                contract::encode_response(&value).expect("bounded fixture Wire response")
            })?
        }
    };
}
bind!(Initialize, controls::Initialize, Bytes<29>);
bind!(Upgrade, controls::Upgrade, ());
bind!(Prepare, controls::Prepare, controls::PrepareInput, Result<u64, Refusal>);
bind!(Retrieve, controls::Retrieve, controls::SignatureInput, Result<ic_auth_wire::model::IcCanisterSignatureProofV1, Refusal>);
bind!(Prune, controls::Prune, u32, Result<u32, Refusal>);
bind!(SetAsset, controls::SetAsset, u64, Result<(), Refusal>);
bind!(AssetVersion, controls::AssetVersion, (), u64);
bind!(
    OwnerOperation,
    controls::OwnerOperation,
    (),
    Result<Bytes<29>, Refusal>
);
bind!(InstallReceiver, controls::InstallReceiver, controls::model::ReceiverConfig, Result<(), Refusal>);
bind!(InvalidateAuthority, controls::InvalidateAuthority, bool, Result<(), Refusal>);
bind!(ProfileSession, controls::ProfileSession, (), Result<controls::model::SessionProfile, Refusal>);
bind!(
    ProfileAdmission,
    controls::ProfileAdmission,
    ic_auth_wire::SessionInput,
    controls::model::AdmissionProfile
);
bind!(EnableCheckpoint, controls::EnableCheckpoint, (), Result<(), Refusal>);
bind!(ConfirmAuthority, controls::ConfirmAuthority, controls::model::ReceiverConfig, Result<(), Refusal>);
bind!(SetSessionAsset, controls::SetSessionAsset, u64, Result<(), Refusal>);
bind!(ProfileCertified, controls::ProfileCertified, ic_auth_wire::SessionInput, Result<controls::model::CertifiedProfile, controls::model::CertifiedProfileFailure>);
bind!(EstablishSession, ic_auth_wire::EstablishSession, ic_auth_wire::SessionInput, Result<ic_auth_wire::SessionView, ic_auth_wire::model::AdmissionRefusal>);
bind!(ReadSession, ic_auth_wire::ReadSession, (), Result<ic_auth_wire::SessionView, ic_auth_wire::model::SessionReadRefusal>);
bind!(
    Logout,
    ic_auth_wire::Logout,
    (),
    ic_auth_wire::model::LogoutResult
);
bind!(PersistentIssuer, controls::PersistentIssuer, controls::persistent::Command, Result<controls::persistent::Report, Refusal>);
bind!(PersistentAdmission, controls::PersistentAdmission, ic_auth_wire::SessionInput, Result<ic_auth_wire::SessionView, ic_auth_wire::model::AdmissionRefusal>);
bind!(PersistentRead, controls::PersistentRead, (), Result<ic_auth_wire::SessionView, ic_auth_wire::model::SessionReadRefusal>);
bind!(
    PersistentLogout,
    controls::PersistentLogout,
    (),
    ic_auth_wire::model::LogoutResult
);
