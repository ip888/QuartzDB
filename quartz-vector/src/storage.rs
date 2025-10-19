//! Storage integration for vector index persistence

use crate::{Result, Vector, VectorError, VectorId, VectorIndex, VectorIndexConfig};
use quartz_storage::StorageEngine;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Metadata for a persisted vector index
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexMetadata {
    dimension: usize,
    metric: crate::DistanceMetric,
    hnsw_config: crate::HnswConfig,
    version: u32,
}

const INDEX_VERSION: u32 = 1;
const METADATA_KEY: &[u8] = b"__vector_index_metadata__";
const VECTOR_PREFIX: &[u8] = b"__vector__";
const METADATA_PREFIX: &[u8] = b"__vector_meta__";

/// Persistent vector index backed by QuartzDB storage
pub struct PersistentVectorIndex {
    index: VectorIndex,
    storage: StorageEngine,
}

impl PersistentVectorIndex {
    /// Create a new persistent vector index
    pub async fn create<P: AsRef<Path>>(path: P, config: VectorIndexConfig) -> Result<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| VectorError::InvalidVector("Invalid path".to_string()))?;

        let storage = StorageEngine::new(path_str).map_err(VectorError::StorageError)?;

        let index = VectorIndex::with_config(config.clone())?;

        // Save metadata
        let metadata = IndexMetadata {
            dimension: config.dimension,
            metric: config.metric,
            hnsw_config: config.hnsw_config,
            version: INDEX_VERSION,
        };

        let metadata_bytes = bincode::serialize(&metadata)
            .map_err(|e| VectorError::SerializationError(e.to_string()))?;

        storage
            .put(METADATA_KEY, &metadata_bytes)
            .await
            .map_err(VectorError::StorageError)?;

        Ok(Self { index, storage })
    }

    /// Open an existing persistent vector index
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| VectorError::InvalidVector("Invalid path".to_string()))?;

        let storage = StorageEngine::new(path_str).map_err(VectorError::StorageError)?;

        // Load metadata
        let metadata_bytes = storage
            .get(METADATA_KEY)
            .await
            .map_err(VectorError::StorageError)?
            .ok_or_else(|| VectorError::InvalidVector("Index metadata not found".to_string()))?;

        let metadata: IndexMetadata = bincode::deserialize(&metadata_bytes)
            .map_err(|e| VectorError::SerializationError(e.to_string()))?;

        if metadata.version != INDEX_VERSION {
            return Err(VectorError::InvalidVector(format!(
                "Unsupported index version: {}",
                metadata.version
            )));
        }

        let config = VectorIndexConfig {
            dimension: metadata.dimension,
            metric: metadata.metric,
            hnsw_config: metadata.hnsw_config,
        };

        let mut index = VectorIndex::with_config(config)?;

        // TODO: Load HNSW index structure from storage
        // For now, we'll rebuild by loading all vectors

        // Load all vectors
        // Note: In production, we'd want a more efficient way to iterate keys
        // This is a simplified version
        let vector_ids = Self::list_vector_ids(&storage).await?;

        for id in vector_ids {
            let vector_key = Self::vector_key(id);
            if let Some(vector_bytes) = storage
                .get(&vector_key)
                .await
                .map_err(VectorError::StorageError)?
            {
                let vector: Vector = bincode::deserialize(&vector_bytes)
                    .map_err(|e| VectorError::SerializationError(e.to_string()))?;

                // Load metadata if exists
                let meta_key = Self::metadata_key(id);
                let metadata = if let Some(meta_bytes) = storage
                    .get(&meta_key)
                    .await
                    .map_err(VectorError::StorageError)?
                {
                    let meta: String = bincode::deserialize(&meta_bytes)
                        .map_err(|e| VectorError::SerializationError(e.to_string()))?;
                    Some(meta)
                } else {
                    None
                };

                index.insert_with_metadata(id, vector, metadata).await?;
            }
        }

        Ok(Self { index, storage })
    }

    /// Insert a vector with automatic persistence
    pub async fn insert(&mut self, id: VectorId, vector: Vector) -> Result<()> {
        self.insert_with_metadata(id, vector, None).await
    }

    /// Insert a vector with metadata and automatic persistence
    pub async fn insert_with_metadata(
        &mut self,
        id: VectorId,
        vector: Vector,
        metadata: Option<String>,
    ) -> Result<()> {
        // Insert into in-memory index
        self.index
            .insert_with_metadata(id, vector.clone(), metadata.clone())
            .await?;

        // Persist vector
        let vector_key = Self::vector_key(id);
        let vector_bytes = bincode::serialize(&vector)
            .map_err(|e| VectorError::SerializationError(e.to_string()))?;
        self.storage
            .put(&vector_key, &vector_bytes)
            .await
            .map_err(VectorError::StorageError)?;

        // Persist metadata if provided
        if let Some(meta) = metadata {
            let meta_key = Self::metadata_key(id);
            let meta_bytes = bincode::serialize(&meta)
                .map_err(|e| VectorError::SerializationError(e.to_string()))?;
            self.storage
                .put(&meta_key, &meta_bytes)
                .await
                .map_err(VectorError::StorageError)?;
        }

        // TODO: Persist HNSW structure incrementally
        Ok(())
    }

    /// Search for k nearest neighbors
    pub async fn search(&self, query: &Vector, k: usize) -> Result<Vec<crate::SearchResult>> {
        self.index.search(query, k).await
    }

    /// Get a vector by ID
    pub fn get(&self, id: VectorId) -> Option<&Vector> {
        self.index.get(id)
    }

    /// Delete a vector with automatic persistence
    pub async fn delete(&mut self, id: VectorId) -> Result<()> {
        // Delete from in-memory index
        self.index.delete(id).await?;

        // Delete from storage
        let vector_key = Self::vector_key(id);
        self.storage
            .delete(&vector_key)
            .await
            .map_err(VectorError::StorageError)?;

        // Delete metadata
        let meta_key = Self::metadata_key(id);
        let _ = self.storage.delete(&meta_key).await; // Ignore error if metadata doesn't exist

        Ok(())
    }

    /// Get the number of vectors in the index
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Get all vector IDs
    pub fn ids(&self) -> Vec<VectorId> {
        self.index.ids()
    }

    /// Get the dimension of vectors in this index
    pub fn dimension(&self) -> usize {
        self.index.dimension()
    }

    /// Get the distance metric used by this index
    pub fn metric(&self) -> crate::DistanceMetric {
        self.index.metric()
    }

    /// Flush all changes to disk
    pub async fn flush(&self) -> Result<()> {
        // StorageEngine auto-flushes via WAL, but we could trigger compaction here
        Ok(())
    }

    // Helper functions

    fn vector_key(id: VectorId) -> Vec<u8> {
        let mut key = VECTOR_PREFIX.to_vec();
        key.extend_from_slice(&id.to_be_bytes());
        key
    }

    fn metadata_key(id: VectorId) -> Vec<u8> {
        let mut key = METADATA_PREFIX.to_vec();
        key.extend_from_slice(&id.to_be_bytes());
        key
    }

    async fn list_vector_ids(storage: &StorageEngine) -> Result<Vec<VectorId>> {
        // This is a simplified version - in production we'd want a prefix scan
        // For now, we'll try common ID ranges
        let mut ids = Vec::new();

        // Try IDs from 0 to 100000
        for id in 0..100000 {
            let key = Self::vector_key(id);
            if storage
                .get(&key)
                .await
                .map_err(VectorError::StorageError)?
                .is_some()
            {
                ids.push(id);
            } else if id > 1000 && ids.is_empty() {
                // Optimization: if we haven't found anything in the first 1000, stop
                break;
            }
        }

        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DistanceMetric;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_persistent_index_create_and_open() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_index");

        // Create index
        {
            let config = VectorIndexConfig::new(3, DistanceMetric::Cosine);
            let mut index = PersistentVectorIndex::create(&path, config).await.unwrap();

            // Insert vectors
            index
                .insert(1, Vector::new(vec![1.0, 0.0, 0.0]))
                .await
                .unwrap();
            index
                .insert(2, Vector::new(vec![0.0, 1.0, 0.0]))
                .await
                .unwrap();

            assert_eq!(index.len(), 2);
        }

        // Reopen index
        {
            let index = PersistentVectorIndex::open(&path).await.unwrap();
            assert_eq!(index.len(), 2);

            // Verify vectors are still there
            assert!(index.get(1).is_some());
            assert!(index.get(2).is_some());
        }
    }

    #[tokio::test]
    async fn test_persistent_index_search() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_search");

        let config = VectorIndexConfig::new(3, DistanceMetric::Cosine);
        let mut index = PersistentVectorIndex::create(&path, config).await.unwrap();

        // Insert vectors
        let v1 = Vector::new(vec![1.0, 0.0, 0.0]);
        let v2 = Vector::new(vec![0.9, 0.1, 0.0]);
        let v3 = Vector::new(vec![0.0, 1.0, 0.0]);

        index.insert(1, v1.clone()).await.unwrap();
        index.insert(2, v2).await.unwrap();
        index.insert(3, v3).await.unwrap();

        // Search
        let results = index.search(&v1, 2).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_persistent_index_delete() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_delete");

        let config = VectorIndexConfig::new(3, DistanceMetric::Cosine);
        let mut index = PersistentVectorIndex::create(&path, config).await.unwrap();

        // Insert and delete
        index
            .insert(1, Vector::new(vec![1.0, 0.0, 0.0]))
            .await
            .unwrap();
        assert_eq!(index.len(), 1);

        index.delete(1).await.unwrap();
        assert_eq!(index.len(), 0);
    }

    #[tokio::test]
    async fn test_persistent_index_with_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_metadata");

        let config = VectorIndexConfig::new(3, DistanceMetric::Cosine);
        let mut index = PersistentVectorIndex::create(&path, config).await.unwrap();

        // Insert with metadata
        index
            .insert_with_metadata(
                1,
                Vector::new(vec![1.0, 0.0, 0.0]),
                Some("test document".to_string()),
            )
            .await
            .unwrap();

        // Retrieve metadata
        let metadata = index.index.get_metadata(1);
        assert_eq!(metadata, Some("test document"));
    }
}
