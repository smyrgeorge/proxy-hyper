use crate::utils::strip_whitespaces;
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
    pub auth: AuthConf,
    pub hosts: Vec<ProxyHost>,
}

#[derive(Debug, Deserialize)]
pub struct AuthConf {
    pub auth: bool,

    pub alg: String,
    pub rsa_modulus: String,
    pub rsa_exponent: String,
}

#[derive(Debug, Deserialize)]
pub struct ProxyHost {
    pub path: String,
    pub host: String,
}

// Implementation found here:
// https://github.com/mehcode/config-rs/tree/master/examples/hierarchical-env
impl Conf {
    pub fn new(config_file: &str, private_config_file: &str) -> Result<Self, config::ConfigError> {
        let mut conf = Config::new();

        // Start off by merging in the "default" configuration file.
        conf.merge(File::with_name(config_file))?;

        // Add in a local configuration file.
        // NOTE: This file shouldn't be commited to git.
        conf.merge(File::with_name(private_config_file).required(false))?;

        // Removes special characters (eg. whitespaces).
        let rsa_modulus = conf.get_str("proxy.auth.rsa_modulus")?;
        conf.set("proxy.auth.rsa_modulus", strip_whitespaces(rsa_modulus))?;

        // You can deserialize (and thus freeze) the entire configuration as
        conf.try_into()
    }

    /// Start the logger (log4rs).
    pub fn log(log_file: &str) -> Result<(), log4rs::Error> {
        Ok(log4rs::init_file(log_file, Default::default())?)
    }

    /// Build server address.
    pub fn server_addr(&self) -> Result<SocketAddr, AddrParseError> {
        Ok(format!("{}:{}", self.server.host, self.server.port).parse()?)
    }
}
