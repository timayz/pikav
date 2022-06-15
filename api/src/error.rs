use actix_web::{http::StatusCode, HttpResponse, HttpResponseBuilder, ResponseError};

use pikav::topic::TopicFilterError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("internal server error")]
    InternalServerError(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("not found")]
    NotFound,
}

impl Error {
    pub fn into_response(self) -> Result<HttpResponse, Self> {
        Err(self)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Error::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            Error::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let mut res = HttpResponseBuilder::new(self.status_code());

        // if let Error::InternalServerError(e) = self {
        //     error!("{}", e);
        // }

        res.json(
            serde_json::json!({"code": self.status_code().as_u16(), "message": self.to_string()}),
        )
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::InternalServerError(e.to_string())
    }
}

impl From<TopicFilterError> for Error {
    fn from(e: TopicFilterError) -> Self {
        Error::BadRequest(e.to_string())
    }
}

impl From<josekit::JoseError> for Error {
    fn from(e: josekit::JoseError) -> Self {
        Error::InternalServerError(e.to_string())
    }
}

impl From<pikav::Error> for Error {
    fn from(_: pikav::Error) -> Self {
        Error::NotFound
    }
}
