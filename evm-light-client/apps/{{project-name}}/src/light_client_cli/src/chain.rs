use super::errors::Error;
use crate::consensus::src::{
    config::{self, Config},
    types::H256,
};
use crate::light_client_verifier::src::updates::deneb::LightClientBootstrapInfo;
use crate::lodestar_rpc::src::client::RPCClient;
use std::str::FromStr;

type Result<T> = core::result::Result<T, Error>;

pub struct Chain {
    pub(crate) rpc_client: RPCClient,
}

impl Chain {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            rpc_client: RPCClient::new(endpoint),
        }
    }

    pub fn get_bootstrap<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    >(
        &self,
        finalized_root: Option<H256>,
    ) -> Result<
        LightClientBootstrapInfo<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    > {
        let finalized_root = if let Some(finalized_root) = finalized_root {
            finalized_root
        } else {
            self.rpc_client
                .get_finality_checkpoints()
                ?
                .data
                .finalized
                .root
        };
        Ok(LightClientBootstrapInfo(
            self.rpc_client
                .get_bootstrap::<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>(
                    finalized_root,
                )
                ?
                .data
                .into(),
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Network {
    Minimal,
    Mainnet,
    Holesky,
    Sepolia,
}

impl Network {
    pub fn config(&self) -> Config {
        match self {
            Network::Minimal => config::minimal::get_config(),
            Network::Mainnet => config::mainnet::get_config(),
            Network::Holesky => config::holesky::get_config(),
            Network::Sepolia => config::sepolia::get_config(),
        }
    }
}

impl FromStr for Network {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "minimal" => Ok(Network::Minimal),
            "mainnet" => Ok(Network::Mainnet),
            "holesky" => Ok(Network::Holesky),
            "sepolia" => Ok(Network::Sepolia),
            s => Err(Error::Other {
                description: format!("unknown network: {}", s),
            }),
        }
    }
}
