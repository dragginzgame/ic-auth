//! Canonical application scope labels, independent of authentication or authority.
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{fmt, str::FromStr};
use thiserror::Error;

/// A scope label's grammar failure. Offsets and lengths count UTF-8 bytes.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ScopeLabelError {
    /// No bytes were supplied.
    #[error("scope is empty")]
    Empty,
    /// The encoded label exceeds the protocol's byte bound.
    #[error("scope has {bytes} bytes; maximum is {max}")]
    TooLong {
        /// Actual encoded byte length.
        bytes: usize,
        /// Inclusive byte limit.
        max: usize,
    },
    /// A leading, trailing or repeated colon creates an empty segment.
    #[error("scope contains an empty segment")]
    EmptySegment,
    /// A segment starts with something other than a lowercase letter or digit.
    #[error("invalid scope segment start at byte {offset}")]
    InvalidSegmentStart {
        /// Zero-based byte offset of the rejected segment start.
        offset: usize,
    },
    /// A segment contains a byte outside the admitted ASCII grammar.
    #[error("invalid scope byte at offset {offset}")]
    InvalidByte {
        /// Zero-based offset of the rejected byte.
        offset: usize,
    },
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
    /// Borrow the validated label without allocating or changing its spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Recover the owned validated text without copying its allocation.
    #[must_use]
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
mod tests;
