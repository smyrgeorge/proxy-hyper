use hyper::{Body, Client, HeaderMap, Request, Response};
use hyper_tls::HttpsConnector;
use log::{debug, info};

use crate::conf::ProxyConf;
use crate::error::ReverseProxyError;

/// HTTP headers to strip, a whitelist is probably a better idea
const STRIPPED: [&str; 6] = [
    "content-length",
    "transfer-encoding",
    "accept-encoding",
    "content-encoding",
    "host",
    "connection",
];

pub struct ReverseProxy {
    pub client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    pub conf: ProxyConf,
}

impl ReverseProxy {
    pub async fn handle(
        &self,
        mut req: Request<Body>,
    ) -> Result<Response<Body>, ReverseProxyError> {
        //TODO: enable logs from config file.
        debug!("{:?}", req);

        let h = req.headers_mut();

        // remove client headers
        for key in &STRIPPED {
            h.remove(*key);
        }

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

    // fn user_header_of(req_heders: &mut HeaderMap) {
    //     req_heders.append("x-real-name", "blah blah".parse().unwrap());
    // }
}
