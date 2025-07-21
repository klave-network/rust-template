use super::{fork::ForkParameters, preset::Preset, types::U64};

pub mod holesky;
pub mod mainnet;
pub mod minimal;
pub mod sepolia;

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub preset: Preset,
    pub fork_parameters: ForkParameters,
    pub min_genesis_time: U64,
}
