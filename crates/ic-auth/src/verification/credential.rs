//! Protected verifier selection for an owned, untrusted credential envelope.

use super::{VerificationBudget, VerifiedClaims};
use ic_auth_protocol_types::{Credential, CredentialProof};
use std::{error::Error, fmt};

///
/// VerificationProfile
///
/// Verifier selected from protected enrollment, never from the submitted proof tag.
/// Construct this from the host's current isolated authority snapshot. A public
/// constructor/variant cannot establish provenance, readiness or authority itself.
///

#[derive(Clone, Copy, Debug)]
pub enum VerificationProfile<'a> {
    /// Independently enrolled Ed25519 authority and parsed issuer key.
    #[cfg(feature = "ed25519-verification")]
    Ed25519 {
        /// Protected claims context, authenticated caller and trusted time.
        context: &'a super::VerificationContext<'a>,
        /// Protected parsed issuer key, whose binding must match the context.
        key: &'a super::Ed25519Key,
    },
    /// Independently enrolled issuer/root authority and protected IC network root.
    #[cfg(feature = "certified-verification")]
    Certified(&'a super::CertifiedContext<'a>),
}

#[cfg(feature = "receiver-memory")]
impl<'a> VerificationProfile<'a> {
    pub(crate) fn context(self) -> &'a super::VerificationContext<'a> {
        match self {
            #[cfg(feature = "ed25519-verification")]
            Self::Ed25519 { context, .. } => context,
            #[cfg(feature = "certified-verification")]
            Self::Certified(context) => &context.claims,
        }
    }
}

// Admission already owns the canonical decode and lazy exact-byte fingerprint.
// No public prepared input or caller-supplied verification callback is exposed.
#[cfg(feature = "receiver-memory")]
pub(crate) fn verify_decoded(
    decoded: super::DecodedClaims<'_>,
    proof: &CredentialProof,
    profile: VerificationProfile<'_>,
    budget: &mut VerificationBudget,
) -> Result<VerifiedClaims, CredentialError> {
    match (profile, proof) {
        #[cfg(feature = "ed25519-verification")]
        (VerificationProfile::Ed25519 { context, key }, CredentialProof::Ed25519(signature)) => {
            super::ed25519::verify_decoded(decoded, signature, context, key, budget)
                .map_err(CredentialError::Ed25519)
        }
        #[cfg(feature = "certified-verification")]
        (VerificationProfile::Certified(context), CredentialProof::Certified(proof)) => {
            let proof = proof.as_ref().into();
            super::certified::check_bounds(decoded.encoded_len(), proof, context)
                .map_err(CredentialError::Certified)?;
            super::certified::verify_decoded(decoded, proof, context, budget)
                .map_err(CredentialError::Certified)
        }
        _ => Err(CredentialError::ProofMismatch),
    }
}

///
/// CredentialError
///
/// Unified proof refusal, preserving mechanism-specific failure stages and charges.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialError {
    /// Submitted proof tag does not match the independently selected verifier.
    ProofMismatch,
    /// Ed25519 refusal from the existing complete verifier.
    #[cfg(feature = "ed25519-verification")]
    Ed25519(super::VerificationError),
    /// Certified refusal from the existing complete verifier.
    #[cfg(feature = "certified-verification")]
    Certified(super::CertifiedError),
}

impl fmt::Display for CredentialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProofMismatch => f.write_str("proof does not match protected verifier selection"),
            #[cfg(feature = "ed25519-verification")]
            Self::Ed25519(error) => error.fmt(f),
            #[cfg(feature = "certified-verification")]
            Self::Certified(error) => error.fmt(f),
        }
    }
}

impl Error for CredentialError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProofMismatch => None,
            #[cfg(feature = "ed25519-verification")]
            Self::Ed25519(error) => Some(error),
            #[cfg(feature = "certified-verification")]
            Self::Certified(error) => Some(error),
        }
    }
}

/// Verify a credential with a host-selected mechanism and one retained work budget.
///
/// A mismatched proof tag refuses before claims decoding, clock observation or
/// charging. Matching proofs follow their existing bounded complete verifier and
/// preserve its error/charge semantics. No claims are re-encoded or buffers cloned.
/// Success remains proof evidence only: this does not create a session, choose
/// enrollment, consume replay, apply a policy result or publish a storage change.
pub fn verify_credential(
    credential: &Credential,
    profile: VerificationProfile<'_>,
    budget: &mut VerificationBudget,
) -> Result<VerifiedClaims, CredentialError> {
    match (profile, &credential.proof) {
        #[cfg(feature = "ed25519-verification")]
        (VerificationProfile::Ed25519 { context, key }, CredentialProof::Ed25519(signature)) => {
            super::verify_ed25519(&credential.claims, signature, context, key, budget)
                .map_err(CredentialError::Ed25519)
        }
        #[cfg(feature = "certified-verification")]
        (VerificationProfile::Certified(context), CredentialProof::Certified(proof)) => {
            super::verify_certified(&credential.claims, proof.as_ref().into(), context, budget)
                .map_err(CredentialError::Certified)
        }
        _ => Err(CredentialError::ProofMismatch),
    }
}
