use std::str::FromStr;

use hyper::HeaderMap;
use jsonwebtoken::decode;
use jsonwebtoken::errors::Error;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::TokenData;
use jsonwebtoken::Validation;

use crate::conf::AuthConf;
use crate::error::ProxyError;

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: String,
}

pub fn verify(conf: &AuthConf, token: String) -> Result<TokenData<Claims>, Error> {
    let token = decode::<Claims>(
        &token,
        &DecodingKey::from_rsa_components(conf.rsa_modulus.as_str(), conf.rsa_exponent.as_str()),
        &Validation::new(Algorithm::from_str(&conf.alg)?),
    )?;

    Ok(token)
}

pub fn extract_bearer_token(req_headers: &HeaderMap) -> Result<String, ProxyError> {
    // HeaderName.eq is case insensitive?.
    let auth_header = req_headers.iter().find(|h| h.0.eq("Authorization"));
    let jwt = match auth_header {
        // FIXME: reeplace 'trim_start_matches' with 'strip_prefix'.
        Some(h) => h.1.to_str()?.trim_start_matches("Bearer "),
        None => {
            return Err(ProxyError::AuthMissingHeader(
                "Authorization header is absent.".to_string(),
            ))
        }
    };

    Ok(jwt.to_string())
}
