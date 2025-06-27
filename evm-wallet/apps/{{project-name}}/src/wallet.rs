use std::{fmt::{self, Display, Formatter}, str::FromStr};

use alloy_consensus::TxEip1559;
use alloy_network::TxSignerSync;
use alloy_signer::k256::{elliptic_curve::sec1::ToEncodedPoint, PublicKey, SecretKey};
use alloy_signer_local::PrivateKeySigner;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use alloy_consensus::transaction::RlpEcdsaTx;
use alloy_primitives::{hex, keccak256, Address, U256};
use klave::{self, crypto::subtle::{self, CryptoKey}};
use super::klave_networks::networks::Networks;

pub(crate) const WALLET_TABLE: &str = "walletTable";

pub fn generate_keypair(secret_key_str: Option<&str>) -> Result<(SecretKey, PublicKey), Box<dyn std::error::Error>> {
    match secret_key_str {
        Some(secret_key_str) => {
            let secret_key_hex = match hex::decode(secret_key_str) {
                Ok(v) => v,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to decode secret key: {}", e));
                    return Err(e.into());
                }
            };

            let secret_key = match SecretKey::from_slice(&secret_key_hex) {
                Ok(v) => v,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to create secret key: {}", e));
                    return Err(e.into());
                }
            };
            let public_key = secret_key.public_key();
            return Ok((secret_key, public_key));
        }
        None => {
            Ok(match klave::crypto::random::get_random_bytes(size_of::<SecretKey>() as i32)
            .map(|bytes| {
                let secret_key = SecretKey::from_slice(&bytes).unwrap();
                let public_key = secret_key.public_key();
                (secret_key, public_key)
            }) {
                Ok(v) => v,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to generate keypair: {}", e));
                    return Err(e.into());
                }
            })
        }
    }
}

pub fn eth_address(public_key: &PublicKey) -> Address {
    let uncompressed_public_key = public_key.to_encoded_point(false); // false for uncompressed
    let pk_sec1_bytes = uncompressed_public_key.as_bytes()[1..].to_vec();    
    let hash = keccak256(&pk_sec1_bytes);
    Address::from_slice(&hash[12..])
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalNetwork {
    network_name: String,
}

impl Display for LocalNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match serde_json::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                format!("ERROR: failed to serialize LocalNetwork: {}", e)
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallet {
    eth_address: String,

    secret_key: String,
    public_key: String,
    networks: Vec<LocalNetwork>,
}

impl Display for Wallet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match serde_json::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                format!("ERROR: failed to serialize Wallet: {}", e)
            }
        })
    }
}

impl Wallet {
    pub fn new(secret_key: &SecretKey, public_key: &PublicKey) -> Wallet {
        let addr: Address = eth_address(&public_key);
        Wallet {
            secret_key: {
                let bytes = secret_key.to_bytes(); 
                hex::encode(bytes.to_vec())
            },
            public_key: {
                let uncompressed_public_key = public_key.to_encoded_point(false); // false for uncompressed
                let pk_sec1_bytes = uncompressed_public_key.as_bytes()[1..].to_vec();                
                hex::encode(pk_sec1_bytes)
            },
            eth_address: addr.to_string(),
            networks: Vec::new()
        }
    }

