use serde::{Deserialize, Serialize};
use http::Request;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TransactionRequest {
    pub jsonrpc: String,
    pub id: i32,
    pub method: String,
    pub params: Vec<String>
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JsonRpcResponse<T> {
    jsonrpc: String,
    id: u64,
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JsonRpcError {
    code: i64,
    message: String,
}

pub fn request_format_with_auth(uri: &str, body: &str, auth: &str) -> Result<Request<String>, Box<dyn std::error::Error>> {
    let stripped_body = body.replace("\\", ""); // Remove extra backslashes

    let http_request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", auth))
        .body(stripped_body)?;

    Ok(http_request)
}

pub fn request_format(uri: &str, body: &str) -> Result<Request<String>, Box<dyn std::error::Error>> {
    let stripped_body = body.replace("\\", ""); // Remove extra backslashes

    let http_request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(stripped_body)?;
        
    Ok(http_request)
}

pub fn parse_json_rpc_response<T>(response_body: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: for<'de> Deserialize<'de>,
{
    let response: JsonRpcResponse<T> = serde_json::from_str(response_body)?;
    match response.result {
        Some(block) => Ok(block),
        None => Err(format!("Error in response: {:?}", response.error).into()),
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TokenResponse {
    token: String,
}

pub fn parse_token_response<T>(response_body: &str) -> Result<String, Box<dyn std::error::Error>>
where
    T: for<'de> Deserialize<'de>,
{
    let token = match serde_json::from_str::<TokenResponse>(response_body) {
        Ok(response) => response.token,
        Err(e) => return Err(e.into())
    };
    Ok(token)
}
