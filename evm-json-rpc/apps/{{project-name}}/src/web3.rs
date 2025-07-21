use serde_json::Value;
use crate::klave_networks::networks::Networks;

pub fn web3_client_version(cmd: String){        
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
    match network.send::<String>(network_name, &"web3_clientVersion", &[]) {
        Ok(result) => klave::notifier::send_string(&format!("{}", result)),
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
        }
    }
}

pub fn web3_sha3(cmd: String){        
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
    let input = match v["input"].as_str() {
        Some(d) => format!("\"{}\"", d),
        None => {
            klave::notifier::send_string(&format!("ERROR: 'data' field is required"));
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

    match network.send::<String>(network_name, &"web3_sha3", &[&input]) {
        Ok(result) => klave::notifier::send_string(&format!("{}", result)),
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
        }
    }
}

pub fn net_version(cmd: String){        
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
    match network.send::<String>(network_name, &"net_version", &[]) {
        Ok(result) => klave::notifier::send_string(&format!("{}", result)),
        Err(e) => {
            klave::notifier::send_string(&format!("ERROR: failed to send request: {}", e));
        }
    }
}
