//! Error types for QuartzDB FaaS

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FaasError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Vector error: {0}")]
    VectorError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Worker error: {0}")]
    WorkerError(String),
}

impl From<FaasError> for worker::Error {
    fn from(err: FaasError) -> Self {
        worker::Error::RustError(err.to_string())
    }
}
