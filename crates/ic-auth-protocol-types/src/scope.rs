//! Scope syntax shared by signed grants and unsigned requests.

use std::{error::Error, fmt, str::FromStr};

const MAX_BYTES: usize = 64;

///
/// ScopeLabelError
///
/// A scope syntax failure; lengths and offsets count UTF-8 bytes.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeLabelError {
    /// The label contains no bytes.
    Empty,
    /// The label exceeds the inclusive byte limit.
    TooLong {
        /// Supplied byte length.
        bytes: usize,
        /// Inclusive protocol limit.
        max: usize,
    },
    /// A leading, repeated or trailing colon creates an empty segment.
    EmptySegment,
    /// A segment does not begin with an ASCII lowercase letter or digit.
    InvalidSegmentStart {
        /// Zero-based byte offset.
        offset: usize,
    },
    /// A continuation byte is outside the ASCII scope grammar.
    InvalidByte {
        /// Zero-based byte offset.
        offset: usize,
    },
}

impl fmt::Display for ScopeLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("scope is empty"),
            Self::TooLong { bytes, max } => write!(f, "scope has {bytes} bytes; maximum is {max}"),
            Self::EmptySegment => f.write_str("scope contains an empty segment"),
            Self::InvalidSegmentStart { offset } => {
                write!(f, "invalid scope segment start at byte {offset}")
            }
            Self::InvalidByte { offset } => write!(f, "invalid scope byte at offset {offset}"),
        }
    }
}

impl Error for ScopeLabelError {}

///
/// ScopeLabel
///
/// An immutable, unnormalized 1–64-byte scope label.
/// Colon-separated segments start with `[a-z0-9]` and continue with
/// `[a-z0-9_-]`. A valid label says nothing about which caller may use it.
/// ```compile_fail
/// use ic_auth_protocol_types::ScopeLabel;
/// let unchecked = ScopeLabel(String::from("Invalid"));
/// ```
///

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "stable-records",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "stable-records", serde(try_from = "String"))]
pub struct ScopeLabel(String);

impl ScopeLabel {
    /// Check grammar without allocating or normalizing input.
    ///
    /// Length errors precede grammar errors; grammar reports the first invalid
    /// segment or byte in input order.
    pub fn validate(text: &str) -> Result<(), ScopeLabelError> {
        if text.is_empty() {
            return Err(ScopeLabelError::Empty);
        }
        if text.len() > MAX_BYTES {
            return Err(ScopeLabelError::TooLong {
                bytes: text.len(),
                max: MAX_BYTES,
            });
        }
        let mut offset = 0;
        for segment in text.split(':') {
            let mut bytes = segment.bytes();
            let first = bytes.next().ok_or(ScopeLabelError::EmptySegment)?;
            if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
                return Err(ScopeLabelError::InvalidSegmentStart { offset });
            }
            for (index, byte) in bytes.enumerate() {
                if !byte.is_ascii_lowercase()
                    && !byte.is_ascii_digit()
                    && !matches!(byte, b'_' | b'-')
                {
                    return Err(ScopeLabelError::InvalidByte {
                        offset: offset + index + 1,
                    });
                }
            }
            offset += segment.len() + 1;
        }
        Ok(())
    }

    /// Borrow the admitted spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Recover the original owned text without copying.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<String> for ScopeLabel {
    type Error = ScopeLabelError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::validate(&text)?;
        Ok(Self(text))
    }
}

impl FromStr for ScopeLabel {
    type Err = ScopeLabelError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::validate(text)?;
        Ok(Self(text.to_owned()))
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
