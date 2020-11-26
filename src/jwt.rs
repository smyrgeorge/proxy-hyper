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

/// Claims model is tested with tokens generated by Keycloak SSO server.
/// May not work properly with other identity servers.
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

    pub realm_access: RealmAccess,
    pub resource_access: ResourceAccess,
}

#[derive(Debug, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceAccess {
    pub account: Account,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UserToken {
    uuid: String,
    username: String,
    email: String,
    first_name: String,
    last_name: String,
    roles: Vec<String>,
}

impl Claims {
    pub fn to_user_token(&self) -> Result<String, ProxyError> {
        // Create UserToken.
        let user_token = UserToken {
            uuid: self.sub.clone(),
            username: self.preferred_username.clone(),
            email: self.email.clone(),
            first_name: self.given_name.clone(),
            last_name: self.family_name.clone(),
            roles: self.realm_access.roles.clone(),
        };

        // Serialize to json.
        let user_token = serde_json::to_string(&user_token)?;

        // Encode to base64 and rturn the result.
        Ok(base64::encode(user_token))
    }
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
    // Find Authorization header.
    let auth_header = req_headers
        .iter()
        .find(|h| h.0.to_string().to_lowercase().eq("authorization"));

    // Extract bearer token from the header.
    let bearer = match auth_header {
        Some(h) => {
            h.1.to_str()?
                .trim_start_matches("Bearer ")
                .trim_start_matches("bearer ")
        }
        None => {
            return Err(ProxyError::AuthMissingHeader(
                "Authorization header is absent.".to_string(),
            ))
        }
    };

    Ok(bearer.to_string())
}
