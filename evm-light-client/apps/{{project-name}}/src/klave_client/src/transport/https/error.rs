use http::StatusCode;
use url::Url;
use reqwest::{Error as ReqwestError};
use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub(crate) enum Kind {
    Builder,
    Request,
    Redirect,
    Status(StatusCode),
    Body,
    Decode,
    Upgrade,
}

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

struct Inner {
    kind: Kind,
    source: Option<BoxError>,
    url: Option<Url>,
}

#[derive(Debug)]
pub(crate) struct TimedOut;

impl fmt::Display for TimedOut {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("operation timed out")
    }
}

impl StdError for TimedOut {}

#[derive(Debug)]
pub struct BadScheme;

impl fmt::Display for BadScheme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("URL scheme is not allowed")
    }
}

impl StdError for BadScheme {}

/// The Errors that may occur when processing a `Request`.
///
/// Note: Errors may include the full URL used to make the `Request`. If the URL
/// contains sensitive information (e.g. an API key as a query parameter), be
/// sure to remove it ([`without_url`](Error::without_url))
pub struct Error {
    inner: Box<Inner>,
}

impl Error {
    pub(crate) fn new<E>(kind: Kind, source: Option<E>) -> Error
    where
        E: Into<BoxError>,
    {
        Error {
            inner: Box::new(Inner {
                kind,
                source: source.map(Into::into),
                url: None,
            }),
        }
    }
    /// Add a url related to this error (overwriting any existing)
    pub fn with_url(mut self, url: Url) -> Self {
        self.inner.url = Some(url);
        self
    }    
}

pub(crate) fn url_bad_scheme(url: Url) -> Error {
    Error::new(Kind::Builder, Some(BadScheme)).with_url(url)
}
