//! QuartzDB HTTP API Server Library
//!
//! This module provides a REST API for QuartzDB storage operations.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use quartz_storage::{StorageEngine, StorageStats};
use quartz_vector::{
    DistanceMetric, HnswConfig, PersistentVectorIndex, Vector, VectorId, VectorIndexConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<StorageEngine>,
    pub storage_path: Arc<String>,
    pub vector_indexes: Arc<tokio::sync::RwLock<HashMap<String, PersistentVectorIndex>>>,
    pub next_vector_ids: Arc<tokio::sync::RwLock<HashMap<String, VectorId>>>,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// API error types
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Storage error: {0}")]
    Storage(#[from] quartz_storage::Error),

    #[error("Vector error: {0}")]
    Vector(#[from] quartz_vector::VectorError),

    #[error("Key not found: {0}")]
    NotFound(String),

    #[error("Vector not found: {0}")]
    VectorNotFound(u64),

    #[error("Index not found: {0}")]
    IndexNotFound(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Vector index not initialized")]
    VectorIndexNotInitialized,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            ApiError::Storage(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                e.to_string(),
            ),
            ApiError::Vector(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "vector_error",
                e.to_string(),
            ),
            ApiError::NotFound(key) => (
                StatusCode::NOT_FOUND,
                "not_found",
                format!("Key '{}' not found", key),
            ),
            ApiError::VectorNotFound(id) => (
                StatusCode::NOT_FOUND,
                "vector_not_found",
                format!("Vector with id {} not found", id),
            ),
            ApiError::IndexNotFound(name) => (
                StatusCode::NOT_FOUND,
                "index_not_found",
                format!("Index '{}' not found", name),
            ),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            ApiError::VectorIndexNotInitialized => (
                StatusCode::SERVICE_UNAVAILABLE,
                "vector_index_not_initialized",
                "Vector index has not been initialized. Please initialize it first.".to_string(),
            ),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message,
        });

        (status, body).into_response()
    }
}

/// Request body for PUT operations
#[derive(Debug, Deserialize)]
pub struct PutRequest {
    pub value: String,
}

/// Response for GET operations
#[derive(Debug, Serialize)]
pub struct GetResponse {
    pub key: String,
    pub value: String,
}

/// Response for PUT operations
#[derive(Debug, Serialize)]
pub struct PutResponse {
    pub key: String,
    pub message: String,
}

/// Response for DELETE operations
#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub key: String,
    pub message: String,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Stats response
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub lsm_levels: usize,
    pub cache_size: usize,
    pub wal_enabled: bool,
}

impl From<StorageStats> for StatsResponse {
    fn from(stats: StorageStats) -> Self {
        Self {
            lsm_levels: stats.lsm_levels,
            cache_size: stats.cache_size,
            wal_enabled: stats.wal_enabled,
        }
    }
}

/// GET /api/v1/kv/{key} - Retrieve a value by key
pub async fn get_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<GetResponse>, ApiError> {
    let value = state.storage.get(key.as_bytes()).await?;

    match value {
        Some(v) => {
            let value_str = String::from_utf8_lossy(&v).to_string();
            Ok(Json(GetResponse {
                key,
                value: value_str,
            }))
        }
        None => Err(ApiError::NotFound(key)),
    }
}

/// PUT /api/v1/kv/{key} - Store a key-value pair
pub async fn put_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(payload): Json<PutRequest>,
) -> Result<Json<PutResponse>, ApiError> {
    if key.is_empty() {
        return Err(ApiError::BadRequest("Key cannot be empty".to_string()));
    }

    state
        .storage
        .put(key.as_bytes(), payload.value.as_bytes())
        .await?;

    Ok(Json(PutResponse {
        key,
        message: "Value stored successfully".to_string(),
    }))
}

/// DELETE /api/v1/kv/{key} - Delete a key-value pair
pub async fn delete_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<DeleteResponse>, ApiError> {
    // Check if key exists first
    let exists = state.storage.get(key.as_bytes()).await?.is_some();

    if !exists {
        return Err(ApiError::NotFound(key));
    }

    state.storage.delete(key.as_bytes()).await?;

    Ok(Json(DeleteResponse {
        key,
        message: "Key deleted successfully".to_string(),
    }))
}

