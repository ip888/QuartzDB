//! Multi-tenant authentication and isolation for QuartzDB
//!
//! Maps API keys to tenant IDs, where each tenant gets isolated vector storage.
//!
//! # KV Storage Design
//!
//! - Namespace binding: `TENANT_KV`
//! - `apikey:{sha256_hash}` → TenantInfo JSON
//! - `tenant:{tenant_id}` → TenantInfo JSON
//!
//! API keys are stored as SHA-256 hashes; raw keys are never persisted in KV values.

use serde::{Deserialize, Serialize};
use worker::*;

const KV_NAMESPACE: &str = "TENANT_KV";
const API_KEY_PREFIX: &str = "qdb_";

// ---------------------------------------------------------------------------
// TenantPlan
// ---------------------------------------------------------------------------

/// Subscription tier determining usage limits and pricing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TenantPlan {
    /// Free tier — 1 000 vectors, 10 000 queries/month.
    Free,
    /// Pro tier ($29/mo) — 100 000 vectors, 100 000 queries/month.
    Pro,
    /// Scale tier ($99/mo) — 1 000 000 vectors, unlimited queries.
    Scale,
}

impl TenantPlan {
    /// Maximum number of vectors a tenant on this plan may store.
    pub fn vector_limit(&self) -> usize {
        match self {
            TenantPlan::Free => 1_000,
            TenantPlan::Pro => 100_000,
            TenantPlan::Scale => 1_000_000,
        }
    }

    /// Maximum queries per month (0 means unlimited).
    pub fn query_limit(&self) -> usize {
        match self {
            TenantPlan::Free => 10_000,
            TenantPlan::Pro => 100_000,
            TenantPlan::Scale => 0, // unlimited
        }
    }

    /// Price in US cents per month.
    pub fn price_cents(&self) -> u32 {
        match self {
            TenantPlan::Free => 0,
            TenantPlan::Pro => 2_900,
            TenantPlan::Scale => 9_900,
        }
    }
}

// ---------------------------------------------------------------------------
// TenantInfo
// ---------------------------------------------------------------------------

/// Persisted tenant record stored in Cloudflare KV.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    /// Unique tenant identifier (e.g. project slug or UUID).
    pub tenant_id: String,
    /// SHA-256 hex digest of the real API key — the raw key is **never** stored.
    pub api_key: String,
    /// Current subscription plan.
    pub plan: TenantPlan,
    /// Creation timestamp in milliseconds since the Unix epoch.
    pub created_at: u64,
    /// Maximum vectors allowed under the current plan.
    pub vector_limit: usize,
    /// Maximum queries per month (0 = unlimited).
    pub query_limit_per_month: usize,
}

// ---------------------------------------------------------------------------
// TenantManager
// ---------------------------------------------------------------------------

/// Stateless helper for tenant CRUD operations against Cloudflare KV.
pub struct TenantManager;

impl TenantManager {
    // -- lookup ---------------------------------------------------------------

    /// Look up a tenant by their raw API key.
    ///
    /// The key is hashed with SHA-256 before the KV lookup so that raw keys
    /// never leave the edge worker.
    pub async fn get_tenant_by_key(env: &Env, api_key: &str) -> Result<Option<TenantInfo>> {
        let hash = Self::sha256_hex(api_key);
        let kv_key = format!("apikey:{hash}");

        let kv = env.kv(KV_NAMESPACE)?;
        match kv.get(&kv_key).text().await? {
            Some(json) => {
                let info: TenantInfo = serde_json::from_str(&json)
                    .map_err(|e| Error::RustError(format!("Tenant JSON parse error: {e}")))?;
                Ok(Some(info))
            }
            None => Ok(None),
        }
    }

    // -- creation -------------------------------------------------------------

    /// Create a new tenant, generate an API key, and persist both index entries
    /// in KV (`apikey:{hash}` and `tenant:{id}`).
    ///
    /// Returns the [`TenantInfo`] **with the raw API key** so it can be shown to
    /// the caller exactly once.  The stored copy uses the SHA-256 hash instead.
    pub async fn create_tenant(
        env: &Env,
        tenant_id: &str,
        plan: TenantPlan,
    ) -> Result<TenantInfo> {
        let raw_key = Self::generate_api_key();
        let key_hash = Self::sha256_hex(&raw_key);

        let now = Date::now().as_millis();

        // Build the stored record (hash, not raw key).
        let stored = TenantInfo {
            tenant_id: tenant_id.to_string(),
            api_key: key_hash.clone(),
            plan: plan.clone(),
            created_at: now,
            vector_limit: plan.vector_limit(),
            query_limit_per_month: plan.query_limit(),
        };

        let json = serde_json::to_string(&stored)
            .map_err(|e| Error::RustError(format!("Tenant serialization error: {e}")))?;

        let kv = env.kv(KV_NAMESPACE)?;

        // Write both lookup keys.
        kv.put(&format!("apikey:{key_hash}"), &json)?
            .execute()
            .await?;
        kv.put(&format!("tenant:{tenant_id}"), &json)?
            .execute()
            .await?;

        // Return a copy that carries the **raw** key so the caller can hand it
        // to the user exactly once.
        let response = TenantInfo {
            api_key: raw_key,
            ..stored
        };

        Ok(response)
    }

