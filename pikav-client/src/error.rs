use thiserror::Error as ThisError;

#[derive(ThisError, Debug, Clone)]
pub enum ClientError {
    #[error("{0}")]
    Unknown(String),
}
