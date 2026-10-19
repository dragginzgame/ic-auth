//! Pure secp256k1 key admission and compact ECDSA verification.
//!
//! These checks establish only key encoding and signature validity. The host
//! still owns key enrollment, algorithm selection, signing authorization and
//! the meaning of the supplied 32-byte prehash. Verification performs no hashing
//! or normalization. Normalization is a separate explicit producer operation.

use ic_platform::ecdsa::{PrehashVerifier, Signature, VerifyingKey};
use thiserror::Error;

/// Rejection of isolated chain-key signature material.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ChainKeySignatureError {
    #[error("malformed secp256k1 public key")]
    MalformedPublicKey,
    #[error("malformed compact secp256k1 signature")]
    MalformedSignature,
    #[error("high-S secp256k1 signature")]
    HighSSignature,
    #[error("invalid secp256k1 signature")]
    InvalidSignature,
}

/// Admit a compressed or uncompressed SEC1 secp256k1 public key using K256.
///
/// Successful admission does not enroll the key or establish trusted authority.
pub fn validate_secp256k1_public_key(public_key: &[u8]) -> Result<(), ChainKeySignatureError> {
    parse_public_key(public_key).map(|_| ())
}

/// Verify a compact 64-byte ECDSA signature over the exact supplied prehash.
///
/// Reject malformed keys first, then malformed signatures, then high-S
/// signatures, then cryptographic mismatches. High-S signatures are refused
/// rather than normalized. DER and recoverable signatures are not accepted.
pub fn verify_secp256k1_prehash(
    public_key: &[u8],
    prehash: &[u8; 32],
    signature: &[u8],
) -> Result<(), ChainKeySignatureError> {
    VerificationMaterial::parse(public_key, signature)?.verify_prehash(prehash)
}

/// Return the low-S representation of a compact 64-byte ECDSA signature.
///
/// This explicitly normalizes producer material using K256; it does not verify
/// a signature, authenticate a management reply or authorize signing. Malformed
/// encodings/scalars return [`ChainKeySignatureError::MalformedSignature`].
/// Already-low-S bytes remain identical. Strict verification still rejects
/// high-S input; it never calls this function implicitly.
pub fn normalize_secp256k1_signature(signature: &[u8]) -> Result<[u8; 64], ChainKeySignatureError> {
    let parsed = parse_signature(signature)?;
    Ok(parsed.normalize_s().unwrap_or(parsed).to_bytes().into())
}

fn parse_public_key(public_key: &[u8]) -> Result<VerifyingKey, ChainKeySignatureError> {
    VerifyingKey::from_sec1_bytes(public_key)
        .map_err(|_| ChainKeySignatureError::MalformedPublicKey)
}

fn parse_signature(signature: &[u8]) -> Result<Signature, ChainKeySignatureError> {
    Signature::from_slice(signature).map_err(|_| ChainKeySignatureError::MalformedSignature)
}

// Keep parsing separate from prehash evaluation so the complete Root verifier
// preserves malformed/high-S rejection before canonical header encoding.
pub(crate) struct VerificationMaterial {
    key: VerifyingKey,
    signature: Signature,
}

impl VerificationMaterial {
    pub(crate) fn parse(
        public_key: &[u8],
        signature: &[u8],
    ) -> Result<Self, ChainKeySignatureError> {
        let key = parse_public_key(public_key)?;
        let signature = parse_signature(signature)?;
        if signature.normalize_s().is_some() {
            return Err(ChainKeySignatureError::HighSSignature);
        }
        Ok(Self { key, signature })
    }

    pub(crate) fn verify_prehash(&self, prehash: &[u8; 32]) -> Result<(), ChainKeySignatureError> {
        self.key
            .verify_prehash(prehash, &self.signature)
            .map_err(|_| ChainKeySignatureError::InvalidSignature)
    }
}