/// GET /api/v1/health - Health check endpoint
pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// GET /api/v1/stats - Get storage statistics
pub async fn stats_handler(State(state): State<AppState>) -> Json<StatsResponse> {
    let stats = state.storage.stats().await;
    Json(stats.into())
}

// ============================================================================
// Vector Search API Types and Handlers
// ============================================================================

/// Request to initialize vector index
#[derive(Debug, Deserialize)]
pub struct InitVectorIndexRequest {
    pub dimension: usize,
    #[serde(default = "default_distance_metric")]
    pub metric: String,
    #[serde(default)]
    pub m: Option<usize>,
    #[serde(default)]
    pub ef_construction: Option<usize>,
}

fn default_distance_metric() -> String {
    "cosine".to_string()
}

/// Response for vector index initialization
#[derive(Debug, Serialize)]
pub struct InitVectorIndexResponse {
    pub message: String,
    pub dimension: usize,
    pub metric: String,
}

/// Request to insert a vector
#[derive(Debug, Deserialize)]
pub struct InsertVectorRequest {
    pub vector: Vec<f32>,
    #[serde(default)]
    pub metadata: Option<String>,
}

/// Response for vector insertion
#[derive(Debug, Serialize)]
pub struct InsertVectorResponse {
    pub id: VectorId,
    pub message: String,
}

/// Request to search for similar vectors
#[derive(Debug, Deserialize)]
pub struct SearchVectorsRequest {
    pub vector: Vec<f32>,
    #[serde(default = "default_k")]
    pub k: usize,
}

fn default_k() -> usize {
    10
}

/// Response for vector search
#[derive(Debug, Serialize)]
pub struct SearchVectorsResponse {
    pub results: Vec<VectorSearchResult>,
}

#[derive(Debug, Serialize)]
pub struct VectorSearchResult {
    pub id: VectorId,
    pub distance: f32,
    pub vector: Vec<f32>,
    pub metadata: Option<String>,
}

/// Response for vector retrieval
#[derive(Debug, Serialize)]
pub struct GetVectorResponse {
    pub id: VectorId,
    pub vector: Vec<f32>,
    pub metadata: Option<String>,
}

/// Response for vector deletion
#[derive(Debug, Serialize)]
pub struct DeleteVectorResponse {
    pub id: VectorId,
    pub message: String,
}

/// POST /api/v1/indexes/{name} - Initialize or open a named vector index
pub async fn init_vector_index_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<InitVectorIndexRequest>,
) -> Result<Json<InitVectorIndexResponse>, ApiError> {
    // Validate index name
    if name.is_empty() || name.contains('/') || name.contains('\\') {
        return Err(ApiError::BadRequest("Invalid index name".to_string()));
    }

    // Parse distance metric
    let metric = match req.metric.to_lowercase().as_str() {
        "cosine" => DistanceMetric::Cosine,
        "euclidean" => DistanceMetric::Euclidean,
        "dotproduct" | "dot_product" => DistanceMetric::DotProduct,
        _ => {
            return Err(ApiError::BadRequest(format!(
                "Invalid distance metric: {}. Must be one of: cosine, euclidean, dotproduct",
                req.metric
            )));
        }
    };

    // Create HNSW config - use presets or defaults
    let hnsw_config = if req.m.is_some() || req.ef_construction.is_some() {
        // If custom parameters provided, use balanced as base and warn
        tracing::warn!("Custom HNSW parameters not directly supported, using balanced preset");
        HnswConfig::balanced()
    } else {
        HnswConfig::balanced()
    };

    // Create vector index config
    let config = VectorIndexConfig {
        dimension: req.dimension,
        metric,
        hnsw_config,
    };

    // Create persistent vector index with storage path
    let vector_path = format!("{}/indexes/{}", state.storage_path.as_str(), name);

    // Try to open existing index first, create new one if it doesn't exist
    let index = match PersistentVectorIndex::open(&vector_path).await {
        Ok(existing_index) => {
            // Verify the existing index has the same configuration
            let existing_dim = existing_index.dimension();
            let existing_metric = existing_index.metric();

            if existing_dim != req.dimension {
                return Err(ApiError::BadRequest(format!(
                    "Index '{}' already exists with dimension {} (requested: {}). Please delete the existing index or use the existing configuration.",
                    name, existing_dim, req.dimension
                )));
            }

            if existing_metric != metric {
                return Err(ApiError::BadRequest(format!(
                    "Index '{}' already exists with metric {:?} (requested: {:?}). Please delete the existing index or use the existing configuration.",
                    name, existing_metric, metric
                )));
            }

            tracing::info!(
                "Opened existing vector index '{}' at {} ({}D, {:?})",
                name,
                vector_path,
                existing_dim,
                existing_metric
            );
            existing_index
        }
        Err(_) => {
            tracing::info!(
                "Creating new vector index '{}' at {} ({}D, {:?})",
                name,
                vector_path,
                req.dimension,
                metric
            );
            PersistentVectorIndex::create(&vector_path, config).await?
        }
    };

    // Store in state
    let mut vector_indexes = state.vector_indexes.write().await;
    vector_indexes.insert(name.clone(), index);

    // Initialize vector ID counter for this index
    let mut next_ids = state.next_vector_ids.write().await;
    next_ids.entry(name.clone()).or_insert(1);

    Ok(Json(InitVectorIndexResponse {
        message: format!("Vector index '{}' initialized successfully", name),
        dimension: req.dimension,
        metric: req.metric,
    }))
}

