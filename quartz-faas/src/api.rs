//! API module for QuartzDB FaaS
//!
//! Defines request/response types and handlers

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PutRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetResponse {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorInsertRequest {
    pub id: u64,
    pub vector: Vec<f32>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorSearchRequest {
    pub query: Vec<f32>,
    pub k: usize,
    pub metric: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorSearchResponse {
    pub results: Vec<VectorSearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: u64,
    pub score: f32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            error: Some(error),
        }
    }
}
