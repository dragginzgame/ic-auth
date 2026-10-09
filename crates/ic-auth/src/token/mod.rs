//! Verification of the existing application-token proof chain and root delegations.
//!
//! Hosts provide protected root-key enrollment, current policy, caller, clock,
//! audience, role, allowed scopes and endpoint requirements. Every invocation
//! of `verify_token` checks both proofs and live policy. Standalone
//! `verify_delegation_proof` authenticates root delegation only. No cache, storage
//! or runtime effect is performed; replay consumption and session admission remain
//! separate.

mod delegation;
mod root;

pub use delegation::{
    DelegationProofVerificationContext, DelegationProofVerificationLimits, VerifiedDelegationProof,
    verify_delegation_proof,
};
pub(crate) mod rules;

use crate::{
    canister_signature::{
        CanisterSignatureError, CanisterSignaturePolicy, domain_separated_message,
        verify_canister_signature,
    },
    canonical::CanonicalAuthError,
};
use ic_auth_protocol_types::{
    AudienceId, AuthRole, ChainKeyAlgorithm, ChainKeyKeyId, DelegatedToken, DelegatedTokenClaims,
    IssuerProof, IssuerProofBinding, Principal,
};
use thiserror::Error;

/// Protected enrollment of one root chain-key. The host selects the network's
/// permitted key names and obtains this public key independently of the proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootKeyPolicy {
    /// Independently enrolled root authority, never inferred from the proof.
    pub root_canister_id: Principal,
    /// Enrolled chain-key algorithm and network-approved key identifier.
    pub algorithm: ChainKeyAlgorithm,
    pub key_id: ChainKeyKeyId,
    /// Canonical hash of the enrolled derivation path, including its domain.
    pub derivation_path_hash: [u8; 32],
    /// Enrolled SEC1-encoded secp256k1 public key.
    pub public_key: Vec<u8>,
    /// Exact enrolled version; the submitted batch must match it.
    pub key_version: u64,
    /// Current lower bounds for authority invalidation, applied on every call.
    pub min_accepted_key_version: u64,
    pub min_accepted_proof_epoch: u64,
    pub min_accepted_registry_epoch: u64,
    /// Enrollment start, subject to the host's explicit future-skew allowance.
    pub valid_from_ns: u64,
    /// Exclusive deadline, checked on every call even for the same token.
    pub accept_until_ns: u64,
    /// Upper bound on batch lifetime, limiting stale root-grant authority.
    pub max_revocation_latency_ns: u64,
}

/// Host-selected bounds. Canic adapters retain their configured values; there
/// are no implicit clock allowances or limits selected from submitted tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TokenVerificationLimits {
    /// Upper bound on root-authorized issuer certificate lifetime.
    pub max_cert_ttl_ns: u64,
    /// Upper bound on the issuer's authorized maximum token lifetime.
    pub max_token_ttl_ns: u64,
    /// Inclusive allowance for future validity starts and certificate time.
    pub max_future_skew_ns: u64,
    /// Inclusive maximum age of the issuer's authenticated IC certificate.
    pub max_certificate_age_ns: u64,
    /// Budget for vector/string bytes plus one byte per variable entry, checked
    /// before canonical allocation or cryptographic work. Not a Candid wire limit.
    pub max_variable_bytes: usize,
    /// Separate bound on issuer signature CBOR before parsing.
    pub max_issuer_signature_bytes: usize,
    /// Separate bound on the root proof's Merkle path length.
    pub max_witness_steps: usize,
}

/// Actual authenticated runtime/configuration inputs for one local operation.
pub struct TokenVerificationContext<'a> {
    /// Actual authenticated ingress caller, equal to presenter and subject.
    pub caller: Principal,
    /// Expected network and fleet identifiers from protected configuration.
    pub audience: AudienceId,
    /// Host-selected local role whose grant is returned after verification.
    pub role: &'a AuthRole,
    /// Current protected scope ceiling for the local role; sorted and unique.
    pub allowed_scopes: &'a [String],
    /// Scopes required by the host's operation, never chosen by the token.
    pub required_scopes: &'a [String],
    /// Current protected root-key enrollment and revocation policy.
    pub root_key: &'a RootKeyPolicy,
    /// Raw, protected 96-byte IC BLS network key for issuer certification.
    pub ic_root_public_key_raw: &'a [u8],
    /// Host clock in nanoseconds; verification never reads a runtime clock.
    pub now_ns: u64,
    pub limits: TokenVerificationLimits,
}

/// Authenticated claims and the local grant. Constructible only by verification;
/// deliberately neither deserializable nor a resource/session authorization.
#[derive(Clone, Debug)]
pub struct VerifiedToken<'a> {
    claims: &'a DelegatedTokenClaims,
    role: &'a AuthRole,
    scopes: &'a [String],
    claims_hash: [u8; 32],
    expires_at_ns: u64,
}

