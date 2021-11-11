use hyper::client::HttpConnector;
use hyper::{Body, Client, HeaderMap, Request, Response};
use lazy_static::lazy_static;

use crate::conf::{AuthConf, ProxyConf, ProxyHost};
use crate::error::ProxyError;
use crate::jwt::Claims;
use crate::jwt::JwtValidator;

pub struct ReverseProxy<'conf> {
    pub client: Client<HttpConnector>,
    pub conf: &'conf ProxyConf,
    pub validator: JwtValidator<'conf>,
}

impl<'conf> ReverseProxy<'conf> {
    pub fn new(
        client: Client<HttpConnector>,
        conf: &'conf ProxyConf,
        validator: JwtValidator<'conf>,
    ) -> Self {
        Self {
            client,
            conf,
            validator,
        }
    }

    pub async fn handle(&self, mut req: Request<Body>) -> Result<Response<Body>, ProxyError> {
        // Build remote uri.
        let (proxy_host, uri) = match req.uri().path_and_query() {
            Some(path) => {
                let scheme = &*self.conf.scheme;
                let proxy_host = build_host(path.as_str(), &self.conf.hosts)?;

                let uri = hyper::Uri::builder()
                    .scheme(scheme)
                    .authority(proxy_host.host.as_str())
                    .path_and_query(path.clone())
                    .build()
                    .map_err(|err| ProxyError::UriError(format!("{}", err)))?;

                (proxy_host, uri)
            }
            None => return Err(ProxyError::UriError(format!("Path cannot be empty."))),
        };

        // Update req uri.
        *req.uri_mut() = uri;

        // If auth is enabled, verify user.
        let claims = self.check_auth(&self.conf.auth, &proxy_host, req.headers_mut())?;

        // Build headers.
        *req.headers_mut() = build_headers(req.headers_mut(), claims)?;

        // Make make the request.
        let response = self
            .client
            .request(req)
            .await
            .map_err(|err| ProxyError::ClientError(format!("{}", err)))?;

        Ok(response)
    }

    /// Check if request is authenticated.
    fn check_auth(
        &self,
        auth_conf: &AuthConf,
        proxy_host: &ProxyHost,
        req_headers: &HeaderMap,
    ) -> Result<Option<Claims>, ProxyError> {
        // Check if auth is enabled.
        let enabled = match proxy_host.auth {
            Some(auth) => auth,
            // Empty value should be converted to True.
            None => true,
        } && auth_conf.auth;

        // If enabled check auth.
        let claims = if enabled {
            let claims = self.validator.check_auth(req_headers)?;
            Some(claims)
        } else {
            None
        };

        Ok(claims)
    }
}

/// Builds remote host.
fn build_host<'host>(
    path: &str,
    hosts: &'host Vec<ProxyHost>,
) -> Result<&'host ProxyHost, ProxyError> {
    for conf in hosts.iter() {
        if path.starts_with(&conf.path) {
            return Ok(conf.clone());
        }
    }

    Err(ProxyError::UnknownPath(format!(
        "Cannot find a valid proxy path for '{}'",
        path
    )))
}

/// Removes headers.
/// If auth is enabled, tries to construct a custom user header (eg. x-real-name).
fn build_headers(req_headers: &HeaderMap, claims: Option<Claims>) -> Result<HeaderMap, ProxyError> {
    let mut headers = remove_hop_headers(req_headers);

    // if request contains auth info, add custom header to proxied request.
    if let Some(claims) = claims {
        let user_token = claims.to_user_token()?;
        headers.append("x-real-name", user_token.parse()?);
    };

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
