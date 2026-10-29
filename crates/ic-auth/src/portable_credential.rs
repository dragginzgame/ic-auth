//! One typed credential boundary over the complete portable proof verifiers.
//! The host selects enrollment independently of the submitted proof variant.

use crate::{admission_budget::SessionAdmissionBudget, portable::VerifiedPortableClaims};
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
impl<'ctx> PortableVerificationContext<'ctx> {
    /// The exact protected claims context used by the selected complete verifier.
    #[cfg(feature = "sessions")]
    pub(crate) fn claims(&self) -> &crate::portable::PortableClaimsContext<'ctx> {
        match self {
            #[cfg(feature = "portable-signature-verification")]
            Self::Ed25519 { claims, .. } => claims,
            #[cfg(feature = "certified-proof-verification")]
            Self::Certified(context) => &context.claims,
        }
    }
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
/// Outer transport decoding needs a host allocation/instruction limit separately.
/// Session admission owns verification and reuses its canonical decode. This
/// entry point independently decodes a standalone credential.
/// Keep this raw dispatch separate: the complete verifier checks certified
/// envelope bounds before decoding, while admission must first resolve retries.
pub fn verify_credential(
    budget: &mut SessionAdmissionBudget,
    credential: &PortableCredential,
    context: &PortableVerificationContext<'_>,
) -> Result<VerifiedPortableClaims, PortableCredentialError> {
    match (context, &credential.proof) {
        #[cfg(feature = "portable-signature-verification")]
        (
            PortableVerificationContext::Ed25519 { claims, public_key },
            PortableProof::Ed25519 { signature },
        ) => Ok(crate::portable_signature::verify_ed25519_claims(
            budget,
            &credential.claims,
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
        ) => Ok(crate::portable_certified::verify_certified_claims(
            budget,
            &credential.claims,
            delegation,
            issuer_signature,
            context,
        )?),
        _ => Err(PortableCredentialError::MechanismMismatch),
    }
}

/// Prepared only inside one serialized engine call. The factory binds the
/// checked claims to the exact verifier selection used after cheap store checks.
#[cfg(feature = "sessions")]
pub(crate) struct PreparedCredential<'a, 'ctx> {
    claims: crate::portable::ContextualClaims<'a, 'ctx>,
    verification: &'a PortableVerificationContext<'ctx>,
}

#[cfg(feature = "sessions")]
impl<'ctx> PortableVerificationContext<'ctx> {
    pub(crate) fn prepare<'a>(
        &'a self,
        decoded: crate::portable::DecodedPortableClaims,
    ) -> Result<PreparedCredential<'a, 'ctx>, crate::portable::PortableClaimsError> {
        Ok(PreparedCredential {
            claims: decoded.validate_context(self.claims())?,
            verification: self,
        })
    }
}

#[cfg(feature = "sessions")]
impl PreparedCredential<'_, '_> {
    pub(crate) fn decoded(&self) -> &crate::portable::DecodedPortableClaims {
        self.claims.decoded()
    }

    pub(crate) fn contextual(&self) -> &crate::portable::ContextualClaims<'_, '_> {
        &self.claims
    }

    pub(crate) fn verify(
        self,
        budget: &mut SessionAdmissionBudget,
        proof: &PortableProof,
    ) -> Result<VerifiedPortableClaims, PortableCredentialError> {
        match (self.verification, proof) {
            #[cfg(feature = "portable-signature-verification")]
            (
                PortableVerificationContext::Ed25519 { public_key, .. },
                PortableProof::Ed25519 { signature },
            ) => Ok(crate::portable_signature::verify_prepared(
                budget,
                self.claims,
                signature,
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
                    self.claims.decoded().encoded_len,
                    delegation,
                    issuer_signature,
                    context,
                )?;
                Ok(crate::portable_certified::verify_prepared(
                    budget,
                    self.claims,
                    delegation,
                    issuer_signature,
                    context,
                )?)
            }
            _ => Err(PortableCredentialError::MechanismMismatch),
        }
    }
}
