use std::{pin::Pin, time::SystemTime};

use actix_web::{
    error::{ErrorBadRequest, ErrorUnauthorized},
    http::header,
    web::Data,
    Error as ActixError, FromRequest, HttpMessage,
};

use futures_util::{
    future::{err, ok, Ready},
    Future,
};
use josekit::{
    jws::RS256,
    jwt::{self, JwtPayloadValidator},
};

use crate::error::ApiError;
use crate::jwks::JwksClient;

pub struct User(pub String, pub String);

impl FromRequest for User {
    type Error = ActixError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone();
        let client = req
            .app_data::<Data<JwksClient>>()
            .expect("JwksClient must be add in app data")
            .clone();

        Box::pin(async move {
            let token = match req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
            {
                Some(value) => value.replace("Bearer ", ""),
                _ => return Err(ErrorBadRequest("authorization is missing from header")),
            };

            let jwk = client.get(&token).await?;
            let verifier = RS256.verifier_from_jwk(&jwk).map_err(ApiError::from)?;
            let (payload, _) =
                jwt::decode_with_verifier(&token, &verifier).map_err(ApiError::from)?;

            let mut validator = JwtPayloadValidator::new();
            validator.set_base_time(SystemTime::now());

            if validator.validate(&payload).is_err() {
                return Err(ErrorUnauthorized("unauthorized"));
            }

            match (validator.validate(&payload), payload.clone().subject()) {
                (Ok(_), Some(sub)) => {
                    req.extensions_mut().insert(payload);

                    Ok(Self(sub.into(), token))
                }
                _ => Err(ErrorUnauthorized("unauthorized")),
            }
        })
    }
}

pub struct Client(pub String);

impl FromRequest for Client {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        match req
            .headers()
            .get("x-pikav-client-id")
            .and_then(|v| v.to_str().ok())
        {
            Some(id) => ok(Self(id.to_owned())),
            None => err(ErrorBadRequest("x-pikav-client-id is missing fromheader")),
        }
    }
}
pub struct PikavInfo(bool);

impl PikavInfo {
    pub fn is_cluster(&self) -> bool {
        self.0
    }
}

impl FromRequest for PikavInfo {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let ua = req
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok());

        let cluster = req
            .headers()
            .get("x-pikav-cluster")
            .and_then(|v| v.to_str().ok());

        let value = match (ua, cluster) {
            (Some(ua), Some(cluster)) => ua.starts_with("Pikav/") && cluster == "true",
            _ => false,
        };

        ok(PikavInfo(value))
    }
}
