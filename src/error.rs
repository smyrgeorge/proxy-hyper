#[derive(Debug)]
pub enum ServerError {
    ConfigError(config::ConfigError),
    AddrParseError(String),
    Log4rsError(log4rs::Error),
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
