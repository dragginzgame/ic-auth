use ic_auth::chain_key_batch::{ChainKeyBatchError, merkle_root_and_witnesses};
use ic_auth_protocol_types::{ChainKeyBatchWitnessStepV1, ChainKeyBatchWitnessV1};
use sha2::{Digest, Sha256};

// Independent proof reconstruction: node bytes are the frozen Canic contract,
// rather than another call into the constructor under test.
fn reconstruct(leaf: [u8; 32], witness: &ChainKeyBatchWitnessV1) -> [u8; 32] {
    witness.steps.iter().fold(leaf, |current, step| {
        let mut bytes = vec![1];
        match step {
            ChainKeyBatchWitnessStepV1::LeftSibling(hash) => {
                bytes.extend_from_slice(hash);
                bytes.extend_from_slice(&current);
            }
            ChainKeyBatchWitnessStepV1::RightSibling(hash) => {
                bytes.extend_from_slice(&current);
                bytes.extend_from_slice(hash);
            }
        }
        Sha256::digest(bytes).into()
    })
}

#[test]
fn odd_batch_preserves_canics_frozen_root_and_witness_directions() {
    // Canic ac55e50334dd6479ec36f404e89e60bcfe9184d6:
    // ops/auth/delegated/chain_key.rs golden fixture.
    let leaf = [
        234, 10, 226, 67, 151, 247, 200, 82, 134, 47, 125, 22, 77, 221, 130, 86, 136, 190, 90, 23,
        133, 137, 48, 221, 175, 77, 81, 232, 85, 11, 198, 2,
    ];
    let expected = [
        73, 50, 92, 95, 67, 117, 166, 33, 181, 113, 62, 57, 56, 186, 157, 136, 114, 178, 138, 96,
        76, 13, 198, 150, 46, 236, 37, 154, 55, 237, 163, 44,
    ];
    let leaves = [[42; 32], leaf, [43; 32]];
    let (root, witnesses) = merkle_root_and_witnesses(&leaves, 3).unwrap();
    assert_eq!(root, expected);
    assert_eq!(
        witnesses[1].steps,
        vec![
            ChainKeyBatchWitnessStepV1::LeftSibling([42; 32]),
            ChainKeyBatchWitnessStepV1::RightSibling([43; 32]),
        ]
    );
    // The third leaf is promoted unchanged, without a self-sibling step.
    assert_eq!(witnesses[2].steps.len(), 1);
    for (leaf, witness) in leaves.into_iter().zip(&witnesses) {
        assert_eq!(reconstruct(leaf, witness), expected);
    }
    let mut reversed = leaves;
    reversed.reverse();
    assert_ne!(merkle_root_and_witnesses(&reversed, 3).unwrap().0, root);
}

#[test]
fn singleton_has_no_internal_hash_or_witness_and_hash_duplicates_are_preserved() {
    let (root, witnesses) = merkle_root_and_witnesses(&[[7; 32]], 1).unwrap();
    assert_eq!(root, [7; 32]);
    assert_eq!(witnesses, vec![ChainKeyBatchWitnessV1 { steps: vec![] }]);
    let leaves = [[7; 32]; 3];
    let (root, witnesses) = merkle_root_and_witnesses(&leaves, 3).unwrap();
    assert_eq!(witnesses.len(), leaves.len());
    for (leaf, witness) in leaves.into_iter().zip(&witnesses) {
        assert_eq!(reconstruct(leaf, witness), root);
    }
}

#[test]
fn protected_capacity_rejects_empty_zero_and_over_budget_batches() {
    assert_eq!(
        merkle_root_and_witnesses(&[], 64),
        Err(ChainKeyBatchError::EmptyBatch)
    );
    assert_eq!(
        merkle_root_and_witnesses(&[[1; 32]], 0),
        Err(ChainKeyBatchError::TooManyLeaves { found: 1, max: 0 })
    );
    assert_eq!(
        merkle_root_and_witnesses(&[[1; 32]; 65], 64),
        Err(ChainKeyBatchError::TooManyLeaves { found: 65, max: 64 })
    );
}

#[test]
fn all_shapes_through_canics_capacity_produce_bounded_ordered_witnesses() {
    for count in 1_u8..=64 {
        let leaves: Vec<_> = (0..count).map(|index| [index; 32]).collect();
        let (root, witnesses) = merkle_root_and_witnesses(&leaves, 64).unwrap();
        assert_eq!(witnesses.len(), leaves.len());
        let height = count.next_power_of_two().ilog2() as usize;
        for (leaf, witness) in leaves.into_iter().zip(&witnesses) {
            assert!(witness.steps.len() <= height);
            assert_eq!(reconstruct(leaf, witness), root);
        }
    }
}
