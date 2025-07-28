use std::fmt::{self, Display, Formatter};
use ::http::Request;
use serde::{Deserialize, Serialize};
use super::http;

pub(crate) const NETWORK_MANAGER_TABLE: &str = "networkManagerTable";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Network {
    pub name: String,
    pub chain_id: Option<u64>,
    pub rpc_url: String,
    pub gas_price: Option<u64>,
    pub credentials: Option<Credentials>,
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match serde_json::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                format!("ERROR: failed to serialize Network: {}", e)
            }
        })
    }
}

impl Network {
    pub fn new(name: &str, chain_id: Option<u64>, rpc_url: &str, gas_price: Option<u64>, credentials_input: Option<&str>) -> Network {
        Network {
            name: name.to_string(),
            chain_id: chain_id,
            rpc_url: rpc_url.to_string(),
            gas_price: gas_price,
            credentials: {
                match credentials_input {
                    None => None,
                    Some(credentials_input) => {
                        match serde_json::from_str::<Credentials>(credentials_input) {
                            Ok(v) => Some(v),
                            Err(_e) => {
                                None
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn load(name: &str) -> Result<Network, Box<dyn std::error::Error>> {
        match klave::ledger::get_table(NETWORK_MANAGER_TABLE).get(name) {
            Ok(v) => {

                let network: Network = serde_json::from_slice(&v)?;
                Ok(network)
            },
            Err(e) => Err(e.into())
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_network = serde_json::to_string(&self)?;
        klave::ledger::get_table(NETWORK_MANAGER_TABLE).set(self.name.as_str(), &serialized_network.as_bytes())?; 
        Ok(())
    }

    pub fn get_rpc_url(&self) -> &str {
        &self.rpc_url
    }

    pub fn get_gas_price(&self) -> Option<u64> {
        self.gas_price
    }

    pub fn get_chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_credentials(&self) -> Option<&Credentials> {
        self.credentials.as_ref()
    }

    pub fn set_gas_price(&mut self, gas_price: Option<u64>) {
        self.gas_price = gas_price;
    }

    pub fn set_chain_id(&mut self, chain_id: Option<u64>) {
        self.chain_id = chain_id;
    }

    pub fn set_rpc_url(&mut self, rpc_url: &str) {
        self.rpc_url = rpc_url.to_string();
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn set_credentials(&mut self, credentials: &Credentials) {
        self.credentials = Some(credentials.clone());
    }

    pub fn generate_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let body = serde_json::to_string::<Credentials>(&self.credentials.clone().expect("credentials not found"))?;
        let http_request = http::request_format(&format!("{}/login", self.get_rpc_url()), &body)?;
        let result = klave::https::request(&http_request)?;
        let token_response = http::parse_token_response::<String>(&result.body())?;
        Ok(token_response)

    }

    pub fn request<T>(&self, body: &str) -> Result<T, Box<dyn std::error::Error>> 
        where
        T: for<'de> Deserialize<'de>,
    {    
        let http_request: Request<String>;
        if self.credentials.is_some() {
            let token = self.generate_token()?;
            http_request = http::request_format_with_auth(self.get_rpc_url(), &body, &token)?;
        }
        else {
            http_request = http::request_format(self.get_rpc_url(), &body)?;
        }

        let result = match klave::https::request(&http_request) {
            Ok(r) => r,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: request - failed to send request: {}, {:?}, {}, {}", &http_request.uri(), &http_request.headers(), &http_request.body(), e));
                return Err(e.into());
            }
        };
        let tx_response = match http::parse_json_rpc_response::<T>(&result.body()) {
            Ok(r) => r,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: request - failed to parse response: {}, {}, {}, {:?}, {}", result.body(), e, &http_request.uri(), &http_request.headers(), &http_request.body()));
                return Err(e.into());
            }
        };
        Ok(tx_response)
    }
}
