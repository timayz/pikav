use actix_web::{error::ErrorBadRequest, Error as ActixError, FromRequest};
use futures_util::future::{err, ok, Ready};

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
