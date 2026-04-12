//! Billing module for QuartzDB - Stripe integration and usage tracking
//!
//! # Overview
//!
//! Handles subscription management via Stripe webhooks and per-tenant
//! usage tracking stored in Cloudflare KV.
//!
//! # Architecture
//!
//! - **Usage Tracking**: Per-tenant, per-month counters stored in KV
//! - **Stripe Webhooks**: Processes subscription lifecycle events
//! - **Plan Enforcement**: Middleware helper to check usage limits
//!
//! # KV Schema
//!
//! - Key format: `usage:{tenant_id}:{YYYY-MM}`
//! - TTL: 90 days (auto-cleanup of old records)
//!
//! # Stripe Events Handled
//!
//! - `checkout.session.completed` → provision new tenant
//! - `customer.subscription.updated` → update plan tier
//! - `customer.subscription.deleted` → downgrade to free

use serde::{Deserialize, Serialize};
use worker::*;

use crate::tenant::{TenantInfo, TenantPlan};

/// KV namespace binding name for tenant data
const TENANT_KV: &str = "TENANT_KV";

/// TTL for usage records: 90 days in seconds
const USAGE_TTL_SECONDS: u64 = 90 * 24 * 60 * 60;

// ---------------------------------------------------------------------------
// Usage Record
// ---------------------------------------------------------------------------

/// Per-tenant monthly usage counters, stored in KV.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub tenant_id: String,
    /// Month in "YYYY-MM" format, e.g. "2026-04"
    pub month: String,
    pub queries: u64,
    pub inserts: u64,
    pub vectors_stored: u64,
    /// Unix timestamp (ms) of last update
    pub last_updated: u64,
}

impl UsageRecord {
    fn new(tenant_id: &str, month: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            month: month.to_string(),
            queries: 0,
            inserts: 0,
            vectors_stored: 0,
            last_updated: crate::platform::now_ms() as u64,
        }
    }
}

// ---------------------------------------------------------------------------
// BillingManager
// ---------------------------------------------------------------------------

/// Stateless helper for usage tracking operations against KV.
pub struct BillingManager;

impl BillingManager {
    /// Increment query count for tenant in the current month.
    pub async fn track_query(env: &Env, tenant_id: &str) -> Result<()> {
        let month = Self::current_month();
        let key = Self::usage_key(tenant_id, &month);
        let kv = env.kv(TENANT_KV)?;

        let mut record = Self::get_or_create_record(&kv, tenant_id, &month, &key).await?;
        record.queries += 1;
        record.last_updated = crate::platform::now_ms() as u64;

        Self::put_record(&kv, &key, &record).await
    }

    /// Increment insert count for tenant in the current month.
    pub async fn track_insert(env: &Env, tenant_id: &str, count: u64) -> Result<()> {
        let month = Self::current_month();
        let key = Self::usage_key(tenant_id, &month);
        let kv = env.kv(TENANT_KV)?;

        let mut record = Self::get_or_create_record(&kv, tenant_id, &month, &key).await?;
        record.inserts += count;
        record.vectors_stored += count;
        record.last_updated = crate::platform::now_ms() as u64;

        Self::put_record(&kv, &key, &record).await
    }

    /// Get current usage for tenant in the current month.
    pub async fn get_usage(env: &Env, tenant_id: &str) -> Result<UsageRecord> {
        let month = Self::current_month();
        let key = Self::usage_key(tenant_id, &month);
        let kv = env.kv(TENANT_KV)?;

        Self::get_or_create_record(&kv, tenant_id, &month, &key).await
    }

    /// Return the current month as `"YYYY-MM"`.
    fn current_month() -> String {
        let (year, month) = crate::platform::current_year_month();
        format!("{:04}-{:02}", year, month)
    }

    /// Build the KV key for a usage record.
    fn usage_key(tenant_id: &str, month: &str) -> String {
        format!("usage:{}:{}", tenant_id, month)
    }

    // -- internal helpers ---------------------------------------------------

    async fn get_or_create_record(
        kv: &kv::KvStore,
        tenant_id: &str,
        month: &str,
        key: &str,
    ) -> Result<UsageRecord> {
        match kv.get(key).text().await? {
            Some(json) => serde_json::from_str::<UsageRecord>(&json)
                .map_err(|e| Error::RustError(format!("deserialize usage: {e}"))),
            None => Ok(UsageRecord::new(tenant_id, month)),
        }
    }

