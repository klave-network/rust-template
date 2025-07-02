use crate::consensus::src::{
    config::Config,
    beacon::Version,
    fork::{
        altair::ALTAIR_FORK_SPEC, bellatrix::BELLATRIX_FORK_SPEC, capella::CAPELLA_FORK_SPEC, deneb::DENEB_FORK_SPEC, electra::ELECTRA_FORK_SPEC, ForkParameter, ForkParameters
    },
    preset,
    types::U64,
};

pub fn get_config() -> Config {
    Config {
        preset: preset::mainnet::PRESET,
        fork_parameters: ForkParameters::new(
            Version([0, 0, 16, 32]),
            vec![
                ForkParameter::new(Version([1, 0, 16, 32]), U64(36660), ALTAIR_FORK_SPEC),
                ForkParameter::new(Version([2, 0, 16, 32]), U64(112260), BELLATRIX_FORK_SPEC),
                ForkParameter::new(Version([3, 0, 16, 32]), U64(162304), CAPELLA_FORK_SPEC),
                ForkParameter::new(Version([4, 0, 16, 32]), U64(231680), DENEB_FORK_SPEC),
                ForkParameter::new(Version([5, 0, 16, 32]), U64(u64::MAX), ELECTRA_FORK_SPEC),
                ForkParameter::new(Version([6, 0, 16, 32]), U64(u64::MAX), Default::default()),
            ],
        )
        .unwrap(),
        min_genesis_time: U64(1614588812),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let _ = get_config();
    }
}
