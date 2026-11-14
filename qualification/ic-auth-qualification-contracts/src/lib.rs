//! Unpublished, bounded Wire controls for the real-canister qualification host.
//! These are test endpoints, not an issuer or application authorization API.
use ic_wire::{Bytes, Limits};
pub mod model;
use model::{AdmissionProfile, ReceiverConfig, SessionProfile};

/// Private control-stage diagnostics. Detailed engine errors stay at their native
/// owners; these bounded codes never grant authority or acknowledge a failed write.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Refusal {
    /// Runtime caller is not an admitted controller for this control.
    ControllerRequired,
    /// Runtime caller is not the installed fixture resource owner.
    OwnerRequired,
    /// Controller input violates the fixture configuration contract.
    InvalidConfiguration,
    /// Submitted enrollment key or authority material is invalid.
    InvalidAuthority,
    /// Authority has not yet been installed or restored.
    AwaitingAuthority,
    /// Configured and retained authority identities do not agree.
    AuthorityMismatch,
    /// Installed authority is outside its accepted lifetime.
    AuthorityExpired,
    /// The receiver fixture is not available.
    ReceiverUnavailable,
    /// Signature preparation or verification failed.
    Signature,
    /// A bounded Wire or storage codec rejected its input.
    Codec,
    /// The session engine refused the requested operation.
    Session,
    /// No session exists for the selected caller.
    MissingSession,
    /// Restored protected metadata differs from the expected checkpoint.
    RestorationMismatch,
    /// The fixture could not admit the requested verification budget.
    AdmissionBudget,
    /// Trusted fixture time violates the required clock fence.
    Clock,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
/// Bounded seed, domain and payload for controller-authorized signature probes.
pub struct SignatureInput {
    /// Submitted seed bytes, further bounded by Auth's signature-key rules.
    pub seed: Bytes<1024>,
    /// IC signature domain, separate from the payload.
    pub domain: Bytes<255>,
    /// Exact payload selected by the controller probe.
    pub message: Bytes<1024>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
/// Signature preparation request for the disposable certification fixture.
pub struct PrepareInput {
    /// Exact seed/domain/message tuple to prepare.
    pub input: SignatureInput,
    /// Requested retrieval retention subject to Auth's maximum.
    pub retention_ns: u64,
}
/// Complete shared controller-request budgets; endpoint authority remains separate.
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
        method!(
            $name,
            $id,
            $entry,
            $request,
            $response,
            CONTROL_LIMITS,
            $limits
        );
    };
    ($name:ident, $id:literal, $entry:literal, $request:ty, $response:ty, $request_limits:expr, $limits:expr) => {
        #[doc = concat!("Private Wire contract for the fixture's `", $entry, "` control. It installs no endpoint or authority by itself.")]
        pub struct $name;
        impl $name {
            /// Exact method identity checked by both request and response codecs.
            pub const ID: u64 = $id;
            /// Entrypoint name implemented by the private qualification canister.
            pub const NAME: &'static str = $entry;
            /// Complete request byte, allocation, node and depth allowances.
            pub const REQUEST_LIMITS: Limits = $request_limits;
            /// Complete response byte, allocation, node and depth allowances.
            pub const RESPONSE_LIMITS: Limits = $limits;
            /// Encode a request under this method identity and its full budget.
            pub fn encode_request(value: &$request) -> Result<Vec<u8>, ic_wire::Error> {
                ic_wire::native::encode_request(Self::ID, value, Self::REQUEST_LIMITS)
            }
            /// Decode a request, rejecting foreign identity and exceeded budgets.
            pub fn decode_request(bytes: &[u8]) -> Result<$request, ic_wire::Error> {
                ic_wire::native::decode_request(Self::ID, bytes, Self::REQUEST_LIMITS)
            }
            /// Encode the typed response under this method identity and response budget.
            pub fn encode_response(value: &$response) -> Result<Vec<u8>, ic_wire::Error> {
                ic_wire::native::encode_response(Self::ID, value, Self::RESPONSE_LIMITS)
            }
            /// Decode a response without bypassing any method budget or identity check.
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
method!(
    ProfileAdmission,
    0x41555449f00c,
    "profile_portable_admission",
    ic_auth_wire::SessionInput,
    AdmissionProfile,
    ic_auth_wire::EstablishSession::REQUEST_LIMITS,
    PROFILE_LIMITS
);
method!(EnableCheckpoint, 0x41555449f00d, "enable_session_checkpoint", (), Result<(), Refusal>, REPLY_LIMITS);
method!(ConfirmAuthority, 0x41555449f00e, "confirm_restored_session_authority", ReceiverConfig, Result<(), Refusal>, REPLY_LIMITS);
method!(SetSessionAsset, 0x41555449f00f, "session_set_asset", u64, Result<(), Refusal>, REPLY_LIMITS);
method!(ProfileCertified, 0x41555449f010, "profile_certified_proof", ic_auth_wire::SessionInput, Result<model::CertifiedProfile, model::CertifiedProfileFailure>, ic_auth_wire::EstablishSession::REQUEST_LIMITS, PROFILE_LIMITS);

pub mod persistent;
method!(PersistentIssuer, 0x41555449f020, "persistent_issuer", persistent::Command, Result<persistent::Report, Refusal>, PROFILE_LIMITS);
method!(PersistentAdmission, 0x41555449f021, "persistent_admission", ic_auth_wire::SessionInput, Result<ic_auth_wire::SessionView, ic_auth_wire::model::AdmissionRefusal>, ic_auth_wire::EstablishSession::REQUEST_LIMITS, REPLY_LIMITS);
method!(PersistentRead, 0x41555449f022, "persistent_read", (), Result<ic_auth_wire::SessionView, ic_auth_wire::model::SessionReadRefusal>, REPLY_LIMITS);
method!(
    PersistentLogout,
    0x41555449f023,
    "persistent_logout",
    (),
    ic_auth_wire::model::LogoutResult,
    REPLY_LIMITS
);