    async fn put_record(kv: &kv::KvStore, key: &str, record: &UsageRecord) -> Result<()> {
        let json = serde_json::to_string(record)
            .map_err(|e| Error::RustError(format!("serialize usage: {e}")))?;
        kv.put(key, json)
            .map_err(|e| Error::RustError(format!("kv put builder: {e}")))?
            .expiration_ttl(USAGE_TTL_SECONDS)
            .execute()
            .await
    }
}

// ---------------------------------------------------------------------------
// Stripe Webhook Handler
// ---------------------------------------------------------------------------

/// Minimal representation of a Stripe webhook event payload.
#[derive(Debug, Deserialize)]
struct StripeEvent {
    #[serde(rename = "type")]
    event_type: String,
    data: StripeEventData,
}

#[derive(Debug, Deserialize)]
struct StripeEventData {
    object: serde_json::Value,
}

/// Handle incoming Stripe webhook events.
///
/// Expected route: `POST /webhooks/stripe`
///
/// Flow:
/// 1. Read raw body
/// 2. Verify webhook signature
/// 3. Parse event JSON
/// 4. Dispatch by event type
/// 5. Return 200 OK
pub async fn handle_stripe_webhook(mut req: Request, env: &Env) -> Result<Response> {
    // 1. Read body
    let body = req.text().await?;

    // 2. Verify signature
    let sig_header = req
        .headers()
        .get("Stripe-Signature")?
        .ok_or_else(|| Error::RustError("Missing Stripe-Signature header".into()))?;

    let webhook_secret = env
        .secret("STRIPE_WEBHOOK_SECRET")
        .map_err(|_| Error::RustError("STRIPE_WEBHOOK_SECRET not configured".into()))?
        .to_string();

    if !verify_stripe_signature(&body, &sig_header, &webhook_secret).await? {
        return Response::error("Invalid signature", 401);
    }

    // 3. Parse event
    let event: StripeEvent = serde_json::from_str(&body)
        .map_err(|e| Error::RustError(format!("parse stripe event: {e}")))?;

    // 4. Dispatch
    match event.event_type.as_str() {
        "checkout.session.completed" => {
            handle_checkout_completed(env, &event.data.object).await?;
        }
        "customer.subscription.updated" => {
            handle_subscription_updated(env, &event.data.object).await?;
        }
        "customer.subscription.deleted" => {
            handle_subscription_deleted(env, &event.data.object).await?;
        }
        other => {
            console_log!("[billing] ignoring event type: {}", other);
        }
    }

    // 5. Acknowledge
    Response::ok("ok")
}

// ---------------------------------------------------------------------------
// Stripe Event Handlers
// ---------------------------------------------------------------------------

/// Provision a new tenant after successful checkout.
///
/// Links the Stripe customer to the existing tenant record using
/// `client_reference_id` (set during checkout session creation),
/// then upgrades the tenant plan to Pro.
async fn handle_checkout_completed(env: &Env, object: &serde_json::Value) -> Result<()> {
    let customer_id = object["customer"]
        .as_str()
        .unwrap_or_default();
    let tenant_id = object["client_reference_id"]
        .as_str()
        .unwrap_or_default();

    if customer_id.is_empty() || tenant_id.is_empty() {
        return Err(Error::RustError(
            "checkout.session.completed missing customer or client_reference_id".into(),
        ));
    }

    let kv = env.kv(TENANT_KV)?;

    // Create reverse mapping: Stripe customer → tenant_id
    kv.put(&format!("stripe:{}", customer_id), tenant_id)
        .map_err(|e| Error::RustError(format!("kv put builder: {e}")))?
        .execute()
        .await?;

    // Upgrade the existing tenant record to Pro
    let tenant_key = format!("tenant:{}", tenant_id);
    if let Some(existing) = kv.get(&tenant_key).text().await? {
        let mut tenant: TenantInfo = serde_json::from_str(&existing)
            .map_err(|e| Error::RustError(format!("parse tenant: {e}")))?;
        tenant.plan = TenantPlan::Pro;
        tenant.vector_limit = TenantPlan::Pro.vector_limit();
        tenant.query_limit_per_month = TenantPlan::Pro.query_limit();

        let json = serde_json::to_string(&tenant)
            .map_err(|e| Error::RustError(format!("serialize tenant: {e}")))?;
        kv.put(&tenant_key, &json)
            .map_err(|e| Error::RustError(format!("kv put builder: {e}")))?
            .execute()
            .await?;
        // Keep the apikey lookup consistent
        kv.put(&format!("apikey:{}", tenant.api_key), &json)
            .map_err(|e| Error::RustError(format!("kv put builder: {e}")))?
            .execute()
            .await?;
    }

    console_log!(
        "[billing] provisioned tenant {} (stripe customer {})",
        tenant_id,
        customer_id
    );
    Ok(())
}

