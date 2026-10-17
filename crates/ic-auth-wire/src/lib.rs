//! Controlled portable-session transport. No endpoint installation, authority,
//! cryptography, session storage or delivery/retry policy is supplied here.
pub mod model;
pub use ic_wire as wire;
use ic_wire::{Limits, List, Method, Text, Wire};
use model::{LogoutResult, PortableCredential};

/// Admission narrowing accompanies the exact opaque signed credential.
#[derive(Clone, Debug, Eq, PartialEq, Wire)]
pub struct SessionInput {
    #[wire(id = 1)]
    pub credential: PortableCredential,
    #[wire(id = 2)]
    pub scopes: List<Text<64>, 16>,
    #[wire(id = 3)]
    pub ttl_secs: Option<u64>,
}
/// A receiver's authenticated local reply, never a transferable authorization.
#[derive(Clone, Debug, Eq, PartialEq, Wire)]
pub struct SessionView {
    #[wire(id = 1)]
    pub caller: ic_wire::Bytes<29>,
    #[wire(id = 2)]
    pub issuer: ic_wire::Bytes<29>,
    #[wire(id = 3)]
    pub subject: ic_wire::Bytes<29>,
    #[wire(id = 4)]
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
impl Method for CredentialEnvelope {
    type Request = PortableCredential;
    type Response = PortableCredential;
    const ID: u64 = 0x415554480001;
    const NAME: &'static str = "credential";
    const REQUEST_LIMITS: Limits = CREDENTIAL_LIMITS;
    const RESPONSE_LIMITS: Limits = CREDENTIAL_LIMITS;
}
/// Controlled admission contract; consumers bind it to their own update export.
pub struct EstablishSession;
impl Method for EstablishSession {
    type Request = SessionInput;
    type Response = Result<SessionView, Text<256>>;
    const ID: u64 = 0x415554480002;
    const NAME: &'static str = "establish_portable_session";
    const REQUEST_LIMITS: Limits = CREDENTIAL_LIMITS;
    const RESPONSE_LIMITS: Limits = REPLY_LIMITS;
}
/// Proof-free local read contract; the runtime caller selects the session.
pub struct ReadSession;
impl Method for ReadSession {
    type Request = ();
    type Response = Result<SessionView, Text<256>>;
    const ID: u64 = 0x415554480003;
    const NAME: &'static str = "portable_session_read";
    const REQUEST_LIMITS: Limits = EMPTY_LIMITS;
    const RESPONSE_LIMITS: Limits = REPLY_LIMITS;
}
/// Caller-authenticated logout; no submitted proof or caller identity is trusted.
pub struct Logout;
impl Method for Logout {
    type Request = ();
    type Response = LogoutResult;
    const ID: u64 = 0x415554480004;
    const NAME: &'static str = "logout_portable_session";
    const REQUEST_LIMITS: Limits = EMPTY_LIMITS;
    const RESPONSE_LIMITS: Limits = REPLY_LIMITS;
}
