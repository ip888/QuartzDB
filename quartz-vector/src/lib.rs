//! Vector Search Module for QuartzDB
//!
//! This module provides high-performance vector similarity search using HNSW indexing.
//! It's designed for AI/ML workloads, particularly for embedding-based semantic search.
//!
//! # Features
//!
//! - **HNSW Indexing**: Hierarchical Navigable Small World graphs for fast approximate nearest neighbor search
//! - **Multiple Distance Metrics**: Cosine similarity, Euclidean distance, Dot product
//! - **Persistence**: Integration with QuartzDB storage layer
//! - **Performance**: Sub-millisecond search on 100K+ vectors
//!
//! # Example
//!
//! ```rust,no_run
//! use quartz_vector::{VectorIndex, Vector, DistanceMetric};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a vector index with cosine similarity
//! let mut index = VectorIndex::new(384, DistanceMetric::Cosine)?;
//!
//! // Insert vectors
//! let vec1 = Vector::new(vec![0.1; 384]);
//! index.insert(1, vec1).await?;
//!
//! // Search for similar vectors
//! let query = Vector::new(vec![0.1; 384]);
//! let results = index.search(&query, 10).await?;
//! # Ok(())
//! # }
//! ```

mod distance;
mod hnsw;
mod storage;
mod types;
mod vector;

pub use distance::DistanceMetric;
pub use hnsw::{HnswConfig, HnswIndex};
pub use storage::PersistentVectorIndex;
pub use types::{SearchResult, VectorId};
pub use vector::{Vector, VectorIndex, VectorIndexConfig};

/// Errors that can occur during vector operations
#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("Invalid vector dimension: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Vector not found: {0}")]
    NotFound(VectorId),

    #[error("Invalid vector: {0}")]
    InvalidVector(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] quartz_storage::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Index error: {0}")]
    IndexError(String),
}

pub type Result<T> = std::result::Result<T, VectorError>;
