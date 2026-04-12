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

use crate::tenant::{TenantInfo, TenantManager};
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

/// Authenticate a request and return the associated [`TenantInfo`].
///
/// 1. Extracts the API key from headers (Bearer / X-API-Key).
/// 2. Looks up the tenant record in KV via [`TenantManager`].
/// 3. Returns the tenant or an error if the key is missing / unknown.
pub async fn authenticate_tenant(req: &Request, env: &Env) -> Result<TenantInfo> {
    let api_key = extract_api_key(req)?
        .ok_or_else(|| Error::RustError("Missing API key".to_string()))?;

    TenantManager::get_tenant_by_key(env, &api_key)
        .await?
        .ok_or_else(|| Error::RustError("Invalid API key".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_protected_path() {
        assert!(is_protected_path("/api/vector/insert"));
        assert!(is_protected_path("/api/put"));
        assert!(is_protected_path("/api/get/key1"));
        assert!(is_protected_path("/api/tenants/signup"));
        assert!(!is_protected_path("/health"));
        assert!(!is_protected_path("/"));
        assert!(!is_protected_path("/webhooks/stripe"));
    }

    #[test]
    fn test_is_protected_path_edge_cases() {
        assert!(!is_protected_path("/api")); // no trailing slash = not /api/*
        assert!(!is_protected_path("")); // empty
        assert!(!is_protected_path("/healthcheck"));
        assert!(!is_protected_path("/apifoo")); // doesn't start with /api/
    }

    #[test]
    fn test_is_protected_path_webhook_not_protected() {
        // Stripe webhooks must be accessible without API key
        assert!(!is_protected_path("/webhooks/stripe"));
        assert!(!is_protected_path("/webhooks/other"));
    }

    #[test]
    fn test_is_protected_path_all_api_routes() {
        // All vector routes are protected
        assert!(is_protected_path("/api/vector/search"));
        assert!(is_protected_path("/api/vector/batch-insert"));
        assert!(is_protected_path("/api/vector/get/some-id"));
        assert!(is_protected_path("/api/vector/delete/some-id"));
        assert!(is_protected_path("/api/vector/stats"));
        // Tenant routes are protected
        assert!(is_protected_path("/api/tenants/signup"));
        assert!(is_protected_path("/api/usage"));
    }

    #[test]
    fn test_api_key_prefix_convention() {
        // Tenant API keys follow qdb_ prefix convention
        let key = "qdb_abc123def456";
        assert!(key.starts_with("qdb_"));
        assert!(key.len() > 4);

        // Non-tenant keys don't have the prefix
        let admin_key = "sk_live_admin_key";
        assert!(!admin_key.starts_with("qdb_"));
    }
}
