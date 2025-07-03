#[allow(warnings)]
mod bindings;
mod klave_networks;
mod solidity;
mod wallet;

use std::str::FromStr;

use alloy_consensus::TxEip1559;
use alloy_primitives::{hex, Address, Bytes, TxKind, U256};
use alloy_rpc_types_eth::AccessList;
use alloy_sol_types::SolCall;
use bindings::Guest;
use klave_networks::{network::Network, networks::Networks};
use serde_json::Value;
use solidity::{burnCall, mintCall};
use wallet::Wallet;

/// Custom function to use the import for random byte generation.
///
/// We do this is because "js" feature is incompatible with the component model
/// if you ever got the __wbindgen_placeholder__ error when trying to use the `js` feature
/// of getrandom,
fn imported_random(dest: &mut [u8]) -> Result<(), getrandom::Error> {
    // iterate over the length of the destination buffer and fill it with random bytes
    let random_bytes =
        klave::crypto::random::get_random_bytes(dest.len().try_into().unwrap()).unwrap();
    dest.copy_from_slice(&random_bytes);

    Ok(())
}

getrandom::register_custom_getrandom!(imported_random);

struct Component;
impl Guest for Component {
    fn register_routes() {
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

    fn network_add(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: network not found");
                return;
            }
        };
        let chain_id = v["chain_id"].as_u64();
        let rpc_url = match v["rpc_url"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: rpc_url not found");
                return;
            }
        };
        let gas_price = v["gas_price"].as_u64();
        let credentials = v["credentials"].as_str();
        let network = Network::new(network_name, chain_id, rpc_url, gas_price, credentials);

        let mut nm = Networks::get();
        match nm.add_network(&network) {
            Ok(_) => {
                klave::notifier::send_string(&format!("network '{network_name}' added"));
            }
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "ERROR: failed to add network '{network_name}': {e}"
                ));
            }
        }
    }

    fn network_set_chain_id(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "ERROR: failed to load network manager: {e}. Create one first."
                ));
                return;
            }
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: network_name not found");
                return;
            }
        };
        let chain_id = match v["chain_id"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: chain_id not found");
                return;
            }
        };
        match nm.update_chain_id(network_name, chain_id) {
            Ok(_) => {
                klave::notifier::send_string(&format!("chain_id '{chain_id}' set as current"));
            }
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "ERROR: failed to set chain_id '{chain_id}': {e}"
                ));
            }
        }
    }

    fn network_set_gas_price(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "ERROR: failed to load network manager: {e}. Create one first."
                ));
                return;
            }
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: network_name not found");
                return;
            }
        };
        let gas_price = match v["gas_price"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: gas_price not found");
                return;
            }
        };
        match nm.update_gas_price(network_name, gas_price) {
            Ok(_) => {
                klave::notifier::send_string(&format!("gas_price '{gas_price}' set as current"));
            }
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "ERROR: failed to set gas_price '{gas_price}': {e}"
                ));
            }
        }
    }

    fn networks_all(_cmd: String) {
        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "ERROR: failed to load network manager: {e}. Create one first."
                ));
                return;
            }
        };

        let mut networks: Vec<String> = Vec::<String>::new();
        for network_name in nm.get_networks() {
            let network = match Network::load(network_name) {
                Ok(n) => n.to_string(),
                Err(e) => {
                    klave::notifier::send_string(&format!(
                        "ERROR: failed to load network '{network_name}': {e}"
                    ));
                    return;
                }
            };
            networks.push(network);
        }

        klave::notifier::send_string(&serde_json::to_string(&networks).unwrap().to_string());
    }

    fn wallet_add(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let (secret_key, public_key) = match wallet::generate_keypair(v["secret_key"].as_str()) {
            Ok((s, p)) => (s, p),
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to generate keypair: {e}"));
                return;
            }
        };
        let mut wallet = Wallet::new(&secret_key, &public_key);

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "ERROR: failed to load network manager: {e}. Create one first."
                ));
                return;
            }
        };

        for network_name in nm.get_networks() {
            match wallet.add_network(network_name) {
                Ok(_) => (),
                Err(e) => {
                    klave::notifier::send_string(&format!(
                        "ERROR: failed to add network {network_name}: {e}"
                    ));
                    return;
                }
            }
        }
        match wallet.save() {
            Ok(_) => (),
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to save wallet: {e}"));
            }
        }
    }

    fn wallet_add_network(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: network_name not found");
                return;
            }
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: eth_address not found");
                return;
            }
        };

        let mut wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {e}"));
                return;
            }
        };

        match wallet.add_network(network_name) {
            Ok(_) => {
                klave::notifier::send_string(&format!("new network {network_name} added to wallet"))
            }
            Err(e) => klave::notifier::send_string(&format!(
                "ERROR: failed to add network {network_name}: {e}"
            )),
        };
    }

    fn wallet_address(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: eth_address not found");
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {e}"));
                return;
            }
        };

        klave::notifier::send_string(wallet.get_eth_address());
    }

    fn wallet_public_key(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: eth_address not found");
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {e}"));
                return;
            }
        };

        klave::notifier::send_string(wallet.get_public_key());
    }

    fn wallet_networks(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: eth_address not found");
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {e}"));
                return;
            }
        };

        let local_networks = wallet.get_networks();
        let local_networks_str: Vec<String> = local_networks
            .iter()
            .map(|network| serde_json::to_string(&network).unwrap().to_string())
            .collect();
        klave::notifier::send_string(
            &serde_json::to_string(&local_networks_str)
                .unwrap()
                .to_string(),
        );
    }

    fn wallet_transfer(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let chain_id = match v["chainId"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: chainId not found");
                return;
            }
        };
        let nonce = match v["nonce"].as_u64() {
            Some(n) => n,
            None => {
                klave::notifier::send_string("ERROR: nonce not found");
                return;
            }
        };
        let gas_limit = match v["gasLimit"].as_u64() {
            Some(g) => g,
            None => {
                klave::notifier::send_string("ERROR: gasLimit not found");
                return;
            }
        };
        let to_str = match v["to"].as_str() {
            Some(t) => t,
            None => {
                klave::notifier::send_string("ERROR: to not found");
                return;
            }
        };
        let to = match Address::from_str(to_str) {
            Ok(a) => a,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to parse address: {e}"));
                return;
            }
        };
        let value = match v["value"].as_str() {
            Some(c) => match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                Ok(v) => v,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to parse value: {e}"));
                    return;
                }
            },
            None => {
                klave::notifier::send_string("ERROR: value not found");
                return;
            }
        };
        let max_fee_per_gas = match v["maxFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string("ERROR: maxFeePerGas not found");
                return;
            }
        };
        let max_priority_fee_per_gas = match v["maxPriorityFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string("ERROR: maxPriorityFeePerGas not found");
                return;
            }
        };

        let tx = TxEip1559 {
            chain_id,
            nonce,
            gas_limit,
            to: to.into(),
            value,
            input: Bytes::new(),
            max_fee_per_gas: max_fee_per_gas as u128,
            max_priority_fee_per_gas: max_priority_fee_per_gas as u128,
            access_list: AccessList::default(),
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: network not found");
                return;
            }
        };
        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: eth_address not found");
                return;
            }
        };

        let mut wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {e}"));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "ERROR: failed to load network manager: {e}. Create one first."
                ));
                return;
            }
        };

        let trace = match v["trace"].as_bool() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: trace not found");
                return;
            }
        };

        if !tx.value.is_zero() && !wallet.can_spend(&nm, network_name, tx.value) {
            return klave::notifier::send_string("ERROR: Insufficient balance");
        }

        match wallet.sign_and_send(&nm, network_name, tx, trace) {
            Ok(result) => klave::notifier::send_string(&result),
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to send transaction: {e}"))
            }
        }
    }

    fn wallet_deploy_contract(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let chain_id = match v["chainId"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: chainId not found");
                return;
            }
        };
        let nonce = match v["nonce"].as_u64() {
            Some(n) => n,
            None => {
                klave::notifier::send_string("ERROR: nonce not found");
                return;
            }
        };
        let gas_limit = match v["gasLimit"].as_u64() {
            Some(g) => g,
            None => {
                klave::notifier::send_string("ERROR: gasLimit not found");
                return;
            }
        };
        let data = match v["data"].as_str() {
            Some(v) => v,
            None => {
                klave::notifier::send_string("ERROR: data not found");
                return;
            }
        };
        let max_fee_per_gas = match v["maxFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string("ERROR: maxFeePerGas not found");
                return;
            }
        };
        let max_priority_fee_per_gas = match v["maxPriorityFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string("ERROR: maxPriorityFeePerGas not found");
                return;
            }
        };

        let tx = TxEip1559 {
            chain_id,
            nonce,
            gas_limit,
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
                klave::notifier::send_string("ERROR: network not found");
                return;
            }
        };
        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: eth_address not found");
                return;
            }
        };

        let mut wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {e}"));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "ERROR: failed to load network manager: {e}. Create one first."
                ));
                return;
            }
        };

        let trace = match v["trace"].as_bool() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: trace not found");
                return;
            }
        };

        match wallet.sign_and_send(&nm, network_name, tx, trace) {
            Ok(result) => klave::notifier::send_string(&result),
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to send transaction: {e}"))
            }
        }
    }

    fn wallet_balance(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let network_name = match v["network_name"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: network not found");
                return;
            }
        };
        let eth_address = match v["eth_address"].as_str() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: eth_address not found");
                return;
            }
        };

        let wallet = match Wallet::load(eth_address) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {e}"));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "ERROR: failed to load network manager: {e}. Create one first."
                ));
                return;
            }
        };

        match wallet.get_balance(&nm, network_name) {
            Ok(result) => klave::notifier::send_string(&result),
            Err(e) => klave::notifier::send_string(&format!("ERROR: failed to send balance: {e}")),
        }
    }

    fn wallet_call_contract(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("ERROR: failed to parse '{cmd}' as json"));
            return;
        };

        let contract_owner_address = match v["contract_owner_address"].as_str() {
            Some(c) => match c.parse::<Address>() {
                Ok(a) => a,
                Err(e) => {
                    klave::notifier::send_string(&format!(
                        "ERROR: failed to parse contract owner address: {e}"
                    ));
                    return;
                }
            },
            None => {
                klave::notifier::send_string("ERROR: contract owner address not found");
                return;
            }
        };

        let contract_address = match v["contract_address"].as_str() {
            Some(c) => match c.parse::<Address>() {
                Ok(a) => a,
                Err(e) => {
                    klave::notifier::send_string(&format!(
                        "ERROR: failed to parse contract address: {e}"
                    ));
                    return;
                }
            },
            None => {
                klave::notifier::send_string("ERROR: contract address not found");
                return;
            }
        };

        let recipient_address = match v["recipient_address"].as_str() {
            Some(c) => match c.parse::<Address>() {
                Ok(a) => a,
                Err(e) => {
                    klave::notifier::send_string(&format!(
                        "ERROR: failed to parse recipient address: {e}"
                    ));
                    return;
                }
            },
            None => {
                klave::notifier::send_string("ERROR: recipient address not found");
                return;
            }
        };
        let value = match v["value"].as_str() {
            Some(c) => match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                Ok(v) => v,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to parse value: {e}"));
                    return;
                }
            },
            None => {
                klave::notifier::send_string("ERROR: value not found");
                return;
            }
        };
        let mut hex_encoded_call = String::new();
        if let Some(d) = v["input"].as_str() {
            match d {
                "mint" => {
                    hex_encoded_call =
                        hex::encode(mintCall::new((recipient_address, value)).abi_encode());
                }
                "burn" => {
                    hex_encoded_call =
                        hex::encode(burnCall::new((recipient_address, value)).abi_encode());
                }
                _ => {
                    klave::notifier::send_string("ERROR: unsupported function call");
                    return;
                }
            }
        };

        let chain_id = match v["chainId"].as_u64() {
            Some(c) => c,
            None => {
                klave::notifier::send_string("ERROR: chainId not found");
                return;
            }
        };
        let nonce = match v["nonce"].as_u64() {
            Some(n) => n,
            None => {
                klave::notifier::send_string("ERROR: nonce not found");
                return;
            }
        };
        let gas_limit = match v["gasLimit"].as_u64() {
            Some(g) => g,
            None => {
                klave::notifier::send_string("ERROR: gasLimit not found");
                return;
            }
        };
        let max_fee_per_gas = match v["maxFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string("ERROR: maxFeePerGas not found");
                return;
            }
        };
        let max_priority_fee_per_gas = match v["maxPriorityFeePerGas"].as_u64() {
            Some(m) => m,
            None => {
                klave::notifier::send_string("ERROR: maxPriorityFeePerGas not found");
                return;
            }
        };

        let tx = TxEip1559 {
            chain_id,
            nonce,
            gas_limit,
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
                klave::notifier::send_string("ERROR: network not found");
                return;
            }
        };
        let mut wallet = match Wallet::load(&contract_owner_address.to_string()) {
            Ok(w) => w,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to load wallet: {e}"));
                return;
            }
        };

        let nm = match Networks::load() {
            Ok(nm) => nm,
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "ERROR: failed to load network manager: {e}. Create one first."
                ));
                return;
            }
        };

        let trace = v["trace"].as_bool().unwrap_or_default();

        match wallet.sign_and_send(&nm, network_name, tx.clone(), trace) {
            Ok(result) => klave::notifier::send_string(&result.to_string()),
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to send transaction: {e}"))
            }
        }
    }
}

bindings::export!(Component with_types_in bindings);
