//! Test-only Wire controls. Never install this canister as an application service.
use serde::{Deserialize, Serialize};

pub mod browser;
pub mod enrollment;
pub mod receiver;

#[cfg(target_family = "wasm")]
mod canister;

/// Qualification control identity; unrelated to product endpoint identities.
pub const CONTROL: u64 = 0x4155_5448_5155_0001;
/// Qualification observation identity.
pub const OBSERVE: u64 = 0x4155_5448_5155_0002;
/// Request limits are checked before copying and during Wire decoding.
pub const REQUEST: ic_wire::Limits = ic_wire::Limits {
    bytes: 256,
    allocation: 1024,
    nodes: 32,
    depth: 4,
};
/// Bounded certificate/claims observation, not a product credential envelope.
pub const RESPONSE: ic_wire::Limits = ic_wire::Limits {
    bytes: 32_768,
    allocation: 1_048_576,
    nodes: 65_536,
    depth: 8,
};
/// Public fixture seed; this is not a signing secret.
pub const SEED: &[u8] = b"auth-host-qualification";

/// Deliberate interruptions inside one synchronous update transaction.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Fault {
    /// Commit normally.
    None,
    /// Trap after mutating the host's signature leaf.
    Leaf,
    /// Trap after publishing the system certification root.
    Root,
    /// Return a host error after root publication; the adapter must trap.
    HostError,
    /// Trap after Auth has retained completion.
    Completion,
}
/// Controller-independent fixture account operations use the actual IC caller.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Command {
    /// Reserve and approve a fixed fixture intent for the authenticated caller.
    Prepare,
    /// Complete that caller's retained intent, injecting an optional host fault.
    Complete(Fault),
    /// Account-authenticated revocation of the retained login.
    Revoke,
}
/// Fixed fixture refusal categories, separate from product Wire error projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Refusal {
    /// No surviving issuer state or terminal closure.
    Closed,
    /// Caller is not the original account, or the login is revoked.
    Denied,
    /// Completion exceeded the original deadline.
    Expired,
    /// Another ordinary Auth refusal.
    Auth,
}
/// Observable retained operation phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Progress {
    /// Genuinely new installation with no operation.
    Empty,
    /// Reserved/approved operation, without a published leaf.
    Pending,
    /// Original completion is retained.
    Completed,
    /// Lost/closed state cannot be reconstructed through these endpoints.
    Closed,
}
/// Query evidence from actual host certificate acquisition and retained state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    /// Retained progress, without creating work.
    pub progress: Progress,
    /// Physical login/operation/result-byte reservations.
    pub usage: (u32, u32, u64),
    /// Retained issuance credit.
    pub credit: u32,
    /// Exact completed claims, empty before completion.
    pub claims: Vec<u8>,
    /// Host signature subtree root, including the empty state.
    pub root: [u8; 32],
    /// Actual IC data certificate; never supplied by an ingress argument.
    pub certificate: Vec<u8>,
    /// Current canister-signature proof, empty without a retained leaf/completion.
    pub signature: Vec<u8>,
    /// Actual host query time.
    pub now_ns: u64,
}

#[cfg(target_family = "wasm")]
fn hash_bytes(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes.as_ref()).into()
}

#[cfg(target_family = "wasm")]
fn endpoint_error<E: std::fmt::Debug>(error: E) -> ! {
    ic_platform_runtime::system::trap(&format!("fixture endpoint refused: {error:?}"))
}
