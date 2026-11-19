//! Role labels have their own grammar, distinct from segmented scopes.

use std::{error::Error, fmt, str::FromStr};

/// A role is empty, over 64 bytes or contains a byte outside `[a-z0-9_:-]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoleError;

impl fmt::Display for RoleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "role must contain 1..=64 bytes using only a-z, 0-9, underscore, colon or hyphen",
        )
    }
}

impl Error for RoleError {}

/// Exact ASCII role spelling. This names a role but grants no permissions.
///
/// Unlike scopes, roles have no segment-start rules; punctuation-only labels
/// remain valid under the signed protocol. No normalization is performed.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "stable-records",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "stable-records", serde(try_from = "String"))]
pub struct AuthRole(String);

impl AuthRole {
    /// Check role syntax without allocating.
    pub fn validate(text: &str) -> Result<(), RoleError> {
        if !(1..=64).contains(&text.len())
            || !text
                .bytes()
                .all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b':' | b'-'))
        {
            return Err(RoleError);
        }
        Ok(())
    }

    /// Borrow the original role spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for AuthRole {
    type Err = RoleError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::validate(text)?;
        Ok(Self(text.to_owned()))
    }
}

impl TryFrom<String> for AuthRole {
    type Error = RoleError;
    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::validate(&text)?;
        Ok(Self(text))
    }
}
