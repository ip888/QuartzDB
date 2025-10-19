use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    #[error("Consensus error: {0}")]
    Consensus(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Query error: {0}")]
    Query(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Client error: {0}")]
    ClientError(String),
}