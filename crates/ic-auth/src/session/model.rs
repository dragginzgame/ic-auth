use super::SessionError;
use crate::{
    canonical::validate_scope_label,
    token::{RootKeyPolicy, TokenVerificationContext, VerifiedToken},
};
use ic_auth_protocol_types::{
    AudienceId, AuthRole, ChainKeyAlgorithm, ChainKeyKeyId, DelegatedToken, Principal, RootProof,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Protected admission policy. Hosts must advance generation atomically with
/// authority changes, including disable/re-enable and issuer revocation.
#[derive(Clone, Copy, Debug)]
pub struct SessionPolicy {
    /// Current host-owned generation, equal to the transactional store's value.
    pub generation: u64,
    /// Protected capability switch; re-enabling requires a new generation.
    pub enabled: bool,
    /// Protected local subject eligibility, independent of resource ownership.
    pub subject_admissible: bool,
    /// Default request TTL and upper bound, in seconds; maximum 1800 seconds.
    pub default_ttl_secs: u64,
    pub max_ttl_secs: u64,
    /// Admission-only proof lifetime bound, at most 60 seconds in nanoseconds.
    pub max_proof_ttl_ns: u64,
}
impl SessionPolicy {
    pub(super) fn validate(self) -> Result<(), SessionError> {
        if self.default_ttl_secs == 0
            || self.default_ttl_secs > self.max_ttl_secs
            || self.max_ttl_secs > 1800
            || self.max_proof_ttl_ns == 0
            || self.max_proof_ttl_ns > 60_000_000_000
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        Ok(())
    }
}

/// Host storage bounds, never selected by submitted tokens. Physical records
/// and their encoded bytes count until pruning; bytes exclude allocator/index overhead.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionLimits {
    pub max_sessions: usize,
    pub max_replays: usize,
    pub max_replays_per_subject: usize,
    pub max_encoded_bytes: usize,
}
impl SessionLimits {
    pub(super) fn validate(self) -> Result<(), SessionError> {
        if self.max_sessions == 0
            || self.max_sessions > 2048
            || self.max_replays == 0
            || self.max_replays > 4096
            || self.max_replays_per_subject == 0
            || self.max_replays_per_subject > 256
            || self.max_encoded_bytes == 0
            || self.max_encoded_bytes > 8 * 1024 * 1024
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        Ok(())
    }
}

/// Canonical narrowing request: 1..=16 unique scopes, at most 1024 scope bytes.
/// Input order is normalized before hashing; duplicates are rejected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionRequest {
    scopes: Vec<String>,
    ttl_secs: Option<u64>,
}
impl SessionRequest {
    pub fn new(mut scopes: Vec<String>, ttl_secs: Option<u64>) -> Result<Self, SessionError> {
        check_scopes(&scopes)?;
        scopes.sort_unstable();
        if scopes.windows(2).any(|pair| pair[0] == pair[1]) || ttl_secs == Some(0) {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        Ok(Self { scopes, ttl_secs })
    }
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }
    /// Exact Canic V1 request hash; None and an explicit default TTL differ.
    pub fn hash(&self) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"canic-application-session-request-v1");
        hash.update((self.scopes.len() as u64).to_be_bytes());
        for scope in &self.scopes {
            hash.update((scope.len() as u64).to_be_bytes());
            hash.update(scope.as_bytes());
        }
        match self.ttl_secs {
            None => hash.update([0]),
            Some(ttl) => {
                hash.update([1]);
                hash.update(ttl.to_be_bytes());
            }
        }
        hash.finalize().into()
    }
}

