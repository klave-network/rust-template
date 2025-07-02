pub mod client;
pub mod transport;
pub mod request;
pub mod error;

use core::str::FromStr;
use url::Url;

/// Connection details for an HTTP transport.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[doc(hidden)]
pub struct HttpConnect {
    /// The URL to connect to.
    url: Url,
}

impl HttpConnect {
    /// Create a new [`HttpConnect`] with the given URL.
    pub const fn new(url: Url) -> Self {
        Self { url }
    }

    /// Get a reference to the URL.
    pub const fn url(&self) -> &Url {
        &self.url
    }
}

impl FromStr for HttpConnect {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse()?))
    }
}

/// An Http transport.
///
/// The user must provide an internal http client and a URL to which to
/// connect. It implements `Service<Box<RawValue>>`, and therefore
/// [`Transport`].
///
/// [`Transport`]: alloy_transport::Transport
///
/// Currently supported clients are:
#[derive(Clone, Debug)]
pub struct Http<T> {
    client: T,
    url: Url,
}

impl<T> Http<T> {
    /// Create a new [`Http`] transport with a custom client.
    pub const fn with_client(client: T, url: Url) -> Self {
        Self { client, url }
    }

    /// Set the URL.
    pub fn set_url(&mut self, url: Url) {
        self.url = url;
    }

    /// Set the client.
    pub fn set_client(&mut self, client: T) {
        self.client = client;
    }

    /// Get a reference to the client.
    pub const fn client(&self) -> &T {
        &self.client
    }

    /// Get a reference to the URL.
    pub fn url(&self) -> &str {
        self.url.as_ref()
    }
}
