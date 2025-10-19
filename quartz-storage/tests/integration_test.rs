use quartz_storage::{StorageConfig, StorageEngine};
use tempfile::TempDir;

/// Helper to create a temporary storage engine for testing
async fn create_test_storage() -> (StorageEngine, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let path = temp_dir.path().to_str().unwrap();
    let engine = StorageEngine::new(path).expect("Failed to create storage engine");
    (engine, temp_dir)
}

/// Helper to create a storage engine with custom config
async fn create_test_storage_with_config(config: StorageConfig) -> (StorageEngine, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let path = temp_dir.path().to_str().unwrap();
    let engine = StorageEngine::with_config(path, config).expect("Failed to create storage engine");
    (engine, temp_dir)
}

#[tokio::test]
async fn test_basic_put_get() {
    let (engine, _temp) = create_test_storage().await;

    // Test basic put and get
    let key = b"test_key";
    let value = b"test_value";

    engine.put(key, value).await.expect("Put failed");
    let result = engine.get(key).await.expect("Get failed");

    assert_eq!(result, Some(value.to_vec()));
}

#[tokio::test]
async fn test_get_nonexistent_key() {
    let (engine, _temp) = create_test_storage().await;

    let result = engine.get(b"nonexistent").await.expect("Get failed");
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_delete() {
    let (engine, _temp) = create_test_storage().await;

    let key = b"delete_test";
    let value = b"to_be_deleted";

    // Put then delete
    engine.put(key, value).await.expect("Put failed");
    engine.delete(key).await.expect("Delete failed");

    let result = engine.get(key).await.expect("Get failed");
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_update_existing_key() {
    let (engine, _temp) = create_test_storage().await;

    let key = b"update_key";
    let value1 = b"original_value";
    let value2 = b"updated_value";

    // Put original value
    engine.put(key, value1).await.expect("First put failed");

    // Update with new value
    engine.put(key, value2).await.expect("Second put failed");

    // Verify updated value
    let result = engine.get(key).await.expect("Get failed");
    assert_eq!(result, Some(value2.to_vec()));
}

#[tokio::test]
async fn test_cache_integration() {
    let (engine, _temp) = create_test_storage().await;

    let key = b"cache_test";
    let value = b"cached_value";

    // First put should write to cache
    engine.put(key, value).await.expect("Put failed");

    // First get should hit cache
    let result1 = engine.get(key).await.expect("First get failed");
    assert_eq!(result1, Some(value.to_vec()));

    // Second get should also hit cache (faster)
    let result2 = engine.get(key).await.expect("Second get failed");
    assert_eq!(result2, Some(value.to_vec()));
}

#[tokio::test]
async fn test_wal_enabled_operations() {
    let config = StorageConfig {
        enable_wal: true,
        ..Default::default()
    };

    let (engine, temp) = create_test_storage_with_config(config).await;

    // Perform operations that should write to WAL
    engine
        .put(b"wal_key1", b"wal_value1")
        .await
        .expect("Put 1 failed");
    engine
        .put(b"wal_key2", b"wal_value2")
        .await
        .expect("Put 2 failed");
    engine.delete(b"wal_key1").await.expect("Delete failed");

    // Verify WAL file exists
    let wal_path = temp.path().join("wal.log");
    assert!(wal_path.exists(), "WAL file should exist");

    // Verify remaining data
    let result = engine.get(b"wal_key2").await.expect("Get failed");
    assert_eq!(result, Some(b"wal_value2".to_vec()));
}

#[tokio::test]
async fn test_wal_disabled_operations() {
    let config = StorageConfig {
        enable_wal: false,
        ..Default::default()
    };

    let (engine, temp) = create_test_storage_with_config(config).await;

    // Perform operations without WAL
    engine
        .put(b"no_wal_key", b"no_wal_value")
        .await
        .expect("Put failed");

    // Verify WAL file does not exist
    let wal_path = temp.path().join("wal.log");
    assert!(
        !wal_path.exists(),
        "WAL file should not exist when disabled"
    );

    // Data should still be accessible
    let result = engine.get(b"no_wal_key").await.expect("Get failed");
    assert_eq!(result, Some(b"no_wal_value".to_vec()));
}

#[tokio::test]
async fn test_compaction_task() {
    let config = StorageConfig {
        compaction_threshold: 2,
        ..Default::default()
    };

    let (engine, _temp) = create_test_storage_with_config(config).await;

    // Start compaction background task
    engine.start_compaction().await;

    // Perform some writes
    for i in 0..10 {
        let key = format!("compact_key_{}", i);
        let value = format!("compact_value_{}", i);
        engine
            .put(key.as_bytes(), value.as_bytes())
            .await
            .expect("Put failed");
    }

    // Wait a bit for compaction to potentially trigger
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Stop compaction
    engine.stop_compaction().await;

    // Verify all data is still accessible
    for i in 0..10 {
        let key = format!("compact_key_{}", i);
        let expected = format!("compact_value_{}", i);
        let result = engine.get(key.as_bytes()).await.expect("Get failed");
        assert_eq!(result, Some(expected.as_bytes().to_vec()));
    }
}

#[tokio::test]
async fn test_storage_stats() {
    let (engine, _temp) = create_test_storage().await;

    let stats = engine.stats().await;

    // Verify default stats
    assert_eq!(stats.cache_size, 1000);
    assert!(stats.wal_enabled);
    assert!(stats.lsm_levels >= 1);
}

#[tokio::test]
async fn test_custom_config() {
    let config = StorageConfig {
        cache_size: 500,
        compaction_threshold: 5,
        max_level_size: 20,
        enable_wal: false,
    };

    let (engine, _temp) = create_test_storage_with_config(config.clone()).await;

    // Verify config is applied
    assert_eq!(engine.config().cache_size, 500);
    assert_eq!(engine.config().compaction_threshold, 5);
    assert_eq!(engine.config().max_level_size, 20);
    assert!(!engine.config().enable_wal);
}

#[tokio::test]
async fn test_large_batch_operations() {
    let (engine, _temp) = create_test_storage().await;

    // Write a batch of entries
    let batch_size = 100;
    for i in 0..batch_size {
        let key = format!("batch_key_{}", i);
        let value = format!("batch_value_{}", i);
        engine
            .put(key.as_bytes(), value.as_bytes())
            .await
            .expect("Batch put failed");
    }

    // Verify all entries
    for i in 0..batch_size {
        let key = format!("batch_key_{}", i);
        let expected = format!("batch_value_{}", i);
        let result = engine.get(key.as_bytes()).await.expect("Batch get failed");
        assert_eq!(result, Some(expected.as_bytes().to_vec()));
    }
}

#[tokio::test]
async fn test_binary_data() {
    let (engine, _temp) = create_test_storage().await;

    // Test with binary data (not UTF-8)
    let key = vec![0xFF, 0xFE, 0xFD, 0xFC];
    let value = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

    engine.put(&key, &value).await.expect("Binary put failed");
    let result = engine.get(&key).await.expect("Binary get failed");

    assert_eq!(result, Some(value));
}

#[tokio::test]
async fn test_empty_key_value() {
    let (engine, _temp) = create_test_storage().await;

    // Test with empty key and value
    let empty_key = b"";
    let empty_value = b"";

    engine
        .put(empty_key, empty_value)
        .await
        .expect("Empty put failed");
    let result = engine.get(empty_key).await.expect("Empty get failed");

    assert_eq!(result, Some(empty_value.to_vec()));
}

#[tokio::test]
async fn test_concurrent_operations() {
    let (engine, _temp) = create_test_storage().await;
    let engine = std::sync::Arc::new(engine);

    // Spawn multiple concurrent tasks
    let mut handles = vec![];

    for i in 0..10 {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            let value = format!("concurrent_value_{}", i);

            engine_clone
                .put(key.as_bytes(), value.as_bytes())
                .await
                .expect("Concurrent put failed");

            let result = engine_clone
                .get(key.as_bytes())
                .await
                .expect("Concurrent get failed");
            assert_eq!(result, Some(value.as_bytes().to_vec()));
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }
}

#[tokio::test]
async fn test_cache_overflow() {
    let config = StorageConfig {
        cache_size: 5, // Small cache to test overflow
        ..Default::default()
    };

    let (engine, _temp) = create_test_storage_with_config(config).await;

    // Write more entries than cache can hold
    for i in 0..10 {
        let key = format!("overflow_key_{}", i);
        let value = format!("overflow_value_{}", i);
        engine
            .put(key.as_bytes(), value.as_bytes())
            .await
            .expect("Overflow put failed");
    }

    // All entries should still be accessible (from RocksDB)
    for i in 0..10 {
        let key = format!("overflow_key_{}", i);
        let expected = format!("overflow_value_{}", i);
        let result = engine
            .get(key.as_bytes())
            .await
            .expect("Overflow get failed");
        assert_eq!(result, Some(expected.as_bytes().to_vec()));
    }
}

#[tokio::test]
async fn test_path_and_config_getters() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let path = temp_dir.path().to_str().unwrap();

    let config = StorageConfig {
        cache_size: 777,
        ..Default::default()
    };

    let engine = StorageEngine::with_config(path, config).expect("Failed to create engine");

    // Test getters
    assert!(
        engine
            .path()
            .to_str()
            .unwrap()
            .contains(temp_dir.path().to_str().unwrap())
    );
    assert_eq!(engine.config().cache_size, 777);
}
