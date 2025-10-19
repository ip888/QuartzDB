//! Storage Engine Implementation
//!
//! This module implements the storage layer of QuartzDB using a LSM-tree based approach
//! with support for distributed storage and edge caching.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("Storage error: {0}")]
    Storage(String),
}

pub type Result<T> = std::result::Result<T, Error>;

mod cache;
mod compaction;
mod engine;
mod lsm;
mod wal;

pub use cache::CacheManager;
pub use engine::{StorageConfig, StorageEngine, StorageStats};
pub use lsm::LSMTree;

use std::sync::Arc;

/// Creates a complete storage stack with LSM tree and compaction
pub async fn create_storage_stack(
    max_level_size: usize,
    compaction_threshold: usize,
) -> Arc<LSMTree> {
    let mut lsm = LSMTree::new(max_level_size);

    // Add an additional level and demonstrate usage
    lsm.add_level().await;

    // Create compaction manager to handle background tasks
    let lsm_arc = Arc::new(lsm);
    let _compaction = compaction::CompactionManager::new(lsm_arc.clone(), compaction_threshold);

    lsm_arc
}
pub use wal::WriteAheadLog;
