extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

use futures::future::Future;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::HeaderMap;
use hyper::{Body, Request, Server};
use log::{debug, error, info};

mod conf;
use conf::Conf;

mod error;
use error::ServerError;

// TODO: keycloak
// TODO: documentation
// TODO: tests
// TODO: uograde hyper_reverse_copy

fn main() -> Result<(), ServerError> {
    let conf = Conf::new()?;
    let _log = conf.log()?;

    let addr = conf.server_addr()?;

    // A `Service` is needed for every connection.
    let make_svc = make_service_fn(|socket: &AddrStream| {
        let remote_addr = socket.remote_addr();

        service_fn(move |mut req: Request<Body>| {
            debug!("{:?}", req);

            let req_headers = req.headers_mut();
            user_header_of(req_headers);

            // forward req to 3000 port (simple http-echo-server).
            return hyper_reverse_proxy::call(remote_addr.ip(), "http://localhost:3000", req);
        })
    });

    let server = Server::bind(&addr)
        .serve(make_svc)
        .map_err(|e| error!("{}", e));

    info!("Server::{}", addr);

    // Run this server for... forever!
    hyper::rt::run(server);

    Ok(())
}

fn user_header_of(req_heders: &mut HeaderMap) {
    req_heders.append("x-real-name", "blah blah".parse().unwrap());
}
