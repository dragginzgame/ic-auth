//! Unpublished, bounded Wire controls for the real-canister qualification host.
//! These are test endpoints, not an issuer or application authorization API.
use ic_wire::{Bytes, Limits};
pub mod model;
use model::{AdmissionProfile, ReceiverConfig, SessionProfile};

pub type Refusal = ic_wire::Text<256>;
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignatureInput {
    pub seed: Bytes<1024>,
    pub domain: Bytes<255>,
    pub message: Bytes<1024>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PrepareInput {
    pub input: SignatureInput,
    pub retention_ns: u64,
}
pub const CONTROL_LIMITS: Limits = Limits {
    bytes: 65536,
    allocation: 1048576,
    nodes: 8192,
    depth: 32,
};
const REPLY_LIMITS: Limits = Limits {
    bytes: 8192,
    allocation: 32768,
    nodes: 256,
    depth: 16,
};
const PROFILE_LIMITS: Limits = Limits {
    allocation: 65536,
    ..REPLY_LIMITS
};
const SIGNATURE_LIMITS: Limits = Limits {
    bytes: 262144,
    allocation: 2097152,
    nodes: 4096,
    depth: 32,
};
macro_rules! method {
    ($name:ident, $id:literal, $entry:literal, $request:ty, $response:ty, $limits:expr) => {
        pub struct $name;
        impl $name {
            pub const ID: u64 = $id;
            pub const NAME: &'static str = $entry;
            pub const REQUEST_LIMITS: Limits = CONTROL_LIMITS;
            pub const RESPONSE_LIMITS: Limits = $limits;
            pub fn encode_request(value: &$request) -> Result<Vec<u8>, ic_wire::Error> {
                ic_wire::native::encode_request(Self::ID, value, Self::REQUEST_LIMITS)
            }
            pub fn decode_request(bytes: &[u8]) -> Result<$request, ic_wire::Error> {
                ic_wire::native::decode_request(Self::ID, bytes, Self::REQUEST_LIMITS)
            }
            pub fn encode_response(value: &$response) -> Result<Vec<u8>, ic_wire::Error> {
                ic_wire::native::encode_response(Self::ID, value, Self::RESPONSE_LIMITS)
            }
            pub fn decode_response(bytes: &[u8]) -> Result<$response, ic_wire::Error> {
                ic_wire::native::decode_response(Self::ID, bytes, Self::RESPONSE_LIMITS)
            }
        }
    };
}
method!(
    Initialize,
    0x41555449f001,
    "init",
    Bytes<29>,
    (),
    REPLY_LIMITS
);
method!(
    Upgrade,
    0x41555449f002,
    "post_upgrade",
    (),
    (),
    REPLY_LIMITS
);
method!(Prepare, 0x41555449f003, "prepare", PrepareInput, Result<u64, Refusal>, REPLY_LIMITS);
method!(Retrieve, 0x41555449f004, "retrieve", SignatureInput, Result<ic_auth_wire::model::IcCanisterSignatureProofV1, Refusal>, SIGNATURE_LIMITS);
method!(Prune, 0x41555449f005, "prune", u32, Result<u32, Refusal>, REPLY_LIMITS);
method!(SetAsset, 0x41555449f006, "set_asset", u64, Result<(), Refusal>, REPLY_LIMITS);
method!(
    AssetVersion,
    0x41555449f007,
    "asset_version",
    (),
    u64,
    REPLY_LIMITS
);
method!(
    OwnerOperation,
    0x41555449f008,
    "owner_checked_operation",
    (),
    Result<Bytes<29>, Refusal>,
    REPLY_LIMITS
);
method!(InstallReceiver, 0x41555449f009, "install_session_receiver", ReceiverConfig, Result<(), Refusal>, REPLY_LIMITS);
method!(InvalidateAuthority, 0x41555449f00a, "invalidate_session_authority", bool, Result<(), Refusal>, REPLY_LIMITS);
method!(ProfileSession, 0x41555449f00b, "profile_portable_session", (), Result<SessionProfile, Refusal>, REPLY_LIMITS);
pub struct ProfileAdmission;
impl ProfileAdmission {
    pub const ID: u64 = 0x41555449f00c;
    pub const NAME: &'static str = "profile_portable_admission";
    pub const REQUEST_LIMITS: Limits = ic_auth_wire::EstablishSession::REQUEST_LIMITS;
    pub const RESPONSE_LIMITS: Limits = PROFILE_LIMITS;
    pub fn encode_request(value: &ic_auth_wire::SessionInput) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_request(Self::ID, value, Self::REQUEST_LIMITS)
    }
    pub fn decode_request(bytes: &[u8]) -> Result<ic_auth_wire::SessionInput, ic_wire::Error> {
        ic_wire::native::decode_request(Self::ID, bytes, Self::REQUEST_LIMITS)
    }
    pub fn encode_response(value: &AdmissionProfile) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_response(Self::ID, value, Self::RESPONSE_LIMITS)
    }
    pub fn decode_response(bytes: &[u8]) -> Result<AdmissionProfile, ic_wire::Error> {
        ic_wire::native::decode_response(Self::ID, bytes, Self::RESPONSE_LIMITS)
    }
}
method!(EnableCheckpoint, 0x41555449f00d, "enable_session_checkpoint", (), Result<(), Refusal>, REPLY_LIMITS);
method!(ConfirmAuthority, 0x41555449f00e, "confirm_restored_session_authority", ReceiverConfig, Result<(), Refusal>, REPLY_LIMITS);
method!(SetSessionAsset, 0x41555449f00f, "session_set_asset", u64, Result<(), Refusal>, REPLY_LIMITS);
pub struct ProfileCertified;
impl ProfileCertified {
    pub const ID: u64 = 0x41555449f010;
    pub const NAME: &'static str = "profile_certified_proof";
    pub const REQUEST_LIMITS: Limits = ic_auth_wire::EstablishSession::REQUEST_LIMITS;
    pub const RESPONSE_LIMITS: Limits = PROFILE_LIMITS;
    pub fn encode_request(value: &ic_auth_wire::SessionInput) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_request(Self::ID, value, Self::REQUEST_LIMITS)
    }
    pub fn decode_request(bytes: &[u8]) -> Result<ic_auth_wire::SessionInput, ic_wire::Error> {
        ic_wire::native::decode_request(Self::ID, bytes, Self::REQUEST_LIMITS)
    }
    pub fn encode_response(
        value: &Result<model::CertifiedProfile, model::CertifiedProfileFailure>,
    ) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_response(Self::ID, value, Self::RESPONSE_LIMITS)
    }
    pub fn decode_response(
        bytes: &[u8],
    ) -> Result<Result<model::CertifiedProfile, model::CertifiedProfileFailure>, ic_wire::Error>
    {
        ic_wire::native::decode_response(Self::ID, bytes, Self::RESPONSE_LIMITS)
    }
}

/// Bounded UTF-8 diagnostics cannot trap and roll back admission charges.
pub fn refusal(mut error: String) -> Refusal {
    let mut end = error.len().min(256);
    while !error.is_char_boundary(end) {
        end -= 1;
    }
    error.truncate(end);
    Refusal::new(error).expect("bounded UTF-8 refusal")
}
