use super::errors::Error;
use crate::consensus::src::{
    beacon::{Epoch, Root, Slot},
    compute::{compute_epoch_at_slot, compute_slot_at_timestamp},
    config::Config,
    context::ChainContext,
    fork::{ForkParameters, ForkSpec},
    types::U64,
};

/// Fraction is a struct representing a fraction for a threshold.
#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Fraction {
    numerator: u64,
    denominator: u64,
}

impl Fraction {
    pub fn new(numerator: u64, denominator: u64) -> Result<Self, Error> {
        if denominator == 0 || numerator > denominator {
            Err(Error::InvalidFraction(Self {
                numerator,
                denominator,
            }))
        } else {
            Ok(Self {
                numerator,
                denominator,
            })
        }
    }

    pub fn numerator(&self) -> u64 {
        self.numerator
    }

    pub fn denominator(&self) -> u64 {
        self.denominator
    }
}

pub trait ConsensusVerificationContext {
    /// The root of the genesis validators corresponding to the target chain
    /// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconstate
    fn genesis_validators_root(&self) -> Root;

    /// A slot based on verifier's local clock
    fn current_slot(&self) -> Slot;

    /// MIN_SYNC_COMMITTEE_PARTICIPANTS from the spec: https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#misc
    fn min_sync_committee_participants(&self) -> usize;

    /// The threshold of sync committee participation required for valid update
    fn signature_threshold(&self) -> Fraction;
}

pub trait ChainConsensusVerificationContext:
    ChainContext + ConsensusVerificationContext + Sized
{
    /// Compute the fork spec at the given slot
    fn compute_fork_spec(&self, slot: U64) -> ForkSpec {
        self.fork_parameters()
            .compute_fork_spec(compute_epoch_at_slot(self, slot))
    }
}

pub struct LightClientContext {
    fork_parameters: ForkParameters,
    seconds_per_slot: U64,
    slots_per_epoch: Slot,
    epochs_per_sync_committee_period: Epoch,
    genesis_time: U64,

    genesis_validators_root: Root,
    min_sync_committee_participants: usize,
    signature_threshold: Fraction,

    current_timestamp: U64,
}

impl LightClientContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fork_parameters: ForkParameters,
        seconds_per_slot: U64,
        slots_per_epoch: Slot,
        epochs_per_sync_committee_period: Epoch,
        genesis_time: U64,

        genesis_validators_root: Root,
        min_sync_committee_participants: usize,
        signature_threshold: Fraction,
        current_timestamp: U64,
    ) -> Self {
        Self {
            fork_parameters,
            seconds_per_slot,
            slots_per_epoch,
            epochs_per_sync_committee_period,
            genesis_time,

            genesis_validators_root,
            min_sync_committee_participants,
            signature_threshold,

            current_timestamp,
        }
    }

    pub fn new_with_config(
        config: Config,
        genesis_validators_root: Root,
        genesis_time: U64,
        signature_threshold: Fraction,
        current_timestamp: U64,
    ) -> Self {
        Self::new(
            config.fork_parameters,
            config.preset.SECONDS_PER_SLOT,
            config.preset.SLOTS_PER_EPOCH,
            config.preset.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
            genesis_time,
            genesis_validators_root,
            config.preset.MIN_SYNC_COMMITTEE_PARTICIPANTS,
            signature_threshold,
            current_timestamp,
        )
    }
}

impl ConsensusVerificationContext for LightClientContext {
    fn genesis_validators_root(&self) -> Root {
        self.genesis_validators_root
    }

    fn min_sync_committee_participants(&self) -> usize {
        self.min_sync_committee_participants
    }

    fn signature_threshold(&self) -> Fraction {
        self.signature_threshold.clone()
    }

    fn current_slot(&self) -> Slot {
        compute_slot_at_timestamp(self, self.current_timestamp)
    }
}

impl ChainContext for LightClientContext {
    fn genesis_time(&self) -> U64 {
        self.genesis_time
    }

    fn fork_parameters(&self) -> &ForkParameters {
        &self.fork_parameters
    }

    fn seconds_per_slot(&self) -> U64 {
        self.seconds_per_slot
    }

    fn slots_per_epoch(&self) -> Slot {
        self.slots_per_epoch
    }

    fn epochs_per_sync_committee_period(&self) -> Epoch {
        self.epochs_per_sync_committee_period
    }
}

impl ChainConsensusVerificationContext for LightClientContext {}
