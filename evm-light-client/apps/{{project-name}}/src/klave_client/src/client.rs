#![allow(unused_imports)]
use std::fs::File;
use std::io::Read;

use http::header::{
    HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE
};
use http::{Method, Request, Response, StatusCode, Uri};
use super::request::RequestBuilder;
use klave;
use url::Url;

#[derive(Clone)]
#[derive(Debug)]
pub struct ClientRef {
    headers: HeaderMap,
}

/// An `Client` to make Requests with.
///
/// The Client has various configuration values to tweak, but the defaults
/// are set to what is usually the most commonly desired value. To configure a
/// `Client`, use `Client::builder()`.
///
/// The `Client` holds a connection pool internally, so it is advised that
/// you create one and **reuse** it.
///
/// You do **not** have to wrap the `Client` in an [`Rc`] or [`Arc`] to **reuse** it,
/// because it already uses an [`Arc`] internally.
///
/// [`Rc`]: std::rc::Rc
#[derive(Clone)]
#[derive(Debug)]
pub struct Client {
    inner: ClientRef,
}

#[allow(dead_code)]
enum HttpVersionPref {
    Http1,
    All,
}

#[derive(Debug)]
struct Config {
    headers: HeaderMap,
}


/// A builder for the transport  [`RpcClient`].
///
/// This is a wrapper around [`tower::ServiceBuilder`]. It allows you to
/// configure middleware layers that will be applied to the transport, and has
/// some shortcuts for common layers and transports.
///
/// A builder accumulates Layers, and then is finished via the
/// [`ClientBuilder::connect`] method, which produces an RPC client.
#[derive(Debug)]
pub struct ClientBuilder {
    config: Config,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self { 
            config: Config { headers: HeaderMap::with_capacity(2) },             
        }
    }
}

impl ClientBuilder {
    /// Constructs a new `ClientBuilder`.
    ///
    /// This is the same as `Client::builder()`.
    pub fn new() -> ClientBuilder {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(2);
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

        ClientBuilder {
            config: Config {
                headers
            }
        }
    }    

    /// Returns a `Client` that uses this `ClientBuilder` configuration.
    ///
    /// # Errors
    ///
    /// This method fails if a TLS backend cannot be initialized, or the resolver
    /// cannot load the system configuration.
    pub fn build(self) -> Result<Client, Box<dyn std::error::Error>> {
        let config = self.config;
        Ok(Client {
            inner: ClientRef {
                headers: config.headers,
            },
        })
    }
}    

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    /// Constructs a new `Client`.
    ///
    /// # Panics
    ///
    /// This method panics if a TLS backend cannot be initialized, or the resolver
    /// cannot load the system configuration.
    ///
    /// Use `Client::builder()` if you wish to handle the failure as an `Error`
    /// instead of panicking.
    pub fn new() -> Client {
        ClientBuilder::new().build().expect("Client::new()")
    }

    /// Creates a `ClientBuilder` to configure a `Client`.
    ///
    /// This is the same as `ClientBuilder::new()`.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Start building a `Request` with the `Method` and `Url`.
    ///
    /// Returns a `RequestBuilder`, which will allow setting headers and
    /// the request body before sending.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn request(&self, method: Method, url: Url) -> RequestBuilder {
        let req = http::Request::new(String::default());
        let (mut parts, body) = req.into_parts();
        parts.method = method;
        let uri: Uri = url.as_str().parse().expect("Failed to parse Url to Uri");
        let uri_parts = uri.into_parts();
        let path_and_query: &http::uri::PathAndQuery = &match uri_parts.path_and_query {
            Some(path_and_query) => path_and_query,
            None => panic!("Url must have a path and query"),
        };
        parts.uri = Uri::from_parts({
            let mut parts = path_and_query.as_str().parse::<http::Uri>().unwrap().into_parts();
            parts.scheme = uri_parts.scheme;
            parts.authority = uri_parts.authority;
            parts
        }).unwrap();
        parts.headers = HeaderMap::with_capacity(2);
        parts.headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        parts.headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let req = Request::from_parts(parts, body);
        
        RequestBuilder::new(self.clone(), req)
    }

    /// Executes a `Request`.
    ///
    /// A `Request` can be built manually with `Request::new()` or obtained
    /// from a RequestBuilder with `RequestBuilder::build()`.
    ///
    /// You should prefer to use the `RequestBuilder` and
    /// `RequestBuilder::send()`.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending request,
    /// redirect loop was detected or redirect limit was exhausted.
    pub fn execute(
        &self,
        request: Request<String>,
        display: bool
    ) -> Result<Response<String>, Box<dyn std::error::Error>> {
        self.execute_request(request, display)
    }
    
    /// Convenience method to make a `GET` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn get(&self, url: Url) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    /// Convenience method to make a `POST` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied `Url` cannot be parsed.
    pub fn post(&self, url: Url) -> RequestBuilder {
        self.request(Method::POST, url)
    }
}

pub trait ExecuteRequest {
    fn execute_request(&self, req: Request<String>, display: bool) -> Result<Response<String>, Box<dyn std::error::Error>>;
}

impl ExecuteRequest for Client {
    fn execute_request(&self, req: Request<String>, display: bool) -> Result<Response<String>, Box<dyn std::error::Error>> {     
        if display {
            klave::notifier::send_string(&format!("execute_request request: {:?}", req));
        }

        let response = klave::https::request(&req)?;
                
        if display {
            klave::notifier::send_string(&format!("execute_request response: {:?}", response));
        }
        
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}, {:?}", response.status(), response).into());
        }
        Ok(response)
    }
}