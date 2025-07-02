use crate::consensus::src::{
    beacon::{BeaconBlockHeader, Slot},
    compute::compute_sync_committee_period_at_slot,
    fork::deneb::{ExecutionPayloadHeader, LightClientBootstrap},
    sync_protocol::SyncCommittee,
    types::{H256, U64},
};
use crate::light_client_verifier::src::{
    context::ChainConsensusVerificationContext,
    state::LightClientStoreReader,
    updates::{ConsensusUpdate, ExecutionUpdate},
};
use serde;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LightClientStore<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    pub latest_finalized_header: BeaconBlockHeader,
    pub latest_execution_payload_header:
        ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub current_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
    pub next_sync_committee: Option<SyncCommittee<SYNC_COMMITTEE_SIZE>>,
}

impl<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    > LightClientStore<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>
{
    pub fn from_bootstrap(
        bootstrap: LightClientBootstrap<
            SYNC_COMMITTEE_SIZE,
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
        latest_execution_payload_header: ExecutionPayloadHeader<
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
    ) -> Self {
        Self {
            latest_finalized_header: bootstrap.header.beacon,
            latest_execution_payload_header,
            current_sync_committee: bootstrap.current_sync_committee,
            next_sync_committee: None,
        }
    }

    pub fn current_slot(&self) -> Slot {
        self.latest_finalized_header.slot
    }

    pub fn apply_light_client_update<
        CC: ChainConsensusVerificationContext,
        CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>,
    >(
        &self,
        ctx: &CC,
        consensus_update: &CU,
    ) -> Result<Option<Self>, super::errors::Error> {
        let mut new_store = self.clone();
        let store_period =
            compute_sync_committee_period_at_slot(ctx, new_store.latest_finalized_header.slot);
        let attested_period = compute_sync_committee_period_at_slot(
            ctx,
            consensus_update.attested_beacon_header().slot,
        );

        let mut updated = if store_period == attested_period {
            if let Some(committee) = consensus_update.next_sync_committee() {
                new_store.next_sync_committee = Some(committee.clone());
                true
            } else {
                false
            }
        } else if store_period + 1 == attested_period {
            if let Some(committee) = new_store.next_sync_committee.as_ref() {
                new_store.current_sync_committee = committee.clone();
                new_store.next_sync_committee = consensus_update.next_sync_committee().cloned();
                true
            } else {
                return Err(super::errors::Error::CannotRotateNextSyncCommittee(
                    store_period,
                    attested_period,
                ));
            }
        } else {
            return Err(super::errors::Error::UnexpectedAttestedPeriod(
                store_period,
                attested_period,
                "attested period must be equal to store_period or store_period+1".into(),
            ));
        };
        if consensus_update.finalized_beacon_header().slot > self.latest_finalized_header.slot {
            new_store.latest_finalized_header = consensus_update.finalized_beacon_header().clone();
            updated = true;
        }
        if updated {
            Ok(Some(new_store))
        } else {
            Ok(None)
        }
    }
}

impl<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    > LightClientStoreReader<SYNC_COMMITTEE_SIZE>
    for LightClientStore<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>
{
    fn current_period<CC: crate::consensus::src::context::ChainContext>(
        &self,
        ctx: &CC,
    ) -> crate::consensus::src::sync_protocol::SyncCommitteePeriod {
        compute_sync_committee_period_at_slot(ctx, self.current_slot())
    }

    fn current_sync_committee(&self) -> Option<SyncCommittee<SYNC_COMMITTEE_SIZE>> {
        Some(self.current_sync_committee.clone())
    }

    fn next_sync_committee(&self) -> Option<SyncCommittee<SYNC_COMMITTEE_SIZE>> {
        self.next_sync_committee.clone()
    }

    fn ensure_relevant_update<
        CC: crate::consensus::src::context::ChainContext,
        C: crate::light_client_verifier::src::updates::ConsensusUpdate<SYNC_COMMITTEE_SIZE>,
    >(
        &self,
        ctx: &CC,
        update: &C,
    ) -> Result<(), crate::light_client_verifier::src::errors::Error> {
        let store_period = compute_sync_committee_period_at_slot(ctx, self.current_slot());
        let update_attested_period =
            compute_sync_committee_period_at_slot(ctx, update.attested_beacon_header().slot);
        let update_has_next_sync_committee = self.next_sync_committee.is_none()
            && (update.next_sync_committee().is_some() && update_attested_period == store_period);

        // https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/altair/light-client/sync-protocol.md#validate_light_client_update
        // assert (update_attested_slot > store.finalized_header.beacon.slot or update_has_next_sync_committee)
        if !(update.attested_beacon_header().slot > self.current_slot()
            || update_has_next_sync_committee)
        {
            return Err(crate::light_client_verifier::src::errors::Error::IrrelevantConsensusUpdates(format!(
                    "attested_beacon_header_slot={} store_slot={} update_has_next_sync_committee={} is_next_sync_committee_known={}",
                    update.attested_beacon_header().slot,
                    self.current_slot(),
                    update_has_next_sync_committee,
                    self.next_sync_committee.is_some()
                )));
        }

        // https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/altair/light-client/sync-protocol.md#process_light_client_update
        // update_has_finalized_next_sync_committee = (
        //     not is_next_sync_committee_known(store)
        //     and is_sync_committee_update(update) and is_finality_update(update) and (
        //         compute_sync_committee_period_at_slot(update.finalized_header.beacon.slot)
        //         == compute_sync_committee_period_at_slot(update.attested_header.beacon.slot)
        //     )
        // )
        let update_has_finalized_next_sync_committee =
            self.next_sync_committee.is_none() && update.next_sync_committee().is_some(); // equivalent to is_sync_committee_update(update)

        // https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/altair/light-client/sync-protocol.md#process_light_client_update
        // update.finalized_header.beacon.slot > store.finalized_header.beacon.slot
        // or update_has_finalized_next_sync_committee
        if !(update_has_finalized_next_sync_committee
            || update.finalized_beacon_header().slot > self.current_slot())
        {
            return Err(crate::light_client_verifier::src::errors::Error::IrrelevantConsensusUpdates(format!(
                    "finalized_beacon_header_slot={} store_slot={} update_has_finalized_next_sync_committee={}",
                    update.finalized_beacon_header().slot, self.current_slot(), update_has_finalized_next_sync_committee
                )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionUpdateInfo {
    pub state_root: H256,
    pub state_root_branch: Vec<H256>,
    pub block_number: U64,
    pub block_number_branch: Vec<H256>,
}

impl ExecutionUpdate for ExecutionUpdateInfo {
    fn state_root(&self) -> H256 {
        self.state_root
    }

    fn state_root_branch(&self) -> Vec<H256> {
        self.state_root_branch.clone()
    }

    fn block_number(&self) -> U64 {
        self.block_number
    }

    fn block_number_branch(&self) -> Vec<H256> {
        self.block_number_branch.clone()
    }
}
