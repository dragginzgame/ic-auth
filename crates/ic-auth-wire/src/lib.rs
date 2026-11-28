//! Bounded authentication transport contracts. Decoding establishes no authority.
//!
//! Hosts supply authenticated callers and implement the operations. This crate
//! supplies neither endpoint installation nor session/revocation state.

pub mod model;
pub mod projection;

pub use ic_wire as wire;
use ic_wire::{Direction, Encoding, Limits, List, Text};
pub use model::SessionView;

/// Browser cap for a complete logout reply, independent of native codec limits.
pub const BROWSER_LOGOUT_RESPONSE_BYTES: usize = 1024;

/// Default browser retention cap for a complete credential frame.
pub const BROWSER_DEFAULT_CREDENTIAL_BYTES: usize = 131_072;
/// Maximum configurable browser retention cap; codec limits still apply.
pub const BROWSER_MAX_CONFIGURED_CREDENTIAL_BYTES: usize = 1_048_576;

///
/// SessionInput
///
/// Unsigned session narrowing accompanying an opaque signed credential.
///

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, ic_wire::Wire)]
pub struct SessionInput {
    /// Claims and proof requiring complete verification against host authority.
    pub credential: model::PortableCredential,
    /// Unnormalized scope text; semantic validation belongs to request admission.
    pub scopes: List<Text<64>, 16>,
    /// Requested seconds; absence selects the protected host default.
    pub ttl_secs: Option<u64>,
}

const CREDENTIAL_LIMITS: Limits = Limits {
    bytes: 524_288,
    allocation: 2_097_152,
    nodes: 65_536,
    depth: 32,
};
const EMPTY_LIMITS: Limits = Limits {
    bytes: 12,
    allocation: 8192,
    nodes: 32,
    depth: 8,
};
const REPLY_LIMITS: Limits = Limits {
    bytes: 4096,
    allocation: 16384,
    nodes: 128,
    depth: 8,
};

const SESSION_REPLY_LIMITS: Limits = Limits {
    bytes: 8192,
    allocation: 131072,
    ..REPLY_LIMITS
};

// Sparse writers charge staging in addition to traversal. Preserve the tighter
// reader allowance when receiving credentials from an untrusted peer.
const CREDENTIAL_WRITE_LIMITS: Limits = Limits {
    allocation: 3 * 1024 * 1024,
    ..CREDENTIAL_LIMITS
};

// Forward framing and admission to Wire; each contract declares its concrete
// types, identity and both budgets below. No local serializer or runtime policy.
macro_rules! contract {
    (@writer CREDENTIAL_LIMITS) => { CREDENTIAL_WRITE_LIMITS };
    (@writer $limits:ident) => { $limits };
    (@encode direct, $value:expr, $encoding:expr, $limits:expr,) => { ic_wire::encode($value, $encoding, $limits) };
    (@decode direct, $bytes:expr, $encoding:expr, $limits:expr,) => { ic_wire::decode($bytes, $encoding, $limits) };
    (@encode wrapped, $value:expr, $encoding:expr, $limits:expr,) => { ic_wire::encode(&ic_wire::Message($value.clone()), $encoding, $limits) };
    (@decode wrapped, $bytes:expr, $encoding:expr, $limits:expr,) => { ic_wire::decode::<ic_wire::Message<_>>($bytes, $encoding, $limits).map(|m| m.0) };

    ($contract:ident, $id:expr, $name:literal, $request:ty, $reqroot:ident, $reply:ty, $reproot:ident, $request_limits:ident, $reply_limits:ident) => {
        impl $contract {
            /// Fixed routing identity, not an authentication or signing domain.
            pub const ID: u64 = $id;
            /// Contract name used by generated clients. The credential envelope has no endpoint.
            pub const NAME: &'static str = $name;
            /// Request reader allowance.
            pub const REQUEST_LIMITS: Limits = $request_limits;
            /// Response reader allowance.
            pub const RESPONSE_LIMITS: Limits = $reply_limits;

            /// Request writer allowance, including sparse preparation.
            pub const REQUEST_WRITE_LIMITS: Limits = contract!(@writer $request_limits);
            /// Response writer allowance, including sparse preparation.
            pub const RESPONSE_WRITE_LIMITS: Limits = contract!(@writer $reply_limits);

            /// Encode this request within all four resource allowances.
            ///
            /// # Errors
            /// Returns the supplier's codec error if admission fails.
            pub fn encode_request(value: &$request) -> Result<Vec<u8>, ic_wire::Error> {
                contract!(@encode $reqroot, value,
                    Self::encoding(Direction::Request),
                    Self::REQUEST_WRITE_LIMITS,
                )
            }

            /// Decode only this contract's request frame, without establishing trust.
            ///
            /// # Errors
            /// Rejects malformed, misrouted, over-budget or trailing input.
            pub fn decode_request(bytes: &[u8]) -> Result<$request, ic_wire::Error> {
                contract!(@decode $reqroot, bytes,
                    Self::encoding(Direction::Request),
                    Self::REQUEST_LIMITS,
                )
            }

            /// Encode this method's reply within all four resource allowances.
            ///
            /// # Errors
            /// Returns the supplier's codec error if admission fails.
            pub fn encode_response(value: &$reply) -> Result<Vec<u8>, ic_wire::Error> {
                contract!(@encode $reproot, value,
                    Self::encoding(Direction::Response),
                    Self::RESPONSE_WRITE_LIMITS,
                )
            }

            /// Decode a reply without treating it as proof of authority.
            ///
            /// # Errors
            /// Rejects malformed, misrouted, over-budget or trailing input.
            pub fn decode_response(bytes: &[u8]) -> Result<$reply, ic_wire::Error> {
                contract!(@decode $reproot, bytes,
                    Self::encoding(Direction::Response),
                    Self::RESPONSE_LIMITS,
                )
            }

            const fn encoding(direction: Direction) -> Encoding {
                Encoding::Frame {
                    method: Self::ID,
                    direction,
                }
            }
        }
    };
}

///
/// ReadSession
///
/// Read the authenticated caller's local session without a submitted credential.
///

pub struct ReadSession;

contract!(
    ReadSession,
    0x415554480113,
    "portable_session_read",
    (), wrapped,
    Result<SessionView, model::SessionReadRefusal>, wrapped,
    EMPTY_LIMITS,
    SESSION_REPLY_LIMITS
);

///
/// Logout
///
/// Request local logout for the authenticated caller only.
///

pub struct Logout;

contract!(
    Logout,
    0x415554480114,
    "logout_portable_session",
    (),
    wrapped,
    model::LogoutResult,
    wrapped,
    EMPTY_LIMITS,
    REPLY_LIMITS
);

///
/// CredentialEnvelope
///
/// Portable credential data envelope, not an issuer endpoint or signing format.
///

pub struct CredentialEnvelope;

contract!(
    CredentialEnvelope,
    0x415554480101,
    "credential",
    model::PortableCredential,
    direct,
    model::PortableCredential,
    direct,
    CREDENTIAL_LIMITS,
    CREDENTIAL_LIMITS
);

///
/// EstablishSession
///
/// Session admission transport; the host binds it to its authenticated update.
///

pub struct EstablishSession;

contract!(
    EstablishSession,
    0x415554480112,
    "establish_portable_session",
    SessionInput, direct,
    Result<SessionView, model::AdmissionRefusal>, wrapped,
    CREDENTIAL_LIMITS,
    SESSION_REPLY_LIMITS
);
