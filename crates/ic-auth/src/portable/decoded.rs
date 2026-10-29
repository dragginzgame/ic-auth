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

/// Static/context checks for this exact owned decode and protected snapshot.
/// Still unauthenticated; only the complete proof verifier can authenticate it.
pub(crate) struct ContextualClaims<'a, 'ctx> {
    decoded: DecodedPortableClaims,
    context: &'a super::PortableClaimsContext<'ctx>,
}

impl DecodedPortableClaims {
    pub(crate) fn validate_context<'a, 'ctx>(
        self,
        context: &'a super::PortableClaimsContext<'ctx>,
    ) -> Result<ContextualClaims<'a, 'ctx>, super::PortableClaimsError> {
        super::rules::validate_decoded_context(&self.claims, context)?;
        Ok(ContextualClaims {
            decoded: self,
            context,
        })
    }
}

impl<'a, 'ctx> ContextualClaims<'a, 'ctx> {
    #[cfg(feature = "sessions")]
    pub(crate) fn decoded(&self) -> &DecodedPortableClaims {
        &self.decoded
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        DecodedPortableClaims,
        &'a super::PortableClaimsContext<'ctx>,
    ) {
        (self.decoded, self.context)
    }
}
