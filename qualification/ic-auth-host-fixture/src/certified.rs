//! Installed-cost fixture using independently OpenSSL-signed root/claims records.

use serde::{Deserialize, Serialize};

/// Isolated test measurement endpoint identity.
pub const MEASURE: u64 = 0x4155_5448_5155_000d;
/// Public fixture seed, not a signing secret.
pub const SEED: &[u8] = b"auth-qualification-issuer";
/// First canister allocated on the qualification application subnet.
pub const ISSUER: [u8; 10] = [0x7f, 0xff, 0xff, 0xff, 0xff, 0xd0, 0, 0, 1, 1];
/// One second in the installed fixture clock.
pub const SECOND: u64 = 1_000_000_000;
/// Fixed simulator time with independently signed windows.
#[must_use]
pub fn at(seconds: u64) -> u64 {
    crate::receiver::TIME_ORIGIN_NS + seconds * SECOND
}

/// Frozen independently assembled and OpenSSL-signed inputs.
/// # Panics
/// Panics if the selected frozen record is missing or malformed.
#[must_use]
pub fn bytes(name: &str) -> Vec<u8> {
    let records =
        include_str!("../../ic-auth-certified-conformance/tests/fixtures/certified-installed.tsv");
    let (_, value) = records
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .find(|(k, _)| *k == name)
        .unwrap();
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}

/// Fixed hash from the independent records.
/// # Panics
/// Panics if the selected frozen record is missing or malformed.
#[must_use]
pub fn hash(name: &str) -> [u8; 32] {
    bytes(name).try_into().unwrap()
}

///
/// Command
///
/// Test operation: only proof bytes are ingress-controlled; trust is installed separately.
///

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    /// Submit current certificate/witness bytes against fixed signed claims.
    Admit(Vec<u8>),
    /// Recheck current retained session without cryptographic verification.
    Authorize,
    /// Revoke the actual caller's login.
    Logout,
}

///
/// Measurement
///
/// Operation cost and retained accounting at the same host boundary.
///

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Measurement {
    /// Ordinary receiver outcome.
    pub result: Result<crate::receiver::Outcome, crate::receiver::Refusal>,
    /// Synchronous control interval including fixture proof assembly.
    pub instructions: u64,
    /// Linear-memory pages, not live allocations.
    pub heap_pages: (u64, u64),
    /// Session/replay occupancy after the operation.
    pub usage: (u64, u64),
    /// Remaining verification credit.
    pub credit: u32,
    /// Ordinary outcome encoding size, excluding measurement metadata.
    pub response_bytes: u64,
}
