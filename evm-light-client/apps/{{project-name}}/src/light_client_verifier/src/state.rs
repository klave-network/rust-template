use super::{errors::Error, updates::ConsensusUpdate};
use crate::consensus::src::{
    context::ChainContext,
    sync_protocol::{SyncCommittee, SyncCommitteePeriod},
};

/// A trait for reading the light client store
pub trait LightClientStoreReader<const SYNC_COMMITTEE_SIZE: usize> {
    /// Returns the current sync committee period
    fn current_period<CC: ChainContext>(&self, ctx: &CC) -> SyncCommitteePeriod;

    /// Returns the current sync committee corresponding to the `current_period()` if available
    fn current_sync_committee(&self) -> Option<SyncCommittee<SYNC_COMMITTEE_SIZE>>;

    /// Returns the next sync committee corresponding to the `current_period() + 1` if available
    fn next_sync_committee(&self) -> Option<SyncCommittee<SYNC_COMMITTEE_SIZE>>;

    /// Returns a error indicating whether the update is relevant to this store.
    ///
    /// This method should be used to determine whether the update should be applied to the store.
    fn ensure_relevant_update<CC: ChainContext, C: ConsensusUpdate<SYNC_COMMITTEE_SIZE>>(
        &self,
        ctx: &CC,
        update: &C,
    ) -> Result<(), Error>;
}

/// Returns the sync committee corresponding to the given signature period if available
pub fn get_sync_committee_at_period<
    CC: ChainContext,
    const SYNC_COMMITTEE_SIZE: usize,
    ST: LightClientStoreReader<SYNC_COMMITTEE_SIZE>,
>(
    ctx: &CC,
    store: &ST,
    signature_period: SyncCommitteePeriod,
) -> Option<SyncCommittee<SYNC_COMMITTEE_SIZE>> {
    let current_period = store.current_period(ctx);
    let next_period = current_period + 1;
    if signature_period == current_period {
        store.current_sync_committee()
    } else if signature_period == next_period {
        store.next_sync_committee()
    } else {
        None
    }
}
