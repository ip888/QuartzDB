//! Rate limiting for QuartzDB
//!
//! # Strategy
//!
//! 1. **Cloudflare Rate Limiting** (primary, configured in wrangler.toml)
//!    - 100 requests/minute per IP (free tier)
//!    - Blocks at edge before reaching worker
//!
//! 2. **Custom Token Bucket** (secondary, per-API-key)
//!    - Stored in Durable Object state
//!    - Refills over time
//!    - Protects against single abusive key
//!
//! # Implementation
//!
//! Token bucket algorithm:
//! - Each API key gets N tokens
//! - Tokens refill at rate R per second
//! - Each request consumes 1 token
//! - If no tokens, reject with 429

use worker::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum tokens in bucket
    pub capacity: u32,
    /// Tokens refilled per second
    pub refill_rate: f64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            capacity: 100,       // 100 requests
            refill_rate: 1.67,   // ~100 per minute
        }
    }
}

/// Token bucket state for one API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBucket {
    /// Current tokens available
    pub tokens: f64,
    /// Last refill timestamp (milliseconds since epoch)
    pub last_refill: f64,
    /// Configuration
    pub config: RateLimitConfig,
}

impl TokenBucket {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            tokens: config.capacity as f64,
            last_refill: js_sys::Date::now(),
            config,
        }
    }
    
    /// Refill tokens based on elapsed time
    pub fn refill(&mut self) {
        let now = js_sys::Date::now();
        let elapsed_seconds = (now - self.last_refill) / 1000.0;
        
        // Add tokens based on elapsed time
        let new_tokens = elapsed_seconds * self.config.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.config.capacity as f64);
        self.last_refill = now;
    }
    
    /// Try to consume one token
    ///
    /// Returns true if successful (has tokens), false if rate limited
    pub fn try_consume(&mut self) -> bool {
        self.refill();
        
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
    
    /// Get remaining tokens
    pub fn available_tokens(&mut self) -> u32 {
        self.refill();
        self.tokens.floor() as u32
    }
}

/// Rate limiter managing multiple API keys
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RateLimiter {
    /// Token buckets per API key (or IP for anonymous)
    buckets: HashMap<String, TokenBucket>,
    /// Default configuration
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            buckets: HashMap::new(),
            config,
        }
    }
    
    /// Check if request allowed and consume token
    ///
    /// Returns (allowed, remaining_tokens, retry_after_seconds)
    pub fn check_rate_limit(&mut self, identifier: &str) -> (bool, u32, Option<u32>) {
        let bucket = self.buckets
            .entry(identifier.to_string())
            .or_insert_with(|| TokenBucket::new(self.config.clone()));
        
        let allowed = bucket.try_consume();
        let remaining = bucket.available_tokens();
        
        let retry_after = if !allowed {
            // Calculate seconds until 1 token available
            let needed_tokens = 1.0 - bucket.tokens;
            let seconds = (needed_tokens / self.config.refill_rate).ceil() as u32;
            Some(seconds.max(1))
        } else {
            None
        };
        
        (allowed, remaining, retry_after)
    }
    
    /// Get stats for monitoring
    pub fn get_stats(&self) -> RateLimiterStats {
        RateLimiterStats {
            num_tracked_keys: self.buckets.len(),
            config: self.config.clone(),
        }
    }
    
    /// Clean up old buckets (full capacity = inactive)
    pub fn cleanup_inactive(&mut self) {
        self.buckets.retain(|_, bucket| {
            bucket.refill();
            bucket.tokens < bucket.config.capacity as f64
        });
    }
}

#[derive(Debug, Serialize)]
pub struct RateLimiterStats {
    pub num_tracked_keys: usize,
    pub config: RateLimitConfig,
}

/// Simple check using request headers (for early validation)
///
/// Checks:
/// - Cloudflare rate limit headers (if present)
/// - Returns Ok if allowed, Err with 429 if blocked
pub fn check_cloudflare_rate_limit(req: &Request) -> Result<()> {
    let headers = req.headers();
    
    // Cloudflare adds these headers when rate limiting is active
    if let Ok(Some(remaining)) = headers.get("CF-RateLimit-Remaining") {
        if remaining == "0" {
            return Err(Error::RustError("Rate limit exceeded".to_string()));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::new(RateLimitConfig {
            capacity: 10,
            refill_rate: 1.0,
        });
        
        // Should allow 10 requests
        for _ in 0..10 {
            assert!(bucket.try_consume());
        }
        
        // 11th should fail
        assert!(!bucket.try_consume());
    }
    
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(RateLimitConfig {
            capacity: 5,
            refill_rate: 1.0,
        });
        
        // First 5 requests from key1 should succeed
        for _ in 0..5 {
            let (allowed, _, _) = limiter.check_rate_limit("key1");
            assert!(allowed);
        }
        
        // 6th should fail
        let (allowed, remaining, retry_after) = limiter.check_rate_limit("key1");
        assert!(!allowed);
        assert_eq!(remaining, 0);
        assert!(retry_after.is_some());
        
        // Different key should have own bucket
        let (allowed, _, _) = limiter.check_rate_limit("key2");
        assert!(allowed);
    }
}
