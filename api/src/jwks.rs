use std::sync::Arc;

use crate::error::ApiError;
use actix_web::http::StatusCode;
use josekit::{
    jwk::{Jwk, JwkSet},
    jwt,
};
use parking_lot::RwLock;

#[derive(Debug, Clone)]
pub struct JwksClient {
    url: String,
    jwk_set: Arc<RwLock<JwkSet>>,
}

impl JwksClient {
    pub fn new<U: Into<String>>(url: U) -> Self {
        Self {
            url: url.into(),
            jwk_set: Arc::new(RwLock::new(JwkSet::new())),
        }
    }

    pub async fn get(&self, input: &str) -> Result<Jwk, ApiError> {
        let header = match jwt::decode_header(input) {
            Ok(header) => header,
            _ => {
                return Err(ApiError::InternalServerError(
                    "Failed to decode jwt header".to_owned(),
                ))
            }
        };

        let key_id = match header.claim("kid").and_then(|key_id| key_id.as_str()) {
            Some(key_id) => key_id,
            _ => {
                return Err(ApiError::InternalServerError(
                    "Key id is missing from jwt header".to_owned(),
                ))
            }
        };

        let alg = match header.claim("alg").and_then(|key_id| key_id.as_str()) {
            Some(alg) => alg,
            _ => {
                return Err(ApiError::InternalServerError(
                    "alg is missing from jwt header".to_owned(),
                ))
            }
        };

        {
            let jwk_set = self.jwk_set.read();

            for jwk in jwk_set.get(key_id.to_string().as_ref()) {
                if jwk.algorithm().unwrap_or("") == alg {
                    return Ok(jwk.clone());
                }
            }
        }

        let fetched_jwk_set = self.fetch_keys().await?;

        for jwk in fetched_jwk_set.get(key_id) {
            if jwk.algorithm().unwrap_or("") != alg {
                continue;
            }

            {
                let mut jwk_set = self.jwk_set.write();
                *jwk_set = fetched_jwk_set.clone();
            }

            return Ok(jwk.clone());
        }

        Err(ApiError::InternalServerError("Jwk not found".to_owned()))
    }

    async fn fetch_keys(&self) -> Result<JwkSet, ApiError> {
        let client = awc::Client::default();

        let req = client.get(&self.url);
        let mut res = req.send().await.unwrap();
        let body = match res.status() {
            StatusCode::OK => res.body().await.unwrap(),
            _ => {
                return Err(ApiError::InternalServerError(format!(
                    "Failed to fetch {} {}",
                    res.status(),
                    self.url
                )))
            }
        };

        Ok(JwkSet::from_bytes(&body)?)
    }
}
