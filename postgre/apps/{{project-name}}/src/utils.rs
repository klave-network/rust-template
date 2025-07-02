use serde_json::Value;

// pub fn get_client_id() -> String {
//     let client_id = match klave::context::get("sender") {
//             Ok(id) => id,
//             Err(e) => {
//                 klave::notifier::send_string(&format!("Failed to get client ID: {}", e));
//                 return String::new();
//             }
//         };
//     return client_id;
// }

pub fn get_serde_value_into_bytes(value: &serde_json::Value) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = match serde_json::to_vec(value) {
        Ok(b) => b,
        Err(e) => {
            klave::notifier::send_string(&format!("Failed to serialize serde_json::Value to bytes: {}", e));
            return Err(e.into());
        }
    };
    Ok(bytes)
}

pub fn flatten_vec_of_vec_values_to_single_string(data: Vec<Vec<Value>>) -> String {
    let inner_strings: Vec<String> = data
        .into_iter() // Take ownership of the outer Vec
        .map(|inner_vec| { // For each inner Vec<Value>
            let values_as_strings: Vec<String> = inner_vec
                .into_iter() // Take ownership of inner Vec's elements
                .map(|value| { // For each Value
                    // Convert the Value enum variant into its string representation
                    match value {
                        Value::String(s) => format!("'{}'",s),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => "null".to_string(),
                        Value::Array(arr) => {
                            arr.into_iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(";")
                        },
                        Value::Object(obj) => {
                            obj.into_iter()
                                .map(|(k, v)| format!("{}:{}", k, v.to_string()))
                                .collect::<Vec<String>>()
                                .join(";")
                        },
                    }
                })
                .collect(); // Collect all processed Value strings into a Vec<String>

            // Join the values for this inner_vec with a comma, then wrap in parentheses
            format!("({})", values_as_strings.join(","))
        })
        .collect(); // Collect all the resulting parenthesized strings into a Vec<String>

    // Finally, join all these parenthesized strings into one single String
    inner_strings.join(",")
}