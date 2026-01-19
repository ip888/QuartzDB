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
mod error;
mod durable;
mod monitoring;
mod ratelimit;
mod timeout;
mod validation;
mod vector;
mod sharding;

pub use api::*;
pub use auth::*;
pub use error::*;
pub use monitoring::*;
pub use ratelimit::*;
pub use timeout::*;
pub use validation::*;
pub use vector::*;
pub use sharding::*;

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
    let timer = Timer::new();
    
    // Clone env for analytics (router consumes original env)
    // This is cheap - Env is just a handle to the JS environment
    let env_clone = env.clone();
    
    // Handle CORS preflight
    if method == "OPTIONS" {
        return Response::ok("")
            .map(|r| add_cors_headers(r));
    }
    
    // Rate limiting check (before auth, uses IP or API key)
    if is_protected_path(&path) {
        if let Err(_) = check_cloudflare_rate_limit(&req) {
            metrics.finish(429, timer.elapsed_ms());
            metrics.log();
            let _ = metrics.track(&env_clone);
            
            return Response::error("Too Many Requests", 429)
                .map(|r| {
                    let mut r = add_cors_headers(r);
                    let _ = r.headers_mut().set("Retry-After", "60");
                    r
                });
        }
    }
    
    // Authentication check (before routing)
    if is_protected_path(&path) {
        match require_auth(&req, &env) {
            Ok(_) => {},
            Err(_) => {
                metrics.finish(401, timer.elapsed_ms());
                metrics.log();
                let _ = metrics.track(&env_clone);
                
                return Response::error("Unauthorized: Invalid or missing API key", 401)
                    .map(|r| add_cors_headers(r));
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
            let body: serde_json::Value = req.json().await?;
            
            // Validate request
            if let Err(e) = validate_insert_request(&body) {
                return Response::error(&format!("Validation error: {}", e), 400)
                    .map(|r| add_cors_headers(r));
            }
            
            // Get Vector Index Durable Object stub
            let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
            let stub = namespace.id_from_name("default")?.get_stub()?;
            
            // Forward request to Durable Object
            let mut do_req = Request::new_with_init(
                "https://fake-host/insert",
                RequestInit::new()
                    .with_method(Method::Post)
                    .with_body(Some(serde_json::to_string(&body)?.into()))
            )?;
            do_req.headers_mut()?.set("Content-Type", "application/json")?;
            
            let response = stub.fetch_with_request(do_req).await?;
            Ok(add_cors_headers(response))
        })
        .post_async("/api/vector/batch-insert", |mut req, ctx| async move {
            let body: serde_json::Value = req.json().await?;
            
            // Validate batch request
            if let Err(e) = validate_batch_insert_request(&body) {
                return Response::error(&format!("Validation error: {}", e), 400)
                    .map(|r| add_cors_headers(r));
            }
            
            // Get Vector Index Durable Object stub
            let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
            let stub = namespace.id_from_name("default")?.get_stub()?;
            
            // Forward request to Durable Object
            let mut do_req = Request::new_with_init(
                "https://fake-host/batch-insert",
                RequestInit::new()
                    .with_method(Method::Post)
                    .with_body(Some(serde_json::to_string(&body)?.into()))
            )?;
            do_req.headers_mut()?.set("Content-Type", "application/json")?;
            
            let response = stub.fetch_with_request(do_req).await?;
            Ok(add_cors_headers(response))
        })
        .get_async("/api/vector/get/:id", |_, ctx| async move {
            if let Some(id) = ctx.param("id") {
                // Validate ID
                if let Err(e) = validate_vector_id(id) {
                    return Response::error(&format!("Validation error: {}", e), 400)
                        .map(|r| add_cors_headers(r));
                }
                
                // Get Vector Index Durable Object stub
                let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
                let stub = namespace.id_from_name("default")?.get_stub()?;
                
                // Forward request to Durable Object
                let do_req = Request::new_with_init(
                    &format!("https://fake-host/get/{}", id),
                    RequestInit::new().with_method(Method::Get)
                )?;
                
                let response = stub.fetch_with_request(do_req).await?;
                Ok(add_cors_headers(response))
            } else {
                Response::error("Missing id parameter", 400)
                    .map(|r| add_cors_headers(r))
            }
        })
        .post_async("/api/vector/search", |mut req, ctx| async move {
            let body: serde_json::Value = req.json().await?;
            
            // Validate request
            if let Err(e) = validate_search_request(&body) {
                return Response::error(&format!("Validation error: {}", e), 400)
                    .map(|r| add_cors_headers(r));
            }
            
            // Get Vector Index Durable Object stub
            let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
            let stub = namespace.id_from_name("default")?.get_stub()?;
            
            // Forward request to Durable Object
            let mut do_req = Request::new_with_init(
                "https://fake-host/search",
                RequestInit::new()
                    .with_method(Method::Post)
                    .with_body(Some(serde_json::to_string(&body)?.into()))
            )?;
            do_req.headers_mut()?.set("Content-Type", "application/json")?;
            
            let response = stub.fetch_with_request(do_req).await?;
            Ok(add_cors_headers(response))
        })
        .delete_async("/api/vector/delete/:id", |_, ctx| async move {
            if let Some(id) = ctx.param("id") {
                // Validate ID
                if let Err(e) = validate_vector_id(id) {
                    return Response::error(&format!("Validation error: {}", e), 400)
                        .map(|r| add_cors_headers(r));
                }
                
                // Get Vector Index Durable Object stub
                let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
                let stub = namespace.id_from_name("default")?.get_stub()?;
                
                // Forward request to Durable Object
                let do_req = Request::new_with_init(
                    &format!("https://fake-host/delete/{}", id),
                    RequestInit::new().with_method(Method::Delete)
                )?;
                
                let response = stub.fetch_with_request(do_req).await?;
                Ok(add_cors_headers(response))
            } else {
                Response::error("Missing id parameter", 400)
                    .map(|r| add_cors_headers(r))
            }
        })
        .get_async("/api/vector/stats", |_, ctx| async move {
            // Get Vector Index Durable Object stub
            let namespace = ctx.env.durable_object("VECTOR_INDEX")?;
            let stub = namespace.id_from_name("default")?.get_stub()?;
            
            // Forward request to Durable Object
            let do_req = Request::new_with_init(
                "https://fake-host/stats",
                RequestInit::new().with_method(Method::Get)
            )?;
            
            let response = stub.fetch_with_request(do_req).await?;
            Ok(add_cors_headers(response))
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

    // Add CORS headers to all responses
    response.map(|r| add_cors_headers(r))
}

/// Add CORS headers to response
///
/// Allows cross-origin requests from web applications
fn add_cors_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    
    // Allow all origins (production should restrict this)
    let _ = headers.set("Access-Control-Allow-Origin", "*");
    
    // Allow common methods
    let _ = headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
    
    // Allow common headers
    let _ = headers.set(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization, X-API-Key"
    );
    
    // Cache preflight for 24 hours
    let _ = headers.set("Access-Control-Max-Age", "86400");
    
    response
}
