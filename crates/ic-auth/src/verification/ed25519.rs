//! Strict Ed25519 proof verification through Platform.
use super::{
    BudgetError, ContextError, DecodedClaims, VerificationBudget, VerificationContext,
    VerifiedClaims, context,
};
use crate::claims::ClaimsError;
use ic_auth_protocol_types::PortableMechanism;
use ic_platform::ed25519::{Scalar, Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use std::{error::Error, fmt};

///
/// VerificationError
///
/// Failure of fresh proof verification; not a session-admission projection.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationError {
    /// Bounded canonical claims decoding failed before charging.
    Encoding(ClaimsError),
    /// Protected context/claims checks failed before charging.
    Context(ContextError),
    /// The proposed protected issuer key is malformed or weak.
    InvalidPublicKey,
    /// Signature length is not exactly 64 bytes; no work is charged.
    MalformedSignature,
    /// Noncanonical scalar (uncharged) or failed strict verification (charged).
    InvalidSignature,
    /// The host-owned work allowance refused the attempt before cryptography.
    Budget(BudgetError),
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encoding(error) => error.fmt(f),
            Self::Context(error) => error.fmt(f),
            Self::Budget(error) => error.fmt(f),
            Self::InvalidPublicKey => f.write_str("invalid or weak Ed25519 public key"),
            Self::MalformedSignature => f.write_str("Ed25519 signature must be exactly 64 bytes"),
            Self::InvalidSignature => f.write_str("invalid Ed25519 signature"),
        }
    }
}

impl Error for VerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encoding(error) => Some(error),
            Self::Context(error) => Some(error),
            Self::Budget(error) => Some(error),
            _ => None,
        }
    }
}

///
/// Ed25519Key
///
/// Parsed, nonweak issuer key and SHA-256 binding; this does not enroll the key.
/// Keep it with protected enrollment and reconstruct it after configuration
/// decoding. No unchecked deserializer or mutable key access is provided.
/// ```compile_fail
/// use ic_auth::verification::Ed25519Key;
/// fn change(key: &mut Ed25519Key) { key.as_bytes()[0] = 0; }
/// ```
///

#[derive(Debug, Eq, PartialEq)]
pub struct Ed25519Key {
    key: VerifyingKey,
    binding: [u8; 32],
}

impl Ed25519Key {
    /// Parse through Platform and reject weak public keys before use.
    pub fn new(bytes: [u8; 32]) -> Result<Self, VerificationError> {
        let key =
            VerifyingKey::from_bytes(&bytes).map_err(|_| VerificationError::InvalidPublicKey)?;
        if key.is_weak() {
            return Err(VerificationError::InvalidPublicKey);
        }
        Ok(Self {
            key,
            binding: Sha256::digest(bytes).into(),
        })
    }

    /// Exact admitted key bytes, without normalization or authority claims.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.key.as_bytes()
    }

    /// Key binding compared with protected enrollment and signed claims.
    #[must_use]
    pub const fn binding(&self) -> [u8; 32] {
        self.binding
    }
}

/// Verify ordinary strict Ed25519 over SHA-256 of exact canonical claims bytes.
///
/// `host` and `key` must come from protected enrollment and actual runtime inputs.
/// Decode, mechanism/context, key binding, length and canonical-scalar checks all
/// precede charging. The retained budget is charged immediately before Platform's
/// strict verifier; invalid signatures and small-order R consume credit. This is
/// ordinary Ed25519 over a 32-byte message, never Ed25519ph or a raw-claims signature.
/// No replay/login/session state is read or changed here.
pub fn verify_ed25519(
    encoded_claims: &[u8],
    signature: &[u8],
    host: &VerificationContext<'_>,
    key: &Ed25519Key,
    budget: &mut VerificationBudget,
) -> Result<VerifiedClaims, VerificationError> {
    verify_decoded(
        DecodedClaims::new(encoded_claims).map_err(VerificationError::Encoding)?,
        signature,
        host,
        key,
        budget,
    )
}

pub(super) fn verify_decoded(
    decoded: DecodedClaims<'_>,
    signature: &[u8],
    host: &VerificationContext<'_>,
    key: &Ed25519Key,
    budget: &mut VerificationBudget,
) -> Result<VerifiedClaims, VerificationError> {
    use VerificationError as E;
    let claims = &decoded.claims;
    if host.mechanism != PortableMechanism::Ed25519 {
        return Err(E::Context(ContextError::MechanismMismatch));
    }
    let caller = context::check(claims, host).map_err(E::Context)?;
    if key.binding != host.key_binding_id {
        return Err(E::Context(ContextError::KeyMismatch));
    }
    let signature = Signature::from_slice(signature).map_err(|_| E::MalformedSignature)?;
    // Platform's upstream scalar admission remains strict even if a downstream
    // enables Dalek legacy compatibility through Cargo feature unification.
    if bool::from(Scalar::from_canonical_bytes(*signature.s_bytes()).is_none()) {
        return Err(E::InvalidSignature);
    }
    let fingerprint = decoded.fingerprint();
    budget
        .charge(host.now_ns, PortableMechanism::Ed25519)
        .map_err(E::Budget)?;
    key.key
        .verify_strict(&fingerprint, &signature)
        .map_err(|_| E::InvalidSignature)?;
    Ok(VerifiedClaims::from_verification(
        decoded.claims,
        fingerprint,
        caller,
        host,
    ))
}
