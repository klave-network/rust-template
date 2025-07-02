use serde_json::Value;

use crate::{database::{self, EncryptedQueryWithEncryptedUser}};


pub fn read_encrypted_data_per_user(cmd: String) {
    let input: database::ReadEncryptedTablePerUserInput = match serde_json::from_str(&cmd) {
        Ok(input) => input,
        Err(err) => {
            klave::notifier::send_string(&format!("Invalid input: {}", err));
            return;
        }
    };
    let mut client: database::Client = match database::Client::load(input.database_id.clone()) {
        Ok(c) => c,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to load client: {}", err));
            return;
        }
    };

    // Connect to the DB and establish a handle
    let _ = match client.connect() {
        Ok(_) => (),
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to connect to client: {}", err));
            return;
        }
    };

    // Build query where first name and last name have been replaced with corresponding encrypted values
    let query: EncryptedQueryWithEncryptedUser = match client.build_encrypted_query_per_user(&input) {
        Ok(res) => res,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to create query: {}", err));
            return;
        }
    };

    let mut result = match client.query::<Vec<Vec<Value>>>(&query.query) {
        Ok(res) => res,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to query the DB: {}", err));
            return;
        }
    };

    let first_name_cleartext = input.first_name.trim().to_string();
    let last_name_cleartext = input.last_name.trim().to_string();

    for elem in result.resultset.iter_mut() {
        let first_name_value = match elem.get_mut(0) {
            Some(res) => res,
            None => {
                klave::notifier::send_string(&format!("Missing first name"));
                return;
            }
        };
        if let Some(val) = first_name_value.as_str() {
            if val == query.first_name_encryption {
                *first_name_value = serde_json::Value::String(first_name_cleartext.clone());
            }
        };

        let last_name_value = match elem.get_mut(1) {
            Some(res) => res,
            None => {
                klave::notifier::send_string(&format!("Missing last name"));
                return;
            }
        };
        if let Some(val) = last_name_value.as_str() {
            if val == query.last_name_encryption {
                *last_name_value = serde_json::Value::String(last_name_cleartext.clone());
            }
        };
    }

    let _ = klave::notifier::send_json(&result);
    return;
}

pub fn avg_age_for_male(cmd: String) {
    let input: database::DatabaseIdInput = match serde_json::from_str(&cmd) {
        Ok(input) => input,
        Err(err) => {
            klave::notifier::send_string(&format!("Invalid input: {}", err));
            return;
        }
    };
    let mut client: database::Client = match database::Client::load(input.database_id.clone()) {
        Ok(c) => c,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to load client: {}", err));
            return;
        }
    };

    // Connect to the DB and establish a handle
    let _ = match client.connect() {
        Ok(_) => (),
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to connect to client: {}", err));
            return;
        }
    };

    // Query
    let query = match client.build_encrypted_query_per_gender(&"Male".to_string()) {
        Ok(res) => res,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to build the query: {}", err));
            return;
        }
    };

    // Run query
    let _ = match client.query::<Vec<Vec<Value>>>(&query) {
        Ok(res) => {
            let _ = klave::notifier::send_json(&res);
            return;
        },
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to run the query: {}", err));
            return;
        }
    };
}

pub fn avg_age_for_female(cmd: String) {
    let input: database::DatabaseIdInput = match serde_json::from_str(&cmd) {
        Ok(input) => input,
        Err(err) => {
            klave::notifier::send_string(&format!("Invalid input: {}", err));
            return;
        }
    };
    let mut client: database::Client = match database::Client::load(input.database_id.clone()) {
        Ok(c) => c,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to load client: {}", err));
            return;
        }
    };

    // Connect to the DB and establish a handle
    let _ = match client.connect() {
        Ok(_) => (),
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to connect to client: {}", err));
            return;
        }
    };

    // Query
    let query = match client.build_encrypted_query_per_gender(&"Female".to_string()) {
        Ok(res) => res,
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to build the query: {}", err));
            return;
        }
    };

    // Run query
    let _ = match client.query::<Vec<Vec<Value>>>(&query) {
        Ok(res) => {
            let _ = klave::notifier::send_json(&res);
            return;
        },
        Err(err) => {
            klave::notifier::send_string(&format!("Failed to run the query: {}", err));
            return;
        }
    };
}
