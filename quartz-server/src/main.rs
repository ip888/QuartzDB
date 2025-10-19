//! QuartzDB HTTP API Server
//!
//! A high-performance REST API server for QuartzDB.

use quartz_server::{create_router, AppState};
use quartz_storage::{StorageConfig, StorageEngine};
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
    compression::CompressionLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "quartz_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting QuartzDB HTTP API Server");

    // Configure storage
    let storage_path = std::env::var("QUARTZ_DATA_PATH")
        .unwrap_or_else(|_| "./data/quartz_server".to_string());

    let config = StorageConfig {
        cache_size: std::env::var("QUARTZ_CACHE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10000),
        compaction_threshold: 4,
        max_level_size: 10,
        enable_wal: true,
    };

    tracing::info!("📁 Storage path: {}", storage_path);
    tracing::info!("⚙️  Cache size: {} entries", config.cache_size);
    tracing::info!("📝 WAL enabled: {}", config.enable_wal);

    // Create storage engine
    let storage = StorageEngine::with_config(&storage_path, config)?;
    let storage = Arc::new(storage);

    tracing::info!("✅ Storage engine initialized");

    // Start background compaction
    storage.start_compaction().await;
    tracing::info!("🔧 Background compaction started");

    // Create application state
    let state = AppState {
        storage: Arc::clone(&storage),
        storage_path: Arc::new(storage_path.clone()),
        vector_indexes: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        next_vector_ids: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    // Build router with middleware
    let app = create_router(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

    // Configure server address
    let host = std::env::var("QUARTZ_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("QUARTZ_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🌐 Server listening on http://{}", addr);
    tracing::info!("📚 API Documentation:");
    tracing::info!("   GET    /api/v1/health                      - Health check");
    tracing::info!("   GET    /api/v1/stats                       - Storage statistics");
    tracing::info!("   GET    /api/v1/kv/{{key}}                     - Retrieve value");
    tracing::info!("   POST   /api/v1/kv/{{key}}                     - Store value");
    tracing::info!("   DELETE /api/v1/kv/{{key}}                     - Delete value");
    tracing::info!("");
    tracing::info!("🔍 Vector Search API (Named Indexes):");
    tracing::info!("   GET    /api/v1/indexes                     - List all indexes");
    tracing::info!("   POST   /api/v1/indexes/{{name}}               - Create/open index");
    tracing::info!("   DELETE /api/v1/indexes/{{name}}               - Delete index");
    tracing::info!("   POST   /api/v1/indexes/{{name}}/vectors       - Insert vector");
    tracing::info!("   POST   /api/v1/indexes/{{name}}/vectors/search - Search vectors");
    tracing::info!("   GET    /api/v1/indexes/{{name}}/vectors/{{id}}  - Retrieve vector");
    tracing::info!("   DELETE /api/v1/indexes/{{name}}/vectors/{{id}}  - Delete vector");
    tracing::info!("");
    tracing::info!("🎯 Ready to accept requests!");

    // Run server
    axum::serve(listener, app)
        .await?;

    // Cleanup on shutdown
    storage.stop_compaction().await;
    tracing::info!("👋 Server shutdown complete");

    Ok(())
}
