use std::task;

use super::{Http, HttpConnect};
use alloy_transport::TransportError;
use alloy_transport::TransportFut;
use http::{Request, Response, status};
use alloy_json_rpc::{RequestPacket, ResponsePacket};
use url::Url;
use log::debug;
use log::trace;
use http_body_util::{BodyExt, Full};
use tower::Service;

use super::client::Client;

type Klave = super::client::Client<
    client::connect::HttpConnector,
    http_body_util::Full<::hyper::body::Bytes>,
>;

/// Type for holding HTTP errors such as 429 rate limit error.
#[derive(Debug, thiserror::Error)]
#[error("HTTP error {status} with body: {body}")]
pub struct HttpError {
    /// The HTTP status code.
    pub status: u16,
    /// The HTTP response body.
    pub body: String,
}

/// Connection details for a [`ReqwestTransport`].
pub type KlaveConnect = HttpConnect;

impl Http<Client> {
    /// Create a new [`Http`] transport.
    pub fn new(url: Url) -> Self {
        Self { client: Default::default(), url }
    }

    fn do_klave(self, req: RequestPacket) -> Result<ResponsePacket, Box<dyn std::error::Error>> {
        let request_builder = self
            .client
            .post(self.url)
            .json(&req);

        //Now I need to convert a reqwest::RequestBuilder into a Request<String>
        let request = request_builder.build().map_err(|e| Box::<dyn std::error::Error>::from(e))?;
        
        if request.url().scheme() != "http" && request.url().scheme() != "https" {
            return Err(format!("Wrong url: {}", request.url()).into());
        }
       
        //Use Klave to send the request
        //Copy the headers from the reqwest request to the klave request
        let mut klave_request_builder = http::request::Builder::new()
            .method(request.method().clone())
            .uri(request.url().to_string());

        for (key, value) in request.headers().iter() {
            klave_request_builder = klave_request_builder.header(key, value);
        }
        let klave_request = match request.body() {
            Some(body) => klave_request_builder.body(String::from_utf8(body.as_bytes().expect("There should have been a body in that branch").to_vec())?)?, 
            None => { return Err("No body in request".into()); }     
        };                        
        let response = klave::https::request(&klave_request).map_err(|e| Box::<dyn std::error::Error>::from(e))?;
        
        //Convert the Response<String> into a ResponsePacket
        let response_packet: ResponsePacket = serde_json::from_slice(response.body().as_bytes())?;
        Ok(response_packet)
    }
}

impl Service<RequestPacket> for Http<Client>
{
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        // `hyper` always returns `Ok(())`.
        task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: RequestPacket) -> Self::Future {
        let this = self.clone();        
        this.do_klave(req);
    }
}
