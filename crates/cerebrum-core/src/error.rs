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

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Service unavailable: {0}")]
    Unavailable(String),
}