fn check_scopes(scopes: &[String]) -> Result<(), SessionError> {
    if scopes.is_empty() || scopes.len() > 16 {
        return Err(SessionError::InvalidPolicyOrRecord);
    }
    let mut bytes = 0usize;
    for scope in scopes {
        validate_scope_label(scope)?;
        bytes += scope.len(); // Each validated scope is <=64 bytes, count <=16.
    }
    if bytes > 1024 {
        return Err(SessionError::InvalidPolicyOrRecord);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RootAuthority {
    root: Principal,
    algorithm: ChainKeyAlgorithm,
    key_id: ChainKeyKeyId,
    path_hash: [u8; 32],
    public_key: Vec<u8>,
    key_version: u64,
    proof_epoch: u64,
    registry_epoch: u64,
}
impl RootAuthority {
    fn current(&self, policy: &RootKeyPolicy) -> bool {
        self.root == policy.root_canister_id
            && self.algorithm == policy.algorithm
            && self.key_id == policy.key_id
            && self.path_hash == policy.derivation_path_hash
            && self.public_key == policy.public_key
            && self.key_version == policy.key_version
            && self.key_version >= policy.min_accepted_key_version
            && self.proof_epoch >= policy.min_accepted_proof_epoch
            && self.registry_epoch >= policy.min_accepted_registry_epoch
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionData {
    caller: Principal,
    subject: Principal,
    issuer: Principal,
    audience: AudienceId,
    role: AuthRole,
    scopes: Vec<String>,
    generation: u64,
    established_at_ns: u64,
    expires_at_ns: u64,
    fingerprint: [u8; 32],
    request_hash: [u8; 32],
    root: RootAuthority,
}

/// A structurally checked host-owned session record, never an ingress credential.
/// Decoding is only for trusted host storage; live authorization is always required.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session(SessionData);
impl Session {
    pub fn caller(&self) -> Principal {
        self.0.caller
    }
    pub fn subject(&self) -> Principal {
        self.0.subject
    }
    pub fn issuer(&self) -> Principal {
        self.0.issuer
    }
    pub fn scopes(&self) -> &[String] {
        &self.0.scopes
    }
    pub fn generation(&self) -> u64 {
        self.0.generation
    }
    pub fn established_at_ns(&self) -> u64 {
        self.0.established_at_ns
    }
    pub fn expires_at_ns(&self) -> u64 {
        self.0.expires_at_ns
    }
    pub fn fingerprint(&self) -> [u8; 32] {
        self.0.fingerprint
    }
    pub fn request_hash(&self) -> [u8; 32] {
        self.0.request_hash
    }

    /// Encode one bounded current record, never an entire store.
    pub fn encode(&self) -> Result<Vec<u8>, SessionError> {
        let bytes = serde_cbor::to_vec(&self.0).map_err(|_| SessionError::InvalidPolicyOrRecord)?;
        if bytes.len() > 2048 {
            return Err(SessionError::ByteCapacity);
        }
        Ok(bytes)
    }
    /// Restore a bounded record from trusted host storage and check invariants.
    /// Successful decoding is structural evidence, not proof authentication.
    pub fn decode(bytes: &[u8]) -> Result<Self, SessionError> {
        if bytes.len() > 2048 {
            return Err(SessionError::ByteCapacity);
        }
        let data: SessionData =
            serde_cbor::from_slice(bytes).map_err(|_| SessionError::InvalidPolicyOrRecord)?;
        check_scopes(&data.scopes)?;
        if data.caller == Principal::anonymous()
            || data.caller != data.subject
            || data.issuer == Principal::anonymous()
            || data.root.root == Principal::anonymous()
            || data.established_at_ns >= data.expires_at_ns
            || data.expires_at_ns - data.established_at_ns > 1_800_000_000_000
            || data.scopes.windows(2).any(|pair| pair[0] >= pair[1])
            || data.root.key_id.name.is_empty()
            || data.root.key_id.name.len() > 128
            || !matches!(data.root.public_key.len(), 33 | 65)
        {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        Ok(Self(data))
    }
    pub(super) fn admit(
        verified: &VerifiedToken<'_>,
        token: &DelegatedToken,
        request: &SessionRequest,
        ctx: &TokenVerificationContext<'_>,
        policy: SessionPolicy,
    ) -> Result<Self, SessionError> {
        let ttl = request.ttl_secs.unwrap_or(policy.default_ttl_secs);
        if ttl == 0 || ttl > policy.max_ttl_secs {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        let expires = ctx
            .now_ns
            .checked_add(
                ttl.checked_mul(1_000_000_000)
                    .ok_or(SessionError::InvalidPolicyOrRecord)?,
            )
            .ok_or(SessionError::InvalidPolicyOrRecord)?;
        let RootProof::IcChainKeyBatchSignatureV1(root) = &token.proof.root_proof;
        let key = ctx.root_key;
        if key.key_id.name.is_empty() || key.key_id.name.len() > 128 {
            return Err(SessionError::InvalidPolicyOrRecord);
        }
        let session = Self(SessionData {
            caller: ctx.caller,
            subject: verified.claims().subject,
            issuer: verified.claims().issuer_pid,
            audience: ctx.audience,
            role: verified.role().clone(),
            scopes: request.scopes.clone(),
            generation: policy.generation,
            established_at_ns: ctx.now_ns,
            expires_at_ns: expires.min(key.accept_until_ns),
            fingerprint: verified.claims_hash(),
            request_hash: request.hash(),
            root: RootAuthority {
                root: key.root_canister_id,
                algorithm: key.algorithm,
                key_id: key.key_id.clone(),
                path_hash: key.derivation_path_hash,
                public_key: key.public_key.clone(),
                key_version: key.key_version,
                proof_epoch: root.header.proof_epoch,
                registry_epoch: root.header.registry_epoch,
            },
        });
        session.check_live(ctx, policy)?;
        session.encode()?;
        Ok(session)
    }
    pub(super) fn check_live(
        &self,
        ctx: &TokenVerificationContext<'_>,
        policy: SessionPolicy,
    ) -> Result<(), SessionError> {
        if !policy.enabled || !policy.subject_admissible {
            return Err(SessionError::Disabled);
        }
        if ctx.now_ns < self.0.established_at_ns || ctx.now_ns >= self.0.expires_at_ns {
            return Err(SessionError::MissingOrExpired);
        }
        if ctx.caller == Principal::anonymous()
            || self.0.caller != ctx.caller
            || self.0.subject != ctx.caller
            || self.0.audience != ctx.audience
            || &self.0.role != ctx.role
            || self.0.generation != policy.generation
            || !self.0.root.current(ctx.root_key)
            || ctx.now_ns < ctx.root_key.valid_from_ns
            || ctx.now_ns >= ctx.root_key.accept_until_ns
            || self.0.expires_at_ns > ctx.root_key.accept_until_ns
            || self.0.expires_at_ns - self.0.established_at_ns > policy.max_ttl_secs * 1_000_000_000
        {
            return Err(SessionError::StaleAuthority);
        }
        let mut previous: Option<&str> = None;
        for scope in ctx.allowed_scopes {
            validate_scope_label(scope)?;
            if previous.is_some_and(|p| p >= scope.as_str()) {
                return Err(SessionError::InvalidPolicyOrRecord);
            }
            previous = Some(scope);
        }
        if self
            .0
            .scopes
            .iter()
            .any(|s| !ctx.allowed_scopes.contains(s))
        {
            return Err(SessionError::ScopeRejected);
        }
        Ok(())
    }
}

/// Replay tombstone retained independently of a session and of logout.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Replay {
    fingerprint: [u8; 32],
    subject: Principal,
    generation: u64,
    remove_at_ns: u64,
}
impl Replay {
    pub(super) fn new(session: &Session, remove_at_ns: u64) -> Self {
        Self {
            fingerprint: session.fingerprint(),
            subject: session.subject(),
            generation: session.generation(),
            remove_at_ns,
        }
    }
    pub fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }
    pub fn subject(&self) -> Principal {
        self.subject
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn remove_at_ns(&self) -> u64 {
        self.remove_at_ns
    }
    pub(super) fn encoded_len(&self) -> Result<usize, SessionError> {
        serde_cbor::to_vec(self)
            .map(|b| b.len())
            .map_err(|_| SessionError::InvalidPolicyOrRecord)
    }
}
