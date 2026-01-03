use worker::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::cell::RefCell;

// ============================================================================
// Storage Durable Object - Key-Value Store with Persistence
// ============================================================================

#[durable_object]
pub struct StorageObject {
    state: State,
    // In-memory cache for faster reads (RefCell for interior mutability)
    cache: RefCell<HashMap<String, String>>,
    initialized: RefCell<bool>,
}

impl DurableObject for StorageObject {
    fn new(state: State, _env: Env) -> Self {
        Self {
            state,
            cache: RefCell::new(HashMap::new()),
            initialized: RefCell::new(false),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        // Initialize cache on first request
        if !*self.initialized.borrow() {
            if let Err(e) = self.init_cache().await {
                console_error!("Failed to initialize cache: {:?}", e);
            }
            *self.initialized.borrow_mut() = true;
        }

        let url = req.url()?;
        let path = url.path();
        let method = req.method();

        match (method, path) {
            (Method::Post, "/put") => self.handle_put(req).await,
            (Method::Get, path) if path.starts_with("/get/") => {
                let key = path.strip_prefix("/get/").unwrap_or("");
                self.handle_get(key).await
            }
            (Method::Delete, path) if path.starts_with("/delete/") => {
                let key = path.strip_prefix("/delete/").unwrap_or("");
                self.handle_delete(key).await
            }
            (Method::Get, "/list") => self.handle_list().await,
            (Method::Get, "/health") => Response::ok("StorageObject healthy"),
            _ => Response::error("Not Found", 404),
        }
    }
}

impl StorageObject {
    async fn init_cache(&self) -> Result<()> {
        // Load all keys from durable storage into cache
        let keys = self.state.storage().list().await?;
        for key_result in keys.keys() {
            if let Ok(key_value) = key_result {
                if let Some(key) = key_value.as_string() {
                    if let Some(value) = self.state.storage().get::<String>(&key).await? {
                        self.cache.borrow_mut().insert(key, value);
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_put(&self, mut req: Request) -> Result<Response> {
        #[derive(Deserialize)]
        struct PutRequest {
            key: String,
            value: String,
        }

        let body = match req.json::<PutRequest>().await {
            Ok(b) => b,
            Err(_) => return Response::error("Invalid JSON body", 400),
        };

        if body.key.is_empty() {
            return Response::error("Key cannot be empty", 400);
        }

        // Update cache
        self.cache.borrow_mut().insert(body.key.clone(), body.value.clone());

        // Persist to durable storage
        self.state
            .storage()
            .put(&body.key, body.value.clone())
            .await?;

        Response::from_json(&serde_json::json!({
            "success": true,
            "key": body.key,
            "message": "Value stored successfully"
        }))
    }

    async fn handle_get(&self, key: &str) -> Result<Response> {
        if key.is_empty() {
            return Response::error("Key cannot be empty", 400);
        }

        // Try cache first for fast reads
        if let Some(value) = self.cache.borrow().get(key).cloned() {
            return Response::from_json(&serde_json::json!({
                "success": true,
                "key": key,
                "value": value,
                "source": "cache"
            }));
        }

        // Fallback to durable storage
        match self.state.storage().get::<String>(key).await {
            Ok(Some(value)) => Response::from_json(&serde_json::json!({
                "success": true,
                "key": key,
                "value": value,
                "source": "storage"
            })),
            _ => Response::error("Key not found", 404),
        }
    }

    async fn handle_delete(&self, key: &str) -> Result<Response> {
        if key.is_empty() {
            return Response::error("Key cannot be empty", 400);
        }

        // Remove from cache
        self.cache.borrow_mut().remove(key);

        // Remove from durable storage
        self.state.storage().delete(key).await?;

        Response::from_json(&serde_json::json!({
            "success": true,
            "key": key,
            "message": "Value deleted successfully"
        }))
    }

    async fn handle_list(&self) -> Result<Response> {
        let keys: Vec<String> = self.cache.borrow().keys().cloned().collect();
        Response::from_json(&serde_json::json!({
            "success": true,
            "count": keys.len(),
            "keys": keys
        }))
    }
}

// ============================================================================
// Vector Index Durable Object - Vector Similarity Search
// ============================================================================

#[derive(Serialize, Deserialize, Clone)]
struct VectorEntry {
    id: String,
    vector: Vec<f32>,
    metadata: Option<serde_json::Value>,
}

#[durable_object]
pub struct VectorIndexObject {
    state: State,
    vectors: RefCell<HashMap<String, VectorEntry>>,
    initialized: RefCell<bool>,
}

impl DurableObject for VectorIndexObject {
    fn new(state: State, _env: Env) -> Self {
        Self {
            state,
            vectors: RefCell::new(HashMap::new()),
            initialized: RefCell::new(false),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        // Initialize vectors on first request
        if !*self.initialized.borrow() {
            if let Err(e) = self.init_vectors().await {
                console_error!("Failed to initialize vectors: {:?}", e);
            }
            *self.initialized.borrow_mut() = true;
        }

        let url = req.url()?;
        let path = url.path();
        let method = req.method();

        match (method, path) {
            (Method::Post, "/insert") => self.handle_insert(req).await,
            (Method::Post, "/search") => self.handle_search(req).await,
            (Method::Delete, path) if path.starts_with("/delete/") => {
                let id = path.strip_prefix("/delete/").unwrap_or("");
                self.handle_delete(id).await
            }
            (Method::Get, "/stats") => self.handle_stats().await,
            (Method::Get, "/health") => Response::ok("VectorIndexObject healthy"),
            _ => Response::error("Not Found", 404),
        }
    }
}

impl VectorIndexObject {
    async fn init_vectors(&self) -> Result<()> {
        // Load all vectors from durable storage
        let keys = self.state.storage().list().await?;
        for key_result in keys.keys() {
            if let Ok(key_value) = key_result {
                if let Some(key) = key_value.as_string() {
                    if let Some(entry) = self.state.storage().get::<VectorEntry>(&key).await? {
                        self.vectors.borrow_mut().insert(key, entry);
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_insert(&self, mut req: Request) -> Result<Response> {
        #[derive(Deserialize)]
        struct InsertRequest {
            id: String,
            vector: Vec<f32>,
            metadata: Option<serde_json::Value>,
        }

        let body = match req.json::<InsertRequest>().await {
            Ok(b) => b,
            Err(_) => return Response::error("Invalid JSON body", 400),
        };

        if body.id.is_empty() {
            return Response::error("ID cannot be empty", 400);
        }

        if body.vector.is_empty() {
            return Response::error("Vector cannot be empty", 400);
        }

        let entry = VectorEntry {
            id: body.id.clone(),
            vector: body.vector,
            metadata: body.metadata,
        };

        // Store in memory
        self.vectors.borrow_mut().insert(body.id.clone(), entry.clone());

        // Persist to durable storage
        self.state.storage().put(&body.id, entry).await?;

        Response::from_json(&serde_json::json!({
            "success": true,
            "id": body.id,
            "message": "Vector inserted successfully"
        }))
    }

    async fn handle_search(&self, mut req: Request) -> Result<Response> {
        #[derive(Deserialize)]
        struct SearchRequest {
            vector: Vec<f32>,
            k: Option<usize>,
        }

        let body = match req.json::<SearchRequest>().await {
            Ok(b) => b,
            Err(_) => return Response::error("Invalid JSON body", 400),
        };

        if body.vector.is_empty() {
            return Response::error("Query vector cannot be empty", 400);
        }

        let k = body.k.unwrap_or(10).min(100); // Max 100 results

        // Compute cosine similarity for all vectors
        let vectors = self.vectors.borrow();
        let mut results: Vec<_> = vectors
            .values()
            .map(|entry| {
                let similarity = cosine_similarity(&body.vector, &entry.vector);
                (entry.id.clone(), similarity, entry.metadata.clone())
            })
            .collect();

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top k
        let top_k: Vec<_> = results
            .into_iter()
            .take(k)
            .map(|(id, score, metadata)| {
                serde_json::json!({
                    "id": id,
                    "score": score,
                    "metadata": metadata
                })
            })
            .collect();

        Response::from_json(&serde_json::json!({
            "success": true,
            "results": top_k,
            "count": top_k.len()
        }))
    }

    async fn handle_delete(&self, id: &str) -> Result<Response> {
        if id.is_empty() {
            return Response::error("ID cannot be empty", 400);
        }

        // Remove from memory
        self.vectors.borrow_mut().remove(id);

        // Remove from durable storage
        self.state.storage().delete(id).await?;

        Response::from_json(&serde_json::json!({
            "success": true,
            "id": id,
            "message": "Vector deleted successfully"
        }))
    }

    async fn handle_stats(&self) -> Result<Response> {
        let vectors = self.vectors.borrow();
        let count = vectors.len();
        let dimensions = vectors
            .values()
            .next()
            .map(|v| v.vector.len())
            .unwrap_or(0);

        Response::from_json(&serde_json::json!({
            "success": true,
            "count": count,
            "dimensions": dimensions
        }))
    }
}

// ============================================================================
// Vector Math Utilities
// ============================================================================

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v1, &v2);
        assert!((sim - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let v1 = vec![1.0, 0.0];
        let v2 = vec![0.0, 1.0];
        let sim = cosine_similarity(&v1, &v2);
        assert!((sim - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![-1.0, -2.0, -3.0];
        let sim = cosine_similarity(&v1, &v2);
        assert!((sim + 1.0).abs() < 0.0001);
    }
}
