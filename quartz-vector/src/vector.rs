//! Vector type and index implementation

use crate::{DistanceMetric, HnswConfig, HnswIndex, Result, SearchResult, VectorError, VectorId};
use serde::{Deserialize, Serialize};

/// A vector with a fixed dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    /// The vector data
    pub data: Vec<f32>,
}

impl Vector {
    /// Create a new vector
    pub fn new(data: Vec<f32>) -> Self {
        Self { data }
    }

    /// Get the dimension of this vector
    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// Normalize this vector to unit length
    pub fn normalize(&mut self) {
        crate::distance::normalize(&mut self.data);
    }

    /// Get a normalized copy of this vector
    pub fn normalized(&self) -> Self {
        let mut copy = self.clone();
        copy.normalize();
        copy
    }

    /// Calculate magnitude (L2 norm)
    pub fn magnitude(&self) -> f32 {
        crate::distance::magnitude(&self.data)
    }
}

impl From<Vec<f32>> for Vector {
    fn from(data: Vec<f32>) -> Self {
        Self::new(data)
    }
}

impl AsRef<[f32]> for Vector {
    fn as_ref(&self) -> &[f32] {
        &self.data
    }
}

/// Configuration for vector index
#[derive(Debug, Clone)]
pub struct VectorIndexConfig {
    /// Dimension of vectors
    pub dimension: usize,
    /// Distance metric to use
    pub metric: DistanceMetric,
    /// HNSW configuration
    pub hnsw_config: HnswConfig,
}

impl VectorIndexConfig {
    /// Create a new configuration with default HNSW parameters
    pub fn new(dimension: usize, metric: DistanceMetric) -> Self {
        Self {
            dimension,
            metric,
            hnsw_config: HnswConfig::default(),
        }
    }

    /// Set HNSW configuration
    pub fn with_hnsw_config(mut self, config: HnswConfig) -> Self {
        self.hnsw_config = config;
        self
    }
}

/// Vector index with HNSW for fast similarity search
pub struct VectorIndex {
    /// Configuration
    config: VectorIndexConfig,
    /// HNSW index
    hnsw: HnswIndex,
    /// Storage for vector data
    vectors: std::collections::HashMap<VectorId, Vector>,
    /// Optional metadata for vectors
    metadata: std::collections::HashMap<VectorId, String>,
}

impl VectorIndex {
    /// Create a new vector index
    pub fn new(dimension: usize, metric: DistanceMetric) -> Result<Self> {
        let config = VectorIndexConfig::new(dimension, metric);
        Self::with_config(config)
    }

    /// Create a vector index with custom configuration
    pub fn with_config(config: VectorIndexConfig) -> Result<Self> {
        let hnsw = HnswIndex::new(config.hnsw_config.clone(), config.metric);
        Ok(Self {
            config,
            hnsw,
            vectors: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Get the dimension of vectors in this index
    pub fn dimension(&self) -> usize {
        self.config.dimension
    }

    /// Get the distance metric used by this index
    pub fn metric(&self) -> DistanceMetric {
        self.config.metric
    }

    /// Get the number of vectors in the index
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// Insert a vector into the index
    pub async fn insert(&mut self, id: VectorId, vector: Vector) -> Result<()> {
        self.insert_with_metadata(id, vector, None).await
    }

    /// Insert a vector with metadata into the index
    pub async fn insert_with_metadata(
        &mut self,
        id: VectorId,
        vector: Vector,
        metadata: Option<String>,
    ) -> Result<()> {
        // Validate dimension
        if vector.dim() != self.config.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.config.dimension,
                actual: vector.dim(),
            });
        }

        // Insert into HNSW index
        self.hnsw.insert(id, &vector.data)?;

        // Store vector data
        self.vectors.insert(id, vector);

        // Store metadata if provided
        if let Some(meta) = metadata {
            self.metadata.insert(id, meta);
        }

        Ok(())
    }

    /// Search for k nearest neighbors
    pub async fn search(&self, query: &Vector, k: usize) -> Result<Vec<SearchResult>> {
        // Validate dimension
        if query.dim() != self.config.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.config.dimension,
                actual: query.dim(),
            });
        }

        // Search HNSW index
        let results = self.hnsw.search(&query.data, k)?;

        // Add metadata if available
        let results_with_metadata = results
            .into_iter()
            .map(|mut r| {
                if let Some(meta) = self.metadata.get(&r.id) {
                    r.metadata = Some(meta.clone());
                }
                r
            })
            .collect();

        Ok(results_with_metadata)
    }

    /// Get a vector by ID
    pub fn get(&self, id: VectorId) -> Option<&Vector> {
        self.vectors.get(&id)
    }

    /// Get metadata for a vector
    pub fn get_metadata(&self, id: VectorId) -> Option<&str> {
        self.metadata.get(&id).map(|s| s.as_str())
    }

    /// Delete a vector from the index
    pub async fn delete(&mut self, id: VectorId) -> Result<()> {
        if !self.vectors.contains_key(&id) {
            return Err(VectorError::NotFound(id));
        }

        // Remove from HNSW index
        self.hnsw.delete(id)?;

        // Remove vector data
        self.vectors.remove(&id);

        // Remove metadata
        self.metadata.remove(&id);

        Ok(())
    }

    /// Get all vector IDs in the index
    pub fn ids(&self) -> Vec<VectorId> {
        self.vectors.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation() {
        let v = Vector::new(vec![1.0, 2.0, 3.0]);
        assert_eq!(v.dim(), 3);
        assert_eq!(v.data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_vector_normalize() {
        let mut v = Vector::new(vec![3.0, 4.0, 0.0]);
        v.normalize();
        let mag = v.magnitude();
        assert!((mag - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_normalized() {
        let v = Vector::new(vec![3.0, 4.0, 0.0]);
        let normalized = v.normalized();
        assert!((normalized.magnitude() - 1.0).abs() < 1e-6);
        // Original should be unchanged
        assert_eq!(v.data, vec![3.0, 4.0, 0.0]);
    }

    #[tokio::test]
    async fn test_vector_index_creation() {
        let index = VectorIndex::new(128, DistanceMetric::Cosine).unwrap();
        assert_eq!(index.dimension(), 128);
        assert_eq!(index.metric(), DistanceMetric::Cosine);
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
    }

    #[tokio::test]
    async fn test_vector_index_dimension_mismatch() {
        let mut index = VectorIndex::new(3, DistanceMetric::Cosine).unwrap();
        let v = Vector::new(vec![1.0, 2.0]); // Wrong dimension

        let result = index.insert(1, v).await;
        assert!(matches!(result, Err(VectorError::DimensionMismatch { .. })));
    }
}
