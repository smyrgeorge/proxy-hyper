extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

use hyper::{
    service::{make_service_fn, service_fn},
    Client, Server,
};
use hyper_tls::HttpsConnector;
use log::{error, info};
use std::sync::Arc;

mod conf;
use conf::Conf;

mod error;
use error::ServerError;

mod proxy;
use proxy::ReverseProxy;

// TODO: keycloak
// TODO: documentation
// TODO: tests

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let conf = Conf::new()?;
    let _log = conf.log();

    let addr = conf.server_addr()?;
    let proxy: Arc<ReverseProxy> = make_reverse_proxy(conf.proxy);

    let make_svc = make_service_fn(move |_| {
        let proxy = proxy.clone();
        async {
            Ok::<_, ServerError>(service_fn(move |req| {
                let proxy = proxy.clone();
                async move { proxy.handle(req).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    info!("Server::{}", addr);

    if let Err(e) = server.await {
        error!("{}", e);
        std::process::abort();
    }

    Ok(())
}

fn make_reverse_proxy(conf: conf::ProxyConf) -> Arc<ReverseProxy> {
    let https = HttpsConnector::new();
    let client = Client::builder().build(https);
    Arc::new(ReverseProxy { client, conf })
}
