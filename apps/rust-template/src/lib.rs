#[allow(warnings)]
mod bindings;

use bindings::Guest;
use klave;
use serde_json::Value;
use serde_json::json;
struct Component;

impl Guest for Component {

    fn register_routes(){
        klave::router::add_user_query("load-from-ledger");
        klave::router::add_user_transaction("insert-in-ledger");
        klave::router::add_user_query("ping");
        klave::router::add_user_query("ping2");
    }

    fn load_from_ledger(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::notify_error(&format!("failed to parse '{}' as json", cmd));
            return
        };
        let key = v["key"].as_str().unwrap();
        let Ok(res) = klave::ledger::get_table("my_table").get(key) else {
            klave::notifier::notify_error(&format!("failed to read from ledger: '{}'", cmd));
            return
        };
        let msg = if res.is_empty() {
            format!("the key '{}' was not found in table my_table", cmd)
        } else {
            let result_as_json = json!({
                "value": res,
            });
            format!("{}", result_as_json.to_string())
        };
        klave::notifier::notify(&msg);
    }

    fn insert_in_ledger(cmd: String){
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::notify_error(&format!("failed to parse '{}' as json", cmd));
            // klave:: cancel_transaction();
            return
        };
        let key = v["key"].as_str().unwrap();
        let value = v["value"].as_str().unwrap();
        match klave::ledger::get_table("my_table").set(key, value) {
            Err(e) => {
                klave::notifier::notify_error(&format!("failed to write to ledger: '{}'", e));
                // sdk::cancel_transaction();
                return
            }
            _ => {}
        }

        let result_as_json = json!({
            "inserted": true,
            "key": key,
            "value": value
            });
        klave::notifier::notify(&result_as_json.to_string());
    }

    fn ping() {
        klave::notifier::notify("pong");
    }

    fn ping2() {
        klave::notifier::notify("pang2");
    }
}

bindings::export!(Component with_types_in bindings);
