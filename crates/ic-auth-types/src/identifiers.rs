//! Validated protocol identifiers; deployment identity and trust enrollment stay
//! with the host. Existing text wire representations are deliberately preserved.

use candid::{CandidType, types};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{fmt, str::FromStr};
use thiserror::Error;

/// Invalid protocol identifier syntax. This says nothing about authorization.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum IdentifierError {
    #[error("identifier must contain exactly 64 lowercase hexadecimal characters")]
    NonCanonicalId,
    #[error("role must be nonempty and contain only a-z, 0-9, underscore, colon or hyphen")]
    InvalidRole,
}

/// Exactly 32 identity bytes, represented as lowercase hexadecimal on the wire.
/// No sentinel bytes are reserved by this protocol. A host must compare this
/// value with its protected identity, never enroll trust from a client claim.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CanonicalId([u8; 32]);

impl CanonicalId {
    /// Project an already resolved host identity without changing its bytes.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Exact bytes used in canonical signed encoding.
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

impl CandidType for CanonicalId {
    fn _ty() -> types::Type {
        types::TypeInner::Text.into()
    }

    fn idl_serialize<S: types::Serializer>(&self, serializer: S) -> Result<(), S::Error> {
        serializer.serialize_text(&self.to_string())
    }
}

/// Exact network-qualified audience, with existing protocol field labels.
/// The host owns the meaning and trusted source of both identifiers; this type
/// contains no fleet topology, membership, derivation or installation policy.
#[derive(
    CandidType, Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct AudienceId {
    pub canonical_network_id: CanonicalId,
    pub fleet_id: CanonicalId,
}

/// Canonical role label, checked at construction and deserialization.
/// The grammar matches the existing signed protocol; it adds no case folding,
/// normalization, built-in roles or implicit role authority.
#[derive(CandidType, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct AuthRole(String);

impl AuthRole {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for AuthRole {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty()
            || !value.bytes().all(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b':' | b'-')
            })
        {
            return Err(IdentifierError::InvalidRole);
        }
        Ok(Self(value.to_owned()))
    }
}

impl<'de> Deserialize<'de> for AuthRole {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
