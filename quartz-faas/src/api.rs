//! API module for QuartzDB FaaS
//!
//! Defines request/response types and handlers

use serde::{Deserialize, Serialize};

/// Request body for KV PUT operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct PutRequest {
    pub key: String,
    pub value: String,
}

/// Response body for KV GET operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetResponse {
    pub key: String,
    pub value: Option<String>,
}

/// Request body for single-vector insert (`POST /api/vector/insert`).
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorInsertRequest {
    pub id: u64,
    pub vector: Vec<f32>,
    pub metadata: Option<serde_json::Value>,
}

/// Request body for vector search (`POST /api/vector/search`).
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorSearchRequest {
    pub query: Vec<f32>,
    pub k: usize,
    pub metric: Option<String>,
}

/// Response body for vector search containing ranked results.
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorSearchResponse {
    pub results: Vec<VectorSearchResult>,
}

/// A single search result entry with score and optional metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: u64,
    pub score: f32,
    pub metadata: Option<serde_json::Value>,
}

/// Generic envelope for all API JSON responses.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Build a success response wrapping `data`.
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            data: Some(data),
            error: None,
        }
    }

    /// Build an error response with a human-readable message.
    pub fn error(error: String) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            error: Some(error),
        }
    }
}
