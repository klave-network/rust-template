use super::{beacon::Root, errors::MerkleError, types::H256};
use sha2::{Digest, Sha256};

use klave;

/// MerkleTree is a merkle tree implementation using sha256 as a hashing algorithm.
pub type MerkleTree = rs_merkle::MerkleTree<rs_merkle::algorithms::Sha256>;

pub fn is_valid_normalized_merkle_branch(
    leaf: H256,
    branch: &[H256],
    gindex: u32,
    root: Root,
) -> Result<(), MerkleError> {
    if gindex == 0 {
        return Err(MerkleError::InvalidGeneralIndex(gindex as i64));
    }
    let depth = get_depth(gindex);
    let subtree_index = get_subtree_index(gindex);
    is_valid_merkle_branch(leaf, branch, depth, subtree_index, root)
}

/// Check if ``leaf`` at ``index`` verifies against the Merkle ``root`` and ``branch``.
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#is_valid_merkle_branch
pub fn is_valid_merkle_branch(
    leaf: H256,
    branch: &[H256],
    depth: u32,
    subtree_index: u32,
    root: Root,
) -> Result<(), MerkleError> {
    if depth != branch.len() as u32 {
        return Err(MerkleError::InvalidMerkleBranchLength(
            depth,
            leaf,
            branch.to_vec(),
            subtree_index,
            root,
        ));
    }
    let mut value = leaf;
    for (i, b) in branch.iter().enumerate() {
        // klave::notifier::send_string(&format!("Iterating over branch: {}, {:?}", i, b));
        if let Some(v) = 2u32.checked_pow(i as u32) {
            if subtree_index / v % 2 == 1 {
                value = hash([b.as_bytes(), value.as_bytes()].concat());
            } else {
                value = hash([value.as_bytes(), b.as_bytes()].concat());
            }
        } else {
            return Err(MerkleError::TooLongMerkleBranchLength(
                depth,
                leaf,
                branch.to_vec(),
                subtree_index,
                root,
            ));
        }
    }
    if value == root {
        // klave::notifier::send_string(&format!("Merkle branch is valid for leaf: {:?}", leaf));
        Ok(())
    } else {
        Err(MerkleError::InvalidMerkleBranch(
            leaf,
            branch.to_vec(),
            subtree_index,
            root,
            value,
        ))
    }
}

pub const fn get_depth(gindex: u32) -> u32 {
    gindex.ilog2()
}

pub const fn get_subtree_index(gindex: u32) -> u32 {
    gindex % 2u32.pow(get_depth(gindex))
}

fn hash(bz: Vec<u8>) -> H256 {
    let mut output = H256::default();
    output.0.copy_from_slice(Sha256::digest(bz).as_slice());
    output
}
