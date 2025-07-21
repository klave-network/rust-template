pub mod altair;
pub mod bellatrix;
pub mod capella;
pub mod deneb;
pub mod electra;

use super::beacon::{Epoch, Slot, Version};
use super::errors::Error;
use super::types::U64;

pub const GENESIS_SPEC: ForkSpec = ForkSpec {
    finalized_root_gindex: 105,
    current_sync_committee_gindex: 0,
    next_sync_committee_gindex: 0,
    execution_payload_gindex: 0,
    execution_payload_state_root_gindex: 0,
    execution_payload_block_number_gindex: 0,
};

pub const ALTAIR_INDEX: usize = 0;
pub const BELLATRIX_INDEX: usize = 1;
pub const CAPELLA_INDEX: usize = 2;
pub const DENEB_INDEX: usize = 3;
pub const ELECTRA_INDEX: usize = 4;

/// Fork parameters for the beacon chain
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForkParameters {
    genesis_version: Version,
    /// Forks in order of ascending epoch
    /// The first element is the first fork after genesis
    /// i.e., [Altair, Bellatrix, Capella, Deneb, ...]
    forks: Vec<ForkParameter>,
}

impl ForkParameters {
    pub fn new(genesis_version: Version, forks: Vec<ForkParameter>) -> Result<Self, Error> {
        let this = Self {
            genesis_version,
            forks,
        };
        this.validate()?;
        Ok(this)
    }

    fn validate(&self) -> Result<(), Error> {
        if self.forks.is_empty() {
            return Err(Error::NotSupportedLightClient);
        }
        if self.forks.windows(2).all(|f| f[0].epoch <= f[1].epoch) {
            Ok(())
        } else {
            Err(Error::InvalidForkParamersOrder(self.clone()))
        }
    }

    pub fn genesis_slot(&self) -> Slot {
        U64(0)
    }

    pub fn genesis_version(&self) -> &Version {
        &self.genesis_version
    }

    pub fn forks(&self) -> &[ForkParameter] {
        &self.forks
    }

    /// Compute the fork version for the given epoch
    pub fn compute_fork_version(&self, epoch: Epoch) -> Version {
        self.compute_fork(epoch)
            .map(|(_, f)| f.version.clone())
            .unwrap_or(self.genesis_version.clone())
    }

    /// Compute the fork spec for the given epoch
    pub fn compute_fork_spec(&self, epoch: Epoch) -> ForkSpec {
        self.compute_fork(epoch)
            .map(|(_, f)| f.spec.clone())
            .unwrap_or(GENESIS_SPEC)
    }

    /// Returns a boolean indicating whether the given epoch is after the fork
    pub fn is_fork(&self, epoch: Epoch, fork_index: usize) -> bool {
        if let Some((current, _)) = self.compute_fork(epoch) {
            current >= fork_index
        } else {
            false
        }
    }

    fn compute_fork(&self, epoch: Epoch) -> Option<(usize, &ForkParameter)> {
        self.forks
            .iter()
            .enumerate()
            .rev()
            .find(|(_, f)| epoch >= f.epoch)
    }
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#constants
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForkSpec {
    /// get_generalized_index(BeaconState, 'finalized_checkpoint', 'root')
    pub finalized_root_gindex: u32,
    /// get_generalized_index(BeaconState, 'current_sync_committee')
    pub current_sync_committee_gindex: u32,
    /// get_generalized_index(BeaconState, 'next_sync_committee')
    pub next_sync_committee_gindex: u32,
    /// get_generalized_index(BeaconBlockBody, 'execution_payload')
    pub execution_payload_gindex: u32,
    /// get_generalized_index(ExecutionPayload, 'state_root')
    pub execution_payload_state_root_gindex: u32,
    /// get_generalized_index(ExecutionPayload, 'block_number')
    pub execution_payload_block_number_gindex: u32,
}

/// Fork parameters for each fork
/// In the mainnet, you can find the parameters here: https://github.com/ethereum/consensus-specs/blob/9849fb39e75e6228ebd610ef0ad22f5b41543cd5/configs/mainnet.yaml#L35
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForkParameter {
    pub version: Version,
    pub epoch: Epoch,
    pub spec: ForkSpec,
}

impl ForkParameter {
    pub const fn new(version: Version, epoch: Epoch, spec: ForkSpec) -> Self {
        Self {
            version,
            epoch,
            spec,
        }
    }
}

#[cfg(test)]
mod tests {
    use altair::ALTAIR_FORK_SPEC;
    use bellatrix::BELLATRIX_FORK_SPEC;
    use capella::CAPELLA_FORK_SPEC;
    use deneb::DENEB_FORK_SPEC;
    use electra::ELECTRA_FORK_SPEC;

