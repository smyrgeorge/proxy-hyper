#[derive(Debug)]
pub enum ServerError {
    ConfigError(config::ConfigError),
    AddrParseError(String),
    LogInitError,
    UnknownError(String),
}

#[derive(Debug)]
pub enum ProxyError {
    UriError(String),
    UnknownPath(String),
    ClientError(String),

    AuthMissingHeader(String),
    AuthCannotParseHeader(String),
    AuthCannotCreateHeader(String),
    AuthTokenError(jsonwebtoken::errors::Error),
}

impl std::error::Error for ServerError {}
impl std::error::Error for ProxyError {}

impl std::fmt::Display for ServerError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

// impl From<log::SetLoggerError> for ServerError {
//     fn from(v: log::SetLoggerError) -> Self {
//         ServerError::LogError(v)
//     }
// }

impl From<config::ConfigError> for ServerError {
    fn from(err: config::ConfigError) -> Self {
        ServerError::ConfigError(err)
    }
}

impl From<std::net::AddrParseError> for ServerError {
    fn from(_: std::net::AddrParseError) -> Self {
        ServerError::AddrParseError(
            "Server host/port is malformed. Please check your configuration file.".into(),
        )
    }
}

impl From<hyper::header::ToStrError> for ProxyError {
    fn from(_: hyper::header::ToStrError) -> Self {
        ProxyError::AuthCannotParseHeader("Cannot parse authorization header.".into())
    }
}

impl From<jsonwebtoken::errors::Error> for ProxyError {
    fn from(v: jsonwebtoken::errors::Error) -> Self {
        ProxyError::AuthTokenError(v)
    }
}

impl From<hyper::header::InvalidHeaderValue> for ProxyError {
    fn from(_: hyper::header::InvalidHeaderValue) -> Self {
        ProxyError::AuthCannotCreateHeader("Cannot create custom auth header".into())
    }
}

impl From<serde_json::Error> for ProxyError {
    fn from(_: serde_json::Error) -> Self {
        ProxyError::AuthCannotCreateHeader("Cannot create custom header".into())
    }
}
