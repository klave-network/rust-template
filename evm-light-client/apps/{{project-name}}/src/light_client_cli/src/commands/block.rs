use crate::light_client_cli::src::context::Context;
use crate::lodestar_rpc::src::client::RPCClient;
use anyhow::Result;
use clap::Parser;

#[derive(Clone, Debug, Parser, PartialEq)]
pub struct BlockCommand {
    #[clap(long = "slot", help = "Slot number")]
    pub slot: Option<u64>,
}

impl BlockCommand {
    pub fn run<
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const SYNC_COMMITTEE_SIZE: usize,
    >(
        self,
        ctx: Context<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE>,
    ) -> Result<()> {
        let client = RPCClient::new(ctx.beacon_endpoint());

        let res = match self.slot {
            Some(slot) => client.get_beacon_block_by_slot(slot.into())?,
            None => {
                let res = client.get_finality_update::<
                    SYNC_COMMITTEE_SIZE,
                    BYTES_PER_LOGS_BLOOM,
                    MAX_EXTRA_DATA_BYTES,
                >()?;
                client.get_beacon_block_by_slot(res.data.finalized_header.beacon.slot)?
            }
        };
        klave::notifier::send_string(&(serde_json::to_string(&res.data.message)?).to_string());
        Ok(())
    }
}
