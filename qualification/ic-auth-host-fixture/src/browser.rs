//! Combined issuer/receiver for a real browser journey, never a deployable service.

use serde::{Deserialize, Serialize};

/// Fixture credential operation: false issues once, true only reconciles.
pub const CREDENTIAL: u64 = 0x4155_5448_5155_0007;
/// Protected application write, with unit request and Result<u32, ()> reply.
pub const WRITE: u64 = 0x4155_5448_5155_0008;
/// Passive native-harness observation.
pub const OBSERVE: u64 = 0x4155_5448_5155_0009;
/// Counts independently checked by the native harness after browser completion.
#[derive(Debug, Deserialize, Serialize)]
pub struct Observation {
    /// Actual host signing invocations.
    pub signatures: u32,
    /// Protected application effects.
    pub writes: u32,
    /// Remaining live receiver sessions.
    pub sessions: u32,
    /// Retained receiver login fences.
    pub logins: u32,
    /// Retained issuer operations.
    pub operations: u32,
}
