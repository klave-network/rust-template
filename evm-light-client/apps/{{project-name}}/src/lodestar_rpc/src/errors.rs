use displaydoc::Display;

#[derive(Debug, Display)]
pub enum Error {
    /// http error: `{0:?}`
    HTTPError(http::Error),
    /// RPC internal server error: `{0}`
    RPCInternalServerError(String),
    /// json decode error: `{0}`
    JSONDecodeError(serde_json::Error),
    /// other error: `{description}`
    Other { description: String },
}

impl From<http::Error> for Error {
    fn from(value: http::Error) -> Self {
        Self::HTTPError(value)
    }
}

impl std::error::Error for Error {}
