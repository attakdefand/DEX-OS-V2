//! Rate limiting implementation using the Token Bucket algorithm.
//!
//! This module provides a thread-safe rate limiter to protect the API from abuse.
//! It supports:
//! - Configurable rate (tokens per second) and burst capacity.
//! - Per-key limiting (e.g., per IP or per User ID).
//! - Automatic cleanup of stale buckets to prevent memory leaks.
//! - Integration with SRE Patterns for handling overload conditions.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during rate limiting operations.
#[derive(Error, Debug, PartialEq)]
pub enum RateLimitError {
    /// Invalid rate limit configuration.
    #[error("Invalid rate limit configuration: {0}")]
    InvalidConfiguration(String),
    
    /// Rate limit exceeded.
    #[error("Rate limit exceeded for key: {0}")]
    RateLimitExceeded(String),
}

/// Configuration for a rate limit rule.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RateLimitConfig {
    /// Maximum number of tokens in the bucket (burst capacity).
    pub capacity: u32,
    /// Number of tokens refilled per second.
    pub refill_rate_per_sec: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            capacity: 100,
            refill_rate_per_sec: 10,
        }
    }
}

impl RateLimitConfig {
    /// Create a new rate limit configuration.
    pub fn new(capacity: u32, refill_rate_per_sec: u32) -> Result<Self, RateLimitError> {
        if capacity == 0 || refill_rate_per_sec == 0 {
            return Err(RateLimitError::InvalidConfiguration(
                "Capacity and refill rate must be greater than 0".to_string()
            ));
        }
        
        Ok(Self {
            capacity,
            refill_rate_per_sec,
        })
    }
}

/// A single token bucket.
#[derive(Debug)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

impl Bucket {
    fn new(capacity: u32) -> Self {
        Self {
            tokens: capacity as f64,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self, config: &RateLimitConfig) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = elapsed * config.refill_rate_per_sec as f64;

        if tokens_to_add > 0.0 {
            self.tokens = (self.tokens + tokens_to_add).min(config.capacity as f64);
            self.last_refill = now;
        }
    }

    /// Attempt to consume tokens. Returns true if successful.
    fn try_consume(&mut self, tokens: u32, config: &RateLimitConfig) -> bool {
        self.refill(config);
        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            true
        } else {
            false
        }
    }
    
    /// Get the current number of tokens in the bucket.
    fn tokens(&self) -> f64 {
        self.tokens
    }
}

/// Thread-safe Rate Limiter.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new RateLimiter with the given configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Check if a request for the given key is allowed.
    /// Consumes 1 token if allowed.
    pub fn check(&self, key: &str) -> Result<bool, RateLimitError> {
        self.check_n(key, 1)
    }

    /// Check if a request for the given key is allowed, consuming `n` tokens.
    pub fn check_n(&self, key: &str, tokens: u32) -> Result<bool, RateLimitError> {
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| Bucket::new(self.config.capacity));
        
        if bucket.try_consume(tokens, &self.config) {
            Ok(true)
        } else {
            Err(RateLimitError::RateLimitExceeded(key.to_string()))
        }
    }

    /// Remove stale buckets that haven't been used for a while.
    /// This prevents memory leaks from ephemeral keys (like random IPs).
    pub fn cleanup(&self, max_idle: Duration) {
        let mut buckets = self.buckets.lock().unwrap();
        let now = Instant::now();
        buckets.retain(|_, bucket| now.duration_since(bucket.last_refill) < max_idle);
    }
    
    /// Get statistics for a specific key.
    pub fn get_stats(&self, key: &str) -> Option<RateLimitStats> {
        let buckets = self.buckets.lock().unwrap();
        buckets.get(key).map(|bucket| RateLimitStats {
            tokens: bucket.tokens(),
            capacity: self.config.capacity as f64,
            refill_rate: self.config.refill_rate_per_sec,
        })
    }
    
    /// Get the configuration of this rate limiter.
    pub fn config(&self) -> RateLimitConfig {
        self.config
    }
}

/// Statistics for a rate-limited key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RateLimitStats {
    /// Current number of tokens.
    pub tokens: f64,
    /// Maximum capacity of the bucket.
    pub capacity: f64,
    /// Refill rate in tokens per second.
    pub refill_rate: u32,
}

/// Advanced rate limiter with multiple tiers for handling overload conditions.
pub struct TieredRateLimiter {
    /// Global rate limiter for overall system protection.
    global_limiter: RateLimiter,
    /// Per-key rate limiters for granular control.
    key_limiters: Arc<Mutex<HashMap<String, RateLimiter>>>,
    /// Default configuration for new key limiters.
    default_config: RateLimitConfig,
}

impl TieredRateLimiter {
    /// Create a new tiered rate limiter.
    pub fn new(global_config: RateLimitConfig, default_config: RateLimitConfig) -> Self {
        Self {
            global_limiter: RateLimiter::new(global_config),
            key_limiters: Arc::new(Mutex::new(HashMap::new())),
            default_config,
        }
    }
    
