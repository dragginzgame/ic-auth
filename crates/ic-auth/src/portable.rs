//! Canonical portable credential claims and explicit protected-context checks.
//! Encoding, decoding and hashing alone do not verify time or authority.
//! Context checks do not verify signatures, enrollment provenance or revocation.
//! No value returned by this module is sufficient for session admission.

#[cfg(any(
    feature = "portable-signature-verification",
    feature = "certified-proof-verification"
))]
mod decoded;
mod grants;
mod revocation;
mod rules;
mod verified;
#[cfg(any(
    feature = "portable-signature-verification",
    feature = "certified-proof-verification"
))]
pub(crate) use decoded::{ContextualClaims, DecodedPortableClaims};
pub use grants::ValidatedGrants;
pub use revocation::{
    LoginBinding, LoginRecordState, LoginRevocationChange, LoginRevocationError,
    MemoryLoginRevocations,
};
pub use rules::{
    PortableAuthority, PortableClaimsContext, PortableClaimsError, validate_claims_context,
};
pub use verified::VerifiedPortableClaims;

use crate::canonical::{CanonicalAuthError, encode_principal, encode_role_grants};
use crate::certificate_issuance::{GrantValidationError, validate_grants};
use crate::limits::{MAX_GRANTS, MAX_LABEL_BYTES, MAX_SCOPES_PER_GRANT};
use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};
use sha2::{Digest, Sha256};
use thiserror::Error;

const DOMAIN: &[u8] = b"IC-AUTH-PORTABLE-CREDENTIAL\0";
const SCHEMA: u16 = 1;

/// Hard cap on the signed claims blob, independent of outer transport limits.
pub use ic_auth_protocol_types::limits::MAX_CLAIMS_BYTES;

