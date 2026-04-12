//! Input validation for QuartzDB
//!
//! # Validation Rules
//!
//! ## Vector Operations
//! - ID: 1-256 characters, alphanumeric + underscore + hyphen
//! - Vector: 1-4096 dimensions, all finite f32 values
//! - Metadata: <32KB JSON
/// - k (search): 1-100
//!
//! ## KV Operations
//! - Key: 1-512 characters
//! - Value: <10MB
//!
//! All validations return descriptive errors for API responses

use serde_json::Value;
use worker::*;

/// Maximum allowed dimensions for vectors
pub const MAX_VECTOR_DIMENSIONS: usize = 4096;

/// Maximum ID length
pub const MAX_ID_LENGTH: usize = 256;

/// Maximum metadata size (bytes)
pub const MAX_METADATA_SIZE: usize = 32 * 1024; // 32KB

/// Maximum k value for search
pub const MAX_SEARCH_K: usize = 100;

/// Maximum key length for KV
pub const MAX_KEY_LENGTH: usize = 512;

/// Validate vector ID
///
/// Rules:
/// - Non-empty
/// - Max 256 characters
/// - Alphanumeric, underscore, hyphen only
pub fn validate_vector_id(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(Error::RustError("Vector ID cannot be empty".to_string()));
    }
    
    if id.len() > MAX_ID_LENGTH {
        return Err(Error::RustError(
            format!("Vector ID too long (max {} chars)", MAX_ID_LENGTH)
        ));
    }
    
    // Allow alphanumeric, underscore, hyphen
    if !id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(Error::RustError(
            "Vector ID must be alphanumeric (plus _ and -)".to_string()
        ));
    }
    
    Ok(())
}

/// Validate vector dimensions
///
/// Rules:
/// - Non-empty
/// - Max 4096 dimensions
/// - All values must be finite (no NaN, no infinity)
pub fn validate_vector(vector: &[f32]) -> Result<()> {
    if vector.is_empty() {
        return Err(Error::RustError("Vector cannot be empty".to_string()));
    }
    
    if vector.len() > MAX_VECTOR_DIMENSIONS {
        return Err(Error::RustError(
            format!("Vector too large (max {} dimensions)", MAX_VECTOR_DIMENSIONS)
        ));
    }
    
    // Check for NaN or infinity
    for (i, &value) in vector.iter().enumerate() {
        if !value.is_finite() {
            return Err(Error::RustError(
                format!("Vector contains invalid value at index {}: {}", i, value)
            ));
        }
    }
    
    Ok(())
}

/// Validate metadata
///
/// Rules:
/// - Optional (can be null)
/// - Must be valid JSON object
/// - Max 32KB when serialized
pub fn validate_metadata(metadata: &Option<Value>) -> Result<()> {
    if let Some(meta) = metadata {
        // Must be an object or null
        if !meta.is_object() && !meta.is_null() {
            return Err(Error::RustError(
                "Metadata must be a JSON object".to_string()
            ));
        }
        
        // Check size
        let serialized = serde_json::to_string(meta)
            .map_err(|e| Error::RustError(format!("Invalid metadata JSON: {}", e)))?;
        
        if serialized.len() > MAX_METADATA_SIZE {
            return Err(Error::RustError(
                format!("Metadata too large (max {} bytes)", MAX_METADATA_SIZE)
            ));
        }
    }
    
    Ok(())
}

/// Validate search k parameter
///
/// Rules:
/// - Must be >= 1
/// - Must be <= 1000
pub fn validate_search_k(k: usize) -> Result<()> {
    if k == 0 {
        return Err(Error::RustError("k must be at least 1".to_string()));
    }
    
    if k > MAX_SEARCH_K {
        return Err(Error::RustError(
            format!("k too large (max {})", MAX_SEARCH_K)
        ));
    }
    
    Ok(())
}

/// Validate KV key
///
/// Rules:
/// - Non-empty
/// - Max 512 characters
pub fn validate_kv_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(Error::RustError("Key cannot be empty".to_string()));
    }
    
    if key.len() > MAX_KEY_LENGTH {
        return Err(Error::RustError(
            format!("Key too long (max {} chars)", MAX_KEY_LENGTH)
        ));
    }
    
    Ok(())
}

