use super::{
    bls::{is_equal_pubkeys_and_aggregate_pub_key, PublicKey, Signature},
    errors::Error,
    types::U64,
};
use ssz_rs::prelude::{Bitvector, Vector};
use ssz_rs::{Deserialize, Sized};
use ssz_rs_derive::SimpleSerialize;

pub type SyncCommitteePeriod = U64;

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#synccommittee
#[derive(
    Clone, Debug, PartialEq, Eq, Default, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
pub struct SyncCommittee<const SYNC_COMMITTEE_SIZE: usize> {
    pub pubkeys: Vector<PublicKey, SYNC_COMMITTEE_SIZE>,
    pub aggregate_pubkey: PublicKey,
}

impl<const SYNC_COMMITTEE_SIZE: usize> SyncCommittee<SYNC_COMMITTEE_SIZE> {
    pub fn validate(&self) -> Result<(), Error> {
        is_equal_pubkeys_and_aggregate_pub_key(&self.pubkeys, &self.aggregate_pubkey)
    }
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#syncaggregate
/**
 *  sync_committee_bits: Bitvector[SYNC_COMMITTEE_SIZE]
 *  sync_committee_signature: BLSSignature
 */
#[derive(
    Clone, Debug, Default, PartialEq, Eq, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
pub struct SyncAggregate<const SYNC_COMMITTEE_SIZE: usize> {
    pub sync_committee_bits: Bitvector<SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: Signature,
}

impl<const SYNC_COMMITTEE_SIZE: usize> SyncAggregate<SYNC_COMMITTEE_SIZE> {
    pub fn count_participants(&self) -> usize {
        self.sync_committee_bits.count_ones()
    }
}
