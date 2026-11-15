//! Bounded schema-2 signed bytes. Decode, encode and hashing grant no authority.
//!
//! Only the current fresh-credential format is supported. Signed labels remain
//! in supplied order; unlike unsigned session requests, they are never sorted.

use ic_auth_protocol_types::{
    AudienceId, CanonicalId, DelegatedRoleGrant, PortableClaims, PortableMechanism, Principal,
};
use sha2::{Digest, Sha256};
use std::{error::Error, fmt};

const DOMAIN: &[u8] = b"IC-AUTH-PORTABLE-CREDENTIAL\0";
const SCHEMA: u16 = 2;
const MAX_ROLES: usize = 16;
const MAX_SCOPES_PER_ROLE: usize = 32;
const MAX_SCOPE_ENTRIES: usize = 32;
const MAX_LABEL_BYTES: usize = 64;
const MAX_GRANT_BYTES: usize = 4096;

/// Signed-claims byte ceiling, independent of the outer Wire and proof budgets.
pub const MAX_CLAIMS_BYTES: usize = 65_536;

// Bounded labels/counts imply the complete grant list fits without measuring it
// with another encoder. Principal lengths and all other fields are fixed/bounded.
const _: () = assert!(
    4 + MAX_ROLES * (8 + MAX_LABEL_BYTES) + MAX_SCOPE_ENTRIES * (4 + MAX_LABEL_BYTES)
        <= MAX_GRANT_BYTES
);
// The schema-2 prefix is at most 282 bytes with two maximum-length principals.
const _: () = assert!(282 + MAX_GRANT_BYTES <= MAX_CLAIMS_BYTES);

/// Structural rejection, distinct from signature, time and enrollment failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimsError {
    /// The entire blob or remaining grant payload exceeds its defensive byte cap.
    TooLarge,
    /// A required field is incomplete.
    Truncated,
    /// Domain, schema or mechanism is unsupported; no legacy fallback is tried.
    WrongDomain,
    /// Data follows the complete record.
    TrailingBytes,
    /// A length, label, principal or device fence is structurally invalid.
    InvalidShape,
    /// The claimed account subject is anonymous.
    AnonymousSubject,
    /// Roles are duplicated or not strictly ascending.
    NonCanonicalRoles,
    /// Scopes within a role are duplicated or not strictly ascending.
    NonCanonicalScopes,
    /// The in-memory grant list is empty or exceeds 16 roles.
    InvalidGrantCount {
        /// Submitted number of grants.
        found: usize,
    },
    /// An in-memory role has no scopes or exceeds 32 scopes.
    InvalidScopeCount {
        /// Submitted number of scopes in that role.
        found: usize,
    },
    /// The total number of role–scope entries exceeds the fresh-credential cap.
    TooManyScopeEntries {
        /// Cumulative entries, counting the same label under each role separately.
        found: usize,
        /// Inclusive entry ceiling.
        max: usize,
    },
}

impl fmt::Display for ClaimsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TooLarge => "claims exceed their byte bound",
            Self::Truncated => "truncated claims",
            Self::WrongDomain => "unsupported claims domain, schema or mechanism",
            Self::TrailingBytes => "trailing claims bytes",
            Self::InvalidShape => "invalid claims structure",
            Self::AnonymousSubject => "anonymous account subject",
            Self::NonCanonicalRoles => "roles must be strictly ordered and unique",
            Self::NonCanonicalScopes => "scopes must be strictly ordered and unique",
            Self::InvalidGrantCount { found } => {
                return write!(f, "grant count {found} is outside 1..=16");
            }
            Self::InvalidScopeCount { found } => {
                return write!(f, "scope count {found} is outside 1..=32");
            }
            Self::TooManyScopeEntries { found, max } => {
                return write!(f, "credential has {found} scope entries; maximum is {max}");
            }
        };
        f.write_str(message)
    }
}

impl Error for ClaimsError {}

fn check_entries(found: usize) -> Result<(), ClaimsError> {
    if found > MAX_SCOPE_ENTRIES {
        return Err(ClaimsError::TooManyScopeEntries {
            found,
            max: MAX_SCOPE_ENTRIES,
        });
    }
    Ok(())
}

