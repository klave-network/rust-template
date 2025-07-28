use super::{ConsensusUpdate, ExecutionUpdate, LightClientBootstrap};
use crate::consensus::src::{
    beacon::{BeaconBlockHeader, Slot},
    fork::bellatrix::LightClientUpdate,
    sync_protocol::{SyncAggregate, SyncCommittee},
    types::{H256, U64},
};

#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct LightClientBootstrapInfo<const SYNC_COMMITTEE_SIZE: usize>(
    pub crate::consensus::src::fork::bellatrix::LightClientBootstrap<SYNC_COMMITTEE_SIZE>,
);

impl<const SYNC_COMMITTEE_SIZE: usize> LightClientBootstrap<SYNC_COMMITTEE_SIZE>
    for LightClientBootstrapInfo<SYNC_COMMITTEE_SIZE>
{
    fn beacon_header(&self) -> &BeaconBlockHeader {
        &self.0.beacon_header
    }
    fn current_sync_committee(&self) -> &SyncCommittee<SYNC_COMMITTEE_SIZE> {
        &self.0.current_sync_committee
    }
    fn current_sync_committee_branch(&self) -> Vec<H256> {
        self.0.current_sync_committee_branch.to_vec()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConsensusUpdateInfo<const SYNC_COMMITTEE_SIZE: usize> {
    pub light_client_update: LightClientUpdate<SYNC_COMMITTEE_SIZE>,
    pub finalized_execution_root: H256,
    pub finalized_execution_branch: Vec<H256>,
}

impl<const SYNC_COMMITTEE_SIZE: usize> ConsensusUpdate<SYNC_COMMITTEE_SIZE>
    for ConsensusUpdateInfo<SYNC_COMMITTEE_SIZE>
{
    fn attested_beacon_header(&self) -> &BeaconBlockHeader {
        &self.light_client_update.attested_header
    }
    fn next_sync_committee(&self) -> Option<&SyncCommittee<SYNC_COMMITTEE_SIZE>> {
        self.light_client_update
            .next_sync_committee
            .as_ref()
            .map(|c| &c.0)
    }
    fn next_sync_committee_branch(&self) -> Option<Vec<H256>> {
        self.light_client_update
            .next_sync_committee
            .as_ref()
            .map(|c| c.1.to_vec())
    }
    fn finalized_beacon_header(&self) -> &BeaconBlockHeader {
        &self.light_client_update.finalized_header.0
    }
    fn finalized_beacon_header_branch(&self) -> Vec<H256> {
        self.light_client_update.finalized_header.1.to_vec()
    }
    fn finalized_execution_root(&self) -> H256 {
        self.finalized_execution_root
    }
    fn finalized_execution_branch(&self) -> Vec<H256> {
        self.finalized_execution_branch.to_vec()
    }
    fn sync_aggregate(&self) -> &SyncAggregate<SYNC_COMMITTEE_SIZE> {
        &self.light_client_update.sync_aggregate
    }
    fn signature_slot(&self) -> Slot {
        self.light_client_update.signature_slot
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
