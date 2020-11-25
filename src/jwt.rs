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
    // issued at
    pub iat: u32,
    // expires at
    pub exp: u32,

    pub sub: String,
    pub email_verified: bool,
    pub name: String,
    pub preferred_username: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
}

/// Checks if request is authenticated.
pub fn check_auth(conf: &AuthConf, req_headers: &HeaderMap) -> Result<Claims, ProxyError> {
    let jwt = extract_bearer_token(req_headers)?;
    let res = verify(conf, jwt)?;
    Ok(res.claims)
}

/// Verify token.
fn verify(conf: &AuthConf, token: String) -> Result<TokenData<Claims>, Error> {
    let token = decode::<Claims>(
        &token,
        &DecodingKey::from_rsa_components(conf.rsa_modulus.as_str(), conf.rsa_exponent.as_str()),
        &Validation::new(Algorithm::from_str(&conf.alg)?),
    )?;

    Ok(token)
}

/// Extracts a jwt token from 'Authentication' header.
fn extract_bearer_token(req_headers: &HeaderMap) -> Result<String, ProxyError> {
    // HeaderName.eq is case insensitive?.
    let auth_header = req_headers.iter().find(|h| h.0.eq("Authorization"));
    let bearer = match auth_header {
        Some(h) => h.1.to_str()?.trim_start_matches("Bearer "),
        None => {
            return Err(ProxyError::AuthMissingHeader(
                "Authorization header is absent.".to_string(),
            ))
        }
    };

    Ok(bearer.to_string())
}
