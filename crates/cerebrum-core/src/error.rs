pub type Result<T> = std::result::Result<T, CerebrumError>;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CerebrumError {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Invalid ID: {0}")]
    InvalidId(String),

    #[error("Persistence error: {0}")]
    Persistence(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
