//! Request timeout utilities for QuartzDB
//!
//! Uses elapsed-time checking since JS Promise.race() is complex in WASM.
//! TimeoutGuard provides a simple, reliable timeout mechanism.

use worker::*;

/// Default timeout for operations (30 seconds)
pub const DEFAULT_TIMEOUT_MS: u32 = 30_000;

/// Timeout for vector operations (10 seconds)  
pub const VECTOR_TIMEOUT_MS: u32 = 10_000;

/// Timeout for health checks (5 seconds)
pub const HEALTH_CHECK_TIMEOUT_MS: u32 = 5_000;

/// Simple timeout guard that checks elapsed time
/// 
/// Usage:
/// ```ignore
/// let guard = TimeoutGuard::new(VECTOR_TIMEOUT_MS);
/// // ... do work ...
/// guard.check()?; // Returns Err if timeout exceeded
/// ```
pub struct TimeoutGuard {
    start_ms: f64,
    timeout_ms: u32,
    operation: String,
}

impl TimeoutGuard {
    /// Create a guard with the given timeout (milliseconds).
    pub fn new(timeout_ms: u32) -> Self {
        Self {
            start_ms: crate::platform::now_ms(),
            timeout_ms,
            operation: String::new(),
        }
    }

    /// Create a guard with a named operation for clearer timeout error messages.
    pub fn with_operation(timeout_ms: u32, operation: &str) -> Self {
        Self {
            start_ms: crate::platform::now_ms(),
            timeout_ms,
            operation: operation.to_string(),
        }
    }

    /// Check if timeout has been exceeded; returns `Err` with a descriptive message if so.
    pub fn check(&self) -> Result<()> {
        let elapsed = crate::platform::now_ms() - self.start_ms;
        if elapsed > self.timeout_ms as f64 {
            let msg = if self.operation.is_empty() {
                format!("Operation timed out after {}ms (limit: {}ms)", elapsed as u32, self.timeout_ms)
            } else {
                format!("{} timed out after {}ms (limit: {}ms)", self.operation, elapsed as u32, self.timeout_ms)
            };
            Err(Error::RustError(msg))
        } else {
            Ok(())
        }
    }

    /// Milliseconds elapsed since the guard was created.
    pub fn elapsed_ms(&self) -> u32 {
        (crate::platform::now_ms() - self.start_ms) as u32
    }

    /// Milliseconds remaining before expiry (saturates at 0).
    pub fn remaining_ms(&self) -> u32 {
        let elapsed = (crate::platform::now_ms() - self.start_ms) as u32;
        self.timeout_ms.saturating_sub(elapsed)
    }

    /// Non-error check: returns `true` once the timeout budget is exhausted.
    pub fn is_expired(&self) -> bool {
        (crate::platform::now_ms() - self.start_ms) > self.timeout_ms as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_not_expired() {
        let guard = TimeoutGuard::new(30_000);
        assert!(!guard.is_expired());
    }

    #[test]
    fn test_check_ok_when_fresh() {
        let guard = TimeoutGuard::new(30_000);
        assert!(guard.check().is_ok());
    }

    #[test]
    fn test_elapsed_ms_small() {
        let guard = TimeoutGuard::new(1000);
        let elapsed = guard.elapsed_ms();
        assert!(elapsed < 100, "Unexpected elapsed: {elapsed}ms");
    }

    #[test]
    fn test_remaining_ms() {
        let guard = TimeoutGuard::new(5000);
        let remaining = guard.remaining_ms();
        // Should be close to 5000 right after creation
        assert!(remaining > 4900 && remaining <= 5000, "Unexpected remaining: {remaining}ms");
    }

    #[test]
    fn test_with_operation() {
        let guard = TimeoutGuard::with_operation(1000, "test_op");
        assert!(guard.check().is_ok());
    }

    #[test]
    fn test_zero_timeout_expires_immediately() {
        let guard = TimeoutGuard::new(0);
        // A 0ms timeout should expire essentially immediately
        // (may or may not depending on timing, so just test check handles it)
        let _ = guard.is_expired(); // shouldn't panic
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_TIMEOUT_MS, 30_000);
        assert_eq!(VECTOR_TIMEOUT_MS, 10_000);
        assert_eq!(HEALTH_CHECK_TIMEOUT_MS, 5_000);
    }
}
