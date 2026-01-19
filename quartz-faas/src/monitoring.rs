//! Monitoring and Analytics for QuartzDB
//!
//! # Overview
//!
//! Provides observability into QuartzDB operations through:
//! - Request metrics (latency, status, path)
//! - Console logging (visible in wrangler tail)
//! - Analytics Engine integration (structured analytics)
//! - Health checks for Durable Objects
//!
//! # Design Philosophy: Best-Effort Monitoring
//!
//! **Key Principle**: Never fail a user request due to monitoring failures
//!
//! All monitoring operations:
//! - Use Result<()> but ignore errors in calling code
//! - Log warnings on failure but continue processing
//! - Degrade gracefully if Analytics Engine unavailable
//!
//! Why? Monitoring is observability, not critical path. User data
//! operations must succeed even if analytics fail.
//!
//! # Analytics Engine Architecture
//!
//! ```text
//! Request Processing
//!        ↓
//! Timer Start
//!        ↓
//! Handle Request
//!        ↓
//! Timer Stop
//!        ↓
//! Log to Console
//!        ↓
//! Write to Analytics Engine (best-effort)
//!        ↓
//! Return Response
//! ```
//!
//! # Metrics Collected
//!
//! - **Latency**: End-to-end request duration (ms)
//! - **Status**: HTTP status code (2xx, 4xx, 5xx)
//! - **Method**: HTTP method (GET, POST, DELETE)
//! - **Path**: Request path (/api/put, /vector/search, etc.)
//! - **Version**: Application version (for A/B testing)
//!
//! # Performance Impact
//!
//! - Timer operations: <0.1ms (JS Date.now)
//! - Console logging: ~0.5-1ms (async I/O)
//! - Analytics write: ~1-2ms (async, non-blocking)
//! - Total overhead: ~2-3ms per request
//!
//! This is acceptable for our use case where DO operations are ~5-10ms.

use std::sync::atomic::{AtomicU64, Ordering};
use worker::*;

static STARTUP_TIME: AtomicU64 = AtomicU64::new(0);

/// Track request metrics for observability
///
/// # Usage Pattern:
///
/// ```ignore
/// let metrics = RequestMetrics::new(method, path);
/// let timer = Timer::new();
/// 
/// // ... handle request ...
/// 
/// let status = response.status_code();
/// let duration = timer.elapsed_ms();
/// metrics.finish(status, duration);
/// metrics.log();                    // Console output
/// let _ = metrics.track(&env);      // Analytics Engine (best-effort)
/// ```
///
/// # Why Separate finish() and log()/track()?
///
/// - **finish()**: Updates mutable state with results
/// - **log()**: Immutable read for console output
/// - **track()**: Immutable read for analytics
///
/// This separation allows flexible composition and clear ownership.
pub struct RequestMetrics {
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u128,
    pub timestamp: u64,
}

impl RequestMetrics {
    pub fn new(method: String, path: String) -> Self {
        Self {
            method,
            path,
            status: 0,
            duration_ms: 0,
            timestamp: js_sys::Date::now() as u64,
        }
    }

    pub fn finish(&mut self, status: u16, duration_ms: u128) {
        self.status = status;
        self.duration_ms = duration_ms;
    }

    /// Log metrics to console (visible in wrangler tail)
    pub fn log(&self) {
        let log_level = if self.status >= 500 {
            "ERROR"
        } else if self.status >= 400 {
            "WARN"
        } else if self.duration_ms > 500 {
            "WARN"
        } else {
            "INFO"
        };

        console_log!(
            "[{}] {} {} - {} ({}ms)",
            log_level,
            self.method,
            self.path,
            self.status,
            self.duration_ms
        );
    }

