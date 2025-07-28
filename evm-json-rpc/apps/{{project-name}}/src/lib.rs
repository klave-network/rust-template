    #[allow(warnings)]
mod bindings;

use bindings::Guest;
use klave;
use serde_json::Value;
use crate::klave_networks::{networks::Networks, network::Network};

pub mod eth;
pub mod web3;
pub mod solidity;
pub mod klave_networks;

/// Custom function to use the import for random byte generation.
///
/// We do this is because "js" feature is incompatible with the component model
/// if you ever got the __wbindgen_placeholder__ error when trying to use the `js` feature
/// of getrandom,
fn imported_random(dest: &mut [u8]) -> Result<(), getrandom::Error> {
    // iterate over the length of the destination buffer and fill it with random bytes
    let random_bytes = klave::crypto::random::get_random_bytes(dest.len().try_into().unwrap()).unwrap();
    dest.copy_from_slice(&random_bytes);

    Ok(())
}

getrandom::register_custom_getrandom!(imported_random);

struct Component;
impl Guest for Component {

    fn register_routes(){
        klave::router::add_user_transaction(&String::from("network_add"));
        klave::router::add_user_transaction(&String::from("network_set_chain_id"));
        klave::router::add_user_transaction(&String::from("network_set_gas_price"));
        klave::router::add_user_query(&String::from("networks_all"));

        klave::router::add_user_query(&String::from("eth_block_number"));
        klave::router::add_user_query(&String::from("eth_get_block_by_number"));
        klave::router::add_user_query(&String::from("eth_gas_price"));
        klave::router::add_user_query(&String::from("eth_estimate_gas"));
        klave::router::add_user_query(&String::from("eth_call_contract"));
        klave::router::add_user_query(&String::from("eth_protocol_version"));
        klave::router::add_user_query(&String::from("eth_chain_id"));
        klave::router::add_user_query(&String::from("eth_get_transaction_by_hash"));
        klave::router::add_user_query(&String::from("eth_get_transaction_receipt")); 
        klave::router::add_user_query(&String::from("eth_get_transaction_count"));   

        klave::router::add_user_query(&String::from("web3_client_version"));
        klave::router::add_user_query(&String::from("web3_sha3"));
        klave::router::add_user_query(&String::from("net_version"));

        klave::router::add_user_query(&String::from("get_sender"));
        klave::router::add_user_query(&String::from("get_trusted_time"));
    }

    fn network_add(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network not found"));
                return;
            }
        };
        let chain_id = v["chain_id"].as_u64();
        let rpc_url = match v["rpc_url"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: rpc_url not found"));
                return;
            }
        };
        let gas_price = v["gas_price"].as_u64();
        let credentials = v["credentials"].as_str();
        let network = Network::new(network_name, chain_id, rpc_url, gas_price, credentials);        

        let mut nm = Networks::get();
        match nm.add_network(&network) {
            Ok(_) => {
                klave::notifier::send_string(&format!("network '{}' added", network_name));
            },
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to add network '{}': {}", network_name, e));
            }
        }
    }

    fn network_set_chain_id(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return
        };
    
        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network_name not found"));
                return;
            }
        };
        let chain_id = match v["chain_id"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: chain_id not found"));
                return;
            }
        };
        let _ = nm.update_chain_id(network_name, chain_id);
    }

    fn network_set_gas_price(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return
        };
    
        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network_name not found"));
                return;
            }
        };
        let gas_price = match v["gas_price"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: gas_price not found"));
                return;
            }
        };
        let _ = nm.update_gas_price(network_name, gas_price);
    }

    fn networks_all(_cmd: String){
        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };
                
        let mut networks: Vec<String> = Vec::<String>::new();
        for network_name in nm.get_networks() {
            let network = match Network::load(network_name) {
                Ok(n) => n.to_string(),
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to load network '{}': {}", network_name, e));
                    return;
                }
            };
            networks.push(network);
        }

        klave::notifier::send_string(&format!("{}", &serde_json::to_string(&networks).unwrap()));
    }

    fn eth_block_number(cmd: String){
        eth::eth_block_number(cmd);
    }

    fn eth_get_block_by_number(cmd: String){
        eth::eth_get_block_by_number(cmd);
    }

    fn eth_gas_price(cmd: String){
        eth::eth_gas_price(cmd);
    }

    fn eth_estimate_gas(cmd: String){
        eth::eth_estimate_gas(cmd);
    }

    fn eth_call_contract(cmd: String){
        eth::eth_call_contract(cmd);
    }

    fn eth_protocol_version(cmd: String){
        eth::eth_protocol_version(cmd);
    }

    fn eth_chain_id(cmd: String){
        eth::eth_chain_id(cmd);
    }

    fn eth_get_transaction_by_hash(cmd: String){
        eth::eth_get_transaction_by_hash(cmd);
    }

    fn eth_get_transaction_receipt(cmd: String){
        eth::eth_get_transaction_receipt(cmd);
    }

    fn eth_get_transaction_count(cmd: String){
        eth::eth_get_transaction_count(cmd);
    }

    fn web_client_version(cmd: String){
        web3::web3_client_version(cmd);
    }

    fn web_sha3(cmd: String){
        web3::web3_sha3(cmd);
    }

    fn net_version(cmd: String){
        web3::net_version(cmd);
    }

    fn get_sender(_cmd: String){
        klave::notifier::send_string(&match klave::context::get("sender") {
            Ok(s) => s,
            Err(e) => e.to_string()
        });
    }

    fn get_trusted_time(_cmd: String){
        klave::notifier::send_string(&match klave::context::get("trusted_time") {
            Ok(s) => s,
            Err(e) => e.to_string()
        });
    }
}

bindings::export!(Component with_types_in bindings);
