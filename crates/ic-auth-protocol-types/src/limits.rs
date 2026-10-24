//! Structural protocol ceilings shared by validation and bounded transport.
//! Hosts may impose tighter policy; transport allocation budgets are separate.

/// Maximum role grants in a credential or delegation certificate.
pub const MAX_GRANTS: usize = 16;
/// Maximum scopes in a role grant.
pub const MAX_SCOPES_PER_GRANT: usize = 32;
/// Maximum requested scopes in a receiver session.
pub const MAX_SESSION_SCOPES: usize = 16;
/// Maximum bytes in a role or scope label.
pub const MAX_LABEL_BYTES: usize = 64;
/// Hard cap on signed claims, independent of outer transport limits.
pub const MAX_CLAIMS_BYTES: usize = 65_536;
