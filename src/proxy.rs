use hyper::{Body, Client, HeaderMap, Request, Response};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use log::debug;

use crate::conf::ProxyConf;
use crate::error::ServerError;

pub struct ReverseProxy {
    pub client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    pub conf: ProxyConf,
}

//TODO: add x-forwarded-for, x-forwarded-proto
impl ReverseProxy {
    pub async fn handle(&self, mut req: Request<Body>) -> Result<Response<Body>, ServerError> {
        //TODO: enable logs from config file.
        debug!("{:?}", req);

        *req.headers_mut() = remove_hop_headers(req.headers_mut());

        let mut builder = hyper::Uri::builder()
            .scheme(&*self.conf.scheme)
            .authority(&*self.conf.host);

        if let Some(pq) = req.uri().path_and_query() {
            builder = builder.path_and_query(pq.clone());
        }

        *req.uri_mut() = builder.build()?;

        let response = self.client.request(req).await?;
        debug!("{:?}", response);

        Ok(response)
    }
}

// fn user_header_of(req_heders: &mut HeaderMap) {
//     req_heders.append("x-real-name", "blah blah".parse().unwrap());
// }

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
