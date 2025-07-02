use displaydoc::Display;
use crate::consensus::src::sync_protocol::SyncCommitteePeriod;

#[derive(Debug, Display)]
pub enum Error {
    /// rpc error: `{0}`
    RPCError(crate::lodestar_rpc::src::errors::Error),
    /// io error: `{0}`
    IOError(std::io::Error),
    /// serde error: `{0}`
    SerdeError(serde_json::Error),
    /// verifier error: `{0}`
    VerifierError(crate::light_client_verifier::src::errors::Error),
    /// common error: `{0}`
    CommontError(crate::consensus::src::errors::Error),
    /// finalized header not found
    FinalizedHeaderNotFound,
    /// unexpected attested period: `store={0} attested={1} reason={2}`
    UnexpectedAttestedPeriod(SyncCommitteePeriod, SyncCommitteePeriod, String),
    /// cannot rotate to next sync committee: `store={0} finalized={1}`
    CannotRotateNextSyncCommittee(SyncCommitteePeriod, SyncCommitteePeriod),
    /// other error: `{description}`
    Other { description: String },
}

impl std::error::Error for Error {}

impl From<crate::lodestar_rpc::src::errors::Error> for Error {
    fn from(value: crate::lodestar_rpc::src::errors::Error) -> Self {
        Self::RPCError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeError(value)
    }
}

impl From<crate::light_client_verifier::src::errors::Error> for Error {
    fn from(value: crate::light_client_verifier::src::errors::Error) -> Self {
        Self::VerifierError(value)
    }
}

impl From<crate::consensus::src::errors::Error> for Error {
    fn from(value: crate::consensus::src::errors::Error) -> Self {
        Self::CommontError(value)
    }
}
