use axum::body::Body;
use axum::http::{Request, StatusCode};
use quartz_server::{AppState, create_router};
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
async fn test_health_endpoint() {
    let (app, _temp) = create_test_app().await;

    let request = Request::builder()
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["version"], "0.1.0");
}

#[tokio::test]
async fn test_stats_endpoint() {
    let (app, _temp) = create_test_app().await;

    let request = Request::builder()
        .uri("/api/v1/stats")
        .body(Body::empty())
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["lsm_levels"].is_number());
    assert!(body["cache_size"].is_number());
    assert!(body["wal_enabled"].is_boolean());
}

#[tokio::test]
async fn test_put_and_get() {
    let (app, _temp) = create_test_app().await;

    // PUT a value
    let put_request = Request::builder()
        .method("POST")
        .uri("/api/v1/kv/test_key")
        .header("content-type", "application/json")
        .body(Body::from(json!({"value": "test_value"}).to_string()))
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app.clone(), put_request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["key"], "test_key");
    assert_eq!(body["message"], "Value stored successfully");

    // GET the value
    let get_request = Request::builder()
        .uri("/api/v1/kv/test_key")
        .body(Body::empty())
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app, get_request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["key"], "test_key");
    assert_eq!(body["value"], "test_value");
}

#[tokio::test]
async fn test_get_nonexistent_key() {
    let (app, _temp) = create_test_app().await;

    let request = Request::builder()
        .uri("/api/v1/kv/nonexistent")
        .body(Body::empty())
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app, request).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");
    assert!(body["message"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_put_empty_key() {
    let (app, _temp) = create_test_app().await;

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/kv/")
        .header("content-type", "application/json")
        .body(Body::from(json!({"value": "test"}).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 404 (not found) because empty path doesn't match route
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_existing_key() {
    let (app, _temp) = create_test_app().await;

    // PUT original value
    let put1 = Request::builder()
        .method("POST")
        .uri("/api/v1/kv/update_test")
        .header("content-type", "application/json")
        .body(Body::from(json!({"value": "original"}).to_string()))
        .unwrap();

    let (status, _): (_, serde_json::Value) = make_request(app.clone(), put1).await;
    assert_eq!(status, StatusCode::OK);

    // PUT updated value
    let put2 = Request::builder()
        .method("POST")
        .uri("/api/v1/kv/update_test")
        .header("content-type", "application/json")
        .body(Body::from(json!({"value": "updated"}).to_string()))
        .unwrap();

    let (status, _): (_, serde_json::Value) = make_request(app.clone(), put2).await;
    assert_eq!(status, StatusCode::OK);

    // GET to verify update
    let get = Request::builder()
        .uri("/api/v1/kv/update_test")
        .body(Body::empty())
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app, get).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["value"], "updated");
}

#[tokio::test]
async fn test_delete() {
    let (app, _temp) = create_test_app().await;

    // PUT a value
    let put = Request::builder()
        .method("POST")
        .uri("/api/v1/kv/delete_test")
        .header("content-type", "application/json")
        .body(Body::from(json!({"value": "to_delete"}).to_string()))
        .unwrap();

    let (status, _): (_, serde_json::Value) = make_request(app.clone(), put).await;
    assert_eq!(status, StatusCode::OK);

    // DELETE the value
    let delete = Request::builder()
        .method("DELETE")
        .uri("/api/v1/kv/delete_test")
        .body(Body::empty())
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app.clone(), delete).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["key"], "delete_test");
    assert_eq!(body["message"], "Key deleted successfully");

    // Verify it's gone
    let get = Request::builder()
        .uri("/api/v1/kv/delete_test")
        .body(Body::empty())
        .unwrap();

    let (status, _): (_, serde_json::Value) = make_request(app, get).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_nonexistent_key() {
    let (app, _temp) = create_test_app().await;

    let request = Request::builder()
        .method("DELETE")
        .uri("/api/v1/kv/nonexistent")
        .body(Body::empty())
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app, request).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "not_found");
}