    // -- key generation -------------------------------------------------------

    /// Generate a secure random API key: `qdb_` + 32 hex characters.
    ///
    /// Uses `js_sys::Math::random()` since this runs inside WASM on V8.
    pub fn generate_api_key() -> String {
        let mut hex = String::with_capacity(32);
        for _ in 0..32 {
            // Math::random() returns [0, 1); multiply by 16 then floor → 0..15
            let nibble = (crate::platform::random_f64() * 16.0).floor() as u8;
            hex.push(char::from(if nibble < 10 {
                b'0' + nibble
            } else {
                b'a' + (nibble - 10)
            }));
        }
        format!("{API_KEY_PREFIX}{hex}")
    }

    // -- shard routing --------------------------------------------------------

    /// Derive the Durable Object shard name that isolates this tenant's vectors.
    pub fn get_tenant_shard_name(tenant_id: &str, shard_id: usize) -> String {
        format!("tenant:{tenant_id}:shard:{shard_id}")
    }

    // -- plan limits ----------------------------------------------------------

    /// Verify that the tenant has not exceeded their plan's vector limit.
    pub async fn check_limits(
        _env: &Env,
        tenant: &TenantInfo,
        current_vectors: usize,
    ) -> Result<()> {
        if current_vectors >= tenant.vector_limit {
            return Err(Error::RustError(format!(
                "Vector limit exceeded: plan allows {} vectors, currently at {}",
                tenant.vector_limit, current_vectors
            )));
        }
        Ok(())
    }

    // -- helpers (private) ----------------------------------------------------

    /// Compute the SHA-256 hex digest of `input`.
    ///
    /// Uses a simple pure implementation suitable for WASM — no external crate
    /// required.  The hash is used for KV key derivation, not for
    /// cryptographic signatures, so a straightforward implementation suffices.
    fn sha256_hex(input: &str) -> String {
        let hash = crate::platform::sha256(input.as_bytes());
        crate::platform::hex_encode(&hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_known_vector() {
        // SHA-256("hello") == 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        let digest = TenantManager::sha256_hex("hello");
        assert_eq!(
            digest,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_plan_limits() {
        assert_eq!(TenantPlan::Free.vector_limit(), 1_000);
        assert_eq!(TenantPlan::Pro.vector_limit(), 100_000);
        assert_eq!(TenantPlan::Scale.vector_limit(), 1_000_000);

        assert_eq!(TenantPlan::Free.query_limit(), 10_000);
        assert_eq!(TenantPlan::Pro.query_limit(), 100_000);
        assert_eq!(TenantPlan::Scale.query_limit(), 0);

        assert_eq!(TenantPlan::Free.price_cents(), 0);
        assert_eq!(TenantPlan::Pro.price_cents(), 2_900);
        assert_eq!(TenantPlan::Scale.price_cents(), 9_900);
    }

    #[test]
    fn test_shard_name() {
        assert_eq!(
            TenantManager::get_tenant_shard_name("acme", 3),
            "tenant:acme:shard:3"
        );
    }

    #[test]
    fn test_api_key_format() {
        let key = TenantManager::generate_api_key();
        assert!(key.starts_with("qdb_"));
        assert_eq!(key.len(), 4 + 32); // prefix + 32 hex chars
        assert!(key[4..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_api_key_uniqueness() {
        let key1 = TenantManager::generate_api_key();
        let key2 = TenantManager::generate_api_key();
        assert_ne!(key1, key2, "Two generated keys should not be identical");
    }

    #[test]
    fn test_tenant_shard_name() {
        let name = TenantManager::get_tenant_shard_name("t1", 3);
        assert_eq!(name, "tenant:t1:shard:3");
    }

    #[test]
    fn test_plan_price_cents() {
        assert_eq!(TenantPlan::Free.price_cents(), 0);
        assert_eq!(TenantPlan::Pro.price_cents(), 2_900);
        assert_eq!(TenantPlan::Scale.price_cents(), 9_900);
    }

    #[test]
    fn test_plan_query_limit() {
        assert_eq!(TenantPlan::Free.query_limit(), 10_000);
        assert_eq!(TenantPlan::Pro.query_limit(), 100_000);
        assert_eq!(TenantPlan::Scale.query_limit(), 0);
    }
}
