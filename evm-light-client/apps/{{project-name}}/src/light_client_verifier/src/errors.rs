use crate::light_client_verifier::src::context::Fraction;
use displaydoc::Display;
use crate::consensus::src::{
    beacon::{BeaconBlockHeader, Epoch, Root, Slot},
    bls::PublicKey,
    errors::MerkleError,
    sync_protocol::SyncCommitteePeriod,
    types::H256,
};
use trie_db::TrieError;

type BoxedTrieError = Box<TrieError<primitive_types::H256, rlp::DecoderError>>;

#[derive(Debug, Display)]
pub enum Error {
    /// the light client does not support the fork corresponding to the finalized epoch: `epoch={0}`
    ForkNotSupported(Epoch),
    /// unexpected signature period: `store={0} signature={1} reason={2}`
    UnexpectedSingaturePeriod(SyncCommitteePeriod, SyncCommitteePeriod, String),
    /// unexpected attested period: `store={0} attested={1} reason={2}`
    UnexpectedAttestedPeriod(SyncCommitteePeriod, SyncCommitteePeriod, String),
    /// unexpected finalized period: `store={0} finalized={1} reason={2}`
    UnexpectedFinalizedPeriod(SyncCommitteePeriod, SyncCommitteePeriod, String),
    /// store does not cover the signature period: `store={0} signature={1}`
    StoreNotCoveredSignaturePeriod(SyncCommitteePeriod, SyncCommitteePeriod),
    /// cannot rotate to next sync committee: `store={0} finalized={1}`
    CannotRotateNextSyncCommittee(SyncCommitteePeriod, SyncCommitteePeriod),
    /// no next sync committee in store: `store_period={0} signature_period={1}`
    NoNextSyncCommitteeInStore(u64, u64),
    /// the beacon header at genesis slot must be empty: `slot={0}`
    NonEmptyBeaconHeaderAtGenesisSlot(u64),
    /// verify membership error
    VerifyMembershipError(),
    /// trusted root mismatch: `expected={0:?} actual={1:?}`
    TrustedRootMismatch(Root, Root),
    /// less than the minimal participants' `actual={0} minimal={1}`
    LessThanMinimalParticipants(usize, usize),
    /// insufficient participants: `actual={0} total={1}`
    InsufficientParticipants(u64, u64),
    /// execution payload's state root branch is empty
    EmptyExecutionPayloadStateRootBranch,
    /// execution payload's block number branch is empty
    EmptyExecutionPayloadBlockNumberBranch,
    /// invalid bls signatures
    InvalidBLSSignatures,
    /// finalized header not found
    FinalizedHeaderNotFound,
    /// inconsistent slot order: `current={0} signature={1} attested={2} finalized={3}`
    InconsistentSlotOrder(Slot, Slot, Slot, Slot),
    /// irrelevant consensus updates error: `{0}`
    IrrelevantConsensusUpdates(String),
    /// trie error
    TrieError(BoxedTrieError),
    /// ethereum common error: `{0:?}`
    CommonError(crate::consensus::src::errors::Error),
    /// rlp decoder error: `{0:?}`
    RlpDecoderError(rlp::DecoderError),
    /// the next sync committee is not finalized: `finalized={0} attested={1}`
    NotFinalizedNextSyncCommittee(SyncCommitteePeriod, SyncCommitteePeriod),
    /// both updates of misbehaviour data must have same period: {0} != {1}
    DifferentPeriodInNextSyncCommitteeMisbehaviour(SyncCommitteePeriod, SyncCommitteePeriod),
    /// both updates of misbehaviour data must have next sync committee
    NoNextSyncCommitteeInNextSyncCommitteeMisbehaviour,
    /// both updates of misbehaviour data must have different next sync committee: aggregate_pubkey={0:?}
    SameNextSyncCommitteeInNextSyncCommitteeMisbehaviour(PublicKey),
    /// both updates of misbehaviour data must have same finalized slot: {0} != {1}
    DifferentSlotInFinalizedHeaderMisbehaviour(Slot, Slot),
    /// both updates of misbehaviour data must have different finalized header: {0:?}
    SameFinalizedHeaderInFinalizedHeaderMisbehaviour(BeaconBlockHeader),
    /// non-existence error in execution layer
    ExecutionValueNonExist,
    /// existence error in execution layer
    ExecutionValueExist,
    /// value mismatch error in execution layer: {0:?} != {1:?}
    ExecutionValueMismatch(Vec<u8>, Vec<u8>),
    /// invalid merkle branch of finalized beacon header: `error={0}`
    InvalidFinalizedBeaconHeaderMerkleBranch(MerkleError),
    /// invalid merkle branch of finalized execution payload: `error={0}`
    InvalidFinalizedExecutionPayload(MerkleError),
    /// invalid merkle branch of next sync committee: `error={0}`
    InvalidNextSyncCommitteeMerkleBranch(MerkleError),
    /// next sync committee must be empty: `actual={0:?}`
    NonEmptyNextSyncCommittee(Vec<H256>),
    /// invalid merkle branch of current sync committee: `error={0}`
    InvalidCurrentSyncCommitteeMerkleBranch(MerkleError),
    /// invalid merkle branch of execution state root: `error={0}`
    InvalidExecutionStateRootMerkleBranch(MerkleError),
    /// the current fork does not contain the execution payload in the beacon block
    NoExecutionPayloadInBeaconBlock,
    /// invalid merkle branch of execution block number: `error={0}`
    InvalidExecutionBlockNumberMerkleBranch(MerkleError),
    /// inconsistent next sync committee: `store:{0:?}` != `update:{1:?}`
    InconsistentNextSyncCommittee(PublicKey, PublicKey),
    /// invalid fraction: `fraction={0:?}`
    InvalidFraction(Fraction),
    /// other error: `{description}`
    Other { description: String },
}

impl std::error::Error for Error {}

impl From<BoxedTrieError> for Error {
    fn from(value: BoxedTrieError) -> Self {
        Self::TrieError(value)
    }
}

impl From<crate::consensus::src::errors::Error> for Error {
    fn from(value: crate::consensus::src::errors::Error) -> Self {
        Self::CommonError(value)
    }
}

impl From<rlp::DecoderError> for Error {
    fn from(value: rlp::DecoderError) -> Self {
        Self::RlpDecoderError(value)
    }
}
