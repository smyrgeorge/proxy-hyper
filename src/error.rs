#[derive(Debug)]
pub enum ServerError {
    ConfigError(config::ConfigError),
    AddrParseError(String),
    Log4rsError(log4rs::Error),
    Hyper(hyper::Error),
    HyperHttp(hyper::http::Error),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl From<log4rs::Error> for ServerError {
    fn from(v: log4rs::Error) -> Self {
        ServerError::Log4rsError(v)
    }
}

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

impl From<hyper::Error> for ServerError {
    fn from(e: hyper::Error) -> Self {
        ServerError::Hyper(e)
    }
}

impl From<hyper::http::Error> for ServerError {
    fn from(e: hyper::http::Error) -> Self {
        ServerError::HyperHttp(e)
    }
}

impl std::error::Error for ServerError {}
