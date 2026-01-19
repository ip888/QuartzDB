//! Authentication middleware for QuartzDB
//!
//! # Security Model
//!
//! - API Key authentication (Bearer token or X-API-Key header)
//! - Keys stored in Cloudflare secrets (not in code)
//! - Public endpoints: /health, /
//! - Protected endpoints: /api/*
//!
//! # Usage
//!
//! ```ignore
//! let api_key = extract_api_key(&request)?;
//! validate_api_key(&api_key, &env)?;
//! ```

use worker::*;

/// Extract API key from request headers
///
/// Supports two formats:
/// 1. Authorization: Bearer <key>
/// 2. X-API-Key: <key>
///
/// Returns None if no key found (public endpoint)
pub fn extract_api_key(req: &Request) -> Result<Option<String>> {
    let headers = req.headers();
    
    // Try Authorization header first
    if let Ok(auth_header) = headers.get("Authorization") {
        if let Some(auth) = auth_header {
            if let Some(key) = auth.strip_prefix("Bearer ") {
                return Ok(Some(key.to_string()));
            }
        }
    }
    
    // Try X-API-Key header
    if let Ok(api_key_header) = headers.get("X-API-Key") {
        if let Some(key) = api_key_header {
            return Ok(Some(key.to_string()));
        }
    }
    
    Ok(None)
}

/// Validate API key against configured keys
///
/// Checks:
/// 1. QUARTZ_API_KEY secret (single key for now)
/// 2. Future: QUARTZ_API_KEYS (comma-separated list)
///
/// Returns Ok(()) if valid, Err if invalid
pub fn validate_api_key(key: &str, env: &Env) -> Result<()> {
    // Check against primary API key
    if let Ok(expected_key) = env.secret("QUARTZ_API_KEY") {
        if key == expected_key.to_string() {
            return Ok(());
        }
    }
    
    // Future: Check against multiple keys (QUARTZ_API_KEYS)
    // if let Ok(keys_csv) = env.secret("QUARTZ_API_KEYS") {
    //     for expected in keys_csv.to_string().split(',') {
    //         if key == expected.trim() {
    //             return Ok(());
    //         }
    //     }
    // }
    
    Err(Error::RustError("Invalid API key".to_string()))
}

/// Check if path requires authentication
///
/// Public endpoints: /, /health
/// Protected: /api/*
pub fn is_protected_path(path: &str) -> bool {
    path.starts_with("/api/")
}

/// Middleware: Require authentication for protected endpoints
///
/// Usage in router:
/// ```ignore
/// if is_protected_path(&path) {
///     let api_key = extract_api_key(&req)?
///         .ok_or_else(|| Error::RustError("Missing API key".to_string()))?;
///     validate_api_key(&api_key, &env)?;
/// }
/// ```
pub fn require_auth(req: &Request, env: &Env) -> Result<()> {
    let path = req.path();
    
    if is_protected_path(&path) {
        let api_key = extract_api_key(req)?
            .ok_or_else(|| Error::RustError("Missing API key".to_string()))?;
        validate_api_key(&api_key, env)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_protected_path() {
        assert!(is_protected_path("/api/vector/insert"));
        assert!(is_protected_path("/api/put"));
        assert!(!is_protected_path("/health"));
        assert!(!is_protected_path("/"));
    }
}
