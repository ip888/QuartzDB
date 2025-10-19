use crate::Result;
use crate::cache::CacheManager;
use crate::compaction::CompactionManager;
use crate::lsm::LSMTree;
use crate::wal::WriteAheadLog;
use rocksdb::{DB, Options};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Configuration for the storage engine
#[derive(Clone, Debug)]
pub struct StorageConfig {
    /// Maximum cache size (number of entries)
    pub cache_size: usize,
    /// Compaction threshold (number of LSM levels before compaction triggers)
    pub compaction_threshold: usize,
    /// Maximum LSM level size
    pub max_level_size: usize,
    /// Enable write-ahead logging
    pub enable_wal: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            compaction_threshold: 4,
            max_level_size: 10,
            enable_wal: true,
        }
    }
}

/// Integrated storage engine combining RocksDB, LSM tree, cache, and WAL
pub struct StorageEngine {
    db: DB,
    path: PathBuf,
    cache: Arc<CacheManager>,
    lsm: Arc<LSMTree>,
    compaction_manager: Arc<CompactionManager>,
    wal: Arc<Mutex<Option<WriteAheadLog>>>,
    config: StorageConfig,
    compaction_handle: Mutex<Option<JoinHandle<()>>>,
}

impl StorageEngine {
    /// Create a new storage engine with default configuration
    pub fn new(path: &str) -> Result<Self> {
        Self::with_config(path, StorageConfig::default())
    }

    /// Create a new storage engine with custom configuration
    pub fn with_config(path: &str, config: StorageConfig) -> Result<Self> {
        let path_buf = PathBuf::from(path);

        // Initialize RocksDB
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, &path_buf)?;

        // Initialize cache
        let cache = Arc::new(CacheManager::new(config.cache_size));

        // Initialize LSM tree
        let lsm = Arc::new(LSMTree::new(config.max_level_size));

        // Initialize compaction manager
        let compaction_manager = Arc::new(CompactionManager::new(
            Arc::clone(&lsm),
            config.compaction_threshold,
        ));

        // Initialize WAL if enabled
        let wal = if config.enable_wal {
            let wal_path = path_buf.join("wal.log");
            let wal = WriteAheadLog::new(wal_path).map_err(crate::Error::Io)?;
            Arc::new(Mutex::new(Some(wal)))
        } else {
            Arc::new(Mutex::new(None))
        };

        Ok(Self {
            db,
            path: path_buf,
            cache,
            lsm,
            compaction_manager,
            wal,
            config,
            compaction_handle: Mutex::new(None),
        })
    }

    /// Start background compaction task
    pub async fn start_compaction(&self) {
        let compaction_manager = Arc::clone(&self.compaction_manager);
        let handle = tokio::spawn(async move {
            loop {
                compaction_manager.check_and_compact().await;
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });

        let mut compaction_handle = self.compaction_handle.lock().await;
        *compaction_handle = Some(handle);
    }

    /// Stop background compaction task
    pub async fn stop_compaction(&self) {
        let mut handle = self.compaction_handle.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
        }
    }

    /// Get a value by key (checks cache first, then RocksDB)
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // Check cache first
        if let Some(value) = self.cache.get(key).await {
            return Ok(Some(value));
        }

        // Check LSM tree for file location
        if let Some(_file_id) = self.lsm.search(key).await {
            // In a real implementation, we'd read from the SSTable file
            // For now, fall back to RocksDB
        }

        // Fall back to RocksDB
        let value = self.db.get(key)?;

        // Update cache if found
        if let Some(ref v) = value {
            self.cache.put(key.to_vec(), v.clone()).await;
        }

        Ok(value)
    }

    /// Put a key-value pair (writes to WAL, cache, and RocksDB)
    pub async fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        // Write to WAL first (durability)
        if self.config.enable_wal {
            let mut wal = self.wal.lock().await;
            if let Some(wal) = wal.as_mut() {
                let record = format!(
                    "PUT:{}:{}",
                    String::from_utf8_lossy(key),
                    String::from_utf8_lossy(value)
                );
                wal.append(record.as_bytes()).map_err(crate::Error::Io)?;
            }
        }

        // Write to RocksDB
        self.db.put(key, value)?;

        // Update cache
        self.cache.put(key.to_vec(), value.to_vec()).await;

        Ok(())
    }

    /// Delete a key (writes to WAL and RocksDB)
    pub async fn delete(&self, key: &[u8]) -> Result<()> {
        // Write to WAL first
        if self.config.enable_wal {
            let mut wal = self.wal.lock().await;
            if let Some(wal) = wal.as_mut() {
                let record = format!("DELETE:{}", String::from_utf8_lossy(key));
                wal.append(record.as_bytes()).map_err(crate::Error::Io)?;
            }
        }

        // Delete from RocksDB
        self.db.delete(key)?;

        // Remove from cache
        self.cache.remove(key).await;

        Ok(())
    }

    /// Get storage statistics
    pub async fn stats(&self) -> StorageStats {
        StorageStats {
            lsm_levels: self.lsm.level_count(),
            cache_size: self.config.cache_size,
            wal_enabled: self.config.enable_wal,
        }
    }

    /// Get the storage path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the configuration
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }
}

impl Drop for StorageEngine {
    fn drop(&mut self) {
        // Note: We can't await in Drop, so we need to handle cleanup differently
        // In a real implementation, we'd use tokio::runtime::Handle::current().block_on()
        // or require explicit shutdown
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub lsm_levels: usize,
    pub cache_size: usize,
    pub wal_enabled: bool,
}
