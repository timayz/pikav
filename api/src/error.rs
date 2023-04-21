use actix_web::{http::StatusCode, HttpResponse, HttpResponseBuilder, ResponseError};

use pikav::topic::TopicFilterError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum ApiError {
    #[error("internal server error")]
    InternalServerError(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("not found")]
    NotFound,
}

impl ApiError {
    pub fn into_response(self) -> Result<HttpResponse, Self> {
        Err(self)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let mut res = HttpResponseBuilder::new(self.status_code());

        if let ApiError::InternalServerError(e) = self {
            tracing::error!("{}", e);
        }

        res.json(
            serde_json::json!({"code": self.status_code().as_u16(), "message": self.to_string()}),
        )
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError::InternalServerError(e.to_string())
    }
}

impl From<TopicFilterError> for ApiError {
    fn from(e: TopicFilterError) -> Self {
        ApiError::BadRequest(e.to_string())
    }
}

impl From<pikav::Error> for ApiError {
    fn from(_: pikav::Error) -> Self {
        ApiError::NotFound
    }
}

impl From<pikav_client::Status> for ApiError {
    fn from(e: pikav_client::Status) -> Self {
        ApiError::InternalServerError(e.to_string())
    }
}