/// POST /api/v1/indexes/{name}/vectors - Insert a vector into a named index
pub async fn insert_vector_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<InsertVectorRequest>,
) -> Result<Json<InsertVectorResponse>, ApiError> {
    let mut vector_indexes = state.vector_indexes.write().await;
    let index = vector_indexes
        .get_mut(&name)
        .ok_or_else(|| ApiError::IndexNotFound(name.clone()))?;

    // Generate new ID
    let mut next_ids = state.next_vector_ids.write().await;
    let id = next_ids.entry(name.clone()).or_insert(1);
    let current_id = *id;
    *id += 1;

    let vector = Vector::new(req.vector);

    if let Some(metadata) = req.metadata {
        index
            .insert_with_metadata(current_id, vector, Some(metadata))
            .await?;
    } else {
        index.insert(current_id, vector).await?;
    }

    Ok(Json(InsertVectorResponse {
        id: current_id,
        message: format!("Vector inserted into index '{}' successfully", name),
    }))
}

/// POST /api/v1/indexes/{name}/vectors/search - Search for similar vectors in a named index
pub async fn search_vectors_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<SearchVectorsRequest>,
) -> Result<Json<SearchVectorsResponse>, ApiError> {
    let vector_indexes = state.vector_indexes.read().await;
    let index = vector_indexes
        .get(&name)
        .ok_or_else(|| ApiError::IndexNotFound(name.clone()))?;

    let query_vector = Vector::new(req.vector);

    let results = index.search(&query_vector, req.k).await?;

    let mut response_results = Vec::new();
    for result in results {
        if let Some(vector) = index.get(result.id) {
            response_results.push(VectorSearchResult {
                id: result.id,
                distance: result.score,
                vector: vector.data.clone(),
                metadata: result.metadata,
            });
        }
    }

    Ok(Json(SearchVectorsResponse {
        results: response_results,
    }))
}

/// GET /api/v1/indexes/{name}/vectors/{id} - Retrieve a vector by ID from a named index
pub async fn get_vector_handler(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, VectorId)>,
) -> Result<Json<GetVectorResponse>, ApiError> {
    let vector_indexes = state.vector_indexes.read().await;
    let index = vector_indexes
        .get(&name)
        .ok_or_else(|| ApiError::IndexNotFound(name.clone()))?;

    let vector = index.get(id).ok_or(ApiError::VectorNotFound(id))?;

    // Extract metadata from SearchResult if we need it separately
    // For now, metadata is None since get() doesn't return it
    let metadata = None;

    Ok(Json(GetVectorResponse {
        id,
        vector: vector.data.clone(),
        metadata,
    }))
}

