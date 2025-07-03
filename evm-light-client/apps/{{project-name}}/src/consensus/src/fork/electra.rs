use crate::consensus::src::fork::{deneb, ForkSpec};

/// https://github.com/ethereum/consensus-specs/blob/a09d0c321550c5411557674a981e2b444a1178c0/specs/electra/light-client/sync-protocol.md#new-constants
pub const ELECTRA_FORK_SPEC: ForkSpec = ForkSpec {
    finalized_root_gindex: 169,
    current_sync_committee_gindex: 86,
    next_sync_committee_gindex: 87,
    ..deneb::DENEB_FORK_SPEC
};

pub type ExecutionPayloadHeader<
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> = deneb::ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>;

pub type LightClientHeader<const BYTES_PER_LOGS_BLOOM: usize, const MAX_EXTRA_DATA_BYTES: usize> =
    deneb::LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>;