fn validate(claims: &PortableClaims) -> Result<(), ClaimsError> {
    if claims.device_slot > 7 || claims.device_generation == 0 {
        return Err(ClaimsError::InvalidShape);
    }
    if claims.account_subject == Principal::anonymous() {
        return Err(ClaimsError::AnonymousSubject);
    }
    validate_grants(&claims.grants)?;
    check_entries(claims.grants.iter().map(|grant| grant.scopes.len()).sum())
}

// Protected authority may contain 32 scopes for each of 16 roles. Fresh
// credentials separately enforce the narrower aggregate 32-entry ceiling.
pub(crate) fn validate_grants(grants: &[DelegatedRoleGrant]) -> Result<(), ClaimsError> {
    if !(1..=MAX_ROLES).contains(&grants.len()) {
        return Err(ClaimsError::InvalidGrantCount {
            found: grants.len(),
        });
    }
    for (index, grant) in grants.iter().enumerate() {
        if index > 0 && grants[index - 1].target >= grant.target {
            return Err(ClaimsError::NonCanonicalRoles);
        }
        if !(1..=MAX_SCOPES_PER_ROLE).contains(&grant.scopes.len()) {
            return Err(ClaimsError::InvalidScopeCount {
                found: grant.scopes.len(),
            });
        }
        if grant.scopes.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(ClaimsError::NonCanonicalScopes);
        }
    }
    Ok(())
}

/// Encode current canonical claims after checking all structural invariants.
///
/// Labels are not normalized. Time relationships and trust require the complete
/// verifier; a valid encoding may describe an expired or unauthorized credential.
pub fn encode_claims(claims: &PortableClaims) -> Result<Vec<u8>, ClaimsError> {
    validate(claims)?;
    let mut output = Vec::new();
    write_fields(claims, |bytes| output.extend_from_slice(bytes));
    Ok(output)
}

/// Hash the exact canonical claims bytes with SHA-256, excluding proof material.
///
/// This performs the same shape checks as encoding and streams directly into
/// the published hash implementation. A digest is not verification evidence.
pub fn claims_hash(claims: &PortableClaims) -> Result<[u8; 32], ClaimsError> {
    validate(claims)?;
    let mut digest = Sha256::new();
    write_fields(claims, |bytes| digest.update(bytes));
    Ok(digest.finalize().into())
}

// One private field traversal owns the signed layout for encoding and hashing.
// Callers validate first; all casts are bounded by that admission or Principal.
fn write_fields(claims: &PortableClaims, mut emit: impl FnMut(&[u8])) {
    emit(DOMAIN);
    emit(&SCHEMA.to_be_bytes());
    emit(&[match claims.mechanism {
        PortableMechanism::Certified => 0,
        PortableMechanism::Ed25519 => 1,
    }]);
    emit(claims.audience.canonical_network_id.as_bytes());
    emit(claims.audience.fleet_id.as_bytes());
    emit(&(u32::from(claims.issuer.len())).to_be_bytes());
    emit(claims.issuer.as_slice());
    emit(&claims.key_binding_id);
    emit(&claims.browser_public_key);
    emit(&(u32::from(claims.account_subject.len())).to_be_bytes());
    emit(claims.account_subject.as_slice());
    emit(&[claims.device_slot]);
    for value in [
        claims.device_generation,
        claims.login_started_at_ns,
        claims.login_deadline_ns,
        claims.issued_at_ns,
        claims.admission_expires_at_ns,
    ] {
        emit(&value.to_be_bytes());
    }
    emit(&claims.nonce);
    emit(&(claims.grants.len() as u32).to_be_bytes());
    for grant in &claims.grants {
        emit(&(grant.target.as_str().len() as u32).to_be_bytes());
        emit(grant.target.as_str().as_bytes());
        emit(&(grant.scopes.len() as u32).to_be_bytes());
        for scope in &grant.scopes {
            emit(&(scope.as_str().len() as u32).to_be_bytes());
            emit(scope.as_str().as_bytes());
        }
    }
}

