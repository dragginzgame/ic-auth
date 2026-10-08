//! IC canister signatures, with explicit protected trust and freshness inputs.
//!
//! The host chooses the signing canister, seed hash, network root, clock and
//! limits independently of the submitted proof. No clock, storage, certification
//! root publication or signing operation is performed here. Successful signature
//! verification authenticates only the supplied message; it grants no authority.

use ic_auth_protocol_types::{IcCanisterSignatureProofV1, Principal};
use ic_canister_sig_creation::CanisterSigPublicKey;
use ic_certification::{Certificate, LookupResult};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Protected inputs for one verification. Reconstruct these on every call from
/// authenticated configuration/state, including after a policy change.
#[derive(Clone, Debug)]
pub struct CanisterSignaturePolicy<'a> {
    /// Approved signer, never inferred from the proof's DER key.
    pub signing_canister: Principal,
    /// Approved SHA-256 hash of the key's seed. For application tokens this must
    /// come from an authenticated delegation binding or protected issuer policy.
    pub seed_hash: [u8; 32],
    /// Protected network trust anchor: raw 96-byte BLS public key, not DER.
    pub ic_root_public_key_raw: &'a [u8],
    /// Host clock in nanoseconds since the Unix epoch.
    pub now_ns: u64,
    /// Largest permitted age of the signing certificate, inclusive.
    pub max_certificate_age_ns: u64,
    /// Largest permitted future clock skew, inclusive.
    pub max_future_skew_ns: u64,
    /// Maximum CBOR signature bytes, checked before decoding.
    pub max_signature_bytes: usize,
    /// Maximum signed message bytes, checked before hashing.
    pub max_message_bytes: usize,
}

/// Maintained failure categories; upstream diagnostic strings are not contracts.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CanisterSignatureError {
    #[error("signature exceeds the host's byte limit")]
    SignatureTooLarge,
    #[error("message exceeds the host's byte limit")]
    MessageTooLarge,
    #[error("invalid or unsupported canister signature public key DER")]
    InvalidPublicKey,
    #[error("signing canister does not match protected authority")]
    CanisterMismatch,
    #[error("signature seed does not match protected authority")]
    SeedMismatch,
    #[error("network root public key must contain 96 raw bytes")]
    InvalidRootKeyLength,
    #[error("invalid canister signature or certificate: {0}")]
    InvalidSignature(String),
    #[error("certificate time is absent or not a complete u64 LEB128 value")]
    InvalidCertificateTime,
    #[error("signing certificate is too old")]
    CertificateTooOld,
    #[error("signing certificate is too far in the future")]
    CertificateInFuture,
    #[error("signature domain exceeds the IC's one-byte length prefix")]
    DomainTooLong,
}

/// Prefix a payload hash with the IC signature domain's one-byte length and
/// domain bytes. Canic's existing issuer domain is `canic-issuer-delegated-token`;
/// its role-attestation domain is `canic-root-role-attestation`. Hosts retain
/// domain selection; neither domain is an IC ingress delegation.
pub fn domain_separated_message(
    domain: &[u8],
    payload_hash: [u8; 32],
) -> Result<Vec<u8>, CanisterSignatureError> {
    let length = u8::try_from(domain.len()).map_err(|_| CanisterSignatureError::DomainTooLong)?;
    let mut message = Vec::with_capacity(1 + domain.len() + payload_hash.len());
    message.push(length);
    message.extend_from_slice(domain);
    message.extend_from_slice(&payload_hash);
    Ok(message)
}

/// Verify the exact message, expected signer/seed, certified witness, subnet
/// authority and BLS signature, then enforce signing-certificate freshness.
///
/// Supports the upstream parser's short-form DER key encoding (20–129 bytes).
/// Re-encoding rejects inconsistent DER lengths, tags and padding. The size
/// check also protects the upstream parser from its unchecked short-input slice.
/// The envelope is bounded by the host; CBOR retains its default recursion limit.
/// A subnet delegation is authenticated by the upstream verifier, but its older
/// root certificate is not subjected to the signing-certificate freshness window.
/// This operation has no state effects and does not verify an application token.
pub fn verify_canister_signature(
    message: &[u8],
    proof: &IcCanisterSignatureProofV1,
    policy: &CanisterSignaturePolicy<'_>,
) -> Result<(), CanisterSignatureError> {
    if proof.signature_cbor.len() > policy.max_signature_bytes {
        return Err(CanisterSignatureError::SignatureTooLarge);
    }
    if message.len() > policy.max_message_bytes {
        return Err(CanisterSignatureError::MessageTooLarge);
    }
    // Upstream encoding uses one-byte DER lengths. Long-form keys are outside
    // this supported boundary; both Canic seeds fit the short-form encoding.
    if !(20..=129).contains(&proof.public_key_der.len()) {
        return Err(CanisterSignatureError::InvalidPublicKey);
    }
    let key = CanisterSigPublicKey::try_from(proof.public_key_der.as_slice())
        .map_err(|_| CanisterSignatureError::InvalidPublicKey)?;
    if key.to_der() != proof.public_key_der {
        return Err(CanisterSignatureError::InvalidPublicKey);
    }
    if key.canister_id != policy.signing_canister {
        return Err(CanisterSignatureError::CanisterMismatch);
    }
    if <[u8; 32]>::from(Sha256::digest(&key.seed)) != policy.seed_hash {
        return Err(CanisterSignatureError::SeedMismatch);
    }
    if policy.ic_root_public_key_raw.len() != 96 {
        return Err(CanisterSignatureError::InvalidRootKeyLength);
    }
    ic_signature_verification::verify_canister_sig(
        message,
        &proof.signature_cbor,
        &proof.public_key_der,
        policy.ic_root_public_key_raw,
    )
    .map_err(CanisterSignatureError::InvalidSignature)?;

    // Only interpret the timestamp as authenticated after upstream verification
    // has checked the certificate, witness and exact seed/message tree path.
    let envelope: SignatureEnvelope = serde_cbor::from_slice(&proof.signature_cbor)
        .map_err(|err| CanisterSignatureError::InvalidSignature(err.to_string()))?;
    let certificate: Certificate = serde_cbor::from_slice(&envelope.certificate)
        .map_err(|err| CanisterSignatureError::InvalidSignature(err.to_string()))?;
    let LookupResult::Found(mut time_bytes) = certificate.tree.lookup_path([b"time".as_slice()])
    else {
        return Err(CanisterSignatureError::InvalidCertificateTime);
    };
    // A u64 takes at most ten bytes. Reject overflow/truncation/trailing bytes.
    if time_bytes.len() > 10 {
        return Err(CanisterSignatureError::InvalidCertificateTime);
    }
    let time_ns = leb128::read::unsigned(&mut time_bytes)
        .map_err(|_| CanisterSignatureError::InvalidCertificateTime)?;
    if !time_bytes.is_empty() {
        return Err(CanisterSignatureError::InvalidCertificateTime);
    }
    if time_ns <= policy.now_ns {
        if policy.now_ns - time_ns > policy.max_certificate_age_ns {
            return Err(CanisterSignatureError::CertificateTooOld);
        }
    } else if time_ns - policy.now_ns > policy.max_future_skew_ns {
        return Err(CanisterSignatureError::CertificateInFuture);
    }
    Ok(())
}

#[derive(Deserialize)]
struct SignatureEnvelope {
    certificate: serde_bytes::ByteBuf,
}