    /// Check if a request is allowed, applying both global and per-key limits.
    pub fn check(&self, key: &str) -> Result<bool, RateLimitError> {
        // First check global limit
        if !self.global_limiter.check("global").unwrap_or(true) {
            return Err(RateLimitError::RateLimitExceeded("global".to_string()));
        }
        
        // Then check per-key limit
        let limiters = self.key_limiters.lock().unwrap();
        if let Some(limiter) = limiters.get(key) {
            limiter.check(key)
        } else {
            // Use default config for new keys
            drop(limiters); // Release the lock
            let mut limiters = self.key_limiters.lock().unwrap();
            let limiter = RateLimiter::new(self.default_config);
            let result = limiter.check(key);
            limiters.insert(key.to_string(), limiter);
            result
        }
    }
    
    /// Set a specific rate limit configuration for a key.
    pub fn set_key_limit(&self, key: &str, config: RateLimitConfig) -> Result<(), RateLimitError> {
        let mut limiters = self.key_limiters.lock().unwrap();
        limiters.insert(key.to_string(), RateLimiter::new(config));
        Ok(())
    }
    
    /// Remove a specific key's rate limit configuration.
    pub fn remove_key_limit(&self, key: &str) {
        let mut limiters = self.key_limiters.lock().unwrap();
        limiters.remove(key);
    }
    
    /// Get statistics for the global limiter.
    pub fn global_stats(&self) -> Option<RateLimitStats> {
        self.global_limiter.get_stats("global")
    }
    
    /// Get statistics for a specific key.
    pub fn key_stats(&self, key: &str) -> Option<RateLimitStats> {
        let limiters = self.key_limiters.lock().unwrap();
        limiters.get(key).and_then(|limiter| limiter.get_stats(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter_allows_burst() {
        let config = RateLimitConfig {
            capacity: 5,
            refill_rate_per_sec: 1,
        };
        let limiter = RateLimiter::new(config);
        let key = "user1";

        // Should allow 5 immediate requests
        for _ in 0..5 {
            assert!(limiter.check(key).unwrap(), "Should allow request within capacity");
        }

        // Should reject the 6th
        assert!(limiter.check(key).is_err(), "Should reject request exceeding capacity");
    }

    #[test]
    fn test_rate_limiter_refills() {
        let config = RateLimitConfig {
            capacity: 1,
            refill_rate_per_sec: 10, // 10 per second = 1 every 100ms
        };
        let limiter = RateLimiter::new(config);
        let key = "user2";

        // Consume capacity
        assert!(limiter.check(key).unwrap());
        assert!(limiter.check(key).is_err());

        // Wait for refill (150ms should be enough for 1 token)
        thread::sleep(Duration::from_millis(150));

        // Should allow again
        assert!(limiter.check(key).unwrap(), "Should allow request after refill");
    }

    #[test]
    fn test_distinct_keys() {
        let config = RateLimitConfig {
            capacity: 1,
            refill_rate_per_sec: 1,
        };
        let limiter = RateLimiter::new(config);

        assert!(limiter.check("userA").unwrap());
        assert!(limiter.check("userA").is_err()); // userA exhausted

        assert!(limiter.check("userB").unwrap()); // userB fresh
        assert!(limiter.check("userB").is_err());
    }

    #[test]
    fn test_cleanup() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        assert!(limiter.check("stale_user").unwrap());
        
        // Wait a bit
        thread::sleep(Duration::from_millis(100));

        // Cleanup with very short idle time
        limiter.cleanup(Duration::from_millis(1));

        let buckets = limiter.buckets.lock().unwrap();
        assert!(buckets.is_empty(), "Should have cleaned up stale bucket");
    }
    
    #[test]
    fn test_rate_limit_config_validation() {
        assert!(RateLimitConfig::new(0, 10).is_err());
        assert!(RateLimitConfig::new(10, 0).is_err());
        assert!(RateLimitConfig::new(10, 10).is_ok());
    }
    
    #[test]
    fn test_tiered_rate_limiter() {
        let global_config = RateLimitConfig::new(100, 50).unwrap();
        let default_config = RateLimitConfig::new(10, 5).unwrap();
        let limiter = TieredRateLimiter::new(global_config, default_config);
        
        // Should allow requests within limits
        for _ in 0..10 {
            assert!(limiter.check("user1").unwrap());
        }
        
        // Should reject when limit is exceeded
        assert!(limiter.check("user1").is_err());
    }
    
    #[test]
    fn test_custom_key_limits() {
        let global_config = RateLimitConfig::new(100, 50).unwrap();
        let default_config = RateLimitConfig::new(10, 5).unwrap();
        let limiter = TieredRateLimiter::new(global_config, default_config);
        
        // Set custom limit for a key
        let custom_config = RateLimitConfig::new(20, 10).unwrap();
        assert!(limiter.set_key_limit("high-priority", custom_config).is_ok());
        
        // Should allow more requests for high-priority key
        for _ in 0..20 {
            assert!(limiter.check("high-priority").unwrap());
        }
        
        // Should reject when limit is exceeded
        assert!(limiter.check("high-priority").is_err());
    }
}