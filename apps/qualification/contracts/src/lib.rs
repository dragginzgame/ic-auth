//! Unpublished, bounded Wire controls for the real-canister qualification host.
//! These are test endpoints, not an issuer or application authorization API.
use ic_wire::{Bytes, Limits, Method, Wire};
pub mod model;
use model::{AdmissionProfile, ReceiverConfig, SessionProfile};

pub type Refusal = ic_wire::Text<256>;
#[derive(Clone, Debug, Eq, PartialEq, Wire)]
pub struct SignatureInput {
    #[wire(id = 1)]
    pub seed: Bytes<1024>,
    #[wire(id = 2)]
    pub domain: Bytes<255>,
    #[wire(id = 3)]
    pub message: Bytes<1024>,
}
#[derive(Clone, Debug, Eq, PartialEq, Wire)]
pub struct PrepareInput {
    #[wire(id = 1)]
    pub input: SignatureInput,
    #[wire(id = 2)]
    pub retention_ns: u64,
}
pub const CONTROL_LIMITS: Limits = Limits {
    bytes: 65536,
    allocation: 262144,
    nodes: 8192,
    depth: 32,
};
const REPLY_LIMITS: Limits = Limits {
    bytes: 8192,
    allocation: 32768,
    nodes: 256,
    depth: 16,
};
const SIGNATURE_LIMITS: Limits = Limits {
    bytes: 262144,
    allocation: 1048576,
    nodes: 4096,
    depth: 32,
};
macro_rules! method {
    ($name:ident, $id:literal, $entry:literal, $request:ty, $response:ty, $limits:expr) => {
        pub struct $name;
        impl Method for $name {
            type Request = $request;
            type Response = $response;
            const ID: u64 = $id;
            const NAME: &'static str = $entry;
            const REQUEST_LIMITS: Limits = CONTROL_LIMITS;
            const RESPONSE_LIMITS: Limits = $limits;
        }
    };
}
method!(
    Initialize,
    0x41555448f001,
    "init",
    Bytes<29>,
    (),
    REPLY_LIMITS
);
method!(
    Upgrade,
    0x41555448f002,
    "post_upgrade",
    (),
    (),
    REPLY_LIMITS
);
method!(Prepare, 0x41555448f003, "prepare", PrepareInput, Result<u64, Refusal>, REPLY_LIMITS);
method!(Retrieve, 0x41555448f004, "retrieve", SignatureInput, Result<ic_auth_wire::model::IcCanisterSignatureProofV1, Refusal>, SIGNATURE_LIMITS);
method!(Prune, 0x41555448f005, "prune", u32, Result<u32, Refusal>, REPLY_LIMITS);
method!(SetAsset, 0x41555448f006, "set_asset", u64, Result<(), Refusal>, REPLY_LIMITS);
method!(
    AssetVersion,
    0x41555448f007,
    "asset_version",
    (),
    u64,
    REPLY_LIMITS
);
method!(
    OwnerOperation,
    0x41555448f008,
    "owner_checked_operation",
    (),
    Result<Bytes<29>, Refusal>,
    REPLY_LIMITS
);
method!(InstallReceiver, 0x41555448f009, "install_session_receiver", ReceiverConfig, Result<(), Refusal>, REPLY_LIMITS);
method!(InvalidateAuthority, 0x41555448f00a, "invalidate_session_authority", bool, Result<(), Refusal>, REPLY_LIMITS);
method!(ProfileSession, 0x41555448f00b, "profile_portable_session", (), Result<SessionProfile, Refusal>, REPLY_LIMITS);
pub struct ProfileAdmission;
impl Method for ProfileAdmission {
    type Request = ic_auth_wire::SessionInput;
    type Response = AdmissionProfile;
    const ID: u64 = 0x41555448f00c;
    const NAME: &'static str = "profile_portable_admission";
    const REQUEST_LIMITS: Limits = <ic_auth_wire::EstablishSession as Method>::REQUEST_LIMITS;
    const RESPONSE_LIMITS: Limits = REPLY_LIMITS;
}
method!(EnableCheckpoint, 0x41555448f00d, "enable_session_checkpoint", (), Result<(), Refusal>, REPLY_LIMITS);
method!(ConfirmAuthority, 0x41555448f00e, "confirm_restored_session_authority", ReceiverConfig, Result<(), Refusal>, REPLY_LIMITS);

/// Bounded UTF-8 diagnostics cannot trap and roll back admission charges.
pub fn refusal(mut error: String) -> Refusal {
    let mut end = error.len().min(256);
    while !error.is_char_boundary(end) {
        end -= 1;
    }
    error.truncate(end);
    Refusal::new(error).expect("bounded UTF-8 refusal")
}
