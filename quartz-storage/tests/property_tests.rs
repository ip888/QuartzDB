//! Property-based tests for storage engine
//!
//! Uses proptest to verify storage engine invariants with random inputs

use proptest::prelude::*;
use quartz_storage::{StorageConfig, StorageEngine};
use std::collections::HashMap;
use tempfile::TempDir;

/// Strategy for generating valid keys (non-empty byte arrays)
fn key_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 1..100)
}

/// Strategy for generating values
fn value_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..1000)
}

proptest! {
    /// Test that put/get roundtrip works correctly
    /// Property: For any key-value pair, if we put it and immediately get it,
    /// we should retrieve the same value
    #[test]
    fn test_put_get_roundtrip(key in key_strategy(), value in value_strategy()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let engine = StorageEngine::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Put the value
            engine.put(&key, &value).await.unwrap();

            // Get it back
            let retrieved = engine.get(&key).await.unwrap();

            // Should match exactly
            prop_assert_eq!(retrieved, Some(value));
            Ok(()) as Result<(), proptest::test_runner::TestCaseError>
        }).unwrap();
    }

    /// Test that delete actually removes data
    /// Property: After deleting a key, get should return None
    #[test]
    fn test_delete_removes_data(key in key_strategy(), value in value_strategy()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let engine = StorageEngine::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Put and verify
            engine.put(&key, &value).await.unwrap();
            let retrieved = engine.get(&key).await.unwrap();
            prop_assert_eq!(retrieved, Some(value.clone()));

            // Delete
            engine.delete(&key).await.unwrap();

            // Should be gone
            let after_delete = engine.get(&key).await.unwrap();
            prop_assert_eq!(after_delete, None);

            Ok(()) as Result<(), proptest::test_runner::TestCaseError>
        }).unwrap();
    }

    /// Test that multiple operations maintain consistency
    /// Property: A sequence of put/get/delete operations should be consistent
    #[test]
    fn test_operation_sequence_consistency(
        ops in prop::collection::vec(
            (key_strategy(), value_strategy(), any::<bool>()),
            1..20
        )
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let engine = StorageEngine::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Track expected state
            let mut expected_state: HashMap<Vec<u8>, Option<Vec<u8>>> = HashMap::new();

            for (key, value, should_delete) in ops {
                if should_delete {
                    // Delete operation
                    engine.delete(&key).await.unwrap();
                    expected_state.insert(key.clone(), None);
                } else {
                    // Put operation
                    engine.put(&key, &value).await.unwrap();
                    expected_state.insert(key.clone(), Some(value));
                }
            }

            // Verify all keys match expected state
            for (key, expected_value) in expected_state {
                let actual_value = engine.get(&key).await.unwrap();
                prop_assert_eq!(actual_value, expected_value);
            }

            Ok(()) as Result<(), proptest::test_runner::TestCaseError>
        }).unwrap();
    }

    /// Test that updates overwrite previous values
    /// Property: The most recent put operation determines the value
    #[test]
    fn test_updates_overwrite(
        key in key_strategy(),
        values in prop::collection::vec(value_strategy(), 2..10)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let engine = StorageEngine::new(temp_dir.path().to_str().unwrap()).unwrap();

            let mut last_value = Vec::new();
            
            // Put multiple values with same key
            for value in values {
                engine.put(&key, &value).await.unwrap();
                last_value = value;
            }

            // Should retrieve the last value
            let retrieved = engine.get(&key).await.unwrap();
            prop_assert_eq!(retrieved, Some(last_value));

            Ok(()) as Result<(), proptest::test_runner::TestCaseError>
        }).unwrap();
    }

    /// Test that WAL disabled doesn't break basic operations
    /// Property: Storage should work correctly even without WAL
    #[test]
    fn test_wal_disabled_consistency(key in key_strategy(), value in value_strategy()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let config = StorageConfig {
                enable_wal: false,
                ..Default::default()
            };
            let engine = StorageEngine::with_config(
                temp_dir.path().to_str().unwrap(),
                config
            ).unwrap();

            // Basic put/get should still work
            engine.put(&key, &value).await.unwrap();
            let retrieved = engine.get(&key).await.unwrap();
            prop_assert_eq!(retrieved, Some(value));

            Ok(()) as Result<(), proptest::test_runner::TestCaseError>
        }).unwrap();
    }

    /// Test cache consistency
    /// Property: Cache should always return the same value as underlying storage
    #[test]
    fn test_cache_consistency(
        key in key_strategy(),
        value in value_strategy(),
        num_reads in 2usize..10usize
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let engine = StorageEngine::new(temp_dir.path().to_str().unwrap()).unwrap();

            // Put once
            engine.put(&key, &value).await.unwrap();

            // Read multiple times (should hit cache after first read)
            for _ in 0..num_reads {
                let retrieved = engine.get(&key).await.unwrap();
                prop_assert_eq!(retrieved, Some(value.clone()));
            }

            Ok(()) as Result<(), proptest::test_runner::TestCaseError>
        }).unwrap();
    }
}

#[cfg(test)]
mod deterministic_tests {
    use super::*;

    /// Test edge case: empty value
    #[tokio::test]
    async fn test_empty_value() {
        let temp_dir = TempDir::new().unwrap();
        let engine = StorageEngine::new(temp_dir.path().to_str().unwrap()).unwrap();

        let key = b"test_key";
        let value = b"";

        engine.put(key, value).await.unwrap();
        let retrieved = engine.get(key).await.unwrap();

        assert_eq!(retrieved, Some(value.to_vec()));
    }

    /// Test edge case: large value (1MB)
    #[tokio::test]
    async fn test_large_value() {
        let temp_dir = TempDir::new().unwrap();
        let engine = StorageEngine::new(temp_dir.path().to_str().unwrap()).unwrap();

        let key = b"large_key";
        let value = vec![42u8; 1024 * 1024]; // 1MB

        engine.put(key, &value).await.unwrap();
        let retrieved = engine.get(key).await.unwrap();

        assert_eq!(retrieved, Some(value));
    }

    /// Test edge case: special characters in data
    #[tokio::test]
    async fn test_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let engine = StorageEngine::new(temp_dir.path().to_str().unwrap()).unwrap();

        let key = b"\x00\x01\xFF\xFE";
        let value = b"value\nwith\nnewlines\0and\0nulls";

        engine.put(key, value).await.unwrap();
        let retrieved = engine.get(key).await.unwrap();

        assert_eq!(retrieved, Some(value.to_vec()));
    }
}