/// Validate insert request body
pub fn validate_insert_request(body: &Value) -> Result<()> {
    // Extract fields
    let id = body.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::RustError("Missing or invalid 'id' field".to_string()))?;
    
    let vector_json = body.get("vector")
        .ok_or_else(|| Error::RustError("Missing 'vector' field".to_string()))?;
    
    let vector: Vec<f32> = serde_json::from_value(vector_json.clone())
        .map_err(|e| Error::RustError(format!("Invalid vector format: {}", e)))?;
    
    let metadata = body.get("metadata").cloned();
    
    // Validate each field
    validate_vector_id(id)?;
    validate_vector(&vector)?;
    validate_metadata(&metadata)?;
    
    Ok(())
}

/// Validate search request body
pub fn validate_search_request(body: &Value) -> Result<()> {
    // Extract fields
    let vector_json = body.get("vector")
        .ok_or_else(|| Error::RustError("Missing 'vector' field".to_string()))?;
    
    let vector: Vec<f32> = serde_json::from_value(vector_json.clone())
        .map_err(|e| Error::RustError(format!("Invalid vector format: {}", e)))?;
    
    let k = body.get("k")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    
    // Validate
    validate_vector(&vector)?;
    validate_search_k(k)?;
    
    Ok(())
}

/// Maximum vectors in a single batch operation
pub const MAX_BATCH_SIZE: usize = 100;

