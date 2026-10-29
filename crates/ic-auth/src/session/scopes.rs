use super::SessionError;

/// Immutable scope requirements selected by a protected application operation.
/// Validate once at configuration/operation setup and reconstruct after decoding.
/// This proves grammar and bounds only: every call still checks the current
/// authority, session, revocation and scope membership. Empty sets, duplicates
/// and arbitrary order are accepted, as with raw operation requirements.
/// Requirements cannot be edited without re-admission:
/// ```compile_fail
/// use ic_auth::session::RequiredScopes;
/// fn change(scopes: &mut RequiredScopes) {
///     scopes.as_slice()[0].clear();
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredScopes(Vec<String>);

impl RequiredScopes {
    /// No operation-specific scopes; all other authorization checks still apply.
    pub const NONE: &'static Self = &Self(Vec::new());

    pub fn new(scopes: Vec<String>) -> Result<Self, SessionError> {
        if scopes.len() > crate::limits::MAX_SESSION_SCOPES {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        for scope in &scopes {
            crate::canonical::validate_scope_label(scope)?;
        }
        Ok(Self(scopes))
    }

    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}
