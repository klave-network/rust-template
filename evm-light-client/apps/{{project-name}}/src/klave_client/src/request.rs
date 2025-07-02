use super::client::Client;
use http::{Request, Response};

/// A builder to construct the properties of a `Request`.
///
/// To construct a `RequestBuilder`, refer to the `Client` documentation.
#[derive(Debug)]
#[must_use = "RequestBuilder does nothing until you 'send' it"]
pub struct RequestBuilder {
    client: Client,
    request: Request<String>,
}

impl RequestBuilder {
    pub fn new(client: Client, request: Request<String>) -> RequestBuilder {
        let builder = RequestBuilder { client, request };
        builder
    }

    pub fn send(self, display: bool) -> Result<Response<String>, Box<dyn std::error::Error>> {
        match self.client.execute(self.request, display) {
            Ok(response) => Ok(response),
            Err(e) => Err(e),
        }
    }    
}