/// Validate batch insert request body
pub fn validate_batch_insert_request(body: &Value) -> Result<()> {
    let vectors = body.get("vectors")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Error::RustError("Missing or invalid 'vectors' array".to_string()))?;
    
    if vectors.is_empty() {
        return Err(Error::RustError("Vectors array cannot be empty".to_string()));
    }
    
    if vectors.len() > MAX_BATCH_SIZE {
        return Err(Error::RustError(
            format!("Batch too large (max {} vectors)", MAX_BATCH_SIZE)
        ));
    }
    
    // Validate each vector in batch
    for (i, item) in vectors.iter().enumerate() {
        let id = item.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::RustError(format!("Missing 'id' at index {}", i)))?;
        
        let vector_json = item.get("vector")
            .ok_or_else(|| Error::RustError(format!("Missing 'vector' at index {}", i)))?;
        
        let vector: Vec<f32> = serde_json::from_value(vector_json.clone())
            .map_err(|e| Error::RustError(format!("Invalid vector at index {}: {}", i, e)))?;
        
        let metadata = item.get("metadata").cloned();
        
        validate_vector_id(id)?;
        validate_vector(&vector)?;
        validate_metadata(&metadata)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_vector_id() {
        assert!(validate_vector_id("valid_id-123").is_ok());
        assert!(validate_vector_id("").is_err());
        assert!(validate_vector_id(&"a".repeat(300)).is_err());
        assert!(validate_vector_id("invalid id!").is_err());
    }

    #[test]
    fn test_validate_vector() {
        assert!(validate_vector(&[0.1, 0.2, 0.3]).is_ok());
        assert!(validate_vector(&[]).is_err());
        assert!(validate_vector(&vec![0.0; 5000]).is_err());
        assert!(validate_vector(&[f32::NAN]).is_err());
        assert!(validate_vector(&[f32::INFINITY]).is_err());
    }

    #[test]
    fn test_validate_search_k() {
        assert!(validate_search_k(10).is_ok());
        assert!(validate_search_k(0).is_err());
        assert!(validate_search_k(2000).is_err());
    }

    #[test]
    fn test_validate_metadata_valid() {
        assert!(validate_metadata(&None).is_ok());
        assert!(validate_metadata(&Some(json!(null))).is_ok());
        assert!(validate_metadata(&Some(json!({"key": "value"}))).is_ok());
        assert!(validate_metadata(&Some(json!({}))).is_ok());
    }

    #[test]
    fn test_validate_metadata_invalid_type() {
        assert!(validate_metadata(&Some(json!("string"))).is_err());
        assert!(validate_metadata(&Some(json!(42))).is_err());
        assert!(validate_metadata(&Some(json!([1, 2, 3]))).is_err());
    }

    #[test]
    fn test_validate_metadata_too_large() {
        let big_value = "x".repeat(MAX_METADATA_SIZE + 1);
        let meta = json!({"data": big_value});
        assert!(validate_metadata(&Some(meta)).is_err());
    }

    #[test]
    fn test_validate_kv_key() {
        assert!(validate_kv_key("valid-key").is_ok());
        assert!(validate_kv_key("").is_err());
        assert!(validate_kv_key(&"k".repeat(600)).is_err());
    }

    #[test]
    fn test_validate_insert_request_valid() {
        let body = json!({
            "id": "vec-1",
            "vector": [1.0, 2.0, 3.0],
            "metadata": {"tag": "test"}
        });
        assert!(validate_insert_request(&body).is_ok());
    }

    #[test]
    fn test_validate_insert_request_no_metadata() {
        let body = json!({
            "id": "vec-1",
            "vector": [1.0, 2.0, 3.0]
        });
        assert!(validate_insert_request(&body).is_ok());
    }

    #[test]
    fn test_validate_insert_request_missing_id() {
        let body = json!({"vector": [1.0, 2.0]});
        assert!(validate_insert_request(&body).is_err());
    }

    #[test]
    fn test_validate_insert_request_missing_vector() {
        let body = json!({"id": "v1"});
        assert!(validate_insert_request(&body).is_err());
    }

    #[test]
    fn test_validate_insert_request_invalid_vector() {
        let body = json!({"id": "v1", "vector": "not_a_vector"});
        assert!(validate_insert_request(&body).is_err());
    }

    #[test]
    fn test_validate_search_request_valid() {
        let body = json!({
            "vector": [1.0, 2.0, 3.0],
            "k": 5
        });
        assert!(validate_search_request(&body).is_ok());
    }

    #[test]
    fn test_validate_search_request_default_k() {
        // When k is omitted, defaults to 10
        let body = json!({"vector": [1.0, 2.0]});
        assert!(validate_search_request(&body).is_ok());
    }

    #[test]
    fn test_validate_search_request_missing_vector() {
        let body = json!({"k": 5});
        assert!(validate_search_request(&body).is_err());
    }

    #[test]
    fn test_validate_search_request_invalid_k() {
        let body = json!({"vector": [1.0], "k": 0});
        assert!(validate_search_request(&body).is_err());
    }

    #[test]
    fn test_validate_batch_insert_valid() {
        let body = json!({
            "vectors": [
                {"id": "a", "vector": [1.0, 2.0]},
                {"id": "b", "vector": [3.0, 4.0]}
            ]
        });
        assert!(validate_batch_insert_request(&body).is_ok());
    }

    #[test]
    fn test_validate_batch_insert_empty() {
        let body = json!({"vectors": []});
        assert!(validate_batch_insert_request(&body).is_err());
    }

    #[test]
    fn test_validate_batch_insert_too_large() {
        let mut vectors = Vec::new();
        for i in 0..=MAX_BATCH_SIZE {
            vectors.push(json!({"id": format!("v{i}"), "vector": [1.0]}));
        }
        let body = json!({"vectors": vectors});
        assert!(validate_batch_insert_request(&body).is_err());
    }

    #[test]
    fn test_validate_batch_insert_missing_vectors() {
        let body = json!({"data": []});
        assert!(validate_batch_insert_request(&body).is_err());
    }

    #[test]
    fn test_validate_batch_insert_invalid_item() {
        let body = json!({
            "vectors": [
                {"id": "a", "vector": [1.0]},
                {"vector": [2.0]}
            ]
        });
        assert!(validate_batch_insert_request(&body).is_err());
    }

    #[test]
    fn test_search_request_uses_k_not_top_k() {
        // The field must be "k", not "top_k"
        let body_with_k = json!({"vector": [1.0, 2.0, 3.0], "k": 5});
        assert!(validate_search_request(&body_with_k).is_ok());

        // "top_k" is ignored — defaults to 10
        let body_with_top_k = json!({"vector": [1.0, 2.0, 3.0], "top_k": 5});
        assert!(validate_search_request(&body_with_top_k).is_ok());
        // But top_k value is NOT read — only "k" is
    }

    #[test]
    fn test_search_request_k_zero_invalid() {
        let body = json!({"vector": [1.0, 2.0, 3.0], "k": 0});
        assert!(validate_search_request(&body).is_err());
    }

    #[test]
    fn test_insert_request_empty_metadata_ok() {
        let body = json!({"id": "vec1", "vector": [1.0, 2.0], "metadata": {}});
        assert!(validate_insert_request(&body).is_ok());
    }
}
