//! Controlled portable-session transport. No endpoint installation, authority,
//! cryptography, session storage or delivery/retry policy is supplied here.
pub mod model;
use ic_auth_protocol_types::limits::{MAX_LABEL_BYTES, MAX_SESSION_SCOPES};
pub use ic_wire as wire;
use ic_wire::{Limits, List, Text};
use model::{LogoutResult, PortableCredential};

/// Default browser retention cap for a complete credential frame. Larger
/// certified envelopes require an explicitly selected client cap; Wire's own
/// frame/field budgets continue to apply. Refusal is client `invalid_material`.
pub const BROWSER_DEFAULT_CREDENTIAL_BYTES: usize = 131_072;
/// Maximum configurable browser retention cap, not an override of Wire limits.
/// Preserves the client option range; actual frames remain bounded by the codec.
pub const BROWSER_MAX_CONFIGURED_CREDENTIAL_BYTES: usize = 1_048_576;
/// Browser logout response frame cap. All current reply variants fit; extensions
/// must review this intentionally narrower policy than the shared reply budget.
pub const BROWSER_LOGOUT_RESPONSE_BYTES: usize = 1024;

/// Admission narrowing accompanies the exact opaque signed credential.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
pub struct SessionInput {
    pub credential: PortableCredential,
    pub scopes: List<Text<MAX_LABEL_BYTES>, MAX_SESSION_SCOPES>,
    pub ttl_secs: Option<u64>,
}
/// A receiver's authenticated local reply, never a transferable authorization.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::Wire))]
pub struct SessionView {
    pub caller: ic_auth_protocol_types::Principal,
    pub issuer: ic_auth_protocol_types::Principal,
    pub subject: ic_auth_protocol_types::Principal,
    pub expires_at_ns: u64,
}
const CREDENTIAL_LIMITS: Limits = Limits {
    bytes: 524288,
    allocation: 2097152,
    nodes: 65536,
    depth: 32,
};
const REPLY_LIMITS: Limits = Limits {
    bytes: 4096,
    allocation: 16384,
    nodes: 128,
    depth: 8,
};
const EMPTY_LIMITS: Limits = Limits {
    bytes: 11,
    allocation: 128,
    nodes: 8,
    depth: 4,
};

// Keep method identities and limits explicit; only the four native codec
// forwarders are mechanical. This macro is private and emits the existing API.
macro_rules! codecs {
    ($request:ty, $response:ty) => {
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
    };
}

/// Data-envelope identity, not a callable issuer method or signing format.
pub struct CredentialEnvelope;
impl CredentialEnvelope {
    pub const ID: u64 = 0x415554480001;
    pub const NAME: &'static str = "credential";
    pub const REQUEST_LIMITS: Limits = CREDENTIAL_LIMITS;
    pub const RESPONSE_LIMITS: Limits = CREDENTIAL_LIMITS;
    codecs!(PortableCredential, PortableCredential);
}
/// Controlled admission contract; consumers bind it to their own update export.
pub struct EstablishSession;
impl EstablishSession {
    pub const ID: u64 = 0x415554480012;
    pub const NAME: &'static str = "establish_portable_session";
    pub const REQUEST_LIMITS: Limits = CREDENTIAL_LIMITS;
    pub const RESPONSE_LIMITS: Limits = REPLY_LIMITS;
    codecs!(SessionInput, Result<SessionView, model::AdmissionRefusal>);
}
/// Proof-free local read contract; the runtime caller selects the session.
pub struct ReadSession;
impl ReadSession {
    pub const ID: u64 = 0x415554480003;
    pub const NAME: &'static str = "portable_session_read";
    pub const REQUEST_LIMITS: Limits = EMPTY_LIMITS;
    pub const RESPONSE_LIMITS: Limits = REPLY_LIMITS;
    codecs!((), Result<SessionView, Text<256>>);
}
/// Caller-authenticated logout; no submitted proof or caller identity is trusted.
pub struct Logout;
impl Logout {
    pub const ID: u64 = 0x415554480004;
    pub const NAME: &'static str = "logout_portable_session";
    pub const REQUEST_LIMITS: Limits = EMPTY_LIMITS;
    pub const RESPONSE_LIMITS: Limits = REPLY_LIMITS;
    codecs!((), LogoutResult);
}
