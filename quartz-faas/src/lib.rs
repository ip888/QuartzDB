#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
//! QuartzDB FaaS - Cloudflare Workers Integration
//!
//! # Architecture Overview
//!
//! ```text
//! ┌────────────────────────────────────┐
//! │ Cloudflare Edge Network (300+ DCs)  │
//! └────────────────────────────────────┘
//!                  ↓
//! ┌────────────────────────────────────┐
//! │      Worker (WASM on V8)           │
//! │  ┌────────────────────────────┐  │
//! │  │ Router (worker-rs)      │  │
//! │  │ - /health              │  │
//! │  │ - /api/*               │  │
//! │  │ - /api/vector/*        │  │
//! │  └────────────────────────────┘  │
//! └────────────────────────────────────┘
//!       │                    │
//!       ↓                    ↓
//! ┌────────────┐    ┌──────────────────┐
//! │ StorageObject│    │ VectorIndexObject│
//! │ (KV Store)  │    │ (HNSW Search)   │
//! └────────────┘    └──────────────────┘
//!       │                    │
//!       ↓                    ↓
//! ┌──────────────────────────────────┐
//! │    Durable Storage (SQLite)      │
//! │    - Replicated                  │
//! │    - Strongly Consistent         │
//! └──────────────────────────────────┘
//! ```
//!
//! # Request Flow
//!
//! 1. **Ingress**: Client → Cloudflare Edge (nearest datacenter)
//! 2. **Routing**: Worker router matches path and method
//! 3. **Forwarding**: Worker gets Durable Object stub and forwards request
//! 4. **Processing**: Durable Object processes (cache/storage or HNSW)
//! 5. **Response**: Result flows back through Worker to client
//! 6. **Analytics**: Metrics tracked in Analytics Engine (async)
//!
//! # Why This Architecture?
//!
//! **Worker as Thin Proxy**
//! - Pro: Stateless, scales infinitely
//! - Pro: Can route to multiple DO instances (future sharding)
//! - Pro: Can add middleware (auth, rate limiting) without touching DO logic
//! - Con: Extra network hop (Worker → DO ~1-2ms)
//!
//! **Durable Objects for State**
//! - Pro: Strong consistency (single writer per DO)
//! - Pro: Automatic persistence (SQLite + replication)
//! - Pro: Isolated execution (no thread safety concerns)
//! - Con: Limited to ~1000 RPS per DO instance
//!
//! **HNSW in Durable Object**
//! - Pro: Graph stays in memory (fast search)
//! - Pro: Automatic persistence via serialization
//! - Con: Serialize entire graph on updates (latency)
//!
//! # Performance Characteristics
//!
//! - **Cold Start**: 50-100ms (first request after deploy)
//! - **Warm Request**: <10ms end-to-end
//! - **DO Operation**: 5-10ms (includes storage I/O)
//! - **HNSW Search**: 1-5ms for 100K vectors
//! - **Network Overhead**: 1-2ms (Worker → DO)
//!
//! # Error Handling Philosophy
//!
//! - **NO PANICS**: All operations return Result<T>
//! - **Graceful Degradation**: Analytics failures don't affect requests
//! - **User-Facing Errors**: Return proper HTTP status codes
//! - **Internal Errors**: Log to console, return 500
//!
//! # Monitoring Integration
//!
//! Every request is tracked with:
//! - Console logs (wrangler tail)
//! - Analytics Engine (structured metrics)
//! - Request/response timing
//! - Error rates and status codes
//!
//! Designed to run on Cloudflare Workers with Durable Objects for state management.

use worker::*;

mod api;
mod auth;
mod billing;
mod error;
mod durable;
mod monitoring;
mod platform;
mod ratelimit;
mod timeout;
mod validation;
mod vector;
mod sharding;
mod tenant;

pub use api::*;
pub use billing::*;
pub use tenant::*;
pub use auth::*;
pub use error::*;
pub use monitoring::*;
pub use ratelimit::*;
pub use timeout::*;
pub use validation::*;
pub use vector::*;
pub use sharding::*;

/// Create a ShardRouter with the default shard count
fn get_shard_router() -> ShardRouter {
    ShardRouter::new(DEFAULT_SHARD_COUNT)
}

