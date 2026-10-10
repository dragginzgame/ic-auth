//! Pure Merkle construction for the existing chain-key batch proof format.
//!
//! Hosts own leaf authorization, canonical certificate hashing, issuer ordering
//! and uniqueness, batch identity, signing and persistence. A tree authenticates
//! none of those inputs by itself. No runtime, clock or signing effect is used.

use ic_auth_protocol_types::{ChainKeyBatchWitnessStepV1, ChainKeyBatchWitnessV1};
use sha2::{Digest, Sha256};
use std::ops::Range;
use thiserror::Error;

/// Rejection before allocating tree or witness material.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ChainKeyBatchError {
    /// A batch cannot commit an empty set of leaves.
    #[error("chain-key batch requires at least one leaf")]
    EmptyBatch,
    /// The supplied hashes exceed the host's explicit construction budget.
    #[error("chain-key batch has {found} leaves, exceeding the limit {max}")]
    TooManyLeaves { found: usize, max: usize },
}

/// Build a root and one witness per leaf, preserving the supplied order.
///
/// Supply canonical certificate hashes from
/// [`crate::canonical::chain_key_delegation_cert_hash`] in the host's authorized
/// issuer order. The host selects `max_leaves` from protected configuration;
/// empty and over-budget inputs reject before tree allocation. Total witness
/// material is bounded by `n * ceil(log2(n))` steps for `n <= max_leaves`.
///
/// Internal nodes hash `0x01 || left || right` with SHA-256. An unpaired node
/// advances unchanged, without duplicating itself or adding a witness step.
/// A singleton's root is its supplied leaf hash and its witness is empty.
/// Duplicate hashes are preserved; issuer uniqueness is a host policy.
pub fn merkle_root_and_witnesses(
    leaf_hashes: &[[u8; 32]],
    max_leaves: usize,
) -> Result<([u8; 32], Vec<ChainKeyBatchWitnessV1>), ChainKeyBatchError> {
    if leaf_hashes.is_empty() {
        return Err(ChainKeyBatchError::EmptyBatch);
    }
    if leaf_hashes.len() > max_leaves {
        return Err(ChainKeyBatchError::TooManyLeaves {
            found: leaf_hashes.len(),
            max: max_leaves,
        });
    }

    let mut witnesses = vec![ChainKeyBatchWitnessV1 { steps: Vec::new() }; leaf_hashes.len()];
    let mut level: Vec<_> = leaf_hashes
        .iter()
        .enumerate()
        .map(|(index, hash)| Node {
            hash: *hash,
            leaves: index..index + 1,
        })
        .collect();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let left = &pair[0];
            if pair.len() == 1 {
                next.push(left.clone());
                continue;
            }
            let right = &pair[1];
            for index in left.leaves.clone() {
                witnesses[index]
                    .steps
                    .push(ChainKeyBatchWitnessStepV1::RightSibling(right.hash));
            }
            for index in right.leaves.clone() {
                witnesses[index]
                    .steps
                    .push(ChainKeyBatchWitnessStepV1::LeftSibling(left.hash));
            }
            next.push(Node {
                hash: node_hash(left.hash, right.hash),
                leaves: left.leaves.start..right.leaves.end,
            });
        }
        level = next;
    }
    Ok((level[0].hash, witnesses))
}

#[derive(Clone)]
struct Node {
    hash: [u8; 32],
    leaves: Range<usize>,
}

// Callers admit proof-size budgets before reconstruction. This helper is also
// the verification side of the constructor's existing node-byte contract.
#[cfg(feature = "token-verification")]
pub(crate) fn witness_root(leaf: [u8; 32], witness: &ChainKeyBatchWitnessV1) -> [u8; 32] {
    witness.steps.iter().fold(leaf, |current, step| match step {
        ChainKeyBatchWitnessStepV1::LeftSibling(hash) => node_hash(*hash, current),
        ChainKeyBatchWitnessStepV1::RightSibling(hash) => node_hash(current, *hash),
    })
}

fn node_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([1]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}
