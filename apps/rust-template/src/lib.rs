#[allow(warnings)]
mod bindings;

use bindings::{Guest, klave::sdk::sdk};
use serde_json::Value;
use serde_json::json;
struct Component;

impl Guest for Component {

    fn register_routes(){
        sdk::add_user_query("load-from-ledger");
        sdk::add_user_transaction("insert-in-ledger");
        sdk::add_user_query("ping");
        sdk::add_user_query("ping2");
    }

    fn load_from_ledger(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            sdk::notify_error(&format!("failed to parse '{}' as json", cmd));
            return
        };
        let key = v["key"].as_str().unwrap().as_bytes();
        let res = sdk::read_ledger("my_table", key);
        let msg = if res.is_empty() {
            format!("the key '{}' was not found in table my_table", cmd)
        } else {
            let result_as_json = json!({
                "value": String::from_utf8(res).unwrap_or("!! utf8 parsing error !!".to_owned()),
            });
            format!("{}", result_as_json.to_string())
        };
        sdk::notify(&msg);
    }

    fn insert_in_ledger(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            sdk::notify_error(&format!("failed to parse '{}' as json", cmd));
            sdk::cancel_transaction();
            return
        };
        let key = v["key"].as_str().unwrap().as_bytes();
        let value = v["value"].as_str().unwrap().as_bytes();
        sdk::write_ledger("my_table", key, value);
        let result_as_json = json!({
            "inserted": true,
            "key": key,
            "value": value
            });
        sdk::notify(&result_as_json.to_string());
    }

    fn ping() {
        sdk::notify("pong");
    }

    fn ping2() {
        sdk::notify("pang");
    }
}

bindings::export!(Component with_types_in bindings);