    /// Send metrics to Cloudflare Analytics Engine
    ///
    /// # What is Analytics Engine?
    ///
    /// Time-series analytics database provided by Cloudflare:
    /// - **Indexes**: Up to 20 numeric fields (fast aggregation)
    /// - **Blobs**: Up to 20 string fields (filtering)
    /// - **Retention**: 3+ months
    /// - **Querying**: SQL via GraphQL API
    ///
    /// # Our Schema:
    ///
    /// - `index1`: version (string) - for A/B testing
    /// - `double1`: HTTP status code - for error rate tracking
    /// - `double2`: latency (ms) - for performance monitoring
    /// - `blob1`: HTTP method - for request pattern analysis
    /// - `blob2`: request path - for endpoint-specific metrics
    ///
    /// # Why This Schema?
    ///
    /// - Status as double: Easy to query (WHERE double1 >= 500 for errors)
    /// - Latency as double: Aggregations (AVG, P50, P95, P99)
    /// - Path as blob: Exact match queries (WHERE blob2 = '/api/vector/search')
    ///
    /// # Error Handling:
    ///
    /// Returns Ok(()) even on failure because:
    /// - Monitoring failures shouldn't affect user requests
    /// - Analytics Engine might not be configured (local dev)
    /// - Network issues to analytics shouldn't break the app
    ///
    /// Errors are logged as warnings but not propagated.
    ///
    /// See: https://developers.cloudflare.com/analytics/analytics-engine/
    pub fn track(&self, env: &Env) -> Result<()> {
        // Try to get Analytics Engine dataset binding
        match env.analytics_engine("ANALYTICS") {
            Ok(dataset) => {
                // Build data point with indexes (fast queries) and blobs (context)
                // Indexes: numeric values for fast aggregation
                // Blobs: string/binary data for filtering
                match AnalyticsEngineDataPointBuilder::new()
                    .indexes(&[env!("CARGO_PKG_VERSION")])  // index1: version
                    .add_double(self.status as f64)          // double1: HTTP status
                    .add_double(self.duration_ms as f64)     // double2: latency
                    .add_blob(self.method.as_str())          // blob1: HTTP method
                    .add_blob(self.path.as_str())            // blob2: request path
                    .write_to(&dataset)
                {
                    Ok(_) => {},
                    Err(e) => {
                        // Best-effort - don't fail request if analytics fail
                        console_warn!("Analytics Engine write failed: {:?}", e);
                    }
                }
            },
            Err(_) => {
                // Analytics Engine not configured - acceptable for local development
                // Production should have [[analytics_engine_datasets]] in wrangler.toml
            }
        }
        Ok(())
    }
}

/// Timer for measuring operation duration using JS Date
pub struct Timer {
    start: f64,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: js_sys::Date::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        (js_sys::Date::now() - self.start) as u128
    }
}

/// Check if Storage Durable Object is accessible
///
/// # Health Check Strategy:
///
/// 1. Get Durable Object namespace from env
/// 2. Get DO stub (doesn't actually call the DO)
/// 3. Return success if stub can be created
///
/// # Why Not Call the DO?
///
/// - Getting stub is fast (<1ms)
/// - Actually calling DO adds 5-10ms latency
/// - If stub creation succeeds, DO is likely healthy
/// - Health endpoint is called frequently (monitoring)
///
/// # Trade-offs:
///
/// - **Pro**: Fast health check (<1ms)
/// - **Con**: Doesn't verify DO is actually responding
/// - **Decision**: Speed over exhaustive checking
///
/// For more thorough health checks, call DO's /health endpoint.
pub async fn check_storage_health(env: &Env) -> bool {
    match env.durable_object("STORAGE") {
        Ok(namespace) => {
            match namespace.id_from_name("health-check") {
                Ok(id) => {
                    match id.get_stub() {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                }
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

pub async fn check_vector_health(env: &Env) -> bool {
    match env.durable_object("VECTOR_INDEX") {
        Ok(namespace) => {
            match namespace.id_from_name("health-check") {
                Ok(id) => {
                    match id.get_stub() {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                }
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// Get system uptime (approximation based on worker lifetime)
pub fn init_uptime() {
    let current = STARTUP_TIME.load(Ordering::Relaxed);
    if current == 0 {
        STARTUP_TIME.store(js_sys::Date::now() as u64, Ordering::Relaxed);
    }
}

pub fn get_uptime_seconds() -> u64 {
    let start = STARTUP_TIME.load(Ordering::Relaxed);
    if start == 0 {
        return 0;
    }
    let now = js_sys::Date::now() as u64;
    (now - start) / 1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_timer() {
        let timer = Timer::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
    }

    // Note: RequestMetrics tests require WASM environment (js_sys::Date)
    // They are tested during actual Worker deployment
}
