use config::{Config, File};
use std::net::{AddrParseError, SocketAddr};

#[derive(Debug, Deserialize)]
pub struct Conf {
    pub server: ServerConf,
    pub proxy: ProxyConf,
}

#[derive(Debug, Deserialize)]
pub struct ServerConf {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct ProxyConf {
    pub scheme: String,

    pub x_forwarded_for: bool,
    pub x_forwarded_proto: bool,

    pub hosts: Vec<ProxyHost>,
}

#[derive(Debug, Deserialize)]
pub struct ProxyHost {
    pub path: String,
    pub host: String,
}

// Implementation found here:
// https://github.com/mehcode/config-rs/tree/master/examples/hierarchical-env
impl Conf {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut settings = Config::new();

        // Start off by merging in the "default" configuration file.
        // TODO: override default file (eg. command line argument).
        settings.merge(File::with_name("config/default"))?;

        // You can deserialize (and thus freeze) the entire configuration as
        settings.try_into()
    }

    pub fn server_addr(&self) -> Result<SocketAddr, AddrParseError> {
        Ok(format!("{}:{}", self.server.host, self.server.port).parse()?)
    }

    pub fn log(&self) -> Result<(), log4rs::Error> {
        Ok(log4rs::init_file("config/log4rs.yml", Default::default())?)
    }
}