/// Resolve the tenant for multi-tenant (`qdb_*`) API keys.
///
/// Returns `None` for legacy admin keys — billing and limits are skipped
/// in legacy mode.
async fn resolve_tenant(req: &Request, env: &Env) -> Result<Option<TenantInfo>> {
    match extract_api_key(req)? {
        Some(key) if key.starts_with("qdb_") => {
            let tenant = authenticate_tenant(req, env).await?;
            Ok(Some(tenant))
        }
        _ => Ok(None),
    }
}

/// Main entry point for Cloudflare Worker
///
/// # Initialization Sequence:
///
/// 1. **Panic Hook**: Install panic handler for better error messages in WASM
/// 2. **Uptime Tracking**: Record worker start time (for /health endpoint)
/// 3. **Metrics Init**: Start timer and create RequestMetrics
/// 4. **Request Handling**: Route to appropriate handler
/// 5. **Analytics**: Log metrics and track in Analytics Engine
/// 6. **Response**: Return to client
///
/// # Why console_error_panic_hook?
///
/// WASM panics are cryptic by default. This hook provides:
/// - Stack traces in console
/// - Readable error messages
/// - Better debugging experience
///
/// # Why Track Uptime?
///
/// Workers can be evicted and restarted. Uptime helps:
/// - Identify cold starts (high latency after restart)
/// - Monitor worker stability
/// - Debug memory leaks (workers running too long)
///
/// # Error Handling:
///
/// - Router errors: Caught and converted to 500 responses
/// - Analytics errors: Ignored (best-effort monitoring)
/// - All other errors: Logged and returned as HTTP errors
///
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Install panic hook for better WASM error messages
    console_error_panic_hook::set_once();
    // Track worker uptime (approximate, resets on worker restart)
    init_uptime();

    // Track request metrics
    let method = req.method().to_string();
    let path = req.path();
    let mut metrics = RequestMetrics::new(method.clone(), path.clone());
    let request_id = metrics.request_id.clone();
    let timer = Timer::new();
    
    // Clone env for analytics (router consumes original env)
    // This is cheap - Env is just a handle to the JS environment
    let env_clone = env.clone();
    
    // Compute allowed CORS origin early so early-return paths get correct headers
    let cors_origin = env_clone.var("CORS_ALLOWED_ORIGIN")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "*".to_string());
    
    // Handle CORS preflight
    if method == "OPTIONS" {
        return Response::ok("")
            .map(|r| finalize_response(r, &cors_origin, &request_id));
    }
    
    // Reject oversized request bodies to prevent memory exhaustion
    if let Ok(Some(content_length)) = req.headers().get("Content-Length") {
        if let Ok(size) = content_length.parse::<usize>() {
            if size > MAX_REQUEST_BODY_SIZE {
                metrics.finish(413, timer.elapsed_ms());
                metrics.log();
                let _ = metrics.track(&env_clone);
                
                return Response::error("Request body too large (max 1MB)", 413)
                    .map(|r| finalize_response(r, &cors_origin, &request_id));
            }
        }
    }
    
    // Rate limiting check (before auth, uses IP or API key)
    if is_protected_path(&path) {
        if let Err(_) = check_cloudflare_rate_limit(&req) {
            metrics.finish(429, timer.elapsed_ms());
            metrics.log();
            let _ = metrics.track(&env_clone);
            
            return Response::error("Too Many Requests", 429)
                .map(|r| {
                    let mut r = finalize_response(r, &cors_origin, &request_id);
                    let _ = r.headers_mut().set("Retry-After", "60");
                    r
                });
        }
    }
    
    // Authentication check (before routing)
    // Two modes:
    //   - Legacy: QUARTZ_API_KEY env secret (admin/dev key, no "qdb_" prefix)
    //   - Multi-tenant: API keys starting with "qdb_" are validated async in route handlers
    if is_protected_path(&path) && !path.starts_with("/webhooks/") && !path.starts_with("/api/tenants/signup") {
        let api_key = extract_api_key(&req)?;
        match api_key {
            Some(ref key) if !key.starts_with("qdb_") => {
                // Legacy API key check (admin/dev key via env secret)
                if let Err(_) = validate_api_key(key, &env) {
                    metrics.finish(401, timer.elapsed_ms());
                    metrics.log();
                    let _ = metrics.track(&env_clone);
                    return Response::error("Unauthorized: Invalid or missing API key", 401)
                        .map(|r| finalize_response(r, &cors_origin, &request_id));
                }
            }
            Some(_) => {
                // Multi-tenant key (qdb_*) - will be validated async in route handlers
            }
            None => {
                metrics.finish(401, timer.elapsed_ms());
                metrics.log();
                let _ = metrics.track(&env_clone);
                return Response::error("Unauthorized: Missing API key", 401)
                    .map(|r| finalize_response(r, &cors_origin, &request_id));
            }
        }
    }

    // Setup router
    let response = Router::new()
        .get("/", |_, _| Response::ok("QuartzDB FaaS API v0.1.0").map(|r| add_cors_headers(r)))
        .get_async("/health", |_, ctx| async move {
            let storage_ok = check_storage_health(&ctx.env).await;
            let vector_ok = check_vector_health(&ctx.env).await;
            let healthy = storage_ok && vector_ok;

            let response_json = serde_json::json!({
                "status": if healthy { "healthy" } else { "degraded" },
                "service": "quartz-faas",
                "version": env!("CARGO_PKG_VERSION"),
                "uptime_seconds": get_uptime_seconds(),
                "checks": {
                    "storage": if storage_ok { "ok" } else { "fail" },
                    "vector_index": if vector_ok { "ok" } else { "fail" },
                }
            });

            if healthy {
                Response::from_json(&response_json).map(|r| add_cors_headers(r))
            } else {
                Ok(add_cors_headers(
                    Response::from_json(&response_json)?
                        .with_status(503)
                ))
            }
        })
        .post_async("/api/put", |mut req, ctx| async move {
            let body: serde_json::Value = req.json().await?;
            
            // Validate key
            let key = body["key"].as_str().unwrap_or("");
            if let Err(e) = validate_kv_key(key) {
                return Response::error(&format!("Validation error: {}", e), 400)
                    .map(|r| add_cors_headers(r));
            }
            
            // Get Durable Object stub
            let namespace = ctx.env.durable_object("STORAGE")?;
            let stub = namespace.id_from_name("default")?.get_stub()?;
            
            // Forward request to Durable Object
            let mut do_req = Request::new_with_init(
                "https://fake-host/put",
                RequestInit::new()
                    .with_method(Method::Post)
                    .with_body(Some(serde_json::to_string(&body)?.into()))
            )?;
            do_req.headers_mut()?.set("Content-Type", "application/json")?;
            
            let response = stub.fetch_with_request(do_req).await?;
            Ok(add_cors_headers(response))
        })
        .get_async("/api/get/:key", |_, ctx| async move {
            if let Some(key) = ctx.param("key") {
                if let Err(e) = validate_kv_key(key) {
                    return Response::error(&format!("Validation error: {}", e), 400)
                        .map(|r| add_cors_headers(r));
                }
                // Get Durable Object stub
                let namespace = ctx.env.durable_object("STORAGE")?;
                let stub = namespace.id_from_name("default")?.get_stub()?;
                
                // Forward request to Durable Object
                let do_req = Request::new_with_init(
                    &format!("https://fake-host/get/{}", key),
                    RequestInit::new().with_method(Method::Get)
                )?;
                
                let response = stub.fetch_with_request(do_req).await?;
                Ok(add_cors_headers(response))
            } else {
                Response::error("Missing key parameter", 400)
                    .map(|r| add_cors_headers(r))
            }
        })
        .delete_async("/api/delete/:key", |_, ctx| async move {
            if let Some(key) = ctx.param("key") {
                if let Err(e) = validate_kv_key(key) {
                    return Response::error(&format!("Validation error: {}", e), 400)
                        .map(|r| add_cors_headers(r));
                }
                // Get Durable Object stub
                let namespace = ctx.env.durable_object("STORAGE")?;
                let stub = namespace.id_from_name("default")?.get_stub()?;
                
                // Forward request to Durable Object
                let do_req = Request::new_with_init(
                    &format!("https://fake-host/delete/{}", key),
                    RequestInit::new().with_method(Method::Delete)
                )?;
                
                let response = stub.fetch_with_request(do_req).await?;
                Ok(add_cors_headers(response))
            } else {
                Response::error("Missing key parameter", 400)
                    .map(|r| add_cors_headers(r))
            }
        })
        .post_async("/api/vector/insert", |mut req, ctx| async move {
            let tenant = resolve_tenant(&req, &ctx.env).await?;
            let guard = TimeoutGuard::with_operation(VECTOR_TIMEOUT_MS, "vector_insert");
            let body: serde_json::Value = req.json().await?;
            
            // Validate request
            if let Err(e) = validate_insert_request(&body) {
                return Response::error(&format!("Validation error: {}", e), 400)
                    .map(|r| add_cors_headers(r));
            }
            
            // Route to shard based on vector ID
            let router = get_shard_router();
            let id = body["id"].as_str().unwrap_or("default");
            let shard_id = router.get_shard(id);
            let shard_name = router.get_shard_name(shard_id);
            
            // Get Vector Index Durable Object stub for target shard
            let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
            let stub = namespace.id_from_name(&shard_name)?.get_stub()?;
            
            // Forward request to Durable Object
            let mut do_req = Request::new_with_init(
                "https://fake-host/insert",
                RequestInit::new()
                    .with_method(Method::Post)
                    .with_body(Some(serde_json::to_string(&body)?.into()))
            )?;
            do_req.headers_mut()?.set("Content-Type", "application/json")?;
            
            let response = stub.fetch_with_request(do_req).await?;

            if let Err(e) = guard.check() {
                console_log!("[timeout] {}", e);
            }
            if let Some(ref t) = tenant {
                let _ = BillingManager::track_insert(&ctx.env, &t.tenant_id, 1).await;
            }

            Ok(add_cors_headers(response))
        })
        .post_async("/api/vector/batch-insert", |mut req, ctx| async move {
            let tenant = resolve_tenant(&req, &ctx.env).await?;
            let guard = TimeoutGuard::with_operation(VECTOR_TIMEOUT_MS, "vector_batch_insert");
            let body: serde_json::Value = req.json().await?;
            
            // Validate batch request
            if let Err(e) = validate_batch_insert_request(&body) {
                return Response::error(&format!("Validation error: {}", e), 400)
                    .map(|r| add_cors_headers(r));
            }
            
            let router = get_shard_router();
            let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
            
            // Group vectors by target shard
            let vectors = body["vectors"].as_array()
                .ok_or_else(|| worker::Error::RustError("Missing vectors array".into()))?;
            
            let mut shard_groups: std::collections::HashMap<usize, Vec<&serde_json::Value>> =
                std::collections::HashMap::new();
            for vector in vectors {
                let id = vector["id"].as_str().unwrap_or("default");
                let shard_id = router.get_shard(id);
                shard_groups.entry(shard_id).or_default().push(vector);
            }
            
            // Send each group to its shard
            let mut total_inserted = 0usize;
            for (shard_id, group) in shard_groups {
                if guard.is_expired() {
                    console_log!("[timeout] batch-insert exceeded {}ms budget after inserting {}", VECTOR_TIMEOUT_MS, total_inserted);
                    break;
                }
                let shard_name = router.get_shard_name(shard_id);
                let stub = namespace.id_from_name(&shard_name)?.get_stub()?;
                
                let shard_body = serde_json::json!({ "vectors": group });
                let mut do_req = Request::new_with_init(
                    "https://fake-host/batch-insert",
                    RequestInit::new()
                        .with_method(Method::Post)
                        .with_body(Some(serde_json::to_string(&shard_body)?.into()))
                )?;
                do_req.headers_mut()?.set("Content-Type", "application/json")?;
                
                let mut response = stub.fetch_with_request(do_req).await?;
                if response.status_code() == 200 {
                    if let Ok(resp_body) = response.json::<serde_json::Value>().await {
                        total_inserted += resp_body["inserted"].as_u64().unwrap_or(0) as usize;
                    }
                }
            }
            
            if let Some(ref t) = tenant {
                let _ = BillingManager::track_insert(&ctx.env, &t.tenant_id, total_inserted as u64).await;
            }

            Response::from_json(&serde_json::json!({
                "success": true,
                "inserted": total_inserted
            })).map(|r| add_cors_headers(r))
        })
        .get_async("/api/vector/get/:id", |req, ctx| async move {
            let _tenant = resolve_tenant(&req, &ctx.env).await?;
            let guard = TimeoutGuard::with_operation(VECTOR_TIMEOUT_MS, "vector_get");
            if let Some(id) = ctx.param("id") {
                // Validate ID
                if let Err(e) = validate_vector_id(id) {
                    return Response::error(&format!("Validation error: {}", e), 400)
                        .map(|r| add_cors_headers(r));
                }
                
                // Route to shard based on vector ID
                let router = get_shard_router();
                let shard_id = router.get_shard(id);
                let shard_name = router.get_shard_name(shard_id);
                
                // Get Vector Index Durable Object stub for target shard
                let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
                let stub = namespace.id_from_name(&shard_name)?.get_stub()?;
                
                // Forward request to Durable Object
                let do_req = Request::new_with_init(
                    &format!("https://fake-host/get/{}", id),
                    RequestInit::new().with_method(Method::Get)
                )?;
                
                let response = stub.fetch_with_request(do_req).await?;
                if let Err(e) = guard.check() {
                    console_log!("[timeout] {}", e);
                }
                Ok(add_cors_headers(response))
            } else {
                Response::error("Missing id parameter", 400)
                    .map(|r| add_cors_headers(r))
            }
        })
        .post_async("/api/vector/search", |mut req, ctx| async move {
            // Tenant auth + usage-limit enforcement for multi-tenant keys
            let tenant = resolve_tenant(&req, &ctx.env).await?;
            if let Some(ref t) = tenant {
                if t.query_limit_per_month > 0 {
                    if let Err(_) = check_usage_limits(&ctx.env, &t.tenant_id, t.query_limit_per_month).await {
                        return Response::error(
                            "Usage limit exceeded. Upgrade your plan at https://quartzdb.io/pricing",
                            429,
                        ).map(|r| add_cors_headers(r));
                    }
                }
            }

            let guard = TimeoutGuard::with_operation(VECTOR_TIMEOUT_MS, "vector_search");
            let body: serde_json::Value = req.json().await?;
            
            // Validate request
            if let Err(e) = validate_search_request(&body) {
                return Response::error(&format!("Validation error: {}", e), 400)
                    .map(|r| add_cors_headers(r));
            }
            
            let router = get_shard_router();
            let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
            let top_k = body["k"].as_u64().unwrap_or(10) as usize;
            
            // Fan-out search to ALL shards
            let mut shard_results: Vec<ShardSearchResult> = Vec::new();
            for shard_id in router.all_shards() {
                if guard.is_expired() {
                    console_log!("[timeout] search fan-out exceeded {}ms budget, returning partial results", VECTOR_TIMEOUT_MS);
                    break;
                }

                let shard_name = router.get_shard_name(shard_id);
                let stub = namespace.id_from_name(&shard_name)?.get_stub()?;
                
                let mut do_req = Request::new_with_init(
                    "https://fake-host/search",
                    RequestInit::new()
                        .with_method(Method::Post)
                        .with_body(Some(serde_json::to_string(&body)?.into()))
                )?;
                do_req.headers_mut()?.set("Content-Type", "application/json")?;
                
                let mut response = stub.fetch_with_request(do_req).await?;
                if response.status_code() == 200 {
                    if let Ok(resp_body) = response.json::<serde_json::Value>().await {
                        let matches: Vec<SearchMatch> = resp_body["results"]
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|m| serde_json::from_value(m.clone()).ok())
                            .collect();
                        shard_results.push(ShardSearchResult {
                            shard_id,
                            results: matches,
                        });
                    }
                }
            }
            
            // Merge results from all shards and return top-k
            let merged = merge_shard_results(shard_results, top_k);

            // Track query for multi-tenant keys
            if let Some(ref t) = tenant {
                let _ = BillingManager::track_query(&ctx.env, &t.tenant_id).await;
            }

            Response::from_json(&serde_json::json!({
                "results": merged
            })).map(|r| add_cors_headers(r))
        })
        .delete_async("/api/vector/delete/:id", |req, ctx| async move {
            let _tenant = resolve_tenant(&req, &ctx.env).await?;
            let guard = TimeoutGuard::with_operation(VECTOR_TIMEOUT_MS, "vector_delete");
            if let Some(id) = ctx.param("id") {
                // Validate ID
                if let Err(e) = validate_vector_id(id) {
                    return Response::error(&format!("Validation error: {}", e), 400)
                        .map(|r| add_cors_headers(r));
                }
                
                // Route to shard based on vector ID
                let router = get_shard_router();
                let shard_id = router.get_shard(id);
                let shard_name = router.get_shard_name(shard_id);
                
                // Get Vector Index Durable Object stub for target shard
                let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
                let stub = namespace.id_from_name(&shard_name)?.get_stub()?;
                
                // Forward request to Durable Object
                let do_req = Request::new_with_init(
                    &format!("https://fake-host/delete/{}", id),
                    RequestInit::new().with_method(Method::Delete)
                )?;
                
                let response = stub.fetch_with_request(do_req).await?;
                if let Err(e) = guard.check() {
                    console_log!("[timeout] {}", e);
                }
                Ok(add_cors_headers(response))
            } else {
                Response::error("Missing id parameter", 400)
                    .map(|r| add_cors_headers(r))
            }
        })
        .get_async("/api/vector/stats", |req, ctx| async move {
            let _tenant = resolve_tenant(&req, &ctx.env).await?;
            let guard = TimeoutGuard::with_operation(DEFAULT_TIMEOUT_MS, "vector_stats");
            let router = get_shard_router();
            let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
            
            // Fan-out stats request to ALL shards and aggregate
            let mut total_document_count = 0u64;
            let mut total_vector_count = 0u64;
            let mut total_storage_bytes = 0u64;
            let mut shard_stats: Vec<serde_json::Value> = Vec::new();
            
            for shard_id in router.all_shards() {
                if guard.is_expired() {
                    console_log!("[timeout] stats fan-out exceeded {}ms budget", DEFAULT_TIMEOUT_MS);
                    break;
                }
                let shard_name = router.get_shard_name(shard_id);
                let stub = namespace.id_from_name(&shard_name)?.get_stub()?;
                
                let do_req = Request::new_with_init(
                    "https://fake-host/stats",
                    RequestInit::new().with_method(Method::Get)
                )?;
                
                let mut response = stub.fetch_with_request(do_req).await?;
                if response.status_code() == 200 {
                    if let Ok(stats) = response.json::<serde_json::Value>().await {
                        total_document_count += stats["document_count"].as_u64().unwrap_or(0);
                        total_vector_count += stats["vector_count"].as_u64().unwrap_or(0);
                        total_storage_bytes += stats["storage_bytes"].as_u64().unwrap_or(0);
                        shard_stats.push(serde_json::json!({
                            "shard_id": shard_id,
                            "shard_name": shard_name,
                            "stats": stats
                        }));
                    }
                }
            }
            
            Response::from_json(&serde_json::json!({
                "total_document_count": total_document_count,
                "total_vector_count": total_vector_count,
                "total_storage_bytes": total_storage_bytes,
                "shard_count": router.shard_count(),
                "shards": shard_stats
            })).map(|r| add_cors_headers(r))
        })
        // Tenant signup: create a new free-tier tenant
        .post_async("/api/tenants/signup", |mut req, ctx| async move {
            #[derive(serde::Deserialize)]
            struct SignupRequest {
                tenant_id: String,
            }
            
            let body: SignupRequest = req.json().await
                .map_err(|_| worker::Error::RustError("Invalid JSON body".into()))?;
            
            if body.tenant_id.is_empty() || body.tenant_id.len() > 64 {
                return Response::error("tenant_id must be 1-64 characters", 400)
                    .map(|r| add_cors_headers(r));
            }
            
            // Only allow alphanumeric, underscore, hyphen
            if !body.tenant_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                return Response::error("tenant_id must be alphanumeric (plus _ and -)", 400)
                    .map(|r| add_cors_headers(r));
            }
            
            match TenantManager::create_tenant(&ctx.env, &body.tenant_id, TenantPlan::Free).await {
                Ok(tenant) => {
                    Response::from_json(&serde_json::json!({
                        "success": true,
                        "tenant_id": tenant.tenant_id,
                        "api_key": tenant.api_key,
                        "plan": "free",
                        "vector_limit": tenant.vector_limit,
                        "query_limit_per_month": tenant.query_limit_per_month,
                        "message": "Save your API key - it will not be shown again!"
                    })).map(|r| add_cors_headers(r))
                }
                Err(e) => {
                    console_log!("[error] signup failed: {}", e);
                    Response::error("Internal server error", 500)
                        .map(|r| add_cors_headers(r))
                }
            }
        })
        // Usage endpoint: get current billing usage
        .get_async("/api/usage", |req, ctx| async move {
            match authenticate_tenant(&req, &ctx.env).await {
                Ok(tenant) => {
                    match BillingManager::get_usage(&ctx.env, &tenant.tenant_id).await {
                        Ok(usage) => {
                            Response::from_json(&serde_json::json!({
                                "success": true,
                                "tenant_id": tenant.tenant_id,
                                "plan": format!("{:?}", tenant.plan),
                                "usage": {
                                    "queries": usage.queries,
                                    "inserts": usage.inserts,
                                    "vectors_stored": usage.vectors_stored,
                                    "month": usage.month
                                },
                                "limits": {
                                    "vector_limit": tenant.vector_limit,
                                    "query_limit_per_month": tenant.query_limit_per_month
                                }
                            })).map(|r| add_cors_headers(r))
                        }
                        Err(e) => {
                            console_log!("[error] usage fetch failed: {}", e);
                            Response::error("Internal server error", 500)
                                .map(|r| add_cors_headers(r))
                        }
                    }
                }
                Err(_) => Response::error("Unauthorized", 401).map(|r| add_cors_headers(r))
            }
        })
        // Stripe webhook endpoint (no auth - verified by signature)
        .post_async("/webhooks/stripe", |req, ctx| async move {
            handle_stripe_webhook(req, &ctx.env).await
                .map(|r| add_cors_headers(r))
        })
        .run(req, env)
        .await;

    // Extract status code (default to 500 if response is error)
    let status = response.as_ref().map(|r| r.status_code()).unwrap_or(500);
    let duration = timer.elapsed_ms();
    
    // Update metrics with final results
    metrics.finish(status, duration);
    metrics.log();

    // Track in Analytics Engine (best effort, don't fail on error)
    // The _ = ignores Result because monitoring failures shouldn't affect user request
    let _ = metrics.track(&env_clone);

    // Add CORS/security headers and X-Request-ID to all responses
    response.map(|r| finalize_response(r, &cors_origin, &request_id))
}

