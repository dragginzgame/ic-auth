//! Bounded, volatile signature retention and host-composed certified witnesses.
//!
//! Preparing a leaf is a signing decision. The host must authorize the message,
//! seed and domain first. It supplies its real canister ID, clock and query data
//! certificate, and publishes one composed certification root after mutations.
//! No runtime clock, certificate acquisition or root publication occurs here.

use ic_auth_protocol_types::{IcCanisterSignatureProofV1, Principal};
use ic_canister_sig_creation::{CanisterSigPublicKey, hash_bytes, hash_with_domain};
use ic_certification::{
    AsHashTree, Certificate, Hash, LookupResult, RbTree, fork, labeled, labeled_hash, pruned,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Maximum number of retained leaves, including expired leaves awaiting cleanup.
pub const MAX_SIGNATURES: usize = 4096;
/// Maximum retrieval retention, in nanoseconds. This is not proof validity.
pub const MAX_RETENTION_NS: u64 = 60_000_000_000;
/// Maximum records removed by one explicit cleanup operation.
pub const MAX_PRUNE_BATCH: usize = 128;
/// Maximum number of host-owned sibling branches along the witness path.
pub const MAX_COMPOSITION_DEPTH: usize = 16;

const LABEL_SIG: &[u8] = b"sig";
type SignatureKey = (Hash, Hash);
type Expiration = (u64, Hash, Hash);

/// Protected limits. Hosts may select lower positive limits, never higher ones.
#[derive(Clone, Copy, Debug)]
pub struct SignatureLimits {
    /// At most 4096 leaves. Duplicate preparation uses one record/index entry.
    pub max_signatures: usize,
    /// At most 64 KiB before hashing the raw payload, excluding the domain.
    pub max_message_bytes: usize,
    /// At most 64 KiB before decoding the host's query certificate.
    pub max_certificate_bytes: usize,
    /// At most 128 KiB of returned CBOR, checked before returning the proof.
    pub max_signature_bytes: usize,
}

impl SignatureLimits {
    fn validate(self) -> Result<(), SignatureStoreError> {
        for (value, maximum) in [
            (self.max_signatures, MAX_SIGNATURES),
            (self.max_message_bytes, 64 * 1024),
            (self.max_certificate_bytes, 64 * 1024),
            (self.max_signature_bytes, 128 * 1024),
        ] {
            if value == 0 || value > maximum {
                return Err(SignatureStoreError::InvalidLimits);
            }
        }
        Ok(())
    }
}

/// Raw payload and host-selected key/domain. The leaf hashes the exact IC
/// one-byte domain length, domain and payload. Do not prefix the payload twice.
#[derive(Clone, Copy, Debug)]
pub struct SignatureInputs<'a> {
    /// Key seed, at most 32 bytes, including the existing Canic seeds.
    pub seed: &'a [u8],
    /// Signature domain, at most 255 bytes.
    pub domain: &'a [u8],
    /// Raw bytes to sign; for Canic proofs this is the canonical payload hash.
    pub message: &'a [u8],
}

/// One pruned host-owned sibling in a composition path, innermost first.
/// The host keeps top-level labels ordered and reserves `/sig` for this store.
#[derive(Clone, Copy, Debug)]
pub enum SignatureSibling {
    /// Compose `fork(pruned(sibling_root), signature_witness)`.
    Left(Hash),
    /// Compose `fork(signature_witness, pruned(sibling_root))`.
    Right(Hash),
}

/// Preparation result. The host composes and publishes `signature_root` along
/// with its other certified state before a query can retrieve this signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreparedSignature {
    /// Exclusive retrieval deadline. A live retry does not extend it.
    pub retrieval_expires_at_ns: u64,
    /// Digest of this store's labeled `/sig` contribution, not a global root.
    pub signature_root: Hash,
}

/// Typed retention, input and composition failures.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SignatureStoreError {
    #[error("invalid signature-store limits")]
    InvalidLimits,
    #[error("anonymous signing canister is invalid")]
    AnonymousSigner,
    #[error("signature seed exceeds 32 bytes")]
    SeedTooLong,
    #[error("signature domain exceeds 255 bytes")]
    DomainTooLong,
    #[error("message exceeds the host's byte limit")]
    MessageTooLarge,
    #[error("retention must be positive and at most 60 seconds")]
    InvalidRetention,
    #[error("retrieval deadline overflows the host clock")]
    DeadlineOverflow,
    #[error("host clock precedes the last accepted mutation")]
    ClockRegressed,
    #[error("signature capacity exhausted; prune expired records explicitly")]
    Capacity,
    #[error("signature was not prepared")]
    NotPrepared,
    #[error("signature retrieval window expired")]
    Expired,
    #[error("cleanup budget exceeds 128 records")]
    InvalidPruneBudget,
    #[error("composition path exceeds 16 sibling branches")]
    CompositionTooDeep,
    #[error("query data certificate is absent")]
    NoCertificate,
    #[error("certificate exceeds the host's byte limit")]
    CertificateTooLarge,
    #[error("query certificate is malformed")]
    InvalidCertificate,
    #[error("certificate does not cover this canister's composed witness root")]
    CertificateRootMismatch,
    #[error("signature exceeds the host's output byte limit")]
    SignatureTooLarge,
    #[error("signature CBOR encoding failed")]
    Encoding,
}

