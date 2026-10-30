//! Canonical application scope labels, independent of authentication or authority.
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{fmt, str::FromStr};
use thiserror::Error;

/// A scope label's grammar failure. Offsets and lengths count UTF-8 bytes.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ScopeLabelError {
    #[error("scope is empty")]
    Empty,
    #[error("scope has {bytes} bytes; maximum is {max}")]
    TooLong { bytes: usize, max: usize },
    #[error("scope contains an empty segment")]
    EmptySegment,
    #[error("invalid scope segment start at byte {offset}")]
    InvalidSegmentStart { offset: usize },
    #[error("invalid scope byte at offset {offset}")]
    InvalidByte { offset: usize },
}

/// Immutable 1..=64-byte scope label. Shape grants no authority.
/// Colon-separated segments start with a lowercase ASCII letter or digit;
/// subsequent bytes may also be underscore or hyphen. No normalization occurs.
/// ```compile_fail
/// use ic_auth_protocol_types::ScopeLabel;
/// fn change(scope: &mut ScopeLabel) { scope.as_str().clear(); }
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ScopeLabel(String);

impl ScopeLabel {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn into_string(self) -> String {
        self.0
    }
    /// Admit an owned sequence without sorting or imposing cardinality policy.
    /// Callers admit their count bound first; the first grammar failure wins.
    pub fn from_strings(values: Vec<String>) -> Result<Vec<Self>, ScopeLabelError> {
        for value in &values {
            Self::validate(value)?;
        }
        Ok(values.into_iter().map(Self).collect())
    }
    /// Allocation-free shape check, also usable by const and raw-proof callers.
    pub const fn validate(scope: &str) -> Result<(), ScopeLabelError> {
        let bytes = scope.as_bytes();
        if bytes.is_empty() {
            return Err(ScopeLabelError::Empty);
        }
        if bytes.len() > crate::limits::MAX_LABEL_BYTES {
            return Err(ScopeLabelError::TooLong {
                bytes: bytes.len(),
                max: crate::limits::MAX_LABEL_BYTES,
            });
        }
        let mut segment_start = true;
        let mut offset = 0;
        while offset < bytes.len() {
            let byte = bytes[offset];
            if byte == b':' {
                if segment_start {
                    return Err(ScopeLabelError::EmptySegment);
                }
                segment_start = true;
            } else if segment_start {
                if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() {
                    return Err(ScopeLabelError::InvalidSegmentStart { offset });
                }
                segment_start = false;
            } else if !byte.is_ascii_lowercase()
                && !byte.is_ascii_digit()
                && byte != b'_'
                && byte != b'-'
            {
                return Err(ScopeLabelError::InvalidByte { offset });
            }
            offset += 1;
        }
        if segment_start {
            return Err(ScopeLabelError::EmptySegment);
        }
        Ok(())
    }
}
impl FromStr for ScopeLabel {
    type Err = ScopeLabelError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::validate(value)?;
        Ok(Self(value.to_owned()))
    }
}
impl TryFrom<String> for ScopeLabel {
    type Error = ScopeLabelError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate(&value)?;
        Ok(Self(value))
    }
}
impl AsRef<str> for ScopeLabel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl fmt::Display for ScopeLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl<'de> Deserialize<'de> for ScopeLabel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_from(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_scope_admission_preserves_bytes_allocation_and_exact_grammar() {
        for text in ["read", "fleet:read_all-1", "0:1", "", "a::b", "Read", "é"] {
            let mut supplied = String::with_capacity(128);
            supplied.push_str(text);
            let original = supplied.as_ptr();
            let owned = ScopeLabel::try_from(supplied);
            assert_eq!(owned, text.parse());
            if let Ok(label) = owned {
                assert_eq!(label.as_str(), text);
                let recovered = label.into_string();
                assert_eq!(recovered.as_ptr(), original);
                assert_eq!(recovered, text);
            }
        }
        assert!(ScopeLabel::try_from("a".repeat(64)).is_ok());
        assert_eq!(
            ScopeLabel::try_from("a".repeat(65)),
            Err(ScopeLabelError::TooLong { bytes: 65, max: 64 })
        );
    }
}
