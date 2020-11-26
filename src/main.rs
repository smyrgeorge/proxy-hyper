extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

use clap::Clap;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Method, Response, Server, StatusCode, Uri};
use hyper_tls::HttpsConnector;
use log::{debug, error, info};
use std::sync::Arc;

mod conf;
use conf::Conf;

mod jwt;
mod utils;

mod error;
use error::ProxyError;
use error::ProxyError::*;
use error::ServerError;

mod proxy;
use proxy::ReverseProxy;

// TODO: x-forwarded-for (and proto) headers.
// TODO: tests

/// Command line arguments.
#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "George S. <smyrgeorge@gmail.com>")]
struct Opts {
    /// Sets a custom config file.
    #[clap(short, long, default_value = "config/default.toml")]
    config_file: String,

    /// Sets a private config file (overrides config file).
    #[clap(long, default_value = "config/private.toml")]
    private_config_file: String,

    /// Sets a custom log config file.
    #[clap(short, long, default_value = "config/log4rs.yml")]
    log_file: String,
}

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    // Parse command line arguments.
    let opts: Opts = Opts::parse();

    // Load log4rs (logger).
    Conf::log(&opts.log_file)?;

    // Load config.
    let conf = Conf::new(&opts.config_file, &opts.private_config_file)?;
    debug!("{:?}", conf);

    // Build ReverseProxy.
    let addr = conf.server_addr()?;
    let proxy: Arc<ReverseProxy> = build_proxy(conf.proxy);

    let make_svc = make_service_fn(move |_| {
        let proxy = proxy.clone();

        async {
            Ok::<_, ProxyError>(service_fn(move |req| {
                let proxy = proxy.clone();

                async move {
                    // Keep request uri and method for logging.
                    let req_uri = req.uri().clone();
                    let req_method = req.method().clone();

                    // Handle errors here (eg. Connection refused).
                    let resp = match proxy.handle(req).await {
                        Ok(resp) => Ok(resp),
                        Err(err) => handle_error(err),
                    };

                    // Log resonses here.
                    log(req_uri, req_method, &resp);

                    // Return result.
                    resp
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

/// Build ReverseProxy.
fn build_proxy(conf: conf::ProxyConf) -> Arc<ReverseProxy> {
    let https = HttpsConnector::new();
    let client = Client::builder().build(https);
    Arc::new(ReverseProxy { client, conf })
}

/// Translate ServerError(s) to rest rsponse.
/// For example a malformed request uri could possibly trigger a panic.
fn handle_error(err: ProxyError) -> Result<Response<Body>, ServerError> {
    let (status, body) = match err {
        UriError(msg) => (StatusCode::BAD_REQUEST, msg),
        UnknownPath(msg) => (StatusCode::NOT_FOUND, msg),
        ClientError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        AuthMissingHeader(msg) => (StatusCode::UNAUTHORIZED, msg),
        AuthCannotParseHeader(msg) => (StatusCode::UNAUTHORIZED, msg),
        AuthTokenError(err) => (StatusCode::UNAUTHORIZED, format!("{:?}", err)),
        AuthCannotCreateHeader(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
    };

    Response::builder()
        .status(status)
        .body(Body::from(body))
        .map_err(|err| ServerError::UnknownError(format!("{}", err)))
}

/// Log responses.
fn log(req_uri: Uri, req_method: Method, resp: &Result<Response<Body>, ServerError>) {
    match resp {
        Ok(resp) => {
            if resp.status().is_client_error() || resp.status().is_server_error() {
                error!("[{}] {} :: {}", req_method, req_uri, resp.status());
            } else {
                info!("[{}] {} :: {}", req_method, req_uri, resp.status());
            }
        }
        Err(err) => error!("[{}] {} :: {}", req_method, req_uri, err),
    };
}
