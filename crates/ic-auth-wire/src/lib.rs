//! Controlled portable-session transport. No endpoint installation, authority,
//! cryptography, session storage or delivery/retry policy is supplied here.
pub mod model;
use ic_auth_protocol_types::limits::{MAX_LABEL_BYTES, MAX_SESSION_SCOPES};
pub use ic_wire as wire;
use ic_wire::{Limits, List, Text};
use model::{LogoutResult, PortableCredential};

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
    pub caller: ic_wire::Bytes<29>,
    pub issuer: ic_wire::Bytes<29>,
    pub subject: ic_wire::Bytes<29>,
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

/// Data-envelope identity, not a callable issuer method or signing format.
pub struct CredentialEnvelope;
impl CredentialEnvelope {
    pub const ID: u64 = 0x415554480001;
    pub const NAME: &'static str = "credential";
    pub const REQUEST_LIMITS: Limits = CREDENTIAL_LIMITS;
    pub const RESPONSE_LIMITS: Limits = CREDENTIAL_LIMITS;
    pub fn encode_request(value: &PortableCredential) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_request(Self::ID, value, Self::REQUEST_LIMITS)
    }
    pub fn decode_request(bytes: &[u8]) -> Result<PortableCredential, ic_wire::Error> {
        ic_wire::native::decode_request(Self::ID, bytes, Self::REQUEST_LIMITS)
    }
    pub fn encode_response(value: &PortableCredential) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_response(Self::ID, value, Self::RESPONSE_LIMITS)
    }
    pub fn decode_response(bytes: &[u8]) -> Result<PortableCredential, ic_wire::Error> {
        ic_wire::native::decode_response(Self::ID, bytes, Self::RESPONSE_LIMITS)
    }
}
/// Controlled admission contract; consumers bind it to their own update export.
pub struct EstablishSession;
impl EstablishSession {
    pub const ID: u64 = 0x415554480012;
    pub const NAME: &'static str = "establish_portable_session";
    pub const REQUEST_LIMITS: Limits = CREDENTIAL_LIMITS;
    pub const RESPONSE_LIMITS: Limits = REPLY_LIMITS;
    pub fn encode_request(value: &SessionInput) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_request(Self::ID, value, Self::REQUEST_LIMITS)
    }
    pub fn decode_request(bytes: &[u8]) -> Result<SessionInput, ic_wire::Error> {
        ic_wire::native::decode_request(Self::ID, bytes, Self::REQUEST_LIMITS)
    }
    pub fn encode_response(
        value: &Result<SessionView, model::AdmissionRefusal>,
    ) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_response(Self::ID, value, Self::RESPONSE_LIMITS)
    }
    pub fn decode_response(
        bytes: &[u8],
    ) -> Result<Result<SessionView, model::AdmissionRefusal>, ic_wire::Error> {
        ic_wire::native::decode_response(Self::ID, bytes, Self::RESPONSE_LIMITS)
    }
}
/// Proof-free local read contract; the runtime caller selects the session.
pub struct ReadSession;
impl ReadSession {
    pub const ID: u64 = 0x415554480003;
    pub const NAME: &'static str = "portable_session_read";
    pub const REQUEST_LIMITS: Limits = EMPTY_LIMITS;
    pub const RESPONSE_LIMITS: Limits = REPLY_LIMITS;
    pub fn encode_request(value: &()) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_request(Self::ID, value, Self::REQUEST_LIMITS)
    }
    pub fn decode_request(bytes: &[u8]) -> Result<(), ic_wire::Error> {
        ic_wire::native::decode_request(Self::ID, bytes, Self::REQUEST_LIMITS)
    }
    pub fn encode_response(
        value: &Result<SessionView, Text<256>>,
    ) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_response(Self::ID, value, Self::RESPONSE_LIMITS)
    }
    pub fn decode_response(bytes: &[u8]) -> Result<Result<SessionView, Text<256>>, ic_wire::Error> {
        ic_wire::native::decode_response(Self::ID, bytes, Self::RESPONSE_LIMITS)
    }
}
/// Caller-authenticated logout; no submitted proof or caller identity is trusted.
pub struct Logout;
impl Logout {
    pub const ID: u64 = 0x415554480004;
    pub const NAME: &'static str = "logout_portable_session";
    pub const REQUEST_LIMITS: Limits = EMPTY_LIMITS;
    pub const RESPONSE_LIMITS: Limits = REPLY_LIMITS;
    pub fn encode_request(value: &()) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_request(Self::ID, value, Self::REQUEST_LIMITS)
    }
    pub fn decode_request(bytes: &[u8]) -> Result<(), ic_wire::Error> {
        ic_wire::native::decode_request(Self::ID, bytes, Self::REQUEST_LIMITS)
    }
    pub fn encode_response(value: &LogoutResult) -> Result<Vec<u8>, ic_wire::Error> {
        ic_wire::native::encode_response(Self::ID, value, Self::RESPONSE_LIMITS)
    }
    pub fn decode_response(bytes: &[u8]) -> Result<LogoutResult, ic_wire::Error> {
        ic_wire::native::decode_response(Self::ID, bytes, Self::RESPONSE_LIMITS)
    }
}