/// Update tenant plan when subscription changes.
async fn handle_subscription_updated(env: &Env, object: &serde_json::Value) -> Result<()> {
    let customer_id = object["customer"]
        .as_str()
        .unwrap_or_default();

    if customer_id.is_empty() {
        return Err(Error::RustError(
            "subscription.updated missing customer".into(),
        ));
    }

    let kv = env.kv(TENANT_KV)?;

    // Reverse lookup: Stripe customer → tenant_id
    let tenant_id = kv
        .get(&format!("stripe:{}", customer_id))
        .text()
        .await?
        .ok_or_else(|| {
            Error::RustError(format!("no tenant mapped to stripe customer {}", customer_id))
        })?;

    let plan_key = object["items"]["data"][0]["price"]["lookup_key"]
        .as_str()
        .unwrap_or("pro");

    let plan = match plan_key {
        "scale" => TenantPlan::Scale,
        "free" => TenantPlan::Free,
        _ => TenantPlan::Pro,
    };

    let tenant_key = format!("tenant:{}", tenant_id);
    if let Some(existing) = kv.get(&tenant_key).text().await? {
        let mut tenant: TenantInfo = serde_json::from_str(&existing)
            .map_err(|e| Error::RustError(format!("parse tenant: {e}")))?;
        tenant.plan = plan.clone();
        tenant.vector_limit = plan.vector_limit();
        tenant.query_limit_per_month = plan.query_limit();

        let json = serde_json::to_string(&tenant)
            .map_err(|e| Error::RustError(format!("serialize tenant: {e}")))?;
        kv.put(&tenant_key, &json)
            .map_err(|e| Error::RustError(format!("kv put builder: {e}")))?
            .execute()
            .await?;
        kv.put(&format!("apikey:{}", tenant.api_key), &json)
            .map_err(|e| Error::RustError(format!("kv put builder: {e}")))?
            .execute()
            .await?;
    }

    console_log!(
        "[billing] updated tenant {} to plan {}",
        tenant_id,
        plan_key
    );
    Ok(())
}

