//! Vector search API integration tests

use axum::body::Body;
use axum::http::{Request, StatusCode};
use quartz_server::{create_router, AppState};
use quartz_storage::{StorageConfig, StorageEngine};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

/// Helper to create a test app with temporary storage
async fn create_test_app() -> (axum::Router, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_str().unwrap();
    let storage = StorageEngine::with_config(path, StorageConfig::default()).unwrap();

    let state = AppState {
        storage: Arc::new(storage),
        storage_path: Arc::new(path.to_string()),
        vector_indexes: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        next_vector_ids: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    let app = create_router(state);
    (app, temp_dir)
}

/// Helper to make a request and parse JSON response
async fn make_request<T: serde::de::DeserializeOwned>(
    app: axum::Router,
    request: Request<Body>,
) -> (StatusCode, T) {
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: T = serde_json::from_slice(&body).unwrap();
    (status, json)
}

#[tokio::test]
async fn test_vector_index_not_initialized() {
    let (app, _temp) = create_test_app().await;

    // Try to insert into non-existent index
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/indexes/test_index/vectors")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "vector": [1.0, 2.0, 3.0]
            })
            .to_string(),
        ))
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app, request).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "index_not_found");
}

#[tokio::test]
async fn test_initialize_vector_index() {
    let (app, _temp) = create_test_app().await;

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/indexes/test_index")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "dimension": 3,
                "metric": "cosine"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["dimension"], 3);
    assert_eq!(body["metric"], "cosine");
    assert!(body["message"].as_str().unwrap().contains("initialized"));
}

#[tokio::test]
async fn test_insert_and_retrieve_vector() {
    let (mut app, _temp) = create_test_app().await;

    // Initialize index
    let init_request = Request::builder()
        .method("POST")
        .uri("/api/v1/indexes/test_index")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "dimension": 3,
                "metric": "euclidean"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, _): (_, serde_json::Value) = make_request(app, init_request).await;
    assert_eq!(status, StatusCode::OK);

    // Need to recreate app for next request
    let (mut app, _temp) = create_test_app().await;

    // Re-initialize
    let init_request = Request::builder()
        .method("POST")
        .uri("/api/v1/indexes/test_index")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "dimension": 3,
                "metric": "euclidean"
            })
            .to_string(),
        ))
        .unwrap();
    let _ = app.clone().oneshot(init_request).await.unwrap();

    // Insert vector
    let insert_request = Request::builder()
        .method("POST")
        .uri("/api/v1/indexes/test_index/vectors")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "vector": [1.0, 2.0, 3.0],
                "metadata": "test vector"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(insert_request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let insert_body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status, StatusCode::OK);
    let vector_id = insert_body["id"].as_u64().unwrap();
    assert_eq!(vector_id, 1);

    // Retrieve vector
    let get_request = Request::builder()
        .uri(format!("/api/v1/indexes/test_index/vectors/{}", vector_id))
        .body(Body::empty())
        .unwrap();

    let (status, get_body): (_, serde_json::Value) = make_request(app, get_request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(get_body["id"], vector_id);
    assert_eq!(get_body["vector"], json!([1.0, 2.0, 3.0]));
}

#[tokio::test]
async fn test_search_vectors() {
    let (mut app, _temp) = create_test_app().await;

    // Initialize index
    let init_request = Request::builder()
        .method("POST")
        .uri("/api/v1/indexes/test_index")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "dimension": 3,
                "metric": "cosine"
            })
            .to_string(),
        ))
        .unwrap();
    let _ = app.clone().oneshot(init_request).await.unwrap();

    // Insert multiple vectors
    for vector in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]] {
        let insert_request = Request::builder()
            .method("POST")
            .uri("/api/v1/indexes/test_index/vectors")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "vector": vector
                })
                .to_string(),
            ))
            .unwrap();
        let _ = app.clone().oneshot(insert_request).await.unwrap();
    }

    // Search for similar vectors
    let search_request = Request::builder()
        .method("POST")
        .uri("/api/v1/indexes/test_index/vectors/search")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "vector": [0.9, 0.1, 0.0],
                "k": 2
            })
            .to_string(),
        ))
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app, search_request).await;

    assert_eq!(status, StatusCode::OK);
    let results = body["results"].as_array().unwrap();
    assert!(results.len() <= 2);
    // The search should find vectors similar to [0.9, 0.1, 0.0]
    assert!(results.len() > 0);
}

#[tokio::test]
async fn test_delete_vector() {
    let (mut app, _temp) = create_test_app().await;

    // Initialize index
    let init_request = Request::builder()
        .method("POST")
        .uri("/api/v1/indexes/test_index")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "dimension": 2,
                "metric": "euclidean"
            })
            .to_string(),
        ))
        .unwrap();
    let _ = app.clone().oneshot(init_request).await.unwrap();

    // Insert vector
    let insert_request = Request::builder()
        .method("POST")
        .uri("/api/v1/indexes/test_index/vectors")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "vector": [1.0, 2.0]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(insert_request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let insert_body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let vector_id = insert_body["id"].as_u64().unwrap();

    // Delete vector
    let delete_request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/indexes/test_index/vectors/{}", vector_id))
        .body(Body::empty())
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app.clone(), delete_request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["message"].as_str().unwrap().contains("deleted"));

    // Try to retrieve deleted vector
    let get_request = Request::builder()
        .uri(format!("/api/v1/indexes/test_index/vectors/{}", vector_id))
        .body(Body::empty())
        .unwrap();

    let (status, _): (_, serde_json::Value) = make_request(app, get_request).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_distance_metric() {
    let (app, _temp) = create_test_app().await;

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/indexes/test_index")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "dimension": 3,
                "metric": "invalid_metric"
            })
            .to_string(),
        ))
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app, request).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"].as_str().unwrap().contains("Invalid distance metric"));
}
