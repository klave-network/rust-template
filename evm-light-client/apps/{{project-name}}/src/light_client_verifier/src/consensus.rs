use super::context::{ChainConsensusVerificationContext, ConsensusVerificationContext};
use super::errors::Error;
use super::misbehaviour::Misbehaviour;
use super::state::{get_sync_committee_at_period, LightClientStoreReader};
use super::updates::{ConsensusUpdate, ExecutionUpdate, LightClientBootstrap};
use core::marker::PhantomData;
use crate::consensus::src::beacon::{BeaconBlockHeader, Root, DOMAIN_SYNC_COMMITTEE};
use crate::consensus::src::bls::{fast_aggregate_verify, BLSPublicKey, BLSSignature};
use crate::consensus::src::compute::{
    compute_domain, compute_epoch_at_slot, compute_fork_version, compute_signing_root,
    compute_sync_committee_period_at_slot, hash_tree_root,
};
use crate::consensus::src::context::ChainContext;
use crate::consensus::src::fork::{ForkSpec, BELLATRIX_INDEX};
use crate::consensus::src::merkle::is_valid_normalized_merkle_branch;
use crate::consensus::src::sync_protocol::SyncCommittee;
use crate::consensus::src::types::H256;

/// SyncProtocolVerifier is a verifier of [light client sync protocol](https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncProtocolVerifier<
    const SYNC_COMMITTEE_SIZE: usize,
    ST: LightClientStoreReader<SYNC_COMMITTEE_SIZE>,
>(PhantomData<ST>);

impl<const SYNC_COMMITTEE_SIZE: usize, ST: LightClientStoreReader<SYNC_COMMITTEE_SIZE>>
    SyncProtocolVerifier<SYNC_COMMITTEE_SIZE, ST>
{
    /// validates a LightClientBootstrap
    pub fn validate_boostrap<
        CC: ChainConsensusVerificationContext,
        LB: LightClientBootstrap<SYNC_COMMITTEE_SIZE>,
    >(
        &self,
        ctx: &CC,
        bootstrap: &LB,
        trusted_block_root: Option<Root>,
    ) -> Result<(), Error> {
        if let Some(trusted_block_root) = trusted_block_root {
            let root = hash_tree_root(bootstrap.beacon_header().clone())?;
            if trusted_block_root != root {
                return Err(Error::TrustedRootMismatch(trusted_block_root, root));
            }
        }
        let fork_spec = ctx.compute_fork_spec(bootstrap.beacon_header().slot);
        is_valid_normalized_merkle_branch(
            hash_tree_root(bootstrap.current_sync_committee().clone())?,
            &bootstrap.current_sync_committee_branch(),
            fork_spec.current_sync_committee_gindex,
            bootstrap.beacon_header().state_root,
        )
        .map_err(Error::InvalidCurrentSyncCommitteeMerkleBranch)?;
        Ok(())
    }

    /// validates consensus update and execution update
    pub fn validate_updates<
        CC: ChainConsensusVerificationContext,
        CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>,
        EU: ExecutionUpdate,
    >(
        &self,
        ctx: &CC,
        store: &ST,
        consensus_update: &CU,
        execution_update: &EU,
    ) -> Result<(), Error> {
        self.validate_consensus_update(ctx, store, consensus_update)?;
        self.validate_execution_update(
            ctx.compute_fork_spec(consensus_update.finalized_beacon_header().slot),
            consensus_update.finalized_execution_root(),
            execution_update,
        )?;
        Ok(())
    }

    /// validate a consensus update with a committee from the trusted store
    /// follow the light client protocol in the consensus spec
    ///
    /// If the return value is `Ok`, the update satisfies the following conditions:
    /// * the update is valid light client update:
    ///   * all merkle branches are valid
    ///   * the number of committee signatures is sufficient
    /// * the update is relevant to the store
    /// * the signature period matches the store's current or next period
    /// * the attested period matches the finalized period or finalized period + 1
    pub fn validate_consensus_update<
        CC: ChainConsensusVerificationContext,
        CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>,
    >(
        &self,
        ctx: &CC,
        store: &ST,
        consensus_update: &CU,
    ) -> Result<(), Error> {
        validate_light_client_update(ctx, store, consensus_update)?;
        let sync_committee = self.get_sync_committee(ctx, store, consensus_update)?;
        verify_sync_committee_attestation(ctx, consensus_update, &sync_committee)?;
        Ok(())
    }

    /// validate an execution update with trusted/verified beacon block body
    pub fn validate_execution_update<EU: ExecutionUpdate>(
        &self,
        update_fork_spec: ForkSpec,
        trusted_execution_root: Root,
        execution_update: &EU,
    ) -> Result<(), Error> {
        execution_update.validate_basic()?;
        if update_fork_spec.execution_payload_gindex == 0 {
            return Err(Error::NoExecutionPayloadInBeaconBlock);
        }
        is_valid_normalized_merkle_branch(
            hash_tree_root(execution_update.state_root())
                .unwrap()
                .0
                .into(),
            &execution_update.state_root_branch(),
            update_fork_spec.execution_payload_state_root_gindex,
            trusted_execution_root,
        )
        .map_err(Error::InvalidExecutionStateRootMerkleBranch)?;

        is_valid_normalized_merkle_branch(
            hash_tree_root(execution_update.block_number())
                .unwrap()
                .0
                .into(),
            &execution_update.block_number_branch(),
            update_fork_spec.execution_payload_block_number_gindex,
            trusted_execution_root,
        )
        .map_err(Error::InvalidExecutionBlockNumberMerkleBranch)?;

        Ok(())
    }

    /// validates a misbehaviour with the store.
    /// it returns `Ok` if the misbehaviour is valid
    pub fn validate_misbehaviour<
        CC: ChainConsensusVerificationContext,
        CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>,
    >(
        &self,
        ctx: &CC,
        store: &ST,
        misbehaviour: Misbehaviour<SYNC_COMMITTEE_SIZE, CU>,
    ) -> Result<(), Error> {
        misbehaviour.validate_basic(ctx)?;
        let (update_1, update_2) = misbehaviour.updates();
        self.validate_consensus_update(ctx, store, &update_1)?;
        self.validate_consensus_update(ctx, store, &update_2)?;
        Ok(())
    }

    /// get the sync committee corresponding to the update signature period from the store
    pub fn get_sync_committee<CC: ChainContext, CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>>(
        &self,
        ctx: &CC,
        store: &ST,
        update: &CU,
    ) -> Result<SyncCommittee<SYNC_COMMITTEE_SIZE>, Error> {
        let update_signature_period =
            compute_sync_committee_period_at_slot(ctx, update.signature_slot());
        if let Some(committee) = get_sync_committee_at_period(ctx, store, update_signature_period) {
            Ok(committee)
        } else {
            Err(Error::UnexpectedSingaturePeriod(
                store.current_period(ctx),
                update_signature_period,
                "store does not have the sync committee corresponding to the update signature period"
                    .into(),
            ))
        }
    }
}

