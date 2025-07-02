#[allow(warnings)]
mod bindings;

use bindings::Guest;
use klave;

pub mod light_client;
pub mod consensus;
pub mod klave_client;
pub mod light_client_cli;
pub mod light_client_verifier;
pub mod lodestar_rpc;
pub mod http;

struct Component;

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

impl Guest for Component {

    fn register_routes(){  
        klave::router::add_user_query(&String::from("light_client_init"));
        klave::router::add_user_query(&String::from("light_client_update"));
        klave::router::add_user_query(&String::from("light_client_update_for_block_number"));
        klave::router::add_user_query(&String::from("light_client_update_for_period"));
        klave::router::add_user_query(&String::from("light_client_update_for_slot"));
        klave::router::add_user_query(&String::from("light_client_fetch_header_from_slot"));
        klave::router::add_user_query(&String::from("light_client_fetch_block_from_slot"));    

        klave::router::add_user_transaction(&String::from("light_client_persist"));
    }

    fn light_client_init(cmd: String){
        light_client::light_client_init(cmd);
    }

    fn light_client_update(cmd: String){
        light_client::light_client_update(cmd);
    }

    fn light_client_update_for_block_number(cmd: String){
        light_client::light_client_update_for_block_number(cmd);
    }

    fn light_client_update_for_period(cmd: String){
        light_client::light_client_update_for_period(cmd);
    }

    fn light_client_update_for_slot(cmd: String){
        light_client::light_client_update_for_slot(cmd);
    }

    fn light_client_fetch_header_from_slot(cmd: String){
        light_client::light_client_fetch_header_from_slot(cmd);
    }

    fn light_client_fetch_block_from_slot(cmd: String){
        light_client::light_client_fetch_block_from_slot(cmd);
    }    

    fn light_client_persist(cmd: String){
        light_client::light_client_persist(cmd);
    }
}

bindings::export!(Component with_types_in bindings);