/// Volatile reference store. No stable-memory layout or restore format is
/// prescribed. A reset loses all pending leaves; the host must republish its
/// composed root before serving queries. Retention indexes stay one-to-one.
pub struct SignatureStore {
    signing_canister: Principal,
    limits: SignatureLimits,
    tree: RbTree<Hash, RbTree<Hash, Vec<u8>>>,
    entries: BTreeMap<SignatureKey, u64>,
    expirations: BTreeSet<Expiration>,
    last_mutation_ns: u64,
}

impl SignatureStore {
    /// Construct an empty store from protected host identity and limits.
    pub fn new(
        signing_canister: Principal,
        limits: SignatureLimits,
    ) -> Result<Self, SignatureStoreError> {
        limits.validate()?;
        if signing_canister == Principal::anonymous() {
            return Err(SignatureStoreError::AnonymousSigner);
        }
        Ok(Self {
            signing_canister,
            limits,
            tree: RbTree::new(),
            entries: BTreeMap::new(),
            expirations: BTreeSet::new(),
            last_mutation_ns: 0,
        })
    }

    /// Physical records, including expired leaves not yet explicitly pruned.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there are no physical records.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Digest of the labeled `/sig` subtree; compose with other owners' roots.
    pub fn root_hash(&self) -> Hash {
        labeled_hash(LABEL_SIG, &self.tree.root_hash())
    }

    /// Prepare one host-authorized message using an explicit monotonic host clock.
    /// Live duplicate preparation returns the original deadline. Repreparing an
    /// expired key replaces its deadline/index entry without increasing capacity.
    /// Expired unrelated entries require explicit bounded cleanup first.
    /// All fallible validation precedes mutation; there is no implicit pruning.
    pub fn prepare(
        &mut self,
        inputs: SignatureInputs<'_>,
        now_ns: u64,
        retention_ns: u64,
    ) -> Result<PreparedSignature, SignatureStoreError> {
        let key = self.key(inputs)?;
        self.check_clock(now_ns)?;
        if retention_ns == 0 || retention_ns > MAX_RETENTION_NS {
            return Err(SignatureStoreError::InvalidRetention);
        }
        let deadline = now_ns
            .checked_add(retention_ns)
            .ok_or(SignatureStoreError::DeadlineOverflow)?;
        let existing = self.entries.get(&key).copied();
        let expires_at = match existing {
            Some(expires_at) if now_ns < expires_at => expires_at,
            _ => deadline,
        };
        if let Some(old) = existing {
            self.expirations.remove(&(old, key.0, key.1));
        } else {
            if self.len() >= self.limits.max_signatures {
                return Err(SignatureStoreError::Capacity);
            }
            if self.tree.get(&key.0).is_none() {
                let mut messages = RbTree::new();
                messages.insert(key.1, Vec::new());
                self.tree.insert(key.0, messages);
            } else {
                self.tree.modify(&key.0, |messages| {
                    messages.insert(key.1, Vec::new());
                });
            }
        }
        self.entries.insert(key, expires_at);
        self.expirations.insert((expires_at, key.0, key.1));
        self.last_mutation_ns = now_ns;
        Ok(PreparedSignature {
            retrieval_expires_at_ns: expires_at,
            signature_root: self.root_hash(),
        })
    }

    /// Stop retrieving one prepared key. Already-issued certificates/signatures
    /// are not revoked by deleting a leaf; their verifier's validity policy applies.
    /// The host republishes the resulting composed root after successful mutation.
    pub fn remove(
        &mut self,
        inputs: SignatureInputs<'_>,
        now_ns: u64,
    ) -> Result<bool, SignatureStoreError> {
        let key = self.key(inputs)?;
        self.check_clock(now_ns)?;
        let removed = self.delete(key);
        self.last_mutation_ns = now_ns;
        Ok(removed)
    }

    /// Remove at most `budget` earliest expired leaves, including the exact
    /// deadline. No scan/clone of the full store; zero is a valid no-op budget.
    pub fn prune(&mut self, now_ns: u64, budget: usize) -> Result<usize, SignatureStoreError> {
        self.check_clock(now_ns)?;
        if budget > MAX_PRUNE_BATCH {
            return Err(SignatureStoreError::InvalidPruneBudget);
        }
        let mut removed = 0;
        while removed < budget {
            let Some(&(deadline, seed, message)) = self.expirations.first() else {
                break;
            };
            if deadline > now_ns {
                break;
            }
            self.delete((seed, message));
            removed += 1;
        }
        self.last_mutation_ns = now_ns;
        Ok(removed)
    }

