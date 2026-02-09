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
/// Uses constant-time comparison to prevent timing side-channel attacks.
///
/// Returns Ok(()) if valid, Err if invalid
pub fn validate_api_key(key: &str, env: &Env) -> Result<()> {
    // Check against primary API key using constant-time comparison
    if let Ok(expected_key) = env.secret("QUARTZ_API_KEY") {
        if constant_time_eq(key.as_bytes(), expected_key.to_string().as_bytes()) {
            return Ok(());
        }
    }
    
    Err(Error::RustError("Invalid API key".to_string()))
}

/// Constant-time byte comparison to prevent timing side-channel attacks.
///
/// Returns true if both slices are equal, false otherwise.
/// Always compares all bytes regardless of where the first difference is,
/// preventing attackers from guessing the API key byte-by-byte.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
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
    
    #[test]
    fn test_constant_time_eq_equal() {
        assert!(constant_time_eq(b"secret_key_123", b"secret_key_123"));
        assert!(constant_time_eq(b"", b""));
        assert!(constant_time_eq(b"a", b"a"));
    }
    
    #[test]
    fn test_constant_time_eq_not_equal() {
        assert!(!constant_time_eq(b"secret_key_123", b"wrong_key_456"));
        assert!(!constant_time_eq(b"short", b"longer_string"));
        assert!(!constant_time_eq(b"abc", b"abd"));
    }
    
    #[test]
    fn test_constant_time_eq_length_mismatch() {
        assert!(!constant_time_eq(b"abc", b"ab"));
        assert!(!constant_time_eq(b"a", b""));
        assert!(!constant_time_eq(b"", b"a"));
    }
}