/// Downgrade tenant to free tier when subscription is deleted.
async fn handle_subscription_deleted(env: &Env, object: &serde_json::Value) -> Result<()> {
    let customer_id = object["customer"]
        .as_str()
        .unwrap_or_default();

    if customer_id.is_empty() {
        return Err(Error::RustError(
            "subscription.deleted missing customer".into(),
        ));
    }

    let kv = env.kv(TENANT_KV)?;

    // Reverse lookup: Stripe customer → tenant_id
    let tenant_id = kv
        .get(&format!("stripe:{}", customer_id))
        .text()
        .await?
        .ok_or_else(|| {
            Error::RustError(format!("no tenant mapped to stripe customer {}", customer_id))
        })?;

    let tenant_key = format!("tenant:{}", tenant_id);
    if let Some(existing) = kv.get(&tenant_key).text().await? {
        let mut tenant: TenantInfo = serde_json::from_str(&existing)
            .map_err(|e| Error::RustError(format!("parse tenant: {e}")))?;
        tenant.plan = TenantPlan::Free;
        tenant.vector_limit = TenantPlan::Free.vector_limit();
        tenant.query_limit_per_month = TenantPlan::Free.query_limit();

        let json = serde_json::to_string(&tenant)
            .map_err(|e| Error::RustError(format!("serialize tenant: {e}")))?;
        kv.put(&tenant_key, &json)
            .map_err(|e| Error::RustError(format!("kv put builder: {e}")))?
            .execute()
            .await?;
        kv.put(&format!("apikey:{}", tenant.api_key), &json)
            .map_err(|e| Error::RustError(format!("kv put builder: {e}")))?
            .execute()
            .await?;
    }

    console_log!(
        "[billing] downgraded tenant {} to free tier",
        tenant_id
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Stripe Signature Verification
// ---------------------------------------------------------------------------

/// Verify the Stripe webhook signature.
///
/// The `Stripe-Signature` header has the format:
///   `t=<timestamp>,v1=<signature>`
///
/// A full implementation would compute HMAC-SHA256 over `"{timestamp}.{payload}"`
/// using the webhook signing secret and compare with the provided signature.
///
/// # Current Implementation
///
/// Parses and validates the header structure and checks timestamp freshness
/// (rejects events older than 5 minutes to prevent replay attacks).
///
/// Verifies the Stripe webhook signature using HMAC-SHA256.
/// Uses the pure-Rust implementation in `crate::platform::hmac_sha256`.
async fn verify_stripe_signature(
    payload: &str,
    signature: &str,
    secret: &str,
) -> Result<bool> {
    // Parse the Stripe-Signature header
    let mut timestamp: Option<&str> = None;
    let mut sig_v1: Option<&str> = None;

    for part in signature.split(',') {
        let part = part.trim();
        if let Some(t) = part.strip_prefix("t=") {
            timestamp = Some(t);
        } else if let Some(v) = part.strip_prefix("v1=") {
            sig_v1 = Some(v);
        }
    }

    let ts = timestamp
        .ok_or_else(|| Error::RustError("Stripe sig missing timestamp".into()))?;
    let v1 = sig_v1
        .ok_or_else(|| Error::RustError("Stripe sig missing v1 signature".into()))?;

    // Validate timestamp freshness (reject events older than 5 minutes)
    let ts_secs: f64 = ts
        .parse()
        .map_err(|_| Error::RustError("invalid timestamp in Stripe sig".into()))?;
    let now_secs = crate::platform::now_ms() / 1000.0;
    let tolerance_secs = 300.0; // 5 minutes

    if (now_secs - ts_secs).abs() > tolerance_secs {
        console_log!(
            "[billing] rejecting stale webhook: ts={}, now={}",
            ts_secs,
            now_secs
        );
        return Ok(false);
    }

    // Full HMAC-SHA256 verification (RFC 2104)
    let signed_payload = format!("{ts}.{payload}");
    let expected_mac = crate::platform::hmac_sha256(
        secret.as_bytes(),
        signed_payload.as_bytes(),
    );
    let expected_hex = crate::platform::hex_encode(&expected_mac);

    if !crate::platform::constant_time_eq(expected_hex.as_bytes(), v1.as_bytes()) {
        console_log!("[billing] webhook signature mismatch");
        return Ok(false);
    }

    Ok(true)
}

// ---------------------------------------------------------------------------
// Usage Limit Enforcement
// ---------------------------------------------------------------------------

/// Check if a tenant has exceeded their plan's query limit.
///
/// Returns `Ok(())` if within limits; returns a 402 Payment Required error
/// if the tenant has exceeded their quota for the current month.
pub async fn check_usage_limits(
    env: &Env,
    tenant_id: &str,
    plan_query_limit: usize,
) -> Result<()> {
    let usage = BillingManager::get_usage(env, tenant_id).await?;

    if usage.queries >= plan_query_limit as u64 {
        return Err(Error::RustError(format!(
            "402:Query limit exceeded ({}/{} this month). Upgrade your plan.",
            usage.queries, plan_query_limit
        )));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_key_format() {
        let key = BillingManager::usage_key("tenant_123", "2026-04");
        assert_eq!(key, "usage:tenant_123:2026-04");
    }

    #[test]
    fn test_current_month_format() {
        let month = BillingManager::current_month();
        // Should match YYYY-MM pattern
        assert_eq!(month.len(), 7);
        assert_eq!(&month[4..5], "-");
        let year: u32 = month[..4].parse().unwrap();
        let mon: u32 = month[5..].parse().unwrap();
        assert!(year >= 2024 && year <= 2100);
        assert!(mon >= 1 && mon <= 12);
    }

    #[test]
    fn test_usage_record_new() {
        let record = UsageRecord::new("t1", "2026-04");
        assert_eq!(record.tenant_id, "t1");
        assert_eq!(record.month, "2026-04");
        assert_eq!(record.queries, 0);
        assert_eq!(record.inserts, 0);
        assert_eq!(record.vectors_stored, 0);
        assert!(record.last_updated > 0);
    }

    #[test]
    fn test_stripe_signature_parse_valid() {
        // Build a valid signature using our HMAC implementation
        let secret = "whsec_test_secret";
        let payload = r#"{"type":"checkout.session.completed","data":{}}"#;
        let ts = (crate::platform::now_ms() / 1000.0) as u64;

        let signed = format!("{ts}.{payload}");
        let mac = crate::platform::hmac_sha256(secret.as_bytes(), signed.as_bytes());
        let sig_hex = crate::platform::hex_encode(&mac);

        let sig_header = format!("t={ts},v1={sig_hex}");

        // Run the async function synchronously in a test
        // We can't easily test the full async function without a runtime,
        // but we can test the signature construction matches
        let expected_mac = crate::platform::hmac_sha256(
            secret.as_bytes(),
            format!("{ts}.{payload}").as_bytes(),
        );
        assert_eq!(crate::platform::hex_encode(&expected_mac), sig_hex);
    }

    #[test]
    fn test_stripe_event_deserialization() {
        let json = r#"{"type":"checkout.session.completed","data":{"object":{"customer":"cus_123","subscription":"sub_456"}}}"#;
        let event: StripeEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "checkout.session.completed");
        assert_eq!(event.data.object["customer"], "cus_123");
    }

    #[test]
    fn test_stripe_signature_constant_time_eq() {
        let a = "abc123def456";
        let b = "abc123def456";
        let c = "abc123def457";
        assert!(crate::platform::constant_time_eq(a.as_bytes(), b.as_bytes()));
        assert!(!crate::platform::constant_time_eq(a.as_bytes(), c.as_bytes()));
    }

    #[test]
    fn test_hmac_sha256_stripe_webhook_roundtrip() {
        let secret = "whsec_realproductionsecret";
        let payload = r#"{"type":"checkout.session.completed"}"#;
        let ts = "1700000000";

        let signed = format!("{ts}.{payload}");
        let mac = crate::platform::hmac_sha256(secret.as_bytes(), signed.as_bytes());
        let sig_hex = crate::platform::hex_encode(&mac);

        // Verify same inputs produce same signature
        let mac2 = crate::platform::hmac_sha256(secret.as_bytes(), signed.as_bytes());
        assert_eq!(crate::platform::hex_encode(&mac2), sig_hex);

        // Verify different secret produces different signature
        let mac_wrong = crate::platform::hmac_sha256(b"wrong_secret", signed.as_bytes());
        assert_ne!(crate::platform::hex_encode(&mac_wrong), sig_hex);
    }

    #[test]
    fn test_stripe_event_with_client_reference_id() {
        let json = r#"{"type":"checkout.session.completed","data":{"object":{"customer":"cus_abc","subscription":"sub_xyz","client_reference_id":"tenant_42"}}}"#;
        let event: StripeEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "checkout.session.completed");
        assert_eq!(event.data.object["client_reference_id"], "tenant_42");
        assert_eq!(event.data.object["customer"], "cus_abc");
    }

    #[test]
    fn test_stripe_event_invoice_paid() {
        let json = r#"{"type":"invoice.paid","data":{"object":{"customer":"cus_abc","amount_paid":2999}}}"#;
        let event: StripeEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "invoice.paid");
        assert_eq!(event.data.object["amount_paid"], 2999);
    }

    #[test]
    fn test_usage_key_different_tenants() {
        let k1 = BillingManager::usage_key("t1", "2026-04");
        let k2 = BillingManager::usage_key("t2", "2026-04");
        assert_ne!(k1, k2);
        assert!(k1.contains("t1"));
        assert!(k2.contains("t2"));
    }

    #[test]
    fn test_usage_key_different_months() {
        let k1 = BillingManager::usage_key("t1", "2026-04");
        let k2 = BillingManager::usage_key("t1", "2026-05");
        assert_ne!(k1, k2);
    }
}