/// verify a sync committee attestation
pub fn verify_sync_committee_attestation<
    const SYNC_COMMITTEE_SIZE: usize,
    CC: ChainContext + ConsensusVerificationContext,
    CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>,
>(
    ctx: &CC,
    consensus_update: &CU,
    sync_committee: &SyncCommittee<SYNC_COMMITTEE_SIZE>,
) -> Result<(), Error> {
    // ensure that suffienct participants exist
    let participants = consensus_update.sync_aggregate().count_participants();
    // from the spec: `assert sum(sync_aggregate.sync_committee_bits) >= MIN_SYNC_COMMITTEE_PARTICIPANTS`
    if participants < ctx.min_sync_committee_participants() {
        return Err(Error::LessThanMinimalParticipants(
            participants,
            ctx.min_sync_committee_participants(),
        ));
    } else if participants as u64 * ctx.signature_threshold().denominator()
        < consensus_update.sync_aggregate().sync_committee_bits.len() as u64
            * ctx.signature_threshold().numerator()
    {
        return Err(Error::InsufficientParticipants(
            participants as u64,
            consensus_update.sync_aggregate().sync_committee_bits.len() as u64,
        ));
    }

    let participant_pubkeys: Vec<BLSPublicKey> = consensus_update
        .sync_aggregate()
        .sync_committee_bits
        .iter()
        .zip(sync_committee.pubkeys.iter())
        .filter(|it| it.0 == true)
        .map(|t| t.1.clone().try_into().unwrap())
        .collect();

    let fork_version_slot = consensus_update.signature_slot().max(1.into()) - 1;
    let fork_version = compute_fork_version(ctx, compute_epoch_at_slot(ctx, fork_version_slot));
    let domain = compute_domain(
        ctx,
        DOMAIN_SYNC_COMMITTEE,
        Some(fork_version),
        Some(ctx.genesis_validators_root()),
    )?;
    let signing_root =
        compute_signing_root(consensus_update.attested_beacon_header().clone(), domain)?;

    verify_bls_signatures(
        participant_pubkeys,
        signing_root,
        consensus_update
            .sync_aggregate()
            .sync_committee_signature
            .clone()
            .try_into()?,
    )
}

/// validate_light_client_update validates a light client update
///
/// NOTE: we can skip the validation of the attested header's execution payload inclusion here because we do not use it in our light client implementation.
pub fn validate_light_client_update<
    const SYNC_COMMITTEE_SIZE: usize,
    CC: ChainConsensusVerificationContext,
    ST: LightClientStoreReader<SYNC_COMMITTEE_SIZE>,
    CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>,