/// DELETE /api/v1/indexes/{name}/vectors/{id} - Delete a vector from a named index
pub async fn delete_vector_handler(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, VectorId)>,
) -> Result<Json<DeleteVectorResponse>, ApiError> {
    let mut vector_indexes = state.vector_indexes.write().await;
    let index = vector_indexes
        .get_mut(&name)
        .ok_or_else(|| ApiError::IndexNotFound(name.clone()))?;

    index.delete(id).await?;

    Ok(Json(DeleteVectorResponse {
        id,
        message: format!("Vector deleted from index '{}' successfully", name),
    }))
}

/// Index information response
#[derive(Debug, Serialize)]
pub struct IndexInfo {
    pub name: String,
    pub dimension: usize,
    pub metric: String,
    pub num_vectors: usize,
}

/// Response for listing indexes
#[derive(Debug, Serialize)]
pub struct ListIndexesResponse {
    pub indexes: Vec<IndexInfo>,
}

/// GET /api/v1/indexes - List all vector indexes
pub async fn list_indexes_handler(State(state): State<AppState>) -> Json<ListIndexesResponse> {
    let vector_indexes = state.vector_indexes.read().await;

    let mut indexes = Vec::new();
    for (name, index) in vector_indexes.iter() {
        indexes.push(IndexInfo {
            name: name.clone(),
            dimension: index.dimension(),
            metric: format!("{:?}", index.metric()),
            num_vectors: index.len(),
        });
    }

    Json(ListIndexesResponse { indexes })
}

/// Response for deleting an index
#[derive(Debug, Serialize)]
pub struct DeleteIndexResponse {
    pub name: String,
    pub message: String,
}

/// DELETE /api/v1/indexes/{name} - Delete a vector index
pub async fn delete_index_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DeleteIndexResponse>, ApiError> {
    let mut vector_indexes = state.vector_indexes.write().await;
    let mut next_ids = state.next_vector_ids.write().await;

    // Remove from state
    vector_indexes
        .remove(&name)
        .ok_or_else(|| ApiError::IndexNotFound(name.clone()))?;
    next_ids.remove(&name);

    // Delete from filesystem
    let index_path = format!("{}/indexes/{}", state.storage_path.as_str(), name);
    if let Err(e) = std::fs::remove_dir_all(&index_path) {
        tracing::warn!("Failed to delete index directory {}: {}", index_path, e);
    }

    Ok(Json(DeleteIndexResponse {
        name: name.clone(),
        message: format!("Index '{}' deleted successfully", name),
    }))
}

/// Create the API router with all endpoints
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Key-value endpoints
        .route("/api/v1/kv/{key}", get(get_handler))
        .route("/api/v1/kv/{key}", post(put_handler))
        .route("/api/v1/kv/{key}", delete(delete_handler))
        // Health and stats
        .route("/api/v1/health", get(health_handler))
        .route("/api/v1/stats", get(stats_handler))
        // Vector search endpoints (named indexes)
        .route("/api/v1/indexes", get(list_indexes_handler))
        .route("/api/v1/indexes/{name}", post(init_vector_index_handler))
        .route("/api/v1/indexes/{name}", delete(delete_index_handler))
        .route(
            "/api/v1/indexes/{name}/vectors",
            post(insert_vector_handler),
        )
        .route(
            "/api/v1/indexes/{name}/vectors/search",
            post(search_vectors_handler),
        )
        .route(
            "/api/v1/indexes/{name}/vectors/{id}",
            get(get_vector_handler),
        )
        .route(
            "/api/v1/indexes/{name}/vectors/{id}",
            delete(delete_vector_handler),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quartz_storage::StorageConfig;
    use tempfile::TempDir;

    async fn create_test_app() -> (Router, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        let storage = StorageEngine::with_config(path, StorageConfig::default()).unwrap();

        let state = AppState {
            storage: Arc::new(storage),
            storage_path: Arc::new(path.to_string()),
            vector_indexes: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            next_vector_ids: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        };

        let app = create_router(state);
        (app, temp_dir)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let (app, _temp) = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
