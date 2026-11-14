//! Validated protocol identifiers; deployment identity and trust enrollment stay
//! with the host. Existing text wire representations are deliberately preserved.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{fmt, str::FromStr};
use thiserror::Error;

/// Invalid protocol identifier syntax. This says nothing about authorization.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum IdentifierError {
    /// The input is not exactly 64 lowercase hexadecimal characters.
    #[error("identifier must contain exactly 64 lowercase hexadecimal characters")]
    NonCanonicalId,
    /// The role is empty, exceeds 64 bytes or contains a forbidden byte.
    #[error("role must contain 1..=64 bytes using only a-z, 0-9, underscore, colon or hyphen")]
    InvalidRole,
}

/// Exactly 32 identity bytes, represented as lowercase hexadecimal on the wire.
/// No sentinel bytes are reserved by this protocol. A host must compare this
/// value with its protected identity, never enroll trust from a client claim.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalId([u8; 32]);

impl CanonicalId {
    /// Project an already resolved host identity without changing its bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Exact bytes used in canonical signed encoding.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for CanonicalId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for CanonicalId {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 64
            || !value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(IdentifierError::NonCanonicalId);
        }
        let mut bytes = [0; 32];
        for (byte, pair) in bytes.iter_mut().zip(value.as_bytes().as_chunks::<2>().0) {
            let nibble = |b| if b <= b'9' { b - b'0' } else { b - b'a' + 10 };
            *byte = (nibble(pair[0]) << 4) | nibble(pair[1]);
        }
        Ok(Self(bytes))
    }
}

impl Serialize for CanonicalId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for CanonicalId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

/// Exact network-qualified audience, with existing protocol field labels.
/// The host owns the meaning and trusted source of both identifiers; this type
/// contains no fleet topology, membership, derivation or installation policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AudienceId {
    /// Host-defined canonical network identity, compared without normalization.
    pub canonical_network_id: CanonicalId,
    /// Host-defined fleet identity within that network.
    pub fleet_id: CanonicalId,
}

/// Canonical 1..=64-byte role label, checked at construction and deserialization.
/// The grammar matches the existing signed protocol; it adds no case folding,
/// normalization, built-in roles or implicit role authority.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct AuthRole(String);

impl AuthRole {
    /// Borrow the validated label without allocation or normalization.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Recover the validated label without copying its allocation.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }

    fn validate(value: &str) -> Result<(), IdentifierError> {
        if value.is_empty()
            || value.len() > crate::limits::MAX_LABEL_BYTES
            || !value.bytes().all(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b':' | b'-')
            })
        {
            return Err(IdentifierError::InvalidRole);
        }
        Ok(())
    }
}

impl FromStr for AuthRole {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::validate(value)?;
        Ok(Self(value.to_owned()))
    }
}

impl TryFrom<String> for AuthRole {
    type Error = IdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate(&value)?;
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for AuthRole {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_from(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests;
