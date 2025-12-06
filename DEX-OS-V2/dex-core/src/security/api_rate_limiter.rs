//! API Rate Limiter for Security Layer 4 - API & Gateway Security
//!
//! Implements rate limiting per API endpoint and per client with sliding window algorithm.
//! From DEX-OS-V2.csv line 238:
//! - Security,Security Layer,Security Layer 4,API & Gateway Security,API Protection,High

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Rate limiter error types
#[derive(Debug, Error, Clone, PartialEq)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for client {client_id}: {limit_type}")]
    RateLimitExceeded {
        client_id: String,
        limit_type: String,
    },
    #[error("Invalid rate limit configuration: {0}")]
    InvalidConfiguration(String),
}

/// Rate limit configuration for an endpoint or client
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimit {
    /// Requests allowed per second
    pub requests_per_second: Option<u32>,
    /// Requests allowed per minute
    pub requests_per_minute: Option<u32>,
    /// Requests allowed per hour
    pub requests_per_hour: Option<u32>,
    /// Burst size (requests allowed above limit in short time)
    pub burst_size: u32,
}

impl RateLimit {
    pub fn new(
        requests_per_second: Option<u32>,
        requests_per_minute: Option<u32>,
        requests_per_hour: Option<u32>,
        burst_size: u32,
    ) -> Self {
        Self {
            requests_per_second,
            requests_per_minute,
            requests_per_hour,
            burst_size,
        }
    }

    /// Create a permissive rate limit (high limits)
    pub fn permissive() -> Self {
        Self::new(Some(100), Some(1000), Some(10000), 50)
    }

    /// Create a strict rate limit (low limits)
    pub fn strict() -> Self {
        Self::new(Some(10), Some(100), Some(1000), 5)
    }

    /// Create an unlimited rate limit (no limits)
    pub fn unlimited() -> Self {
        Self::new(None, None, None, 0)
    }
}

/// Usage tracker for sliding window rate limiting
#[derive(Debug, Clone)]
struct UsageTracker {
    /// Request timestamps (in seconds since UNIX epoch)
    requests: VecDeque<u64>,
    /// Last cleanup time
    last_cleanup: u64,
}