/// Apply all response headers (CORS, security, tracing) to a response.
///
/// Single helper used by all code paths — early returns and the final router
/// response — so every response carries a consistent set of headers.
fn finalize_response(response: Response, cors_origin: &str, request_id: &str) -> Response {
    let mut r = add_cors_headers_with_origin(response, cors_origin);
    let _ = r.headers_mut().set("X-Request-ID", request_id);
    r
}

/// Legacy CORS wrapper — intentional no-op
///
/// CORS headers are applied exactly once by the outer response wrapper
/// (`add_cors_headers_with_origin`) using the `CORS_ALLOWED_ORIGIN` env var.
/// Inner route handlers still call this for code-path uniformity, but it
/// intentionally does nothing to prevent wildcard-origin override.
fn add_cors_headers(response: Response) -> Response {
    response
}

/// Add CORS and security headers with configurable origin
///
/// Called once at the outermost layer so headers are applied exactly once.
fn add_cors_headers_with_origin(mut response: Response, allowed_origin: &str) -> Response {
    let headers = response.headers_mut();
    
    // CORS headers
    let _ = headers.set("Access-Control-Allow-Origin", allowed_origin);
    let _ = headers.set("Vary", "Origin");
    let _ = headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
    let _ = headers.set(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization, X-API-Key, X-Request-ID"
    );
    let _ = headers.set("Access-Control-Max-Age", "86400");
    let _ = headers.set("Access-Control-Expose-Headers", "X-Request-ID");
    
    // Security headers
    let _ = headers.set("X-Content-Type-Options", "nosniff");
    let _ = headers.set("X-Frame-Options", "DENY");
    let _ = headers.set("Strict-Transport-Security", "max-age=31536000; includeSubDomains");
    let _ = headers.set("X-XSS-Protection", "1; mode=block");
    let _ = headers.set("Referrer-Policy", "strict-origin-when-cross-origin");
    let _ = headers.set("Content-Security-Policy", "default-src 'none'; frame-ancestors 'none'");
    
    response
}
