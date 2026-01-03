//! Monitoring and analytics utilities

use std::sync::atomic::{AtomicU64, Ordering};
use worker::*;

static STARTUP_TIME: AtomicU64 = AtomicU64::new(0);

/// Track request metrics
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

    /// Send to Analytics Engine (if enabled)
    /// Note: Analytics Engine implementation is optional
    /// For now, we rely on console_log which appears in wrangler tail
    pub fn track(&self, _env: &Env) -> Result<()> {
        // TODO: Implement Analytics Engine when the API is stable
        // For now, console_log (from the log() method) is sufficient
        // and will show up in wrangler tail
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

/// Health check for Durable Objects
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
    fn test_timer() {
        let timer = Timer::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
    }

    // Note: RequestMetrics tests require WASM environment (js_sys::Date)
    // They are tested during actual Worker deployment
}
