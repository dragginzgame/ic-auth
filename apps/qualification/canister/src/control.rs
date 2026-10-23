//! Fixture-only CDK codec hooks over the canonical native Wire contracts.
use ic_auth_qualification_contracts::{self as controls, CONTROL_LIMITS, refusal};
use ic_wire::Bytes;

pub(crate) fn guard() -> Result<(), String> {
    if ic_platform::ic0::msg_arg_data_size() > CONTROL_LIMITS.bytes {
        Err("wire control argument too large".into())
    } else {
        Ok(())
    }
}

// CDK requires concrete codec hooks; constants and codecs stay with contracts.
macro_rules! bind {
    ($name:ident, $request:ty) => {
        #[allow(non_snake_case)]
        pub(crate) mod $name {
            use super::*;
            pub(crate) fn decode(bytes: Vec<u8>) -> $request {
                controls::$name::decode_request(&bytes).expect("bounded fixture Wire request")
            }
        }
    };
    ($name:ident, $request:ty, result $value:ty) => {
        #[allow(non_snake_case)]
        pub(crate) mod $name {
            use super::*;
            pub(crate) fn decode(bytes: Vec<u8>) -> $request {
                controls::$name::decode_request(&bytes).expect("bounded fixture Wire request")
            }
            pub(crate) fn encode(value: Result<$value, String>) -> Vec<u8> {
                controls::$name::encode_response(&value.map_err(refusal))
                    .expect("bounded fixture Wire response")
            }
        }
    };
    ($name:ident, $request:ty, value $value:ty) => {
        #[allow(non_snake_case)]
        pub(crate) mod $name {
            use super::*;
            pub(crate) fn decode(bytes: Vec<u8>) -> $request {
                controls::$name::decode_request(&bytes).expect("bounded fixture Wire request")
            }
            pub(crate) fn encode(value: $value) -> Vec<u8> {
                controls::$name::encode_response(&value).expect("bounded fixture Wire response")
            }
        }
    };
}
bind!(Initialize, Bytes<29>);
bind!(Upgrade, ());
bind!(Prepare, controls::PrepareInput, result u64);
bind!(Retrieve, controls::SignatureInput, result ic_auth_wire::model::IcCanisterSignatureProofV1);
bind!(Prune, u32, result u32);
bind!(SetAsset, u64, result());
bind!(AssetVersion, (), value u64);
bind!(OwnerOperation, (), result Bytes<29>);
bind!(InstallReceiver, controls::model::ReceiverConfig, result());
bind!(InvalidateAuthority, bool, result());
bind!(ProfileSession, (), result controls::model::SessionProfile);
bind!(ProfileAdmission, ic_auth_wire::SessionInput, value controls::model::AdmissionProfile);
bind!(EnableCheckpoint, (), result());
bind!(ConfirmAuthority, controls::model::ReceiverConfig, result());
