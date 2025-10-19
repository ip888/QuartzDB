//! Example: Using the Integrated Storage Engine
//!
//! This example demonstrates how to use QuartzDB's integrated storage engine
//! with all components: cache, LSM tree, WAL, and compaction.

use quartz_storage::{StorageConfig, StorageEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 QuartzDB Storage Engine Example\n");

    // =====================
    // 1. Basic Usage
    // =====================
    println!("📦 Creating storage engine with default configuration...");
    let storage = StorageEngine::new("./data/example_db")?;

    // Simple put and get
    println!("✏️  Writing data...");
    storage.put(b"user:1", b"Alice").await?;
    storage.put(b"user:2", b"Bob").await?;
    storage.put(b"user:3", b"Charlie").await?;

    println!("📖 Reading data...");
    if let Some(value) = storage.get(b"user:1").await? {
        println!("   user:1 = {}", String::from_utf8_lossy(&value));
    }

    // Update existing value
    println!("🔄 Updating data...");
    storage.put(b"user:1", b"Alice Smith").await?;

    if let Some(value) = storage.get(b"user:1").await? {
        println!("   user:1 = {} (updated)", String::from_utf8_lossy(&value));
    }

    // Delete a key
    println!("🗑️  Deleting data...");
    storage.delete(b"user:3").await?;
    println!("   user:3 deleted");

    // Check statistics
    let stats = storage.stats().await;
    println!("\n📊 Storage Statistics:");
    println!("   Cache size: {}", stats.cache_size);
    println!("   LSM levels: {}", stats.lsm_levels);
    println!("   WAL enabled: {}", stats.wal_enabled);

    // =====================
    // 2. Custom Configuration
    // =====================
    println!("\n⚙️  Creating storage with custom configuration...");
    let config = StorageConfig {
        cache_size: 500,         // Smaller cache
        compaction_threshold: 3, // Trigger compaction at 3 levels
        max_level_size: 15,      // Larger max level
        enable_wal: true,        // Enable WAL for durability
    };

    let custom_storage = StorageEngine::with_config("./data/custom_db", config)?;

    println!("   ✅ Custom storage created");
    println!("   Cache: {} entries", custom_storage.config().cache_size);
    println!(
        "   Compaction threshold: {} levels",
        custom_storage.config().compaction_threshold
    );

    // =====================
    // 3. Background Compaction
    // =====================
    println!("\n🔧 Starting background compaction...");
    custom_storage.start_compaction().await;
    println!("   ✅ Compaction task running");

    // Perform many writes
    println!("✏️  Writing batch of 50 entries...");
    for i in 0..50 {
        let key = format!("batch_key_{}", i);
        let value = format!("batch_value_{}", i);
        custom_storage.put(key.as_bytes(), value.as_bytes()).await?;
    }
    println!("   ✅ Batch write complete");

    // Wait for compaction to potentially run
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Stop compaction
    println!("⏹️  Stopping compaction...");
    custom_storage.stop_compaction().await;
    println!("   ✅ Compaction stopped");

    // =====================
    // 4. Cache Performance
    // =====================
    println!("\n⚡ Demonstrating cache performance...");

    // Write some data
    custom_storage
        .put(b"hot_key", b"frequently_accessed_data")
        .await?;

    // First read (may hit cache or RocksDB)
    let start = std::time::Instant::now();
    custom_storage.get(b"hot_key").await?;
    let first_read = start.elapsed();

    // Second read (should hit cache)
    let start = std::time::Instant::now();
    custom_storage.get(b"hot_key").await?;
    let second_read = start.elapsed();

    println!("   First read:  {:?}", first_read);
    println!("   Second read: {:?} (likely cached)", second_read);

    // =====================
    // 5. WAL Operations
    // =====================
    println!("\n📝 Write-Ahead Log (WAL) operations...");

    // Create storage with WAL enabled
    let wal_config = StorageConfig {
        enable_wal: true,
        ..Default::default()
    };
    let wal_storage = StorageEngine::with_config("./data/wal_db", wal_config)?;

    // These writes will be logged to WAL first
    println!("   Writing with WAL protection...");
    wal_storage.put(b"critical:1", b"important data 1").await?;
    wal_storage.put(b"critical:2", b"important data 2").await?;
    wal_storage.put(b"critical:3", b"important data 3").await?;
    println!("   ✅ All writes persisted to WAL");

    // =====================
    // 6. Concurrent Operations
    // =====================
    println!("\n🔀 Concurrent operations example...");

    use std::sync::Arc;
    let shared_storage = Arc::new(custom_storage);
    let mut handles = vec![];

    for i in 0..5 {
        let storage = Arc::clone(&shared_storage);
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            let value = format!("concurrent_value_{}", i);

            // Write
            storage.put(key.as_bytes(), value.as_bytes()).await.unwrap();

            // Read back
            let result = storage.get(key.as_bytes()).await.unwrap();
            assert!(result.is_some());

            println!("   Task {} completed", i);
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await?;
    }
    println!("   ✅ All concurrent operations completed");

    // =====================
    // 7. Summary
    // =====================
    println!("\n✨ Example completed successfully!");
    println!("\n📚 Key Features Demonstrated:");
    println!("   ✅ Basic CRUD operations (put, get, delete)");
    println!("   ✅ Custom configuration options");
    println!("   ✅ Background compaction management");
    println!("   ✅ Cache-aware reads for performance");
    println!("   ✅ Write-Ahead Log for durability");
    println!("   ✅ Concurrent operations with Arc");

    Ok(())
}
