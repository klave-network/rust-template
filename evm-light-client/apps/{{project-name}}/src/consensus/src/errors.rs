use super::{
    beacon::{Root, Version},
    bls::PublicKey,
    fork::ForkParameters,
    types::{H256, U64},
};
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum Error {
    /// bls amcl error: `{0:?}`
    BLSAmclError(milagro_bls::AmclError),
    /// merkleization error: `{0:?}`
    MerkleizationError(ssz_rs::MerkleizationError),
    /// ssz deserialize error: `{0:?}`
    SSZDeserializeError(ssz_rs::DeserializeError),
    /// hex error: `{0:?}`
    FromHexError(hex::FromHexError),
    /// invalid bls signature length: `expected={0} actual={1}`
    InvalidBLSSignatureLenght(usize, usize),
    /// invalid bls public key length: `expected={0} actual={1}`
    InvalidBLSPublicKeyLength(usize, usize),
    /// bls aggregate public key mismatch: `{0:?} != {1:?}`
    BLSAggregatePublicKeyMismatch(PublicKey, PublicKey),
    /// invalid address length: `expected={0} actual={1}`
    InvalidAddressLength(usize, usize),
    /// invalid fork parameters order: `{0:?}`
    InvalidForkParamersOrder(ForkParameters),
    /// invalid fork version: `epoch={0:?} fork={1:?} index={2}`
    UnknownFork(U64, U64, usize),
    /// the fork version does not support execution payload: `{0:?}`
    NotSupportedExecutionPayload(Version),
    /// the genesis does not support light client protocol
    NotSupportedLightClient,
    /// other error: `{description}`
    Other { description: String },
}

#[derive(Debug, Display)]
pub enum MerkleError {
    /// invalid merkle branch error: leaf={0:?} branch={1:?} subtree_index={2:?} expected={3:?} actual={4:?}
    InvalidMerkleBranch(H256, Vec<H256>, u32, Root, Root),
    /// too long merkle branch error: depth={0:?} leaf={1:?} branch={2:?} subtree_index={3:?} root={4:?}
    TooLongMerkleBranchLength(u32, H256, Vec<H256>, u32, Root),
    /// invalid merkle branch length error: depth={0:?} leaf={1:?} branch={2:?} subtree_index={3:?} root={4:?}
    InvalidMerkleBranchLength(u32, H256, Vec<H256>, u32, Root),
    /// invalid general index error: gindex={0:?}
    InvalidGeneralIndex(i64),
}

impl std::error::Error for Error {}

impl From<ssz_rs::MerkleizationError> for Error {
    fn from(value: ssz_rs::MerkleizationError) -> Self {
        Self::MerkleizationError(value)
    }
}

impl From<ssz_rs::DeserializeError> for Error {
    fn from(value: ssz_rs::DeserializeError) -> Self {
        Self::SSZDeserializeError(value)
    }
}

impl From<milagro_bls::AmclError> for Error {
    fn from(value: milagro_bls::AmclError) -> Self {
        Self::BLSAmclError(value)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(value: hex::FromHexError) -> Self {
        Self::FromHexError(value)
    }
}
