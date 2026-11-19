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

/// Unsigned session narrowing accompanying an opaque signed credential.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "typescript", derive(ic_wire::typescript::Wire))]
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
    bytes: 11,
    allocation: 128,
    nodes: 8,
    depth: 4,
};
const REPLY_LIMITS: Limits = Limits {
    bytes: 4096,
    allocation: 16384,
    nodes: 128,
    depth: 8,
};

// Forward framing and admission to Wire; each contract declares its concrete
// types, identity and both budgets below. No local serializer or runtime policy.
macro_rules! contract {
    ($contract:ident, $id:expr, $name:literal, $request:ty, $reply:ty, $request_limits:ident, $reply_limits:ident) => {
        impl $contract {
            /// Fixed routing identity, not an authentication or signing domain.
            pub const ID: u64 = $id;
            /// Existing host export name.
            pub const NAME: &'static str = $name;
            /// Complete request encoding and decoding allowances.
            pub const REQUEST_LIMITS: Limits = $request_limits;
            /// Complete response encoding and decoding allowances.
            pub const RESPONSE_LIMITS: Limits = $reply_limits;

            /// Encode this request within all four resource allowances.
            ///
            /// # Errors
            /// Returns the supplier's codec error if admission fails.
            pub fn encode_request(value: &$request) -> Result<Vec<u8>, ic_wire::Error> {
                ic_wire::encode(
                    value,
                    Self::encoding(Direction::Request),
                    Self::REQUEST_LIMITS,
                )
            }

            /// Decode only this contract's request frame, without establishing trust.
            ///
            /// # Errors
            /// Rejects malformed, misrouted, over-budget or trailing input.
            pub fn decode_request(bytes: &[u8]) -> Result<$request, ic_wire::Error> {
                ic_wire::decode(
                    bytes,
                    Self::encoding(Direction::Request),
                    Self::REQUEST_LIMITS,
                )
            }

            /// Encode this method's reply within all four resource allowances.
            ///
            /// # Errors
            /// Returns the supplier's codec error if admission fails.
            pub fn encode_response(value: &$reply) -> Result<Vec<u8>, ic_wire::Error> {
                ic_wire::encode(
                    value,
                    Self::encoding(Direction::Response),
                    Self::RESPONSE_LIMITS,
                )
            }

            /// Decode a reply without treating it as proof of authority.
            ///
            /// # Errors
            /// Rejects malformed, misrouted, over-budget or trailing input.
            pub fn decode_response(bytes: &[u8]) -> Result<$reply, ic_wire::Error> {
                ic_wire::decode(
                    bytes,
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

/// Read the authenticated caller's local session without a submitted credential.
pub struct ReadSession;

contract!(
    ReadSession,
    0x415554480013,
    "portable_session_read",
    (),
    Result<SessionView, model::SessionReadRefusal>,
    EMPTY_LIMITS,
    REPLY_LIMITS
);

/// Request local logout for the authenticated caller only.
pub struct Logout;

contract!(
    Logout,
    0x415554480014,
    "logout_portable_session",
    (),
    model::LogoutResult,
    EMPTY_LIMITS,
    REPLY_LIMITS
);

/// Portable credential data envelope, not an issuer endpoint or signing format.
pub struct CredentialEnvelope;

contract!(
    CredentialEnvelope,
    0x415554480001,
    "credential",
    model::PortableCredential,
    model::PortableCredential,
    CREDENTIAL_LIMITS,
    CREDENTIAL_LIMITS
);

/// Session admission transport; the host binds it to its authenticated update.
pub struct EstablishSession;

contract!(
    EstablishSession,
    0x415554480012,
    "establish_portable_session",
    SessionInput,
    Result<SessionView, model::AdmissionRefusal>,
    CREDENTIAL_LIMITS,
    REPLY_LIMITS
);
