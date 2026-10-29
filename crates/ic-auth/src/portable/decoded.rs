//! One owned decode and digest, shared by admission and complete proof verification.
use super::{PortableEncodingError, decode_claims};
use ic_auth_protocol_types::{PortableClaims, Principal};
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
    browser_caller: Principal,
}

impl DecodedPortableClaims {
    pub(crate) fn validate_context<'a, 'ctx>(
        self,
        context: &'a super::PortableClaimsContext<'ctx>,
    ) -> Result<ContextualClaims<'a, 'ctx>, super::PortableClaimsError> {
        let browser_caller = super::browser_principal(&self.claims.browser_public_key);
        super::rules::validate_decoded_context(&self.claims, browser_caller, context)?;
        Ok(ContextualClaims {
            decoded: self,
            context,
            browser_caller,
        })
    }
}

impl<'a, 'ctx> ContextualClaims<'a, 'ctx> {
    #[cfg(feature = "sessions")]
    pub(crate) fn decoded(&self) -> &DecodedPortableClaims {
        &self.decoded
    }

    pub(crate) fn browser_caller(&self) -> Principal {
        self.browser_caller
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        DecodedPortableClaims,
        &'a super::PortableClaimsContext<'ctx>,
        Principal,
    ) {
        (self.decoded, self.context, self.browser_caller)
    }
}