impl UsageTracker {
    fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            requests: VecDeque::new(),
            last_cleanup: now,
        }
    }

    /// Add a request timestamp
    fn add_request(&mut self, timestamp: u64) {
        self.requests.push_back(timestamp);
    }

    /// Remove requests older than the given duration
    fn cleanup(&mut self, max_age_seconds: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let cutoff = now.saturating_sub(max_age_seconds);
        
        while let Some(&timestamp) = self.requests.front() {
            if timestamp < cutoff {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        self.last_cleanup = now;
    }

    /// Count requests in the last N seconds
    fn count_in_window(&self, window_seconds: u64) -> usize {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let cutoff = now.saturating_sub(window_seconds);
        
        self.requests
            .iter()
            .filter(|&&timestamp| timestamp >= cutoff)
            .count()
    }
}

/// API Rate Limiter with per-endpoint and per-client limits
#[derive(Debug, Clone)]
pub struct APIRateLimiter {
    /// Rate limits per endpoint
    endpoint_limits: Arc<RwLock<HashMap<String, RateLimit>>>,
    /// Rate limits per client
    client_limits: Arc<RwLock<HashMap<String, RateLimit>>>,
    /// Usage tracking per client
    client_usage: Arc<RwLock<HashMap<String, UsageTracker>>>,
    /// Usage tracking per endpoint
    endpoint_usage: Arc<RwLock<HashMap<String, UsageTracker>>>,
    /// Global rate limit (applies to all requests)
    global_limit: RateLimit,
    /// Statistics
    total_requests: Arc<RwLock<u64>>,
    blocked_requests: Arc<RwLock<u64>>,
}

use crate::security::ring_buffer::RingBuffer;

impl APIRateLimiter {
    /// Create a new API rate limiter
    pub fn new(global_limit: RateLimit) -> Self {
        Self {
            endpoint_limits: Arc::new(RwLock::new(HashMap::new())),
            client_limits: Arc::new(RwLock::new(HashMap::new())),
            client_usage: Arc::new(RwLock::new(HashMap::new())),
            endpoint_usage: Arc::new(RwLock::new(HashMap::new())),
            global_limit,
            total_requests: Arc::new(RwLock::new(0)),
            blocked_requests: Arc::new(RwLock::new(0)),
        }
    }

    /// Set rate limit for a specific endpoint
    pub fn set_endpoint_limit(&self, endpoint: String, limit: RateLimit) {
        let mut limits = self.endpoint_limits.write().unwrap();
        limits.insert(endpoint, limit);
    }

    /// Set rate limit for a specific client
    pub fn set_client_limit(&self, client_id: String, limit: RateLimit) {
        let mut limits = self.client_limits.write().unwrap();
        limits.insert(client_id, limit);
    }

    /// Check if a request is allowed
    pub fn check_request(&self, client_id: &str, endpoint: &str) -> Result<(), RateLimitError> {
        let mut total = self.total_requests.write().unwrap();
        *total += 1;
        drop(total);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check global limit
        if !self.check_limit_internal(client_id, &self.global_limit, true)? {
            let mut blocked = self.blocked_requests.write().unwrap();
            *blocked += 1;
            return Err(RateLimitError::RateLimitExceeded {
                client_id: client_id.to_string(),
                limit_type: "global".to_string(),
            });
        }

        // Check client-specific limit
        let client_limits = self.client_limits.read().unwrap();
        if let Some(limit) = client_limits.get(client_id) {
            let limit = limit.clone();
            drop(client_limits);
            if !self.check_limit_internal(client_id, &limit, true)? {
                let mut blocked = self.blocked_requests.write().unwrap();
                *blocked += 1;
                return Err(RateLimitError::RateLimitExceeded {
                    client_id: client_id.to_string(),
                    limit_type: "client".to_string(),
                });
            }
        }

        // Check endpoint-specific limit
        let endpoint_limits = self.endpoint_limits.read().unwrap();
        if let Some(limit) = endpoint_limits.get(endpoint) {
            let limit = limit.clone();
            drop(endpoint_limits);
            if !self.check_limit_internal(endpoint, &limit, false)? {
                let mut blocked = self.blocked_requests.write().unwrap();
                *blocked += 1;
                return Err(RateLimitError::RateLimitExceeded {
                    client_id: client_id.to_string(),
                    limit_type: format!("endpoint:{}", endpoint),
                });
            }
        }

        // Record the request
        let mut client_usage = self.client_usage.write().unwrap();
        let tracker = client_usage
            .entry(client_id.to_string())
            .or_insert_with(UsageTracker::new);
        tracker.add_request(now);

        let mut endpoint_usage = self.endpoint_usage.write().unwrap();
        let tracker = endpoint_usage
            .entry(endpoint.to_string())
            .or_insert_with(UsageTracker::new);
        tracker.add_request(now);

        Ok(())
    }

    /// Internal limit checking logic
    fn check_limit_internal(
        &self,
        key: &str,
        limit: &RateLimit,
        is_client: bool,
    ) -> Result<bool, RateLimitError> {
        let usage = if is_client {
            self.client_usage.read().unwrap()
        } else {
            self.endpoint_usage.read().unwrap()
        };

        let tracker = match usage.get(key) {
            Some(t) => t,
            None => return Ok(true), // No usage yet, allow
        };

        // Check per-second limit
        if let Some(rps) = limit.requests_per_second {
            let count = tracker.count_in_window(1);
            if count >= rps as usize + limit.burst_size as usize {
                return Ok(false);
            }
        }

        // Check per-minute limit
        if let Some(rpm) = limit.requests_per_minute {
            let count = tracker.count_in_window(60);
            if count >= rpm as usize + limit.burst_size as usize {
                return Ok(false);
            }
        }

        // Check per-hour limit
        if let Some(rph) = limit.requests_per_hour {
            let count = tracker.count_in_window(3600);
            if count >= rph as usize + limit.burst_size as usize {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get remaining requests for a client
    pub fn get_remaining_requests(&self, client_id: &str) -> RateLimitInfo {
        let client_limits = self.client_limits.read().unwrap();
        let limit = client_limits
            .get(client_id)
            .cloned()
            .unwrap_or_else(|| self.global_limit.clone());

        let client_usage = self.client_usage.read().unwrap();
        let tracker = client_usage.get(client_id);

        let remaining_per_second = limit.requests_per_second.map(|rps| {
            let used = tracker.map(|t| t.count_in_window(1)).unwrap_or(0);
            (rps as usize + limit.burst_size as usize).saturating_sub(used) as u32
        });

        let remaining_per_minute = limit.requests_per_minute.map(|rpm| {
            let used = tracker.map(|t| t.count_in_window(60)).unwrap_or(0);
            (rpm as usize + limit.burst_size as usize).saturating_sub(used) as u32
        });

        let remaining_per_hour = limit.requests_per_hour.map(|rph| {
            let used = tracker.map(|t| t.count_in_window(3600)).unwrap_or(0);
            (rph as usize + limit.burst_size as usize).saturating_sub(used) as u32
        });

        RateLimitInfo {
            remaining_per_second,
            remaining_per_minute,
            remaining_per_hour,
        }
    }

    /// Cleanup old usage data
    pub fn cleanup(&self) {
        let mut client_usage = self.client_usage.write().unwrap();
        for tracker in client_usage.values_mut() {
            tracker.cleanup(3600); // Keep last hour
        }

        let mut endpoint_usage = self.endpoint_usage.write().unwrap();
        for tracker in endpoint_usage.values_mut() {
            tracker.cleanup(3600); // Keep last hour
        }
    }

    /// Get statistics
    pub fn get_statistics(&self) -> RateLimiterStatistics {
        let total = *self.total_requests.read().unwrap();
        let blocked = *self.blocked_requests.read().unwrap();

        RateLimiterStatistics {
            total_requests: total,
            blocked_requests: blocked,
            allowed_requests: total - blocked,
            block_rate: if total > 0 {
                (blocked as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            active_clients: self.client_usage.read().unwrap().len(),
            active_endpoints: self.endpoint_usage.read().unwrap().len(),
        }
    }
}

/// Rate limit information for a client
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub remaining_per_second: Option<u32>,
    pub remaining_per_minute: Option<u32>,
    pub remaining_per_hour: Option<u32>,
}

/// Rate limiter statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimiterStatistics {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub allowed_requests: u64,
    pub block_rate: f64,
    pub active_clients: usize,
    pub active_endpoints: usize,
}

impl Default for APIRateLimiter {
    fn default() -> Self {
        Self::new(RateLimit::permissive())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_creation() {
        let limit = RateLimit::new(Some(10), Some(100), Some(1000), 5);
        assert_eq!(limit.requests_per_second, Some(10));
        assert_eq!(limit.requests_per_minute, Some(100));
        assert_eq!(limit.requests_per_hour, Some(1000));
        assert_eq!(limit.burst_size, 5);
    }

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let limiter = APIRateLimiter::new(RateLimit::new(Some(10), Some(100), None, 2));

        // Should allow first 12 requests (10 + burst of 2)
        for i in 0..12 {
            let result = limiter.check_request("client1", "/api/test");
            assert!(result.is_ok(), "Request {} should be allowed", i);
        }

        // 13th request should be blocked
        let result = limiter.check_request("client1", "/api/test");
        assert!(result.is_err());
    }

    #[test]
    fn test_rate_limiter_per_client() {
        let limiter = APIRateLimiter::new(RateLimit::unlimited());
        limiter.set_client_limit("client1".to_string(), RateLimit::new(Some(5), None, None, 0));

        // Client1 should be limited to 5 requests
        for _ in 0..5 {
            assert!(limiter.check_request("client1", "/api/test").is_ok());
        }
        assert!(limiter.check_request("client1", "/api/test").is_err());

        // Client2 should not be limited
        for _ in 0..10 {
            assert!(limiter.check_request("client2", "/api/test").is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_per_endpoint() {
        let limiter = APIRateLimiter::new(RateLimit::unlimited());
        limiter.set_endpoint_limit("/api/limited".to_string(), RateLimit::new(Some(3), None, None, 0));

        // Limited endpoint should block after 3 requests
        for _ in 0..3 {
            assert!(limiter.check_request("client1", "/api/limited").is_ok());
        }
        assert!(limiter.check_request("client1", "/api/limited").is_err());

        // Unlimited endpoint should work
        for _ in 0..10 {
            assert!(limiter.check_request("client1", "/api/unlimited").is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_statistics() {
        let limiter = APIRateLimiter::new(RateLimit::new(Some(5), None, None, 0));

        for _ in 0..5 {
            let _ = limiter.check_request("client1", "/api/test");
        }
        // This one should be blocked
        let _ = limiter.check_request("client1", "/api/test");

        let stats = limiter.get_statistics();
        assert_eq!(stats.total_requests, 6);
        assert_eq!(stats.blocked_requests, 1);
        assert_eq!(stats.allowed_requests, 5);
    }

    #[test]
    fn test_get_remaining_requests() {
        let limiter = APIRateLimiter::new(RateLimit::new(Some(10), Some(100), None, 2));

        // Make 5 requests
        for _ in 0..5 {
            let _ = limiter.check_request("client1", "/api/test");
        }

        let info = limiter.get_remaining_requests("client1");
        assert_eq!(info.remaining_per_second, Some(7)); // 10 + 2 - 5 = 7
    }
}
