use hyper::client::HttpConnector;
use hyper::{Body, Client, HeaderMap, Request, Response};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;

use crate::conf::{AuthConf, ProxyConf, ProxyHost};
use crate::error::ProxyError;
use crate::jwt;

pub struct ReverseProxy {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub conf: ProxyConf,
}

impl ReverseProxy {
    pub async fn handle(&self, mut req: Request<Body>) -> Result<Response<Body>, ProxyError> {
        // Build headers.
        *req.headers_mut() = build_headers(&self.conf.auth, req.headers_mut())?;

        // Build remote uri.
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

        // Make make the request.
        let response = self
            .client
            .request(req)
            .await
            .map_err(|err| ProxyError::ClientError(format!("{}", err)))?;

        Ok(response)
    }
}

/// Builds remote host.
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

/// Removes headers.
/// If auth is enabled, tries to verify the provided token
/// and construct a custom user header (eg. x-real-name).
fn build_headers(conf: &AuthConf, req_headers: &HeaderMap) -> Result<HeaderMap, ProxyError> {
    let headers = remove_hop_headers(req_headers);

    if conf.auth {
        let jwt = jwt::extract_bearer_token(req_headers)?;
        jwt::verify(conf, jwt)?;
        // TODO: add user header
    }

    Ok(headers)
}

/// Returns a clone of the headers without the [hop-by-hop headers].
/// [hop-by-hop headers]: http://www.w3.org/Protocols/rfc2616/rfc2616-sec13.html
fn remove_hop_headers(req_headers: &HeaderMap) -> HeaderMap {
    let mut result = HeaderMap::with_capacity(req_headers.len());
    for (k, v) in req_headers.iter() {
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
            Ascii::new("Authorization"),
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
