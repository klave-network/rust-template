use super::{
    beacon::{
        BeaconBlockHeader, Domain, DomainType, Epoch, ForkData, Root, SigningData, Slot, Version,
    },
    context::ChainContext,
    errors::Error,
    sync_protocol::SyncCommitteePeriod,
    types::{H256, U64},
};

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#compute_timestamp_at_slot
pub fn compute_timestamp_at_slot<C: ChainContext>(ctx: &C, slot: Slot) -> U64 {
    let slots_since_genesis = slot - ctx.fork_parameters().genesis_slot();
    ctx.genesis_time() + slots_since_genesis * ctx.seconds_per_slot()
}

/// compute_slot_at_timestamp returns the slot number at the given timestamp.
pub fn compute_slot_at_timestamp<C: ChainContext>(ctx: &C, timestamp: U64) -> Slot {
    let slots_since_genesis = (timestamp - ctx.genesis_time()) / ctx.seconds_per_slot();
    ctx.fork_parameters().genesis_slot() + slots_since_genesis
}

/// compute_sync_committee_period_at_slot returns the sync committee period at slot
pub fn compute_sync_committee_period_at_slot<C: ChainContext>(
    ctx: &C,
    slot: Slot,
) -> SyncCommitteePeriod {
    compute_sync_committee_period(ctx, compute_epoch_at_slot(ctx, slot))
}

/// Return the epoch number at slot
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_epoch_at_slot
pub fn compute_epoch_at_slot<C: ChainContext>(ctx: &C, slot: Slot) -> Epoch {
    slot / ctx.slots_per_epoch()
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/validator.md#sync-committee
pub fn compute_sync_committee_period<C: ChainContext>(
    ctx: &C,
    epoch: Epoch,
) -> SyncCommitteePeriod {
    epoch / ctx.epochs_per_sync_committee_period()
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/fork.md#compute_fork_version
pub fn compute_fork_version<C: ChainContext>(ctx: &C, epoch: Epoch) -> Version {
    ctx.fork_parameters().compute_fork_version(epoch)
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_fork_data_root
pub fn compute_fork_data_root(
    current_version: Version,
    genesis_validators_root: Root,
) -> Result<Root, Error> {
    hash_tree_root(ForkData {
        current_version,
        genesis_validators_root,
    })
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_domain
pub fn compute_domain<C: ChainContext>(
    ctx: &C,
    domain_type: DomainType,
    fork_version: Option<Version>,
    genesis_validators_root: Option<Root>,
) -> Result<Domain, Error> {
    let fork_data_root = compute_fork_data_root(
        fork_version.unwrap_or(ctx.fork_parameters().genesis_version().clone()),
        genesis_validators_root.unwrap_or_default(),
    )?;
    let mut domain: [u8; 32] = Default::default();
    domain[..4].copy_from_slice(&domain_type.0);
    domain[4..].copy_from_slice(&fork_data_root.as_bytes()[..28]);
    Ok(Domain(domain))
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_signing_root
pub fn compute_signing_root(header: BeaconBlockHeader, domain: Domain) -> Result<Root, Error> {
    hash_tree_root(SigningData {
        object_root: hash_tree_root(header)?,
        domain,
    })
}

/// hash_tree_root returns the hash tree root of the object
pub fn hash_tree_root<T: ssz_rs::SimpleSerialize>(mut object: T) -> Result<Root, Error> {
    Ok(H256::from_slice(object.hash_tree_root()?.as_bytes()))
}
