//! QuartzDB FaaS - Cloudflare Workers Integration
//!
//! Provides a serverless edge API for QuartzDB operations.
//! Designed to run on Cloudflare Workers with Durable Objects for state management.

use worker::*;

mod api;
mod error;
mod durable;

pub use api::*;
pub use error::*;
pub use durable::*;

/// Main entry point for Cloudflare Worker
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    // Setup router
    Router::new()
        .get("/", |_, _| Response::ok("QuartzDB FaaS API v0.1.0"))
        .get_async("/health", |_, _| async move {
            Response::from_json(&serde_json::json!({
                "status": "healthy",
                "service": "quartz-faas",
                "version": env!("CARGO_PKG_VERSION")
            }))
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
        .await
}
