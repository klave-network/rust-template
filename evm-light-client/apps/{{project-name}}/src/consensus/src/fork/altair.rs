use crate::consensus::src::fork::{ForkSpec, GENESIS_SPEC};

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#constants
pub const ALTAIR_FORK_SPEC: ForkSpec = ForkSpec {
    current_sync_committee_gindex: 54,
    next_sync_committee_gindex: 55,
    ..GENESIS_SPEC
};
