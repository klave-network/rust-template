#[allow(warnings)]
mod bindings;

use bindings::Guest;
use serde_json::json;
use serde_json::Value;
struct Component;

impl Guest for Component {
    fn register_routes() {
        klave::router::add_user_query("get-quote-binary");
        klave::router::add_user_query("verify-quote");
        klave::router::add_user_query("parse-quote");
    }

    fn get_quote_binary(cmd: String) {
        let Ok(challenge) = klave::crypto::random::get_random_bytes(64) else {
            klave::notifier::send_string(&format!(
                "failed to get random bytes for challenge: '{}'",
                cmd
            ));
            return;
        };
        let Ok(quote) = klave::attestation::get_quote(&challenge) else {
            klave::notifier::send_string(&format!("failed to get quote: '{}'", cmd));
            return;
        };

        let _ = klave::notifier::send_json(&json!({
            "quote": quote
        }));
    }

    fn verify_quote(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
            return;
        };

        let Some(quote) = v.get("quote").and_then(|val| val.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_u64().and_then(|n| u8::try_from(n).ok()))
                .collect::<Vec<u8>>()
        }) else {
            klave::notifier::send_string("Failed to extract 'quote' as Vec<u8>");
            return;
        };

        let Ok(current_time_str) = klave::context::get("trusted_time") else {
            klave::notifier::send_string("Failed to get current time");
            return;
        };

        let Ok(current_time) = current_time_str.parse::<i64>() else {
            klave::notifier::send_string("Failed to parse current time as i64");
            return;
        };

        let quote_verification = match klave::attestation::verify_quote(&quote, current_time) {
            Ok(v) => v,
            Err(e) => {
                klave::notifier::send_string(&format!("Failed to verify quote: {}", e));
                return;
            }
        };

        let _ = klave::notifier::send_json(&quote_verification);
    }

    fn parse_quote(cmd: String) {
        let Ok(quote) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("failed to parse '{}' as json", cmd));
            return;
        };
        let Some(quote) = quote
            .get("quote")
            .and_then(|val| val.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_u64().and_then(|n| u8::try_from(n).ok()))
                    .collect::<Vec<u8>>()
            })
        else {
            klave::notifier::send_string("Failed to extract 'quote' as Vec<u8>");
            return;
        };

        let parsed_quote = match klave::attestation::parse_quote(&quote) {
            Ok(q) => q,
            Err(e) => {
                klave::notifier::send_string(&format!("Failed to parse quote: {}", e));
                return;
            }
        };

        let _ = klave::notifier::send_json(&parsed_quote);
    }
}

bindings::export!(Component with_types_in bindings);