/// Decode one bounded canonical schema-2 record, without granting authority.
///
/// Count and byte limits are admitted before allocating their values. Extra
/// bytes, unknown formats and noncanonical signed order refuse without repair.
pub fn decode_claims(bytes: &[u8]) -> Result<PortableClaims, ClaimsError> {
    if bytes.len() > MAX_CLAIMS_BYTES {
        return Err(ClaimsError::TooLarge);
    }
    let mut input = Input { remaining: bytes };
    if input.take(DOMAIN.len())? != DOMAIN || u16::from_be_bytes(input.fixed()?) != SCHEMA {
        return Err(ClaimsError::WrongDomain);
    }
    let mechanism = match input.fixed()? {
        [0] => PortableMechanism::Certified,
        [1] => PortableMechanism::Ed25519,
        _ => return Err(ClaimsError::WrongDomain),
    };
    let audience = AudienceId {
        canonical_network_id: CanonicalId::from_bytes(input.fixed()?),
        fleet_id: CanonicalId::from_bytes(input.fixed()?),
    };
    let issuer = input.principal()?;
    let key_binding_id = input.fixed()?;
    let browser_public_key = input.fixed()?;
    let account_subject = input.principal()?;
    let [device_slot] = input.fixed()?;
    let device_generation = u64::from_be_bytes(input.fixed()?);
    let login_started_at_ns = u64::from_be_bytes(input.fixed()?);
    let login_deadline_ns = u64::from_be_bytes(input.fixed()?);
    let issued_at_ns = u64::from_be_bytes(input.fixed()?);
    let admission_expires_at_ns = u64::from_be_bytes(input.fixed()?);
    let nonce = input.fixed()?;
    // The remaining bytes include the whole grant list and any trailing input.
    if input.remaining.len() > MAX_GRANT_BYTES {
        return Err(ClaimsError::TooLarge);
    }
    let roles = input.size(1, MAX_ROLES)?;
    let mut grants = Vec::with_capacity(roles);
    let mut entries = 0;
    for _ in 0..roles {
        let target = input
            .label()?
            .parse()
            .map_err(|_| ClaimsError::InvalidShape)?;
        let count = input.size(1, MAX_SCOPES_PER_ROLE)?;
        entries += count;
        check_entries(entries)?;
        let mut scopes = Vec::with_capacity(count);
        for _ in 0..count {
            scopes.push(
                input
                    .label()?
                    .parse()
                    .map_err(|_| ClaimsError::InvalidShape)?,
            );
        }
        grants.push(DelegatedRoleGrant { target, scopes });
    }
    if !input.remaining.is_empty() {
        return Err(ClaimsError::TrailingBytes);
    }
    let claims = PortableClaims {
        mechanism,
        audience,
        issuer,
        key_binding_id,
        browser_public_key,
        account_subject,
        device_slot,
        device_generation,
        login_started_at_ns,
        login_deadline_ns,
        issued_at_ns,
        admission_expires_at_ns,
        nonce,
        grants,
    };
    validate(&claims)?;
    Ok(claims)
}

// Claims-only cursor: borrowed slices and checked fixed-width lengths, no generic
// serialization or storage framework. Failed reads never publish a partial value.
struct Input<'a> {
    remaining: &'a [u8],
}

impl<'a> Input<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], ClaimsError> {
        let head = self.remaining.get(..length).ok_or(ClaimsError::Truncated)?;
        self.remaining = &self.remaining[length..];
        Ok(head)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], ClaimsError> {
        self.take(N)?.try_into().map_err(|_| ClaimsError::Truncated)
    }

    fn size(&mut self, minimum: usize, maximum: usize) -> Result<usize, ClaimsError> {
        let size = usize::try_from(u32::from_be_bytes(self.fixed()?))
            .map_err(|_| ClaimsError::InvalidShape)?;
        if !(minimum..=maximum).contains(&size) {
            return Err(ClaimsError::InvalidShape);
        }
        Ok(size)
    }

    fn principal(&mut self) -> Result<Principal, ClaimsError> {
        let size = self.size(0, Principal::MAX_LENGTH_IN_BYTES)?;
        Principal::try_from_slice(self.take(size)?).map_err(|_| ClaimsError::InvalidShape)
    }

    fn label(&mut self) -> Result<&'a str, ClaimsError> {
        let size = self.size(1, MAX_LABEL_BYTES)?;
        std::str::from_utf8(self.take(size)?).map_err(|_| ClaimsError::InvalidShape)
    }
}
