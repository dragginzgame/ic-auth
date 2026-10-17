//! One owned decode and digest, shared by admission and complete proof verification.
use super::{PortableEncodingError, decode_claims};
use ic_auth_protocol_types::PortableClaims;
use sha2::{Digest, Sha256};

/// Canonical claims prepared by the session engine, not authenticated claims.
/// Fields and construction are private: only complete proof verification can turn
/// this value into VerifiedPortableClaims. Consume it in the admission callback
/// using `verify`; do not decode the original credential again.
///
/// ```compile_fail
/// use ic_auth::{portable::DecodedPortableClaims, portable::decode_claims};
/// fn forge(bytes: &[u8]) -> DecodedPortableClaims {
///     decode_claims(bytes).unwrap().into()
/// }
/// ```
#[derive(Debug)]
pub struct DecodedPortableClaims {
    pub(crate) claims: PortableClaims,
    pub(crate) fingerprint: [u8; 32],
    #[cfg(feature = "certified-proof-verification")]
    pub(crate) encoded_len: usize,
}
impl DecodedPortableClaims {
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, PortableEncodingError> {
        let claims = decode_claims(bytes)?;
        Ok(Self {
            claims,
            fingerprint: Sha256::digest(bytes).into(),
            #[cfg(feature = "certified-proof-verification")]
            encoded_len: bytes.len(),
        })
    }
}