/// Structural codec errors, never proof-verification outcomes.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PortableEncodingError {
    #[error("portable claims exceed their byte bound")]
    TooLarge,
    #[error("truncated portable claims")]
    Truncated,
    #[error("wrong portable domain, schema or mechanism")]
    WrongDomain,
    #[error("portable claims contain trailing bytes")]
    TrailingBytes,
    #[error("invalid portable principal, label or count")]
    InvalidShape,
    #[error("anonymous account subject")]
    AnonymousSubject,
    #[error(transparent)]
    Grants(#[from] GrantValidationError),
    #[error(transparent)]
    Canonical(#[from] CanonicalAuthError),
}

/// Encode exactly the frozen 0.6 claims domain. Never sort or normalize input.
/// Time relationships and protected authority are verification responsibilities.
pub fn encode_claims(claims: &PortableClaims) -> Result<Vec<u8>, PortableEncodingError> {
    let mut bytes = Vec::new();
    encode_claims_into(claims, &mut |part| bytes.extend_from_slice(part))?;
    Ok(bytes)
}

fn encode_claims_into(
    claims: &PortableClaims,
    sink: &mut crate::canonical::Sink<'_>,
) -> Result<(), PortableEncodingError> {
    validate_shape(claims)?;
    // The same bounded field traversal produces bytes or feeds SHA-256.
    let mut length = 0usize;
    let mut out = |bytes: &[u8]| {
        length = length.saturating_add(bytes.len());
        sink(bytes);
    };
    out(DOMAIN);
    out(&SCHEMA.to_be_bytes());
    out(&[match claims.mechanism {
        PortableMechanism::Certified => 0,
        PortableMechanism::Ed25519 => 1,
    }]);
    out(claims.audience.canonical_network_id.as_bytes());
    out(claims.audience.fleet_id.as_bytes());
    encode_principal(&mut out, claims.issuer)?;
    out(&claims.key_binding_id);
    out(&claims.browser_public_key);
    encode_principal(&mut out, claims.account_subject)?;
    for time in [
        claims.login_started_at_ns,
        claims.login_deadline_ns,
        claims.issued_at_ns,
        claims.admission_expires_at_ns,
    ] {
        out(&time.to_be_bytes());
    }
    out(&claims.nonce);
    encode_role_grants::<true>(&mut out, &claims.grants)?;
    if length > MAX_CLAIMS_BYTES {
        return Err(PortableEncodingError::TooLarge);
    }
    Ok(())
}

/// Decode bounded canonical claims. Lengths/counts are admitted before allocation;
/// unknown schema/mechanism, trailing bytes and noncanonical grants are rejected.
/// This is deliberately not a deserializer for the previous token domain.
pub fn decode_claims(bytes: &[u8]) -> Result<PortableClaims, PortableEncodingError> {
    if bytes.len() > MAX_CLAIMS_BYTES {
        return Err(PortableEncodingError::TooLarge);
    }
    let mut input = Reader(bytes);
    if input.take(DOMAIN.len())? != DOMAIN || input.array::<2>()? != SCHEMA.to_be_bytes() {
        return Err(PortableEncodingError::WrongDomain);
    }
    let mechanism = match input.array::<1>()? {
        [0] => PortableMechanism::Certified,
        [1] => PortableMechanism::Ed25519,
        _ => return Err(PortableEncodingError::WrongDomain),
    };
    let audience = AudienceId {
        canonical_network_id: CanonicalId::from_bytes(input.array()?),
        fleet_id: CanonicalId::from_bytes(input.array()?),
    };
    let issuer = input.principal()?;
    let key_binding_id = input.array()?;
    let browser_public_key = input.array()?;
    let account_subject = input.principal()?;
    let login_started_at_ns = u64::from_be_bytes(input.array()?);
    let login_deadline_ns = u64::from_be_bytes(input.array()?);
    let issued_at_ns = u64::from_be_bytes(input.array()?);
    let admission_expires_at_ns = u64::from_be_bytes(input.array()?);
    let nonce = input.array()?;
    let count = input.count(MAX_GRANTS)?;
    let mut grants = Vec::with_capacity(count);
    for _ in 0..count {
        let target = input
            .label()?
            .parse()
            .map_err(|_| PortableEncodingError::InvalidShape)?;
        let count = input.count(MAX_SCOPES_PER_GRANT)?;
        let mut scopes = Vec::with_capacity(count);
        for _ in 0..count {
            scopes.push(
                input
                    .label()?
                    .parse()
                    .map_err(|_| PortableEncodingError::InvalidShape)?,
            );
        }
        grants.push(DelegatedRoleGrant { target, scopes });
    }
    if !input.0.is_empty() {
        return Err(PortableEncodingError::TrailingBytes);
    }
    let claims = PortableClaims {
        mechanism,
        audience,
        issuer,
        key_binding_id,
        browser_public_key,
        account_subject,
        login_started_at_ns,
        login_deadline_ns,
        issued_at_ns,
        admission_expires_at_ns,
        nonce,
        grants,
    };
    validate_shape(&claims)?;
    Ok(claims)
}

/// SHA-256 of the exact canonical signed blob, for signing and replay identity.
/// The digest authenticates nothing by itself; signature bytes are not included.
pub fn claims_hash(claims: &PortableClaims) -> Result<[u8; 32], PortableEncodingError> {
    let mut hash = Sha256::new();
    encode_claims_into(claims, &mut |bytes| hash.update(bytes))?;
    Ok(hash.finalize().into())
}

/// Derive the IC sender for a directly authenticated Ed25519 browser key.
/// Hosts must compare this with the actual ingress caller, not a client field.
/// This does not validate curve membership or possession of the key.
pub fn browser_principal(public_key: &[u8; 32]) -> Principal {
    // RFC 8410 Ed25519 SubjectPublicKeyInfo: SEQUENCE(algorithm OID, BIT STRING).
    let mut der = [0; 44];
    der[..12].copy_from_slice(&[
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0,
    ]);
    der[12..].copy_from_slice(public_key);
    Principal::self_authenticating(der)
}

// A login caller must have the complete self-authenticating principal shape.
// This checks neither ingress provenance nor possession of the browser key.
pub(crate) fn is_login_caller(caller: Principal) -> bool {
    let bytes = caller.as_slice();
    bytes.len() == 29 && bytes.last() == Some(&2)
}

fn validate_shape(claims: &PortableClaims) -> Result<(), PortableEncodingError> {
    if claims.account_subject == Principal::anonymous() {
        return Err(PortableEncodingError::AnonymousSubject);
    }
    validate_grants(&claims.grants, MAX_GRANTS, MAX_SCOPES_PER_GRANT)?;
    Ok(())
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], PortableEncodingError> {
        let (head, tail) = self
            .0
            .split_at_checked(len)
            .ok_or(PortableEncodingError::Truncated)?;
        self.0 = tail;
        Ok(head)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], PortableEncodingError> {
        self.take(N)?
            .try_into()
            .map_err(|_| PortableEncodingError::Truncated)
    }

    fn length(&mut self, max: usize) -> Result<usize, PortableEncodingError> {
        let len = u32::from_be_bytes(self.array()?);
        let len = usize::try_from(len).map_err(|_| PortableEncodingError::InvalidShape)?;
        if len > max {
            return Err(PortableEncodingError::InvalidShape);
        }
        Ok(len)
    }

    fn count(&mut self, max: usize) -> Result<usize, PortableEncodingError> {
        let count = self.length(max)?;
        if count == 0 {
            return Err(PortableEncodingError::InvalidShape);
        }
        Ok(count)
    }

    fn principal(&mut self) -> Result<Principal, PortableEncodingError> {
        let len = self.length(29)?;
        Ok(Principal::from_slice(self.take(len)?))
    }

    fn label(&mut self) -> Result<&'a str, PortableEncodingError> {
        let len = self.count(MAX_LABEL_BYTES)?;
        std::str::from_utf8(self.take(len)?).map_err(|_| PortableEncodingError::InvalidShape)
    }
}

#[cfg(test)]
mod tests;
