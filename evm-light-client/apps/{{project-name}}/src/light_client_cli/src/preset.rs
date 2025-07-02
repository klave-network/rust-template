use super::context::Context;
use crate::consensus::src::preset::{mainnet, minimal};

pub type MainnetContext = Context<
    { mainnet::PRESET.BYTES_PER_LOGS_BLOOM },
    { mainnet::PRESET.MAX_EXTRA_DATA_BYTES },
    { mainnet::PRESET.SYNC_COMMITTEE_SIZE },
>;

pub type MinimalContext = Context<
    { minimal::PRESET.BYTES_PER_LOGS_BLOOM },
    { minimal::PRESET.MAX_EXTRA_DATA_BYTES },
    { minimal::PRESET.SYNC_COMMITTEE_SIZE },
>;
