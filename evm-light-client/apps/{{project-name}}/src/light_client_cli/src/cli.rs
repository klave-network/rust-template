use anyhow::Result;
use clap::Parser;
use std::str::FromStr;

use crate::light_client_cli::src::{
    chain::Network,
    commands::Command,
    context::Context,
    preset::{MainnetContext, MinimalContext},
};
    
#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct Cli {
    #[command(flatten)]
    pub opts: Opts,

    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Parser, Clone)]
pub struct Opts {
    #[arg(long = "home", help = "Path to home directory")]
    pub ledger_table: String,

    #[arg(long = "beacon_endpoint")]
    pub beacon_endpoint: String,

    #[arg(long = "network")]
    pub network: String,
}

impl Opts {
    pub fn ledger_table(&self) -> String {
        self.ledger_table.clone()
    }

    pub fn get_network(&self) -> Result<Network> {
        Network::from_str(&self.network).map_err(Into::into)
    }
}

impl Cli {
    pub fn run(self) -> Result<()> {
        let local_network = match self.opts.get_network() {
            Ok(network) => network,
            Err(e) => {
                eprintln!("Invalid network: {}", e);
                return Ok(());
            }
        };

        let opts = self.opts.clone();
        match local_network {
            Network::Mainnet | Network::Holesky | Network::Sepolia => {
                self.run_with_context(MainnetContext::build(local_network, opts)?)                    
            }
            Network::Minimal => {
                self.run_with_context(MinimalContext::build(local_network, opts)?)
                    
            }
        }
    }

    fn run_with_context<
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const SYNC_COMMITTEE_SIZE: usize,
    >(
        self,
        ctx: Context<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE>,
    ) -> Result<()> {
        match self.cmd {
            Command::Init(cmd) => cmd.run(ctx),
            Command::Persist(cmd) => cmd.run(ctx),
            Command::Update(cmd) => cmd.run(ctx),
            Command::Header(cmd) => cmd.run(ctx),
            Command::Block(cmd) => cmd.run(ctx),
        }
    }
}
