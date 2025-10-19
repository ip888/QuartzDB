/// Simple QuartzDB HTTP API client example
///
/// This example demonstrates:
/// - Starting a client connection
/// - Storing key-value pairs
/// - Retrieving values
/// - Deleting keys
/// - Error handling
///
/// Run the server first:
/// ```bash
/// cargo run -p quartz-server
/// ```
///
/// Then run this example:
/// ```bash
/// cargo run -p quartz-server --example simple_client
/// ```
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "http://localhost:3000/api/v1";

#[derive(Serialize)]
struct PutRequest {
    value: String,
}

#[derive(Deserialize, Debug)]
struct PutResponse {
    #[allow(dead_code)]
    key: String,
    message: String,
}

#[derive(Deserialize, Debug)]
struct GetResponse {
    key: String,
    value: String,
}

#[derive(Deserialize, Debug)]
struct DeleteResponse {
    key: String,
    message: String,
}

#[derive(Deserialize, Debug)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Deserialize, Debug)]
struct StatsResponse {
    lsm_levels: usize,
    cache_size: usize,
    wal_enabled: bool,
}

#[derive(Deserialize, Debug)]
struct ErrorResponse {
    #[allow(dead_code)]
    error: String,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    println!("🚀 QuartzDB HTTP API Client Example\n");

    // 1. Check health
    println!("1️⃣  Checking server health...");
    let health: HealthResponse = client
        .get(format!("{}/health", BASE_URL))
        .send()
        .await?
        .json()
        .await?;
    println!(
        "   ✅ Server is {} (version {})\n",
        health.status, health.version
    );

    // 2. Get stats
    println!("2️⃣  Getting storage statistics...");
    let stats: StatsResponse = client
        .get(format!("{}/stats", BASE_URL))
        .send()
        .await?
        .json()
        .await?;
    println!("   📊 LSM Levels: {}", stats.lsm_levels);
    println!("   📊 Cache Size: {}", stats.cache_size);
    println!("   📊 WAL Enabled: {}\n", stats.wal_enabled);

    // 3. Store some values
    println!("3️⃣  Storing key-value pairs...");

    let keys = vec![
        ("user:alice", "Alice Johnson"),
        ("user:bob", "Bob Smith"),
        ("product:laptop", "MacBook Pro - $2499"),
        ("session:abc123", "active"),
    ];

    for (key, value) in &keys {
        let response: PutResponse = client
            .post(format!("{}/kv/{}", BASE_URL, key))
            .json(&PutRequest {
                value: value.to_string(),
            })
            .send()
            .await?
            .json()
            .await?;
        println!("   ✅ {}", response.message);
    }
    println!();

    // 4. Retrieve values
    println!("4️⃣  Retrieving values...");
    for (key, _) in &keys {
        let response: GetResponse = client
            .get(format!("{}/kv/{}", BASE_URL, key))
            .send()
            .await?
            .json()
            .await?;
        println!("   📦 {}: {}", response.key, response.value);
    }
    println!();

    // 5. Update a value
    println!("5️⃣  Updating user:alice...");
    let response: PutResponse = client
        .post(format!("{}/kv/user:alice", BASE_URL))
        .json(&PutRequest {
            value: "Alice Johnson (Updated)".to_string(),
        })
        .send()
        .await?
        .json()
        .await?;
    println!("   ✅ {}", response.message);

    let updated: GetResponse = client
        .get(format!("{}/kv/user:alice", BASE_URL))
        .send()
        .await?
        .json()
        .await?;
    println!("   📦 New value: {}\n", updated.value);

    // 6. Delete a key
    println!("6️⃣  Deleting session:abc123...");
    let response: DeleteResponse = client
        .delete(format!("{}/kv/session:abc123", BASE_URL))
        .send()
        .await?
        .json()
        .await?;
    println!("   ✅ {}\n", response.message);

    // 7. Try to get deleted key (should fail)
    println!("7️⃣  Attempting to get deleted key...");
    let response = client
        .get(format!("{}/kv/session:abc123", BASE_URL))
        .send()
        .await?;

    if response.status() == 404 {
        let error: ErrorResponse = response.json().await?;
        println!("   ⚠️  Expected error: {}\n", error.message);
    }

    // 8. Cleanup
    println!("8️⃣  Cleaning up remaining keys...");
    for (key, _) in &keys {
        if *key != "session:abc123" {
            let response: DeleteResponse = client
                .delete(format!("{}/kv/{}", BASE_URL, key))
                .send()
                .await?
                .json()
                .await?;
            println!("   ✅ Deleted {}", response.key);
        }
    }

    println!("\n✨ Example completed successfully!");

    Ok(())
}
