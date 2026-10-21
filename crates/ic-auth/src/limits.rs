//! Protocol ceilings, shared by live validation and trusted-storage restoration.

pub(crate) const MAX_GRANTS: usize = 16;
pub(crate) const MAX_SCOPES_PER_GRANT: usize = 32;
pub(crate) const MAX_SESSION_SCOPES: usize = 16;
pub(crate) const MAX_LABEL_BYTES: usize = 64;

pub(crate) const NS_PER_SECOND: u64 = 1_000_000_000;
pub(crate) const MAX_LOGIN_NS: u64 = 86_400 * NS_PER_SECOND;
pub(crate) const MAX_PROOF_NS: u64 = 60 * NS_PER_SECOND;
pub(crate) const MAX_SESSION_SECS: u64 = 1_800;
#[cfg(feature = "session-restore")]
pub(crate) const MAX_SESSION_NS: u64 = MAX_SESSION_SECS * NS_PER_SECOND;
