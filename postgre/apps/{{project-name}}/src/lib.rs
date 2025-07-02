#[allow(warnings)]
mod bindings;

use bindings::Guest;
use klave;

pub mod database;
pub mod crypto;
pub mod utils;
pub mod business;

struct Component;
impl Guest for Component {

    fn register_routes(){
        klave::router::add_user_transaction(&String::from("db_setup"));
        klave::router::add_user_query(&String::from("execute_table_encryption"));

        //routes defined in business part
        klave::router::add_user_query(&String::from("read_encrypted_data_per_user"));
        klave::router::add_user_query(&String::from("avg_age_for_male"));
        klave::router::add_user_query(&String::from("avg_age_for_female"));
    }

    //endpoints to test Postgres client management
    fn db_setup(cmd: String) {
        let input: database::DBInputDetails = match serde_json::from_str(&cmd) {
            Ok(input) => input,
            Err(err) => {
                klave::notifier::send_string(&format!("Invalid input: {}", err));
                return;
            }
        };

        let mut clients = match database::Clients::load() {
            Ok(c) => c,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to load clients: {}", err));
                return;
            }
        };

        match clients.add(
            input.clone(),
        ) {
            Ok(database_id) => {
                klave::notifier::send_string(&database_id);
            },
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to add database client: {}", err));
                return;
            }
        };
    }

    fn execute_table_encryption(cmd: String) {
        let db_table: database::DBTable = match serde_json::from_str(&cmd) {
            Ok(input) => input,
            Err(err) => {
                klave::notifier::send_string(&format!("Invalid input: {}", err));
                return;
            }
        };

        let mut client: database::Client = match database::Client::load(db_table.database_id.clone()) {
            Ok(c) => c,
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to load client: {}", err));
                return;
            }
        };
        let _ = match client.connect() {
            Ok(_) => (),
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to connect to client: {}", err));
                return;
            }
        };
        let _ = match client.encrypt_columns(db_table) {
            Ok(_) => (),
            Err(err) => {
                klave::notifier::send_string(&format!("Failed to encrypt columns: {}", err));
                return;
            }
        };
    }

    fn read_encrypted_data_per_user(cmd: String) {
        business::read_encrypted_data_per_user(cmd);
    }

    fn avg_age_for_male(cmd: String) {
        business::avg_age_for_male(cmd);
    }

    fn avg_age_for_female(cmd: String) {
        business::avg_age_for_female(cmd);
    }

}

bindings::export!(Component with_types_in bindings);
