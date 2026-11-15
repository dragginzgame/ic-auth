//! Private canonical input shared by admission and the complete verifiers.

use crate::claims::{ClaimsError, decode_claims};
use ic_auth_protocol_types::PortableClaims;
use sha2::{Digest, Sha256};
use std::cell::OnceCell;

pub(crate) struct DecodedClaims<'a> {
    pub(crate) claims: PortableClaims,
    encoded: &'a [u8],
    fingerprint: OnceCell<[u8; 32]>,
}

impl<'a> DecodedClaims<'a> {
    pub(crate) fn new(encoded: &'a [u8]) -> Result<Self, ClaimsError> {
        Ok(Self {
            claims: decode_claims(encoded)?,
            encoded,
            fingerprint: OnceCell::new(),
        })
    }

    pub(crate) fn fingerprint(&self) -> [u8; 32] {
        *self
            .fingerprint
            .get_or_init(|| Sha256::digest(self.encoded).into())
    }

    #[cfg(all(feature = "receiver-memory", feature = "certified-verification"))]
    pub(super) fn encoded_len(&self) -> usize {
        self.encoded.len()
    }
}