    pub fn load(eth_address: &str) -> Result<Wallet, Box<dyn std::error::Error>> {
        match klave::ledger::get_table(WALLET_TABLE).get(&eth_address) {        
            Ok(v) => {
                let wallet: Wallet = match serde_json::from_slice(&v) {
                    Ok(w) => w,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to deserialize wallet: {}", e));
                        return Err(e.into());
                    }
                };
                Ok(wallet)
            },
            Err(e) => Err(e.into())
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_wallet = match to_string(&self) {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to serialize wallet: {}", e));
                return Err(e.into());
            }
        };
        klave::ledger::get_table(WALLET_TABLE).set(&self.eth_address, &serialized_wallet.as_bytes())
    }

    pub fn add_network(&mut self, network_name: &str) -> Result<(), Box<dyn std::error::Error>> {        
        //Check if network already exists
        if self.networks.iter().find(|x| x.network_name == network_name).is_some() {
            return Ok(());
        }

        //Check if network is valid
        let nm = Networks::load()?;
        if !nm.get_networks().contains(&network_name.to_string()) {
            return Err("Network not found".into());
        }

        self.networks.push(LocalNetwork {
            network_name: network_name.to_string()
        });
        self.save()?;
        Ok(())
    }

    pub fn get_public_key(&self) -> &str {        
        &self.public_key
    }

    pub fn get_eth_address(&self) -> &str {
        &self.eth_address
    }

    pub fn get_networks(&self) -> &Vec<LocalNetwork> {
        &self.networks
    }

    #[allow(dead_code)]
    fn get_crypto_key(&mut self) -> Result<CryptoKey, Box<dyn std::error::Error>> {
        match subtle::load_key(&self.eth_address) {
            Ok(crypto_key) => Ok(crypto_key),
            Err(_e) => {        
                let ec_params = subtle::EcKeyGenParams { named_curve: "P-256".to_string() };
                let gen_algorithm = subtle::KeyGenAlgorithm::Ecc(ec_params);
                let crypto_key = match subtle::import_key("raw", &hex::decode(self.secret_key.clone()).unwrap(), &gen_algorithm, false, &["sign"]) {
                    Ok(result) => result,
                    Err(err) => {
                        klave::notifier::send_string(&err.to_string());
                        return Err(err.into());
                    }
                };
                match subtle::save_key(&crypto_key, &self.eth_address) {
                    Ok(_) => {},
                    Err(err) => {
                        klave::notifier::send_string(&err.to_string());
                        return Err(err.into());
                    }
                };
                Ok(crypto_key)
            }
        }
    }

    pub fn get_balance(&self, nm: &Networks, network_name:  &str) -> Result<String, Box<dyn std::error::Error>> {                
        let result = nm.send(network_name, &"eth_getBalance", &[&format!("\"{}\"",&self.eth_address), "\"latest\""])?;
        Ok(result)
    }        

    pub fn can_spend(&self, nm: &Networks, network_name: &str, value: U256) -> bool {
        let balance = match U256::from_str(&
            match self.get_balance(nm, network_name) {
                Ok(v) => v,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to get balance: {}", e));
                    return false;
                }
            }) {
            Ok(v) => v,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to parse balance: {}", e));
                return false;
            }
        };
        balance >= value
    }

    #[allow(dead_code)]
    pub fn sign(&mut self,   
        mut transaction: TxEip1559
    ) -> Result<String, Box<dyn std::error::Error>> {

        // Instantiate a signer.
        let local_signer= self.secret_key
            .parse::<PrivateKeySigner>().unwrap();

        // Sign it.
        let signature = local_signer.sign_transaction_sync(&mut transaction).unwrap();
        let mut encoded_tx = Vec::new();
        transaction.eip2718_encode(&signature, &mut encoded_tx);
        let rlp_hex = format!("\"{}\"", hex::encode_prefixed(encoded_tx));
        Ok(rlp_hex)
    }

    pub fn sign_and_send(&mut self,   
        nm: &Networks,   
        network_name: &str,
        mut transaction: TxEip1559,
        trace: bool
    ) -> Result<String, Box<dyn std::error::Error>> {

        // Instantiate a signer.
        let local_signer= self.secret_key
            .parse::<PrivateKeySigner>().unwrap();

        // Sign it.
        let signature = local_signer.sign_transaction_sync(&mut transaction).unwrap();
        let mut encoded_tx = Vec::new();
        transaction.eip2718_encode(&signature, &mut encoded_tx);
        let rlp_hex = format!("\"{}\"", hex::encode_prefixed(encoded_tx));

        let result = nm.send(network_name, match trace {
            true => &"trace_rawTransaction",
            false => &"eth_sendRawTransaction"
        }, &[&rlp_hex])?;
        Ok(result)
    }
}


#[test]
fn test_convert_public_key_to_wallet_address(){

    let private_key_str = "0x89D7C6BB9F58F1EECDE6009243B6B3D968277B37A92B4D3C3D5C167E979BCF55";
    let secret_key = SecretKey::from_slice(&hex::decode(private_key_str).unwrap()).unwrap();
    let public_key = secret_key.public_key();

    let expected_public_key_str = "0x112017270089E763C2B6CBCC0E0BBDE379811FC91F0A33CBC25AF268B694A2814F835B9B08F3356EF265E9B871A5A0732186EC9257F450069DC9574AED830621";

    let uncompressed_public_key = public_key.to_encoded_point(false); // false for uncompressed
    let pk_sec1_bytes = uncompressed_public_key.as_bytes()[1..].to_vec();
    assert_eq!(pk_sec1_bytes.len(), 64);

    let computed_public_key_str = format!("0x{}", hex::encode(pk_sec1_bytes));
    assert_eq!(computed_public_key_str.to_lowercase(), expected_public_key_str.to_lowercase());

    let addr = eth_address(&public_key);
    let computed_address = addr.to_string();
    assert_eq!(computed_address.to_lowercase(), String::from("0x8CA23339DCD606267E466E12F8BFD1593E983E3A").to_lowercase());
}
