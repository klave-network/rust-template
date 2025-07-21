use super::context::{ChainConsensusVerificationContext, ConsensusVerificationContext};
use super::errors::Error;
use crate::consensus::src::compute::compute_sync_committee_period_at_slot;
use crate::consensus::src::context::ChainContext;
use crate::consensus::src::{
    beacon::{BeaconBlockHeader, Slot},
    merkle::is_valid_normalized_merkle_branch,
    sync_protocol::{SyncAggregate, SyncCommittee},
    types::{H256, U64},
};
pub mod bellatrix;
pub mod capella;
pub mod deneb;

pub trait LightClientBootstrap<const SYNC_COMMITTEE_SIZE: usize>:
    core::fmt::Debug + Clone + PartialEq + Eq
{
    /// finalized header
    fn beacon_header(&self) -> &BeaconBlockHeader;
    /// current sync committee corresponding to `beacon_header.state_root`
    fn current_sync_committee(&self) -> &SyncCommittee<SYNC_COMMITTEE_SIZE>;
    /// merkle branch of `current_sync_committee` within `BeaconState`
    fn current_sync_committee_branch(&self) -> Vec<H256>;
}

/// ConsensusUpdate is an update info of the consensus layer corresponding to a specific light client update
///
/// NOTE: The design is intended to minimise data type differences between forks.
/// ref.
/// - https://github.com/ethereum/consensus-specs/blob/a09d0c321550c5411557674a981e2b444a1178c0/specs/altair/light-client/sync-protocol.md#lightclientupdate
/// - https://github.com/ethereum/consensus-specs/blob/a09d0c321550c5411557674a981e2b444a1178c0/specs/capella/light-client/sync-protocol.md#modified-lightclientheader
pub trait ConsensusUpdate<const SYNC_COMMITTEE_SIZE: usize>:
    core::fmt::Debug + Clone + PartialEq + Eq
{
    /// header attested to by the sync committee
    fn attested_beacon_header(&self) -> &BeaconBlockHeader;
    /// next sync committee corresponding to `attested_beacon_header.state_root`
    fn next_sync_committee(&self) -> Option<&SyncCommittee<SYNC_COMMITTEE_SIZE>>;
    /// merkle branch of `next_sync_committee` within `BeaconState`
    fn next_sync_committee_branch(&self) -> Option<Vec<H256>>;
    /// finalized header corresponding to `attested_beacon_header.state_root`
    fn finalized_beacon_header(&self) -> &BeaconBlockHeader;
    /// merkle branch of `finalized_checkpoint.root` within `BeaconState`. This is called `finality_branch` in the spec.
    fn finalized_beacon_header_branch(&self) -> Vec<H256>;
    /// sync committee aggregate signature
    fn sync_aggregate(&self) -> &SyncAggregate<SYNC_COMMITTEE_SIZE>;
    /// slot at which the aggregate signature was created (untrusted)
    fn signature_slot(&self) -> Slot;
    /// root of execution payload corresponding to `finalized_beacon_header.body_root`
    fn finalized_execution_root(&self) -> H256;
    /// merkle branch of `execution_payload` within `BeaconBlockBody`
    fn finalized_execution_branch(&self) -> Vec<H256>;

    /// ref. https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/altair/light-client/sync-protocol.md#is_valid_light_client_header
    /// NOTE: There are no validation for the execution payload, so you should implement it if the update contains the execution payload.
    fn is_valid_light_client_finalized_header<C: ChainConsensusVerificationContext>(
        &self,
        ctx: &C,
    ) -> Result<(), Error> {
        let spec = ctx.compute_fork_spec(self.finalized_beacon_header().slot);
        is_valid_normalized_merkle_branch(
            self.finalized_execution_root(),
            &self.finalized_execution_branch(),
            spec.execution_payload_gindex,
            self.finalized_beacon_header().body_root,
        )
        .map_err(Error::InvalidFinalizedExecutionPayload)
    }

    /// Returns whether the contained next sync committee is finalized
    fn has_finalized_next_sync_committee<C: ChainContext>(&self, ctx: &C) -> bool {
        self.next_sync_committee().is_some()
            && compute_sync_committee_period_at_slot(ctx, self.attested_beacon_header().slot)
                == compute_sync_committee_period_at_slot(ctx, self.finalized_beacon_header().slot)
    }

    /// validate the basic properties of the update
    fn validate_basic<C: ConsensusVerificationContext>(&self, ctx: &C) -> Result<(), Error> {
        // ensure that sync committee's aggregated key matches pubkeys
        if let Some(next_sync_committee) = self.next_sync_committee() {
            next_sync_committee.validate()?;
        }

        // ensure that the order of slots is consistent
        // equivalent to:
        // `assert current_slot >= update.signature_slot > update_attested_slot >= update_finalized_slot``
        // from the spec: https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/altair/light-client/sync-protocol.md?plain=1#L380
        if !(ctx.current_slot() >= self.signature_slot()
            && self.signature_slot() > self.attested_beacon_header().slot
            && self.attested_beacon_header().slot >= self.finalized_beacon_header().slot)
        {
            return Err(Error::InconsistentSlotOrder(
                ctx.current_slot(),
                self.signature_slot(),
                self.attested_beacon_header().slot,
                self.finalized_beacon_header().slot,
            ));
        }
        Ok(())
    }
}

/// ExecutionUpdate is an update info of the execution payload
pub trait ExecutionUpdate: core::fmt::Debug + Clone + PartialEq + Eq {
    /// `state_root` of the execution payload
    fn state_root(&self) -> H256;
    /// merkle branch of `state_root` within `ExecutionPayload`
    fn state_root_branch(&self) -> Vec<H256>;
    /// `block_number` of the execution payload
    fn block_number(&self) -> U64;
    /// merkle branch of `block_number` within `ExecutionPayload`
    fn block_number_branch(&self) -> Vec<H256>;
    /// validate the basic properties of the update
    fn validate_basic(&self) -> Result<(), Error> {
        if self.state_root_branch().is_empty() {
            return Err(Error::EmptyExecutionPayloadStateRootBranch);
        }
        if self.block_number_branch().is_empty() {
            return Err(Error::EmptyExecutionPayloadBlockNumberBranch);
        }
        Ok(())
    }
}

/// LightClientBootstrapInfo is a basic type for the light client bootstrap
pub type LightClientBootstrapInfo<const SYNC_COMMITTEE_SIZE: usize> =
    bellatrix::LightClientBootstrapInfo<SYNC_COMMITTEE_SIZE>;

/// LightClientUpdate is a basic type for the light client update
pub type LightClientUpdate<const SYNC_COMMITTEE_SIZE: usize> =
    crate::consensus::src::fork::bellatrix::LightClientUpdate<SYNC_COMMITTEE_SIZE>;

/// ConsensusUpdateInfo is a basic type for the consensus update
pub type ConsensusUpdateInfo<const SYNC_COMMITTEE_SIZE: usize> =
    bellatrix::ConsensusUpdateInfo<SYNC_COMMITTEE_SIZE>;

/// ExecutionUpdateInfo is a basic type for the execution update
pub type ExecutionUpdateInfo = bellatrix::ExecutionUpdateInfo;
