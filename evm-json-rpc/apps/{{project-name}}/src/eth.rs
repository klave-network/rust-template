use alloy_primitives::{hex, Address, U256};
use serde_json::Value;
use crate::klave_networks::networks::Networks;
use crate::solidity::{balanceOfCall, burnCall, decimalsCall, mintCall, nameCall, ownerCall, symbolCall, totalSupplyCall};
use alloy_sol_types::SolCall;

pub fn eth_get_block_by_number(cmd: String){        
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
        return
    };

    let network = match Networks::load() {
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

    let block_number = match v["block_number"].as_str() {
        Some(bn) => format!("\"{}\"", bn),
        None => {
            klave::notifier::send_string(&format!("ERROR: 'block_number' field is required"));
            return
        }
    };

    let trace = match v["trace"].as_bool() {
        Some(c) => c,
        None => false
    };

    match trace {
        true => {
            match network.send::<alloy_rpc_types_eth::Block>(network_name, "trace_block", &[&block_number]) {
                Ok(result) => klave::notifier::send_string(&match serde_json::to_string(&result) {
                    Ok(s) => s,
                    Err(e) => format!("ERROR: failed to serialize response: {}", e)
                }),
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
                }
            }
        },
        false => {
            match network.send::<alloy_rpc_types_eth::Block>(network_name, "eth_getBlockByNumber", &[&block_number, "false"]) {
                Ok(result) => klave::notifier::send_string(&match serde_json::to_string(&result) {
                    Ok(s) => s,
                    Err(e) => format!("ERROR: failed to serialize response: {}", e)
                }),
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
                }
            }
        }    
    }
}

pub fn eth_block_number(cmd: String){    
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
        return
    };

    let network = match Networks::load() {
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
    match network.send::<String>(network_name, &"eth_blockNumber", &[]) {
        Ok(result) => klave::notifier::send_string(&format!("{}", result)),
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
        }
    }
}

pub fn eth_gas_price(cmd: String){        
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
        return
    };

    let network = match Networks::load() {
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
    match network.send::<String>(network_name, &"eth_gasPrice", &[]) {
        Ok(result) => klave::notifier::send_string(&format!("{}", result)),
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
        }
    }
}

