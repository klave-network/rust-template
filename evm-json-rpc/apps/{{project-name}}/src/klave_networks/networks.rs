use std::fmt::{self, Display, Formatter};
use super::network::NETWORK_MANAGER_TABLE;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use super::network::Network;
use super::network::Credentials;
use klave;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Networks {
    networks: Vec<String>,
}

impl Display for Networks {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match serde_json::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                format!("ERROR: failed to serialize Networks: {}", e)
            }
        })
    }
}

impl Networks {
    fn new() -> Networks {
        Networks {
            networks: Vec::new(),
        }
    }

    pub fn load() -> Result<Networks, Box<dyn std::error::Error>> {
        match klave::ledger::get_table(NETWORK_MANAGER_TABLE).get("ALL") {
            Ok(v) => {
                let wallet: Networks = serde_json::from_slice(&v)?;
                Ok(wallet)
            },
            Err(e) => {
                Err(e.into())
            }
        }
    }

    pub fn get() -> Networks {
        match Networks::load() {
            Ok(nm) => nm,
            Err(_) => {
                Networks::new()
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_nm = to_string(&self)?;
        klave::ledger::get_table(NETWORK_MANAGER_TABLE).set("ALL", &serialized_nm.as_bytes())?;
        Ok(())
    }

    pub fn add_network(&mut self, network: &Network) -> Result<(), Box<dyn std::error::Error>> {
        //Check if network exists
        let mut found = false;
        for n in &self.networks {
            if n == &network.name {
                found = true;
                break;
            }
        }
        if found {
            return Err(format!("network {} already exists", network.name).into());
        }        

        network.save()?;

        self.networks.push(network.name.clone());        
        self.save()?;
        Ok(())
    }

    pub fn update_gas_price(&self, network_name: &str, gas_price: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut network = self.get_network(&network_name)?;
        network.set_gas_price(Some(gas_price));
        network.save()?;
        Ok(())
    }

    pub fn update_chain_id(&self, network_name: &str, chain_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut network = self.get_network(&network_name)?;
        network.set_chain_id(Some(chain_id));
        network.save()?;
        Ok(())
    }

    pub fn get_network(&self, name: &str) -> Result<Network, Box<dyn std::error::Error>> {
        for network in &self.networks {
            if network == name {
                let network = Network::load(network)?;
                return Ok(network)
            }
        }
        Err("network not found".into())        
    }

    pub fn get_networks(&self) -> &Vec<String> {
        &self.networks
    }

    #[allow(dead_code)]
    pub fn update_network(&self, network: &Network) -> Result<(), Box<dyn std::error::Error>> {
        let network_name = network.get_name();
        for n in &self.networks {
            if n == network_name {
                let mut local_network = self.get_network(&network_name)?;
                local_network.set_name(network.get_name());
                local_network.set_chain_id(network.get_chain_id());
                local_network.set_rpc_url(network.get_rpc_url());
                local_network.set_gas_price(network.get_gas_price());
                match network.get_credentials() {
                    Some(c) => {
                        local_network.set_credentials(c);
                    }
                    None => {}
                }
                local_network.save()?;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_credentials(&self, network_name: &str, credentials: &Credentials) -> Result<(), Box<dyn std::error::Error>> {
        let mut network = self.get_network(network_name)?;
        network.set_credentials(credentials);
        network.save()?;
        Ok(())
    }

    pub fn send<T>(&self, network_name: &str, method: &str, params: &[&str]) -> Result<T, Box<dyn std::error::Error>> 
    where
        T: for<'de> Deserialize<'de>,
    {
        let network = self.get_network(&network_name)?;       
        let chain_id = match network.get_chain_id() {
            Some(c) => c,
            None => 1            
        };

        let body = r#"{"jsonrpc":"2.0","method":""#.to_string() + method + r#"","params":["# + &params.join(",") + r#"],"id":"# + &chain_id.to_string() + r#"}"#;
        let result = network.request::<T>(&body)?;
        Ok(result)
    }
}