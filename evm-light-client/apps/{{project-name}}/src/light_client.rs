use serde_json::Value;
use crate::light_client_cli::src::{cli::{Cli, Opts}, commands::{BlockCommand, Command, HeaderCommand, InitCommand, PersistCommand, UpdateCommand}};

static LEDGER_TABLE: &str = "light_client";

pub fn light_client_init(cmd: String){
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
        return
    };

    let network_name = match v["network_name"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: network not found"));
            return;
        }
    };
    let beacon_endpoint = match v["beacon_endpoint"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: beacon_endpoint not found"));
            return;
        }
    };

    let command_line = Cli {
        opts: Opts {
            ledger_table: String::from(format!("{}_{}", LEDGER_TABLE, network_name)),
            beacon_endpoint: String::from(beacon_endpoint),
            network: String::from(network_name),
        },
        cmd: Command::Init(InitCommand {
            trusted_block_root: v["trusted_block_root"].as_str().map(|s| s.to_string()),
            untrusted_slot: v["untrusted_slot"].as_u64(),
        }),
    };
    let _ = command_line.run();
}

pub fn light_client_persist(cmd: String){       
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
        return
    };

    let network_name = match v["network_name"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: network not found"));
            return;
        }
    };
    let beacon_endpoint = match v["beacon_endpoint"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: beacon_endpoint not found"));
            return;
        }
    };

    let command_line = Cli {
        opts: Opts {
            ledger_table: String::from(format!("{}_{}", LEDGER_TABLE, network_name)),
            beacon_endpoint: String::from(beacon_endpoint),
            network: String::from(network_name),
        },
        cmd: Command::Persist(PersistCommand {
            bootstrap_info: v["bootstrap_info"].as_str().map(|s| s.to_string()),
            state_info: v["state_info"].as_str().map(|s| s.to_string()),
            genesis_info: v["genesis_info"].as_str().map(|s| s.to_string()),
        }),
    };
    let _ = command_line.run();
}

pub fn light_client_update(cmd: String){       
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
        return
    };

    let network_name = match v["network_name"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: network not found"));
            return;
        }
    };
    let beacon_endpoint = match v["beacon_endpoint"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: beacon_endpoint not found"));
            return;
        }
    };

    let command_line = Cli {
        opts: Opts {
            ledger_table: String::from(format!("{}_{}", LEDGER_TABLE, network_name)),
            beacon_endpoint: String::from(beacon_endpoint),
            network: String::from(network_name),
        },
        cmd: Command::Update(UpdateCommand {
            target: None,
        }),
    };
    let _ = command_line.run();
}

//block number: u64
pub fn light_client_update_for_block_number(cmd: String){       
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
        return
    };

    let network_name = match v["network_name"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: network not found"));
            return;
        }
    };
    let beacon_endpoint = match v["beacon_endpoint"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: beacon_endpoint not found"));
            return;
        }
    };

    let command_line = Cli {
        opts: Opts {
            ledger_table: String::from(format!("{}_{}", LEDGER_TABLE, network_name)),
            beacon_endpoint: String::from(beacon_endpoint),
            network: String::from(network_name),
        },
        cmd: Command::Update(UpdateCommand {
            target: match v["block_number"].as_str() {
                Some(s) => match s.strip_prefix("0x") {
                    Some(s) => Some(s.to_string() + &String::from("bn")),
                    None => {
                        klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
                        return                                                
                    },
                },
                None => {
                    klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
                    return
                }
            }
        }),
    };
    let _ = command_line.run();
}

//period: u64
pub fn light_client_update_for_period(cmd: String){
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
        return
    };

    let network_name = match v["network_name"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: network not found"));
            return;
        }
    };
    let beacon_endpoint = match v["beacon_endpoint"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: beacon_endpoint not found"));
            return;
        }
    };

    let command_line = Cli {
        opts: Opts {
            ledger_table: String::from(format!("{}_{}", LEDGER_TABLE, network_name)),
            beacon_endpoint: String::from(beacon_endpoint),
            network: String::from(network_name),
        },
        cmd: Command::Update(UpdateCommand {
            target: v["period"].as_str().map(|s| s.to_string() + &String::from("period"))
        }),
    };
    let _ = command_line.run();
}

//slot: u64
pub fn light_client_update_for_slot(cmd: String){
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
        return
    };

    let network_name = match v["network_name"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: network not found"));
            return;
        }
    };
    let beacon_endpoint = match v["beacon_endpoint"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: beacon_endpoint not found"));
            return;
        }
    };

    let command_line = Cli {
        opts: Opts {
            ledger_table: String::from(format!("{}_{}", LEDGER_TABLE, network_name)),
            beacon_endpoint: String::from(beacon_endpoint),
            network: String::from(network_name),
        },
        cmd: Command::Update(UpdateCommand {
            target: v["slot"].as_str().map(|s| s.to_string() + &String::from("slot"))
        }),
    };
    let _ = command_line.run();
}

//slot: u64
pub fn light_client_fetch_header_from_slot(cmd: String){       
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
        return
    };

    let network_name = match v["network_name"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: network not found"));
            return;
        }
    };
    let beacon_endpoint = match v["beacon_endpoint"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: beacon_endpoint not found"));
            return;
        }
    };

    let command_line = Cli {
        opts: Opts {
            ledger_table: String::from(format!("{}_{}", LEDGER_TABLE, network_name)),
            beacon_endpoint: String::from(beacon_endpoint),
            network: String::from(network_name),
        },
        cmd: Command::Header(HeaderCommand {
            slot: v["slot"].as_u64(),
        }),
    };
    let _ = command_line.run();
}

//slot: u64
pub fn light_client_fetch_block_from_slot(cmd: String){       
    let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
        klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
        return
    };

    let network_name = match v["network_name"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: network not found"));
            return;
        }
    };
    let beacon_endpoint = match v["beacon_endpoint"].as_str() {
        Some(c) => c,
        None => {
            klave::notifier::send_string(&format!("ERROR: beacon_endpoint not found"));
            return;
        }
    };

    let command_line = Cli {
        opts: Opts {
            ledger_table: String::from(format!("{}_{}", LEDGER_TABLE, network_name)),
            beacon_endpoint: String::from(beacon_endpoint),
            network: String::from(network_name),
        },
        cmd: Command::Block(BlockCommand {
            slot: v["slot"].as_u64(),
        }),
    };
    let _ = command_line.run();
}