pub fn eth_estimate_gas(cmd: String){        
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
        return
    };

    let network = match Networks::load() {
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

    let mut potential_tx: Vec<String> = Vec::new();

    match v["to"].as_str() {
        Some(t) => potential_tx.push(format!("\"to\":\"{}\"", t)),
        None => {}
    };
    match v["input"].as_str() {
        Some(d) => {
            match d {
                "name" => {
                    let hex_encoded_call = hex::encode(nameCall::new(()).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "symbol" => {
                    let hex_encoded_call = hex::encode(symbolCall::new(()).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "decimals" => {
                    let hex_encoded_call = hex::encode(decimalsCall::new(()).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "totalSupply" => {
                    let hex_encoded_call = hex::encode(totalSupplyCall::new(()).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "balanceOf" => {
                        let from_address = match v["from"].as_str() {
                            Some(f) => match f.parse::<Address>() {
                                Ok(a) => {
                                    potential_tx.push(format!("\"from\":\"{}\"", a.to_string()));
                                    a
                                },
                                Err(e) => {
                                    klave::notifier::send_string(&format!("ERROR: failed to parse contract address: {}", e));
                                    return;
                                }            
                            },
                            None => {
                                klave::notifier::send_string(&format!("ERROR: 'from' field is required"));            
                                return
                            }
                        };
                
                        let hex_encoded_call = hex::encode(balanceOfCall::new((from_address,)).abi_encode());
                        potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "mint" => {
                    let value = match v["value"].as_str() {
                        Some(v) => {
                            match U256::from_str_radix(v.trim_start_matches("0x"), 16) {
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
                    let from_address = match v["from"].as_str() {
                        Some(f) => match f.parse::<Address>() {
                            Ok(a) => {
                                potential_tx.push(format!("\"from\":\"{}\"", a.to_string()));
                                a
                            },
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to parse contract address: {}", e));
                                return;
                            }            
                        },
                        None => {
                            klave::notifier::send_string(&format!("ERROR: 'from' field is required"));            
                            return
                        }
                    };

                    let hex_encoded_call = hex::encode(mintCall::new((from_address, value)).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "burn" => {
                    let value = match v["value"].as_str() {
                        Some(v) => {
                            match U256::from_str_radix(v.trim_start_matches("0x"), 16) {
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
                    let from_address = match v["from"].as_str() {
                        Some(f) => match f.parse::<Address>() {
                            Ok(a) => {
                                potential_tx.push(format!("\"from\":\"{}\"", a.to_string()));
                                a
                            },
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to parse contract address: {}", e));
                                return;
                            }            
                        },
                        None => {
                            klave::notifier::send_string(&format!("ERROR: 'from' field is required"));            
                            return
                        }
                    };

                    let hex_encoded_call = hex::encode(burnCall::new((from_address, value)).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                _ => {
                    potential_tx.push(format!("\"input\":\"{}\"", d))
                }
            }

        },
        None => {
            match v["value"].as_str() {
                Some(c) => {
                    match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                        Ok(v) => {
                            potential_tx.push(format!("\"value\":\"{}\"", v));                            
                        },
                        Err(e) => {
                            klave::notifier::send_string(&format!("ERROR: failed to parse value: {}", e));
                            return;
                        }
                    }
                },
                None => {}
            };        
            match v["from"].as_str() {
                Some(f) => match f.parse::<Address>() {
                    Ok(a) => {
                        potential_tx.push(format!("\"from\":\"{}\"", a.to_string()));
                    },
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to parse contract address: {}", e));
                        return;
                    }            
                },
                None => {}
            };

        }
    };
    match v["gas_price"].as_str() {
        Some(gp) => potential_tx.push(format!("\"gasPrice\":\"{}\"", gp)),
        None => {}
    };
    match v["gas"].as_str() {
        Some(g) => potential_tx.push(format!("\"gas\":\"{}\"", g)),
        None => {}
    };
    match v["nonce"].as_str() {
        Some(n) => potential_tx.push(format!("\"nonce\":\"{}\"", n)),
        None => {}
    };

    match network.send::<String>(network_name, &"eth_estimateGas", &[&format!("{{{}}}", &potential_tx.join(","))]) {
        Ok(result) => klave::notifier::send_string(&format!("{}", result)),
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
        }
    }
}

pub fn eth_call_contract(cmd: String){        
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
        return
    };

    let network = match Networks::load() {
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

    let mut potential_tx: Vec<String> = Vec::new();
    match v["to"].as_str() {
        Some(t) => potential_tx.push(format!("\"to\":\"{}\"", t)),
        None => {}
    };
    match v["input"].as_str() {
        Some(d) => {
            match d {
                "name" => {
                    let hex_encoded_call = hex::encode(nameCall::new(()).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "symbol" => {
                    let hex_encoded_call = hex::encode(symbolCall::new(()).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "decimals" => {
                    let hex_encoded_call = hex::encode(decimalsCall::new(()).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "owner" => {
                    let hex_encoded_call = hex::encode(ownerCall::new(()).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "totalSupply" => {
                    let hex_encoded_call = hex::encode(totalSupplyCall::new(()).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "balanceOf" => {
                    let from_address = match v["from"].as_str() {
                        Some(f) => match f.parse::<Address>() {
                            Ok(a) => {
                                potential_tx.push(format!("\"from\":\"{}\"", a.to_string()));
                                a
                            },
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to parse contract address: {}", e));
                                return;
                            }            
                        },
                        None => {
                            klave::notifier::send_string(&format!("ERROR: 'from' field is required"));            
                            return
                        }
                    };

                    let hex_encoded_call = hex::encode(balanceOfCall::new((from_address,)).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "mint" => {
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
                    let from_address = match v["from"].as_str() {
                        Some(f) => match f.parse::<Address>() {
                            Ok(a) => {
                                potential_tx.push(format!("\"from\":\"{}\"", a.to_string()));
                                a
                            },
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to parse contract address: {}", e));
                                return;
                            }            
                        },
                        None => {
                            klave::notifier::send_string(&format!("ERROR: 'from' field is required"));            
                            return
                        }
                    };

                    let hex_encoded_call = hex::encode(mintCall::new((from_address, value)).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                "burn" => {
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
                    let from_address = match v["from"].as_str() {
                        Some(f) => match f.parse::<Address>() {
                            Ok(a) => {
                                potential_tx.push(format!("\"from\":\"{}\"", a.to_string()));
                                a
                            },
                            Err(e) => {
                                klave::notifier::send_string(&format!("ERROR: failed to parse contract address: {}", e));
                                return;
                            }            
                        },
                        None => {
                            klave::notifier::send_string(&format!("ERROR: 'from' field is required"));            
                            return
                        }
                    };

                    let hex_encoded_call = hex::encode(burnCall::new((from_address, value)).abi_encode());
                    potential_tx.push(format!("\"input\":\"0x{}\"", hex_encoded_call))
                },
                _ => {
                    klave::notifier::send_string(&format!("ERROR: unsupported function call"));
                    return;
                }
            }
        },
        None => {
            match v["value"].as_str() {
                Some(c) => {
                    match U256::from_str_radix(c.trim_start_matches("0x"), 16) {
                        Ok(v) => {
                            potential_tx.push(format!("\"value\":\"{}\"", v));                            
                        },
                        Err(e) => {
                            klave::notifier::send_string(&format!("ERROR: failed to parse value: {}", e));
                            return;
                        }
                    }
                },
                None => {}
            }; 
            match v["from"].as_str() {
                Some(f) => match f.parse::<Address>() {
                    Ok(a) => {
                        potential_tx.push(format!("\"from\":\"{}\"", a.to_string()));
                    },
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to parse contract address: {}", e));
                        return;
                    }            
                },
                None => {}
            };
        }
    };

    match v["gas_price"].as_str() {
        Some(gp) => potential_tx.push(format!("\"gasPrice\":\"{}\"", gp)),
        None => {}
    };
    match v["gas"].as_str() {
        Some(g) => potential_tx.push(format!("\"gas\":\"{}\"", g)),
        None => {}
    };
    match v["nonce"].as_str() {
        Some(n) => potential_tx.push(format!("\"nonce\":\"{}\"", n)),
        None => {}
    };
    match v["maxFeePerGas"].as_u64() {
        Some(m) => potential_tx.push(format!("\"maxFeePerGas\":\"{}\"", m)),
        None => {}
    };
    match v["maxPriorityFeePerGas"].as_u64() {
        Some(m) => potential_tx.push(format!("\"maxPriorityFeePerGas\":\"{}\"", m)),
        None => {}
    };

    let trace = match v["trace"].as_bool() {
        Some(c) => c,
        None => false
    };

    match trace {
        true => {
            match network.send::<String>(network_name, &"trace_call", &[&format!("{{{}}}", &potential_tx.join(",")), "[\"trace\", \"vmTrace\", \"stateDiff\"]", "\"latest\""]) {
                Ok(result) => klave::notifier::send_string(&format!("{}", result)),
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
                }
            }
        },
        false => {
            match network.send::<String>(network_name, &"eth_call", &[&format!("{{{}}}", &potential_tx.join(",")), "\"latest\""]) {
                Ok(result) => klave::notifier::send_string(&format!("{}", result)),
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to send request: {} - {:?}", e, potential_tx));
                }
            }
        }
    }
}

pub fn eth_protocol_version(cmd: String){        
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
        return
    };

    let network = match Networks::load() {
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
    match network.send::<String>(network_name, &"eth_protocolVersion", &[]) {
        Ok(result) => klave::notifier::send_string(&format!("{}", result)),
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
        }
    }
}

pub fn eth_chain_id(cmd: String){     
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
        return
    };

    let network = match Networks::load() {
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
    match network.send::<String>(network_name, &"eth_chainId", &[]) {
        Ok(result) => klave::notifier::send_string(&format!("{}", result)),
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
        }
    }
}

pub fn eth_get_transaction_by_hash(cmd: String){        
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
        return
    };

    let network_name = match v["network_name"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: network_name not found"));
            return;
        }
    };
    let tx_hash = match v["tx_hash"].as_str() {
        Some(t) => format!("\"{}\"",t),
        None => {
            klave::notifier::send_string(&format!("ERROR: 'tx_hash' field is required"));
            return
        }
    };
    let network = match Networks::load() {
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

    match network.send::<alloy_rpc_types_eth::Transaction>(network_name, match trace {
        true => &"trace_transaction",
        false => &"eth_getTransactionByHash"
    }, &[&tx_hash]) {
        Ok(result) => klave::notifier::send_string(&match serde_json::to_string(&result) {
            Ok(s) => s,
            Err(e) => format!("ERROR: failed to serialize response: {}", e)
        }),
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
        }
    }
}

pub fn eth_get_transaction_receipt(cmd: String){        
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
        return
    };

    let network = match Networks::load() {
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
    let tx_hash = match v["tx_hash"].as_str() {
        Some(t) => format!("\"{}\"",t),
        None => {
            klave::notifier::send_string(&format!("ERROR: 'tx_hash' field is required"));
            return
        }
    };

    match network.send::<alloy_rpc_types_eth::TransactionReceipt>(network_name, &"eth_getTransactionReceipt", &[&tx_hash]) {
        Ok(result) => klave::notifier::send_string(&match serde_json::to_string(&result) {
            Ok(s) => s,
            Err(e) => format!("ERROR: failed to serialize response: {}", e)
        }),
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
        }
    }
}

pub fn eth_get_transaction_count(cmd: String){        
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("ERROR: failed to parse '{}' as json", cmd));
        return
    };

    let network_name = match v["network_name"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: network_name not found"));
            return;
        }
    };
    let address = match v["address"].as_str() {
        Some(a) => format!("\"{}\"",a),
        None => {
            klave::notifier::send_string(&format!("ERROR: 'address' field is required"));
            return
        }
    };
    let block = match v["block"].as_str() {
        Some(b) => format!("\"{}\"",b),
        None => format!("\"latest\"")
    };
    let network = match Networks::load() {
        Ok(nm) => nm,
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to load network manager: {}. Create one first.", e));                
            return
        }
    };

    match network.send::<String>(network_name, &"eth_getTransactionCount", &[&address, &block]) {
        Ok(result) => klave::notifier::send_string(&format!("{}", result)),
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
        }
    }
}