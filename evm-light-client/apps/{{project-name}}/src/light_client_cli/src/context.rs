use super::{
    chain::Network, 
    cli::Opts,
    db::{FileDB, DB},
    errors::Error,
    state::LightClientStore
};
use crate::consensus::src::{
    config::Config,
    context::ChainContext,
    fork::deneb::LightClientBootstrap
};
use crate::lodestar_rpc::src::types::GenesisData;
use log::*;
use std::str::FromStr;

#[derive(Debug)]
pub struct Context<
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const SYNC_COMMITTEE_SIZE: usize,
> {
    pub(crate) config: Config,
    pub(crate) beacon_endpoint: String,
    pub(crate) network: Network,
    db: FileDB,
}

impl<
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const SYNC_COMMITTEE_SIZE: usize,
    > Context<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE>
{
    pub fn build(network: Network, opts: Opts) -> Result<Self, Error> {
        let home_dir = opts.ledger_table();
        Ok(Self {
            config: network.config(),
            db: FileDB::open(home_dir)?,
            beacon_endpoint: opts.beacon_endpoint,
            network: Network::from_str(&opts.network)?,
        })
    }

    pub fn beacon_endpoint(&self) -> &str {
        &self.beacon_endpoint
    }

    pub fn network(&self) -> Network {
        self.network.clone()
    }

    /// Store accessors
    pub fn get_bootstrap(
        &self,
    ) -> Result<
        LightClientBootstrap<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
        Error,
    > {
        Ok(serde_json::from_slice(&self.db.get("bootstrap")?.ok_or(
            Error::Other {
                description: "bootstrap not found".into(),
            },
        )?)?)
    }

    pub fn store_boostrap(
        &self,
        bootstrap: &LightClientBootstrap<
            SYNC_COMMITTEE_SIZE,
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
    ) -> Result<(), Error> {
        let value = match serde_json::to_string(bootstrap) {
            Ok(value) => value,
            Err(e) => {
                error!("Failed to serialize bootstrap: {:?}", e);
                return Err(Error::Other {
                    description: "Failed to serialize bootstrap".into(),
                });
            }
        };
        self.db.put("bootstrap", value)?;
        Ok(())
    }

    pub fn get_light_client_state(
        &self,
    ) -> Result<
        LightClientStore<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
        Error,
    > {
        Ok(serde_json::from_slice(&self.db.get("state")?.ok_or(
            Error::Other {
                description: "light_client_state not found".into(),
            },
        )?)?)
    }

    pub fn store_light_client_state(
        &self,
        state: &LightClientStore<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    ) -> Result<(), Error> {
        let value = match serde_json::to_string(state) {
            Ok(value) => value,
            Err(e) => {
                error!("Failed to serialize state: {:?}", e);
                return Err(Error::Other {
                    description: "Failed to serialize state".into(),
                });
            }
        };
        self.db.put("state", value)?;
        Ok(())
    }

    /// Store accessors
    pub fn get_genesis(
        &self,
    ) -> Result<
        GenesisData,
        Error,
    > {
        Ok(serde_json::from_slice(&self.db.get("genesis")?.ok_or(
            Error::Other {
                description: "genesis not found".into(),
            },
        )?)?)
    }

    pub fn store_genesis(
        &self,
        genesis: &GenesisData,
    ) -> Result<(), Error> {
        let value = match serde_json::to_string(genesis) {
            Ok(value) => value,
            Err(e) => {
                error!("Failed to serialize genesis: {:?}", e);
                return Err(Error::Other {
                    description: "Failed to serialize genesis".into(),
                });
            }
        };
        self.db.put("genesis", value)?;
        Ok(())
    }
}

impl<
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const SYNC_COMMITTEE_SIZE: usize,
    > ChainContext for Context<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE>
{
    fn genesis_time(&self) -> crate::consensus::src::types::U64 {
        todo!()
    }

    fn fork_parameters(&self) -> &crate::consensus::src::fork::ForkParameters {
        &self.config.fork_parameters
    }

    fn seconds_per_slot(&self) -> crate::consensus::src::types::U64 {
        self.config.preset.SECONDS_PER_SLOT
    }

    fn slots_per_epoch(&self) -> crate::consensus::src::beacon::Slot {
        self.config.preset.SLOTS_PER_EPOCH
    }

    fn epochs_per_sync_committee_period(&self) -> crate::consensus::src::beacon::Epoch {
        self.config.preset.EPOCHS_PER_SYNC_COMMITTEE_PERIOD
    }
}
