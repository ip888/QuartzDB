//! QuartzDB FaaS - Cloudflare Workers Integration
//!
//! Provides a serverless edge API for QuartzDB operations.
//! Designed to run on Cloudflare Workers with Durable Objects for state management.

use worker::*;

mod api;
mod error;
mod durable;
mod monitoring;

pub use api::*;
pub use error::*;
pub use monitoring::*;

/// Main entry point for Cloudflare Worker
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    init_uptime();

    // Track request metrics
    let method = req.method().to_string();
    let path = req.path();
    let mut metrics = RequestMetrics::new(method, path.clone());
    let timer = Timer::new();
    
    // Clone env for later use
    let env_clone = env.clone();

    // Setup router
    let response = Router::new()
        .get("/", |_, _| Response::ok("QuartzDB FaaS API v0.1.0"))
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
                Response::from_json(&response_json)
            } else {
                Ok(Response::from_json(&response_json)?
                    .with_status(503))
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
            
            stub.fetch_with_request(do_req).await
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
                
                stub.fetch_with_request(do_req).await
            } else {
                Response::error("Missing key parameter", 400)
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
                
                stub.fetch_with_request(do_req).await
            } else {
                Response::error("Missing key parameter", 400)
            }
        })
        .post_async("/api/vector/insert", |mut req, ctx| async move {
            let body: serde_json::Value = req.json().await?;
            
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
            
            stub.fetch_with_request(do_req).await
        })
        .post_async("/api/vector/search", |mut req, ctx| async move {
            let body: serde_json::Value = req.json().await?;
            
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
            
            stub.fetch_with_request(do_req).await
        })
        .run(req, env)
        .await;

    // Log metrics
    let status = response.as_ref().map(|r| r.status_code()).unwrap_or(500);
    let duration = timer.elapsed_ms();
    metrics.finish(status, duration);
    metrics.log();

    // Track in Analytics Engine (best effort, don't fail on error)
    let _ = metrics.track(&env_clone);

    response
}
