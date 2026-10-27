//! One owned decode and digest, shared by admission and complete proof verification.
use super::{PortableEncodingError, decode_claims};
use ic_auth_protocol_types::PortableClaims;
use sha2::{Digest, Sha256};

/// Canonical claims prepared by the session engine, not authenticated claims.
/// Only complete proof verification can turn this into VerifiedPortableClaims.
#[derive(Debug)]
pub(crate) struct DecodedPortableClaims {
    pub(crate) claims: PortableClaims,
    pub(crate) fingerprint: [u8; 32],
    #[cfg(all(feature = "sessions", feature = "certified-proof-verification"))]
    pub(crate) encoded_len: usize,
}
impl DecodedPortableClaims {
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, PortableEncodingError> {
        let claims = decode_claims(bytes)?;
        Ok(Self {
            claims,
            fingerprint: Sha256::digest(bytes).into(),
            #[cfg(all(feature = "sessions", feature = "certified-proof-verification"))]
            encoded_len: bytes.len(),
        })
    }
}
