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
        preset: preset::minimal::PRESET,
        fork_parameters: ForkParameters::new(
            Version([0, 0, 0, 1]),
            vec![
                ForkParameter::new(Version([1, 0, 0, 1]), U64(0), ALTAIR_FORK_SPEC),
                ForkParameter::new(Version([2, 0, 0, 1]), U64(0), BELLATRIX_FORK_SPEC),
                ForkParameter::new(Version([3, 0, 0, 1]), U64(0), CAPELLA_FORK_SPEC),
                ForkParameter::new(Version([4, 0, 0, 1]), U64(0), DENEB_FORK_SPEC),
                ForkParameter::new(Version([5, 0, 0, 1]), U64(0), ELECTRA_FORK_SPEC),
            ],
        )
        .unwrap(),
        min_genesis_time: U64(1578009600),
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
