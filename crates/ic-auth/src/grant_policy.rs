//! Shared selection over grants already admitted in canonical order.

use ic_auth_protocol_types::{AuthRole, DelegatedRoleGrant, ScopeLabel};

pub(crate) fn for_role<'a>(
    grants: &'a [DelegatedRoleGrant],
    role: &AuthRole,
) -> Option<&'a DelegatedRoleGrant> {
    grants
        .binary_search_by(|grant| grant.target.cmp(role))
        .ok()
        .map(|index| &grants[index])
}

// Required operation scopes may be unordered or duplicated; allowed scopes
// must be strictly ordered. Membership never unions roles or changes either list.
pub(crate) fn scopes_subset(wanted: &[ScopeLabel], allowed: &[ScopeLabel]) -> bool {
    wanted
        .iter()
        .all(|scope| allowed.binary_search(scope).is_ok())
}
