#[allow(warnings)]
mod bindings;
mod wallet;
mod klave_networks;
mod solidity;

use std::str::FromStr;

use alloy_consensus::TxEip1559;
use alloy_primitives::{hex, Address, Bytes, TxKind, U256};
use alloy_rpc_types_eth::AccessList;
use bindings::Guest;
use klave;
use serde_json::Value;
use wallet::Wallet;
use klave_networks::{networks::Networks, network::Network};
use solidity::{burnCall, mintCall};
use alloy_sol_types::SolCall;


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
        klave::router::add_user_transaction("network_add");
        klave::router::add_user_transaction("network_set_chain_id");
        klave::router::add_user_transaction("network_set_gas_price");
        klave::router::add_user_query("networks_all");

        klave::router::add_user_transaction("wallet_add");
        klave::router::add_user_transaction("wallet_add_network");
        klave::router::add_user_query("wallet_address");
        klave::router::add_user_query("wallet_secret_key");
        klave::router::add_user_query("wallet_public_key");
        klave::router::add_user_query("wallet_balance");
        klave::router::add_user_query("wallet_networks");
        klave::router::add_user_query("wallet_transfer");    
        klave::router::add_user_query("wallet_deploy_contract");
        klave::router::add_user_query("wallet_call_contract");
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
        match nm.update_chain_id(network_name, chain_id) {
            Ok(_) => {
                klave::notifier::send_string(&format!("chain_id '{}' set as current", chain_id));
            },
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to set chain_id '{}': {}", chain_id, e));
            }
        }
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
        match nm.update_gas_price(network_name, gas_price) {
            Ok(_) => {
                klave::notifier::send_string(&format!("gas_price '{}' set as current", gas_price));
            },
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to set gas_price '{}': {}", gas_price, e));
            }
        }
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

    fn wallet_add(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };
        
        let (secret_key, public_key) = match wallet::generate_keypair(v["secret_key"].as_str()) {
            Ok((s, p)) => (s, p),
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to generate keypair: {}", e));
                return;
            }
        };
        let mut wallet = Wallet::new(&secret_key, &public_key);        

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        for network_name in nm.get_networks() {
            match wallet.add_network(network_name) {
                Ok(_) => (),
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to add network {}: {}", network_name, e));
                    return;
                }
            }
        }
        match wallet.save() {
            Ok(_) => (),
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to save wallet: {}", e));
                return;
            }
        };
    }

    fn wallet_add_network(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network_name not found"));
                return;
            }
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let mut wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        match wallet.add_network(network_name) {
            Ok(_) => klave::notifier::send_string(&format!("new network {} added to wallet", network_name)),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to add network {}: {}", network_name, e))
        };        
    }

    fn wallet_address(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        klave::notifier::send_string(&wallet.get_eth_address());
    }

    fn wallet_public_key(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        klave::notifier::send_string(&wallet.get_public_key());
    }

    fn wallet_networks(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let local_networks = wallet.get_networks();
        let local_networks_str: Vec<String> = local_networks.iter().map(|network| format!("{}", serde_json::to_string(&network).unwrap())).collect();
        klave::notifier::send_string(&format!("{}", serde_json::to_string(&local_networks_str).unwrap()));
    }

    fn wallet_transfer(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };    
    
        let chain_id = match v["chainId"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: chainId not found"));
                return;
            }
        };
        let nonce = match v["nonce"].as_u64() {
            Some(n) => n,
            None => {
                klave::notifier::send_string(&format!("ERROR: nonce not found"));
                return;
            }
        };
        let gas_limit = match v["gasLimit"].as_u64(){
            Some(g) => g,
            None => {
                klave::notifier::send_string(&format!("ERROR: gasLimit not found"));
                return;
            }
        };
        let to_str = match v["to"].as_str(){
            Some(t) => t,
            None => {
                klave::notifier::send_string(&format!("ERROR: to not found"));
                return;
            }
        };
        let to = match Address::from_str(&to_str) {
            Ok(a) => a,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to parse address: {}", e));
                return;
            }
        };
        let value = match v["value"].as_str() {
            Some(c) => {
                match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                    Ok(v) => v,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to parse value: {}", e));
                        return;
                    }
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: value not found"));
                return;
            }
        };
        let max_fee_per_gas = match v["maxFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxFeePerGas not found"));
                return;
            }
        };
        let max_priority_fee_per_gas = match v["maxPriorityFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxPriorityFeePerGas not found"));
                return;
            }
        };
    
        let tx = TxEip1559 {
            chain_id: chain_id,
            nonce: nonce,
            gas_limit: gas_limit,
            to: to.into(),
            value: value,
            input: Bytes::new(),
            max_fee_per_gas: max_fee_per_gas as u128,
            max_priority_fee_per_gas: max_priority_fee_per_gas as u128,
            access_list: AccessList::default(),
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network not found"));
                return;
            }
        };
        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let mut wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        let trace = match v["trace"].as_bool() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: trace not found"));
                return;
            }
        };

        if !tx.value.is_zero() && !wallet.can_spend(&nm, network_name, tx.value) {
            return klave::notifier::send_string("ERROR: Insufficient balance");
        }

        match wallet.sign_and_send(&nm, network_name, tx, trace) {
            Ok(result) => klave::notifier::send_string(&result),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to send transaction: {}", e))
        }
    }

    fn wallet_deploy_contract(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };    
    
        let chain_id = match v["chainId"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: chainId not found"));
                return;
            }
        };
        let nonce = match v["nonce"].as_u64() {
            Some(n) => n,
            None => {
                klave::notifier::send_string(&format!("ERROR: nonce not found"));
                return;
            }
        };
        let gas_limit = match v["gasLimit"].as_u64(){
            Some(g) => g,
            None => {
                klave::notifier::send_string(&format!("ERROR: gasLimit not found"));
                return;
            }
        };
        let data = match v["data"].as_str() {
            Some(v) => v,            
            None => {
                klave::notifier::send_string(&format!("ERROR: data not found"));
                return;
            }
        };
        let max_fee_per_gas = match v["maxFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxFeePerGas not found"));
                return;
            }
        };
        let max_priority_fee_per_gas = match v["maxPriorityFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxPriorityFeePerGas not found"));
                return;
            }
        };
    
        let tx = TxEip1559 {
            chain_id: chain_id,
            nonce: nonce,
            gas_limit: gas_limit,
            to: TxKind::Create,
            value: U256::default(),
            input: hex::decode(data.trim_start_matches("0x")).unwrap().into(),
            max_fee_per_gas: max_fee_per_gas as u128,
            max_priority_fee_per_gas: max_priority_fee_per_gas as u128,
            access_list: AccessList::default(),
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network not found"));
                return;
            }
        };
        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let mut wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        let trace = match v["trace"].as_bool() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: trace not found"));
                return;
            }
        };

        match wallet.sign_and_send(&nm, network_name, tx, trace) {
            Ok(result) => klave::notifier::send_string(&result),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to send transaction: {}", e))
        }
    }

    fn wallet_balance(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network not found"));
                return;
            }
        };
        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: eth_address not found"));
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };
        
        match wallet.get_balance(&nm, network_name) {            
            Ok(result) => klave::notifier::send_string(&result),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to send balance: {}", e))
        }
    }   

    fn wallet_call_contract(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
            return;
        };

        let contract_owner_address = match v["contract_owner_address"].as_str() {
            Some(c) => match c.parse::<Address>() {
                Ok(a) => a,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to parse contract owner address: {}", e));
                    return;
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: contract owner address not found"));
                return;
            }
        };

        let contract_address = match v["contract_address"].as_str() {
            Some(c) => match c.parse::<Address>() {
                Ok(a) => a,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to parse contract address: {}", e));
                    return;
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: contract address not found"));
                return;
            }
        };

        let recipient_address = match v["recipient_address"].as_str() {
            Some(c) => match c.parse::<Address>() {
                Ok(a) => a,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to parse recipient address: {}", e));
                    return;
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: recipient address not found"));
                return;
            }
        };
        let value = match v["value"].as_str() {
            Some(c) => {
                match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                    Ok(v) => v,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to parse value: {}", e));
                        return;
                    }
                }
            },
            None => {
                klave::notifier::send_string(&format!("ERROR: value not found"));
                return;
            }
        };
        let mut hex_encoded_call = String::new();
        match v["input"].as_str() {
            Some(d) => {
                match d {
                    "mint" => {
                            hex_encoded_call = hex::encode(mintCall::new((recipient_address, value)).abi_encode());                            
                        },
                    "burn" => {
                            hex_encoded_call = hex::encode(burnCall::new((recipient_address, value)).abi_encode());                            
                        },
                    _ => {
                        klave::notifier::send_string(&format!("ERROR: unsupported function call"));
                        return;
                    }
                }
            },
            None => {}
        };    
        
        let chain_id = match v["chainId"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: chainId not found"));
                return;
            }
        };
        let nonce = match v["nonce"].as_u64() {
            Some(n) => n,
            None => {
                klave::notifier::send_string(&format!("ERROR: nonce not found"));
                return;
            }
        };
        let gas_limit = match v["gasLimit"].as_u64(){
            Some(g) => g,
            None => {
                klave::notifier::send_string(&format!("ERROR: gasLimit not found"));
                return;
            }
        };
        let max_fee_per_gas = match v["maxFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxFeePerGas not found"));
                return;
            }
        };
        let max_priority_fee_per_gas = match v["maxPriorityFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string(&format!("ERROR: maxPriorityFeePerGas not found"));
                return;
            }
        };
    
        let tx = TxEip1559 {
            chain_id: chain_id,
            nonce: nonce,
            gas_limit: gas_limit,
            to: TxKind::Call(contract_address),
            value: U256::default(),
            input: hex::decode(hex_encoded_call).unwrap().into(),
            max_fee_per_gas: max_fee_per_gas as u128,
            max_priority_fee_per_gas: max_priority_fee_per_gas as u128,
            ..Default::default()
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string(&format!("ERROR: network not found"));
                return;
            }
        };
        let mut wallet = match Wallet::load(&contract_owner_address.to_string()) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {}", e));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
                return
            }
        };

        let trace = match v["trace"].as_bool() {
            Some(c) => c,
            None => false
        };

        match wallet.sign_and_send(&nm, network_name, tx.clone(), trace) {
            Ok(result) => {
                klave::notifier::send_string(&format!("{}", result))
            },
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to send transaction: {}", e))
        }        
    }
}

bindings::export!(Component with_types_in bindings);
