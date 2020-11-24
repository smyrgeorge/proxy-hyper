extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

use error::ProxyError;
use hyper::StatusCode;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Client, Response, Server,
};
use hyper_tls::HttpsConnector;
use log::{debug, error, info};
use std::sync::Arc;

mod conf;
use conf::Conf;

mod error;
use error::{
    ProxyError::{ClientError, UnknownPath, UriError},
    ServerError,
};

mod proxy;
use proxy::ReverseProxy;

// TODO: keycloak
// TODO: documentation
// TODO: tests

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    // Load config, and log4rs (logger).
    let conf = Conf::new()?;
    let _log = conf.log();
    debug!("{:?}", conf);

    // Build ReverseProxy.
    let addr = conf.server_addr()?;
    let proxy: Arc<ReverseProxy> = build_reverse_proxy(conf.proxy);

    let make_svc = make_service_fn(move |_| {
        let proxy = proxy.clone();

        async {
            Ok::<_, ProxyError>(service_fn(move |req| {
                let proxy = proxy.clone();

                async move {
                    // Handle errors here (eg. Connection refused).
                    match proxy.handle(req).await {
                        Ok(resp) => Ok(resp),
                        Err(err) => build_error_response(err),
                    }
                }
            }))
        }
    });

    // Build Server.
    let server = Server::bind(&addr).serve(make_svc);
    info!("Server::{}", addr);

    if let Err(e) = server.await {
        error!("{}", e);
        std::process::abort();
    }

    Ok(())
}

/// Translate ServerError(s) to rest rsponse.
/// For example a malformed request uri could possibly trigger a panic.
fn build_error_response(err: ProxyError) -> Result<Response<Body>, ServerError> {
    let (status, body) = match err {
        // REVIEW: think again about the follwing statuses.
        UriError(msg) => (StatusCode::BAD_REQUEST, msg),
        UnknownPath(msg) => (StatusCode::BAD_REQUEST, msg),
        ClientError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
    };

    Response::builder()
        .status(status)
        .body(Body::from(body))
        .map_err(|err| ServerError::UnknownError(format!("{}", err)))
}

/// Build ReverseProxy.
fn build_reverse_proxy(conf: conf::ProxyConf) -> Arc<ReverseProxy> {
    let https = HttpsConnector::new();
    let client = Client::builder().build(https);
    Arc::new(ReverseProxy { client, conf })
}
