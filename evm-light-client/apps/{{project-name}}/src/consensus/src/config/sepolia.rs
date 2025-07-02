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
            Version([144, 0, 0, 105]),
            vec![
                ForkParameter::new(Version([144, 0, 0, 112]), U64(50), ALTAIR_FORK_SPEC),
                ForkParameter::new(Version([144, 0, 0, 113]), U64(100), BELLATRIX_FORK_SPEC),
                ForkParameter::new(Version([144, 0, 0, 114]), U64(56832), CAPELLA_FORK_SPEC),
                ForkParameter::new(Version([144, 0, 0, 115]), U64(132608), DENEB_FORK_SPEC),
                ForkParameter::new(Version([144, 0, 0, 116]), U64(222464), ELECTRA_FORK_SPEC),
                ForkParameter::new(Version([144, 0, 0, 117]), U64(u64::MAX), Default::default()),
            ],
        )
        .unwrap(),
        min_genesis_time: U64(1655647200),
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
