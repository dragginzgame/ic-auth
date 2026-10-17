//! One typed credential boundary over the complete portable proof verifiers.
//! The host selects enrollment independently of the submitted proof variant.

use crate::{
    admission_budget::SessionAdmissionBudget,
    portable::{DecodedPortableClaims, PortableEncodingError, VerifiedPortableClaims},
};
use ic_auth_protocol_types::{PortableCredential, PortableProof};
use thiserror::Error;

/// Protected enrolled verifier selection. Only compiled capabilities can be
/// selected; the request cannot choose a key or enable a different mechanism.
pub enum PortableVerificationContext<'a> {
    #[cfg(feature = "portable-signature-verification")]
    Ed25519 {
        claims: &'a crate::portable::PortableClaimsContext<'a>,
        public_key: &'a [u8; 32],
    },
    #[cfg(feature = "certified-proof-verification")]
    Certified(&'a crate::portable_certified::PortableCertifiedContext<'a>),
}

/// A transport or complete proof refusal, never session authorization.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum PortableCredentialError {
    #[error("credential proof does not match the protected enrolled mechanism")]
    MechanismMismatch,
    #[cfg(feature = "portable-signature-verification")]
    #[error(transparent)]
    Ed25519(#[from] crate::portable_signature::PortableSignatureError),
    #[cfg(feature = "certified-proof-verification")]
    #[error(transparent)]
    Certified(#[from] crate::portable_certified::PortableCertifiedError),
}

/// Verify the envelope using the host-selected mechanism and the retained shared
/// budget. Mismatched variants fail before cryptography or budget consumption.
/// Claims, proof bounds, time and key checks remain owned by the complete verifier.
/// Outer Candid decoding needs a host allocation/instruction limit separately.
/// For session admission, use the callback's DecodedPortableClaims::verify instead
/// with its supplied budget and the same protected authority snapshot. This entry
/// point independently decodes a standalone credential.
pub fn verify_credential(
    budget: &mut SessionAdmissionBudget,
    credential: &PortableCredential,
    context: &PortableVerificationContext<'_>,
) -> Result<VerifiedPortableClaims, PortableCredentialError> {
    verify_claims(
        budget,
        credential.claims.len(),
        || DecodedPortableClaims::decode(&credential.claims),
        &credential.proof,
        context,
    )
}

impl crate::portable::DecodedPortableClaims {
    /// Verify the engine's exact canonical claims with the protected enrolled
    /// mechanism and its supplied shared budget. This consumes the decode without
    /// parsing, hashing or cloning claims again. Success is not session admission.
    pub fn verify(
        self,
        budget: &mut SessionAdmissionBudget,
        proof: &PortableProof,
        context: &PortableVerificationContext<'_>,
    ) -> Result<VerifiedPortableClaims, PortableCredentialError> {
        // Only the certified profile needs the encoded length for its aggregate
        // proof cap. Keep that metadata absent from Ed25519-only decoded values.
        #[cfg(feature = "certified-proof-verification")]
        let encoded_len = self.encoded_len;
        #[cfg(not(feature = "certified-proof-verification"))]
        let encoded_len = 0;
        verify_claims(budget, encoded_len, || Ok(self), proof, context)
    }
}

// Select authority/proof once. Decode lazily so mismatch and certified aggregate
// bounds retain precedence over codec work, while admission reuses its decode.
fn verify_claims(
    budget: &mut SessionAdmissionBudget,
    encoded_len: usize,
    decode: impl FnOnce() -> Result<DecodedPortableClaims, PortableEncodingError>,
    proof: &PortableProof,
    context: &PortableVerificationContext<'_>,
) -> Result<VerifiedPortableClaims, PortableCredentialError> {
    #[cfg(not(feature = "certified-proof-verification"))]
    let _ = encoded_len;
    match (context, proof) {
        #[cfg(feature = "portable-signature-verification")]
        (
            PortableVerificationContext::Ed25519 { claims, public_key },
            PortableProof::Ed25519 { signature },
        ) => Ok(crate::portable_signature::verify_decoded(
            budget,
            decode().map_err(crate::portable_signature::PortableSignatureError::from)?,
            signature,
            claims,
            public_key,
        )?),
        #[cfg(feature = "certified-proof-verification")]
        (
            PortableVerificationContext::Certified(context),
            PortableProof::Certified {
                delegation,
                issuer_signature,
            },
        ) => {
            crate::portable_certified::check_bounds(
                encoded_len,
                delegation,
                issuer_signature,
                context,
            )?;
            Ok(crate::portable_certified::verify_decoded(
                budget,
                decode().map_err(crate::portable_certified::PortableCertifiedError::from)?,
                delegation,
                issuer_signature,
                context,
            )?)
        }
        _ => Err(PortableCredentialError::MechanismMismatch),
    }
}
