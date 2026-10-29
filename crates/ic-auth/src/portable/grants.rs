//! Immutable structural admission for host-selected portable authority grants.
use crate::certificate_issuance::{GrantValidationError, validate_grants};
use ic_auth_protocol_types::DelegatedRoleGrant;

/// Owned canonical grants admitted to the portable limits once, before use.
/// This establishes shape only, never enrollment, provenance or authorization.
/// Hosts must select the current protected set and advance their generation
/// atomically with authority changes. Clock, revocation and scope checks still
/// run for every operation. Construct a fresh value after decoding configuration
/// or restoring it from storage; this type has no unchecked deserializer.
///
/// Read access cannot invalidate admitted ordering or grammar:
/// ```compile_fail
/// use ic_auth::portable::ValidatedGrants;
/// fn change(grants: &mut ValidatedGrants) {
///     grants.as_slice()[0].scopes.clear();
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedGrants(Vec<DelegatedRoleGrant>);

impl ValidatedGrants {
    /// Admit nonempty, strictly ordered role/scope grants without normalization.
    /// Refuse more than 16 roles or 32 scopes per role, empty scope sets and
    /// invalid scope labels through the canonical grant validator.
    pub fn new(grants: Vec<DelegatedRoleGrant>) -> Result<Self, GrantValidationError> {
        validate_grants(
            &grants,
            crate::limits::MAX_GRANTS,
            crate::limits::MAX_SCOPES_PER_GRANT,
        )?;
        Ok(Self(grants))
    }

    pub fn as_slice(&self) -> &[DelegatedRoleGrant] {
        &self.0
    }

    /// Consume structural evidence before editing or projecting the raw grants.
    /// Reconstruct through `new` before using edited grants as authority again.
    pub fn into_vec(self) -> Vec<DelegatedRoleGrant> {
        self.0
    }
}
