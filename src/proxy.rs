use hyper::{Body, Client, HeaderMap, Request, Response};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use log::{debug, error, info};

use crate::conf::{ProxyConf, ProxyHost};
use crate::error::ProxyError;

pub struct ReverseProxy {
    pub client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    pub conf: ProxyConf,
}

impl ReverseProxy {
    pub async fn handle(&self, mut req: Request<Body>) -> Result<Response<Body>, ProxyError> {
        *req.headers_mut() = build_headers(req.headers_mut());
        *req.uri_mut() = match req.uri().path_and_query() {
            Some(path) => {
                let scheme = &*self.conf.scheme;
                let host = build_host(path.as_str(), &self.conf.hosts)?;

                hyper::Uri::builder()
                    .scheme(scheme)
                    .authority(host.as_str())
                    .path_and_query(path.clone())
                    .build()
                    .map_err(|err| ProxyError::UriError(format!("{}", err)))?
            }
            None => return Err(ProxyError::UriError(format!("Path cannot be empty."))),
        };

        let response = self
            .client
            .request(req)
            .await
            .map_err(|err| ProxyError::ClientError(format!("{}", err)))?;

        Ok(response)
    }
}

fn build_host(path: &str, hosts: &Vec<ProxyHost>) -> Result<String, ProxyError> {
    for conf in hosts.iter() {
        // FIXME replace regex match.
        // This implementation is for test purposes.
        if conf.path.starts_with(path) {
            return Ok(conf.host.clone());
        }
    }

    Err(ProxyError::UnknownPath(format!(
        "Cannot find a valid proxy path for '{}'",
        path
    )))
}

fn build_headers(req_headers: &mut HeaderMap) -> HeaderMap {
    // TODO: add x-forwarded-for, x-forwarded-proto
    // TODO: user header (eg. x-real-name).
    remove_hop_headers(req_headers)
}

/// Returns a clone of the headers without the [hop-by-hop headers].
/// [hop-by-hop headers]: http://www.w3.org/Protocols/rfc2616/rfc2616-sec13.html
fn remove_hop_headers(headers: &mut HeaderMap) -> HeaderMap {
    let mut result = HeaderMap::with_capacity(headers.len());
    for (k, v) in headers.iter() {
        if !is_hop_header(k.as_str()) {
            result.insert(k.clone(), v.clone());
        }
    }

    result
}

// REVIEW, maybe could be another way.
/// Checks for hop header.
fn is_hop_header(name: &str) -> bool {
    use unicase::Ascii;

    // A list of the headers, using `unicase` to help us compare without
    // worrying about the case, and `lazy_static!` to prevent reallocation
    // of the vector.
    lazy_static! {
        static ref HOP_HEADERS: Vec<Ascii<&'static str>> = vec![
            Ascii::new("connection"),
            Ascii::new("accept-encoding"),
            Ascii::new("content-length"),
            Ascii::new("content-encoding"),
            Ascii::new("host"),
            Ascii::new("connection"),
            Ascii::new("peep-alive"),
            Ascii::new("proxy-authenticate"),
            Ascii::new("proxy-authorization"),
            Ascii::new("te"),
            Ascii::new("trailers"),
            Ascii::new("transfer-encoding"),
            Ascii::new("upgrade"),
        ];
    }

    HOP_HEADERS.iter().any(|h| h == &name)
}