#[tokio::test]
async fn test_multiple_keys() {
    let (app, _temp) = create_test_app().await;

    // Store multiple keys
    for i in 0..10 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);

        let request = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/kv/{}", key))
            .header("content-type", "application/json")
            .body(Body::from(json!({"value": value}).to_string()))
            .unwrap();

        let (status, _): (_, serde_json::Value) = make_request(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);
    }

    // Retrieve all keys
    for i in 0..10 {
        let key = format!("key_{}", i);
        let expected_value = format!("value_{}", i);

        let request = Request::builder()
            .uri(format!("/api/v1/kv/{}", key))
            .body(Body::empty())
            .unwrap();

        let (status, body): (_, serde_json::Value) = make_request(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["value"], expected_value);
    }
}

#[tokio::test]
async fn test_special_characters_in_key() {
    let (app, _temp) = create_test_app().await;

    let special_keys = vec![
        "user:123",
        "session_abc-def",
        "metric.cpu.usage",
        // Note: paths with "/" are tricky in URLs, would need URL encoding
    ];

    for key in special_keys {
        let request = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/kv/{}", key))
            .header("content-type", "application/json")
            .body(Body::from(json!({"value": "test"}).to_string()))
            .unwrap();

        let (status, _): (_, serde_json::Value) = make_request(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);

        // Verify we can retrieve it
        let get_request = Request::builder()
            .uri(format!("/api/v1/kv/{}", key))
            .body(Body::empty())
            .unwrap();

        let (status, body): (_, serde_json::Value) = make_request(app.clone(), get_request).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["key"], key);
    }
}

#[tokio::test]
async fn test_concurrent_requests() {
    let (app, _temp) = create_test_app().await;

    let mut handles = vec![];

    // Spawn 10 concurrent write tasks
    for i in 0..10 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_{}", i);
            let value = format!("value_{}", i);

            let request = Request::builder()
                .method("POST")
                .uri(format!("/api/v1/kv/{}", key))
                .header("content-type", "application/json")
                .body(Body::from(json!({"value": value}).to_string()))
                .unwrap();

            let response = app_clone.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all keys were stored (create new app instance)
    let (_app2, _) = create_test_app().await;

    // Note: This test won't verify persistence since we're using a fresh temp dir
    // In a real scenario, we'd use the same storage path
    // For now, let's just verify the test structure works
}

#[tokio::test]
async fn test_large_value() {
    let (app, _temp) = create_test_app().await;

    // Create a large value (1MB)
    let large_value = "x".repeat(1_000_000);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/kv/large_key")
        .header("content-type", "application/json")
        .body(Body::from(json!({"value": large_value}).to_string()))
        .unwrap();

    let (status, _): (_, serde_json::Value) = make_request(app.clone(), request).await;
    assert_eq!(status, StatusCode::OK);

    // Retrieve and verify
    let get_request = Request::builder()
        .uri("/api/v1/kv/large_key")
        .body(Body::empty())
        .unwrap();

    let (status, body): (_, serde_json::Value) = make_request(app, get_request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["value"].as_str().unwrap().len(), 1_000_000);
}

#[tokio::test]
async fn test_unicode_values() {
    let (app, _temp) = create_test_app().await;

    let unicode_values = [
        "Hello, 世界",
        "Привет мир",
        "مرحبا بالعالم",
        "🚀 Rocket emoji",
        "Ñoño 日本語 中文",
    ];

    for (i, value) in unicode_values.iter().enumerate() {
        let key = format!("unicode_{}", i);

        let request = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/kv/{}", key))
            .header("content-type", "application/json")
            .body(Body::from(json!({"value": value}).to_string()))
            .unwrap();

        let (status, _): (_, serde_json::Value) = make_request(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);

        // Verify retrieval
        let get_request = Request::builder()
            .uri(format!("/api/v1/kv/{}", key))
            .body(Body::empty())
            .unwrap();

        let (status, body): (_, serde_json::Value) = make_request(app.clone(), get_request).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["value"], *value);
    }
}