>(
    ctx: &CC,
    store: &ST,
    consensus_update: &CU,
) -> Result<(), Error> {
    consensus_update.validate_basic(ctx)?;
    let finalized_epoch =
        compute_epoch_at_slot(ctx, consensus_update.finalized_beacon_header().slot);
    if !ctx
        .fork_parameters()
        .is_fork(finalized_epoch, BELLATRIX_INDEX)
    {
        return Err(Error::ForkNotSupported(finalized_epoch));
    }

    let current_period = store.current_period(ctx);
    let signature_period =
        compute_sync_committee_period_at_slot(ctx, consensus_update.signature_slot());
    // ensure that the update is relevant to the store
    // the `store` only has the current and next sync committee, so the signature period must match the current or next period
    if current_period != signature_period && current_period + 1 != signature_period {
        return Err(Error::StoreNotCoveredSignaturePeriod(
            current_period,
            signature_period,
        ));
    }
    store.ensure_relevant_update(ctx, consensus_update)?;

    // https://github.com/ethereum/consensus-specs/blob/087e7378b44f327cdad4549304fc308613b780c3/specs/altair/light-client/sync-protocol.md#validate_light_client_update
    // Verify that the `finality_branch`, if present, confirms `finalized_header`
    // to match the finalized checkpoint root saved in the state of `attested_header`.
    // Note that the genesis finalized checkpoint root is represented as a zero hash.
    // if not is_finality_update(update):
    //     assert update.finalized_header == LightClientHeader()
    // else:
    //     if update_finalized_slot == GENESIS_SLOT:
    //         assert update.finalized_header == LightClientHeader()
    //         finalized_root = Bytes32()
    //     else:
    //         assert is_valid_light_client_header(update.finalized_header)
    //         finalized_root = hash_tree_root(update.finalized_header.beacon)
    //     assert is_valid_normalized_merkle_branch(
    //         leaf=finalized_root,
    //         branch=update.finality_branch,
    //         gindex=finalized_root_gindex_at_slot(update.attested_header.beacon.slot),
    //         root=update.attested_header.beacon.state_root,
    //     )

    // we assume that the `finalized_beacon_header_branch`` must be non-empty
    if consensus_update.finalized_beacon_header_branch().is_empty() {
        return Err(Error::FinalizedHeaderNotFound);
    }
    let finalized_root = if consensus_update.finalized_beacon_header().slot
        == ctx.fork_parameters().genesis_slot()
    {
        if consensus_update.finalized_beacon_header() != &BeaconBlockHeader::default() {
            return Err(Error::NonEmptyBeaconHeaderAtGenesisSlot(
                ctx.fork_parameters().genesis_slot().into(),
            ));
        }
        Default::default()
    } else {
        // ensure that the finalized header is non-empty
        if consensus_update.finalized_beacon_header() == &BeaconBlockHeader::default() {
            return Err(Error::FinalizedHeaderNotFound);
        }
        consensus_update.is_valid_light_client_finalized_header(ctx)?;
        hash_tree_root(consensus_update.finalized_beacon_header().clone())?
    };
    is_valid_normalized_merkle_branch(
        finalized_root,
        &consensus_update.finalized_beacon_header_branch(),
        ctx.compute_fork_spec(consensus_update.attested_beacon_header().slot)
            .finalized_root_gindex,
        consensus_update.attested_beacon_header().state_root,
    )
    .map_err(Error::InvalidFinalizedBeaconHeaderMerkleBranch)?;

    // # Verify that the `next_sync_committee`, if present, actually is the next sync committee saved in the
    // # state of the `attested_header`
    // if not is_sync_committee_update(update):
    //     assert update.next_sync_committee == SyncCommittee()
    // else:
    //     if update_attested_period == store_period and is_next_sync_committee_known(store):
    //         assert update.next_sync_committee == store.next_sync_committee
    //     assert is_valid_normalized_merkle_branch(
    //         leaf=hash_tree_root(update.next_sync_committee),
    //         branch=update.next_sync_committee_branch,
    //         gindex=next_sync_committee_gindex_at_slot(update.attested_header.beacon.slot),
    //         root=update.attested_header.beacon.state_root,
    //     )
    if let Some(update_next_sync_committee) = consensus_update.next_sync_committee() {
        let update_attested_period = compute_sync_committee_period_at_slot(
            ctx,
            consensus_update.attested_beacon_header().slot,
        );
        if let Some(committee) =
            get_sync_committee_at_period(ctx, store, update_attested_period + 1)
        {
            if committee != *update_next_sync_committee {
                return Err(Error::InconsistentNextSyncCommittee(
                    committee.aggregate_pubkey.clone(),
                    update_next_sync_committee.aggregate_pubkey.clone(),
                ));
            }
        }
        is_valid_normalized_merkle_branch(
            hash_tree_root(update_next_sync_committee.clone())?,
            &consensus_update.next_sync_committee_branch().unwrap(),
            ctx.compute_fork_spec(consensus_update.attested_beacon_header().slot)
                .next_sync_committee_gindex,
            consensus_update.attested_beacon_header().state_root,
        )
        .map_err(Error::InvalidNextSyncCommitteeMerkleBranch)?;
    } else if let Some(branch) = consensus_update.next_sync_committee_branch() {
        return Err(Error::NonEmptyNextSyncCommittee(branch.to_vec()));
    }

    Ok(())
}

pub fn verify_bls_signatures(
    pubkeys: Vec<BLSPublicKey>,
    msg: H256,
    signature: BLSSignature,
) -> Result<(), Error> {
    if fast_aggregate_verify(pubkeys, msg, signature)? {
        Ok(())
    } else {
        Err(Error::InvalidBLSSignatures)
    }
}