impl<'a> VerifiedToken<'a> {
    /// Claims authenticated by the complete proof chain at verification time.
    pub fn claims(&self) -> &'a DelegatedTokenClaims {
        self.claims
    }

    /// The host-selected local role, from the authenticated token grant.
    pub fn role(&self) -> &'a AuthRole {
        self.role
    }

    /// Granted local scopes, already constrained by the live host ceiling.
    pub fn scopes(&self) -> &'a [String] {
        self.scopes
    }

    /// Canonical claims hash authenticated by the issuer's IC signature.
    pub const fn claims_hash(&self) -> [u8; 32] {
        self.claims_hash
    }

    /// Exclusive proof deadline capped by the currently accepted root policy.
    /// Reusing a result later still requires live authority and replay checks.
    pub const fn expires_at_ns(&self) -> u64 {
        self.expires_at_ns
    }
}

/// Typed rejection of bounds, policy, canonical encoding or a proof. Field and
/// target labels provide diagnostics; callers should match the enum variants.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum TokenVerificationError {
    #[error("token variable material exceeds host limits")]
    InputTooLarge,
    #[error("anonymous {field} is not permitted")]
    AnonymousPrincipal { field: &'static str },
    #[error("presenter must equal the authenticated caller")]
    PresenterMismatch,
    #[error("presenter and subject must be the same identity")]
    SubjectMismatch,
    #[error("proof binding mismatch: {field}")]
    BindingMismatch { field: &'static str },
    #[error("invalid {target} validity window")]
    InvalidWindow { target: &'static str },
    #[error("{target} is not yet valid")]
    NotYetValid { target: &'static str },
    #[error("{target} has expired")]
    Expired { target: &'static str },
    #[error("{target} TTL {ttl_ns} exceeds {max_ttl_ns}")]
    TtlExceeded {
        target: &'static str,
        ttl_ns: u64,
        max_ttl_ns: u64,
    },
    #[error("grants must be nonempty, with at most 16 roles and 32 scopes per role")]
    InvalidGrants,
    #[error("audience does not match protected local context")]
    AudienceRejected,
    #[error("token grants exceed the root certificate grants")]
    GrantsNotSubset,
    #[error("local role is not granted")]
    RoleRejected,
    #[error("scope is outside the protected local ceiling or absent from the grant: {scope}")]
    ScopeRejected { scope: String },
    #[error("root proof signature or public key encoding is invalid")]
    RootSignatureInvalid,
    #[error("root proof signature has high-s encoding")]
    HighSSignature,
    #[error("root proof Merkle witness is invalid")]
    InvalidMerkleWitness,
    #[error("root proof {field} is below the protected epoch/version floor")]
    StaleAuthority { field: &'static str },
    #[error(transparent)]
    Canonical(#[from] CanonicalAuthError),
    #[error(transparent)]
    IssuerSignature(#[from] CanisterSignatureError),
}

/// Authenticate both root and issuer proofs and enforce live context, grant
/// narrowing, caller binding and validity. Neither proof verifier is injectable.
/// A valid result does not consume a nonce, establish a session or grant access
/// to an application-owned resource. The host must bound outer request decoding.
pub fn verify_token<'a>(
    token: &'a DelegatedToken,
    context: &TokenVerificationContext<'_>,
) -> Result<VerifiedToken<'a>, TokenVerificationError> {
    rules::check_size(token, context.limits)?;
    let (grant, claims_hash) = rules::verify_material(token, context)?;
    root::verify(
        &token.proof.cert,
        &token.proof.root_proof,
        context.root_key,
        context.now_ns,
        context.limits.max_future_skew_ns,
    )?;

    let cert = &token.proof.cert;
    // The seed and issuer become authenticated only after the root proof above.
    let IssuerProofBinding::IcCanisterSignatureV1 { seed_hash } = cert.issuer_proof_binding;
    let IssuerProof::IcCanisterSignatureV1(proof) = &token.issuer_proof;
    let message = domain_separated_message(b"canic-issuer-delegated-token", claims_hash)?;
    verify_canister_signature(
        &message,
        proof,
        &CanisterSignaturePolicy {
            signing_canister: cert.issuer_pid,
            seed_hash,
            ic_root_public_key_raw: context.ic_root_public_key_raw,
            now_ns: context.now_ns,
            max_certificate_age_ns: context.limits.max_certificate_age_ns,
            max_future_skew_ns: context.limits.max_future_skew_ns,
            max_signature_bytes: context.limits.max_issuer_signature_bytes,
            max_message_bytes: message.len(),
        },
    )?;

    Ok(VerifiedToken {
        claims: &token.claims,
        role: &grant.target,
        scopes: &grant.scopes,
        claims_hash,
        expires_at_ns: token
            .claims
            .expires_at_ns
            .min(context.root_key.accept_until_ns),
    })
}

pub(super) fn binding(condition: bool, field: &'static str) -> Result<(), TokenVerificationError> {
    if condition {
        Ok(())
    } else {
        Err(TokenVerificationError::BindingMismatch { field })
    }
}

pub(super) fn window(
    target: &'static str,
    start: u64,
    end: u64,
    now: u64,
    skew: u64,
) -> Result<(), TokenVerificationError> {
    if start >= end {
        return Err(TokenVerificationError::InvalidWindow { target });
    }
    if start > now && start - now > skew {
        return Err(TokenVerificationError::NotYetValid { target });
    }
    if now >= end {
        return Err(TokenVerificationError::Expired { target });
    }
    Ok(())
}