    /// Retrieve a fresh witness and encode the standard canister-signature DTO.
    /// `certificate` MUST be the host's authenticated IC query data certificate,
    /// never client input. This checks its certified-data path/digest, not its BLS
    /// trust chain or time. Consumers still perform full signature verification.
    /// Siblings are protected host composition state, innermost first.
    pub fn retrieve(
        &self,
        inputs: SignatureInputs<'_>,
        now_ns: u64,
        certificate: &[u8],
        siblings: &[SignatureSibling],
    ) -> Result<IcCanisterSignatureProofV1, SignatureStoreError> {
        let key = self.key(inputs)?;
        self.check_clock(now_ns)?;
        let expires_at = self
            .entries
            .get(&key)
            .ok_or(SignatureStoreError::NotPrepared)?;
        if now_ns >= *expires_at {
            return Err(SignatureStoreError::Expired);
        }
        if siblings.len() > MAX_COMPOSITION_DEPTH {
            return Err(SignatureStoreError::CompositionTooDeep);
        }
        if certificate.is_empty() {
            return Err(SignatureStoreError::NoCertificate);
        }
        if certificate.len() > self.limits.max_certificate_bytes {
            return Err(SignatureStoreError::CertificateTooLarge);
        }
        let decoded: Certificate = serde_cbor::from_slice(certificate)
            .map_err(|_| SignatureStoreError::InvalidCertificate)?;
        let mut tree = labeled(
            LABEL_SIG,
            self.tree
                .nested_witness(&key.0, |messages| messages.witness(&key.1)),
        );
        for sibling in siblings {
            tree = match sibling {
                SignatureSibling::Left(root) => fork(pruned(*root), tree),
                SignatureSibling::Right(root) => fork(tree, pruned(*root)),
            };
        }
        match decoded.tree.lookup_path([
            b"canister".as_slice(),
            self.signing_canister.as_slice(),
            b"certified_data".as_slice(),
        ]) {
            LookupResult::Found(root) if root == tree.digest() => {}
            _ => return Err(SignatureStoreError::CertificateRootMismatch),
        }
        // Both certificate bytes and witness depth/record count are bounded
        // before serialization. This output limit does not measure heap usage.
        #[derive(Serialize)]
        struct Envelope<'a> {
            #[serde(with = "serde_bytes")]
            certificate: &'a [u8],
            tree: ic_certification::HashTree,
        }
        let mut serializer = serde_cbor::Serializer::new(Vec::new());
        serializer
            .self_describe()
            .map_err(|_| SignatureStoreError::Encoding)?;
        Envelope { certificate, tree }
            .serialize(&mut serializer)
            .map_err(|_| SignatureStoreError::Encoding)?;
        let signature_cbor = serializer.into_inner();
        if signature_cbor.len() > self.limits.max_signature_bytes {
            return Err(SignatureStoreError::SignatureTooLarge);
        }
        Ok(IcCanisterSignatureProofV1 {
            signature_cbor,
            public_key_der: CanisterSigPublicKey::new(self.signing_canister, inputs.seed.to_vec())
                .to_der(),
        })
    }

    fn check_clock(&self, now_ns: u64) -> Result<(), SignatureStoreError> {
        if now_ns < self.last_mutation_ns {
            return Err(SignatureStoreError::ClockRegressed);
        }
        Ok(())
    }

    fn key(&self, inputs: SignatureInputs<'_>) -> Result<SignatureKey, SignatureStoreError> {
        if inputs.seed.len() > 32 {
            return Err(SignatureStoreError::SeedTooLong);
        }
        if inputs.domain.len() > 255 {
            return Err(SignatureStoreError::DomainTooLong);
        }
        if inputs.message.len() > self.limits.max_message_bytes {
            return Err(SignatureStoreError::MessageTooLarge);
        }
        Ok((
            hash_bytes(inputs.seed),
            hash_with_domain(inputs.domain, inputs.message),
        ))
    }

    fn delete(&mut self, key: SignatureKey) -> bool {
        let Some(deadline) = self.entries.remove(&key) else {
            return false;
        };
        self.expirations.remove(&(deadline, key.0, key.1));
        let mut empty = false;
        self.tree.modify(&key.0, |messages| {
            messages.delete(&key.1);
            empty = messages.is_empty();
        });
        if empty {
            self.tree.delete(&key.0);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retries_and_expired_repreparation_keep_exactly_one_expiration_entry() {
        let mut store = SignatureStore::new(
            Principal::from_slice(&[1]),
            SignatureLimits {
                max_signatures: 1,
                max_message_bytes: 32,
                max_certificate_bytes: 1024,
                max_signature_bytes: 2048,
            },
        )
        .unwrap();
        let inputs = SignatureInputs {
            seed: b"seed",
            domain: b"domain",
            message: b"message",
        };
        store.prepare(inputs, 0, 2000).unwrap();
        for now in 1..2000 {
            let prepared = store.prepare(inputs, now, 2000).unwrap();
            assert_eq!(prepared.retrieval_expires_at_ns, 2000);
            assert_eq!(store.expirations.len(), 1);
        }
        store.prepare(inputs, 2000, 2000).unwrap();
        assert_eq!(store.expirations.len(), 1);
        assert_eq!(store.prune(2000, 128), Ok(0));
        assert_eq!(store.prune(4000, 128), Ok(1));
        assert!(store.expirations.is_empty());
        assert!(store.is_empty());
    }
}
