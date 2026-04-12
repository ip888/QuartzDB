//! Error types for QuartzDB FaaS

use thiserror::Error;

/// Unified error type for all QuartzDB operations.
#[derive(Debug, Error)]
pub enum FaasError {
    /// Client sent a malformed or invalid request.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Durable Object storage read/write failure.
    #[error("Storage error: {0}")]
    StorageError(String),

    /// HNSW index operation failure (dimension mismatch, missing node, etc.).
    #[error("Vector error: {0}")]
    VectorError(String),

    /// JSON serialization / deserialization failure.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Cloudflare Worker runtime error.
    #[error("Worker error: {0}")]
    WorkerError(String),
}

impl From<FaasError> for worker::Error {
    fn from(err: FaasError) -> Self {
        worker::Error::RustError(err.to_string())
    }
}
