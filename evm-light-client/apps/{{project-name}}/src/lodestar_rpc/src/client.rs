use super::errors::Error;
use super::types::{
    BeaconBlockResponse, BeaconBlockRootResponse, BeaconHeaderResponse,
    FinalityCheckpointsResponse, GenesisDataResponse, LightClientBootstrapResponse,
    LightClientFinalityUpdateResponse, LightClientUpdatesResponse,
};
use crate::consensus::src::beacon::Slot;
use crate::consensus::src::sync_protocol::SyncCommitteePeriod;
use crate::consensus::src::types::H256;
use log::debug;
use url::Url;
use serde::de::DeserializeOwned;
use crate::klave_client::src::client::Client as KlaveClient;
use http;

type Result<T> = core::result::Result<T, Error>;

pub struct RPCClient {
    http_client: KlaveClient,
    endpoint: String,
}

impl RPCClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        let url = Url::parse(&endpoint.into()).expect("Invalid URL");
        if url.scheme() != "http" && url.scheme() != "https" {
            panic!("Invalid URL scheme: {}", url.scheme());
        }
        if url.path() != "/" {
            panic!("Invalid URL path: {}", url.path());
        }
        if url.host().is_none() {
            panic!("Invalid URL host: {}", url.host().unwrap());
        }
        if url.query().is_some() {
            panic!("Invalid URL query: {}", url.query().unwrap());
        }
        if url.fragment().is_some() {
            panic!("Invalid URL fragment: {}", url.fragment().unwrap());
        }
        Self {
            http_client: KlaveClient::new(),
            endpoint: url.as_str().strip_suffix("/").unwrap().to_string(),
        }
    }

    // Beacon API

    pub fn get_genesis(&self) -> Result<GenesisDataResponse> {
        self.request_get("/eth/v1/beacon/genesis", None)
    }

    pub fn get_beacon_block_root(&self, slot: Slot) -> Result<BeaconBlockRootResponse> {
        self.request_get(format!("/eth/v1/beacon/blocks/{}/root", slot), None)
            
    }

    pub fn get_beacon_header_by_slot(&self, slot: Slot) -> Result<BeaconHeaderResponse> {
        self.request_get(format!("/eth/v1/beacon/headers/{}", slot), None)
            
    }

    pub fn get_beacon_block_by_slot(&self, slot: Slot) -> Result<BeaconBlockResponse> {
        self.request_get(format!("/eth/v2/beacon/blocks/{}", slot), None)
            
    }

    pub fn get_finality_checkpoints(&self) -> Result<FinalityCheckpointsResponse> {
        self.request_get("/eth/v1/beacon/states/head/finality_checkpoints", None)
            
    }

    // Light Client API

    pub fn get_finality_update<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    >(
        &self,
    ) -> Result<
        LightClientFinalityUpdateResponse<
            SYNC_COMMITTEE_SIZE,
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
    > {
        self.request_get("/eth/v1/beacon/light_client/finality_update", None)
            
    }

    pub fn get_bootstrap<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    >(
        &self,
        finalized_root: H256,
    ) -> Result<
        LightClientBootstrapResponse<
            SYNC_COMMITTEE_SIZE,
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
    > {
        self.request_get(format!(
            "/eth/v1/beacon/light_client/bootstrap/0x{}",
            finalized_root
        ), None)
        
    }

    pub fn get_light_client_updates<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    >(
        &self,
        start_period: SyncCommitteePeriod,
        count: u64,
    ) -> Result<
        LightClientUpdatesResponse<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    > {
        let count = if count < 1 { 1 } else { count };
        for c in (1..=count).rev() {
            let res = self
                .request_get(format!(
                    "/eth/v1/beacon/light_client/updates?start_period={}&count={}",
                    start_period, c
                ), None)
                ;
            if res.is_ok()
                || !res
                    .as_ref()
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("No partialUpdate available for period")
            {
                return res;
            }
        }
        unreachable!()
    }

    pub fn get_light_client_updates_simple<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    >(
        &self,
        start_period: SyncCommitteePeriod,
        count: u64,
    ) -> Result<
        LightClientUpdatesResponse<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    > {
        let count = if count < 1 { 1 } else { count };
        self.request_get(format!(
            "/eth/v1/beacon/light_client/updates?start_period={}&count={}",
            start_period, count
        ), None)
        
    }

    // Helper functions
    fn request_get<T: DeserializeOwned>(&self, path: impl Into<String>, display: Option<bool>) -> Result<T> {
        let url_str = format!("{}{}", self.endpoint, path.into());
        debug!("request_get: url={}", url_str);        
        let url = match Url::parse(url_str.as_str()) {
            Ok(url) => url,
            Err(e) => {
                return Err(Error::Other {
                    description: format!("{}, {}", e.to_string(), url_str),
                });
            }
        };
        let res = match self.http_client.get(url).send(display.unwrap_or(false)) {
            Ok(res) => res,
            Err(e) => {
                return Err(Error::Other {
                    description: format!("{}, {}", e.to_string(), url_str),
                });
            }
        };
        match res.status() {
            http::StatusCode::OK => {
                let bytes = res.body().as_bytes();
                debug!("request_get: response={}", String::from_utf8_lossy(&bytes));
                Ok(serde_json::from_slice(&bytes).map_err(Error::JSONDecodeError)?)
            }
            http::StatusCode::INTERNAL_SERVER_ERROR => Err(Error::RPCInternalServerError(
                serde_json::from_str::<InternalServerError>(res.body()).unwrap().message,
            )),
            _ => Err(Error::Other {
                description: res.body().to_string(),
            }),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct InternalServerError {
    #[serde(alias = "statusCode", alias = "code")]
    status_code: u64,
    error: Option<String>,
    message: String,
}