    use super::*;

    #[test]
    pub fn test_fork_parameters() {
        let res = ForkParameters::new(
            Version([0, 0, 0, 1]),
            vec![
                ForkParameter::new(Version([1, 0, 0, 1]), U64(0), ALTAIR_FORK_SPEC),
                ForkParameter::new(Version([2, 0, 0, 1]), U64(0), BELLATRIX_FORK_SPEC),
                ForkParameter::new(Version([3, 0, 0, 1]), U64(0), CAPELLA_FORK_SPEC),
                ForkParameter::new(Version([4, 0, 0, 1]), U64(0), DENEB_FORK_SPEC),
                ForkParameter::new(Version([5, 0, 0, 1]), U64(0), ELECTRA_FORK_SPEC),
            ],
        );
        assert!(res.is_ok());
        let params = res.unwrap();
        assert_eq!(params.compute_fork_version(0.into()), Version([4, 0, 0, 1]));

        let res = ForkParameters::new(Version([0, 0, 0, 1]), vec![]);
        assert!(res.is_err());

        let res = ForkParameters::new(
            Version([0, 0, 0, 1]),
            vec![ForkParameter::new(
                Version([1, 0, 0, 1]),
                U64(0),
                ALTAIR_FORK_SPEC,
            )],
        );
        let params = res.unwrap();
        assert_eq!(params.compute_fork_version(0.into()), Version([1, 0, 0, 1]));

        let res = ForkParameters::new(
            Version([0, 0, 0, 1]),
            vec![
                ForkParameter::new(Version([1, 0, 0, 1]), U64(0), ALTAIR_FORK_SPEC),
                ForkParameter::new(Version([2, 0, 0, 1]), U64(1), BELLATRIX_FORK_SPEC),
                ForkParameter::new(Version([3, 0, 0, 1]), U64(2), CAPELLA_FORK_SPEC),
                ForkParameter::new(Version([4, 0, 0, 1]), U64(3), DENEB_FORK_SPEC),
                ForkParameter::new(Version([5, 0, 0, 1]), U64(4), ELECTRA_FORK_SPEC),
            ],
        );
        assert!(res.is_ok());
        let params = res.unwrap();
        assert_eq!(params.compute_fork_version(0.into()), Version([1, 0, 0, 1]));
        assert_eq!(params.compute_fork_version(1.into()), Version([2, 0, 0, 1]));
        assert_eq!(params.compute_fork_version(2.into()), Version([3, 0, 0, 1]));
        assert_eq!(params.compute_fork_version(3.into()), Version([4, 0, 0, 1]));
        assert_eq!(params.compute_fork_version(4.into()), Version([5, 0, 0, 1]));
        assert!(params.is_fork(0.into(), ALTAIR_INDEX));
        assert!(!params.is_fork(0.into(), BELLATRIX_INDEX));
        assert!(params.is_fork(1.into(), ALTAIR_INDEX));
        assert!(params.is_fork(1.into(), BELLATRIX_INDEX));
        assert!(!params.is_fork(1.into(), CAPELLA_INDEX));
        assert!(params.is_fork(3.into(), DENEB_INDEX));
        assert!(params.is_fork(4.into(), DENEB_INDEX));
        assert!(params.is_fork(5.into(), ELECTRA_INDEX));

        let res = ForkParameters::new(
            Version([0, 0, 0, 1]),
            vec![
                ForkParameter::new(Version([2, 0, 0, 1]), U64(1), BELLATRIX_FORK_SPEC),
                ForkParameter::new(Version([1, 0, 0, 1]), U64(0), ALTAIR_FORK_SPEC),
            ],
        );
        assert!(res.is_err());
    }
}
