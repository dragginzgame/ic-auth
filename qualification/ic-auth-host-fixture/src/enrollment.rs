//! Single-key enrollment exchange used only by installed qualification.

use serde::{Deserialize, Serialize};

/// Test RPC identity for an authenticated IC call.
pub const METHOD: u64 = 0x4155_5448_5155_0005;
/// Controller-only authority fault selection.
pub const CONFIGURE: u64 = 0x4155_5448_5155_0006;
/// All four allowances for the bounded single-key exchange.
pub const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 2048,
    allocation: 65_536,
    nodes: 4096,
    depth: 8,
};

///
/// Scenario
///
/// Qualification authority response behavior, selected by its actual controller.
///

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum Scenario {
    /// Original key and generation.
    #[default]
    Initial,
    /// Freshness-only refresh.
    Refresh,
    /// Wrong receiver binding.
    WrongReceiver,
    /// Stale challenge echo.
    WrongChallenge,
    /// Already expired response.
    Expired,
    /// Unknown key substituted by the authority.
    WrongKey,
    /// Revision rollback.
    Regression,
    /// Authority transport rejection.
    Unavailable,
    /// Terminal revocation with a greater generation.
    Revoke,
    /// Attempt to revive the revoked key with still greater metadata.
    Reenable,
    /// Second fresh receipt under the same authority generation.
    SecondRefresh,
}

///
/// Fault
///
/// Interrupt callback publication after the specified successful mutation.
///

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Fault {
    /// Commit normally.
    None,
    /// Trap after receiver metadata installation.
    Metadata,
    /// Trap after host entry installation.
    Entry,
}

///
/// Snapshot
///
/// Bounded reply. Its source is the installed call destination, not a reply field.
///

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    /// Actual requesting canister, observed by the authority.
    pub receiver: Vec<u8>,
    /// Echo of the host's outstanding request.
    pub challenge: [u8; 32],
    /// Monotonic snapshot revision.
    pub revision: u64,
    /// Monotonic authority generation.
    pub generation: u64,
    /// Actual authority issue time.
    pub issued_at_ns: u64,
    /// Exclusive freshness deadline.
    pub expires_at_ns: u64,
    /// The single protected test entry, scoped to the fixed fixture domain/grants.
    pub key: [u8; 32],
    /// Terminal revocation of that entry.
    pub revoked: bool,
}

/// Public test key, shared with independently generated memory vectors (seed 00..1f).
pub const KEY: [u8; 32] = [
    0x03, 0xa1, 0x07, 0xbf, 0xf3, 0xce, 0x10, 0xbe, 0x1d, 0x70, 0xdd, 0x18, 0xe7, 0x4b, 0xc0, 0x99,
    0x67, 0xe4, 0xd6, 0x30, 0x9b, 0xa5, 0x0d, 0x5f, 0x1d, 0xdc, 0x86, 0x64, 0x12, 0x55, 0x31, 0xb8,
];
