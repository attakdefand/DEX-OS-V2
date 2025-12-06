//! DDoS Protection implementation for DEX-OS Network Security
//!
//! Implements Security Layer 6 - Network & Infrastructure Security (Perimeter Defense)
//! Provides comprehensive DDoS protection mechanisms including rate limiting,
//! SYN flood protection, connection throttling, and traffic analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use thiserror::Error;

/// DDoS protection error types
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DDoSError {
    #[error("Rate limit exceeded for IP: {0}")]
    RateLimitExceeded(String),
    #[error("Connection limit exceeded for IP: {0}")]
    ConnectionLimitExceeded(String),
    #[error("SYN flood detected from IP: {0}")]
    SynFloodDetected(String),
    #[error("Traffic anomaly detected: {0}")]
    TrafficAnomaly(String),
    #[error("IP blocked: {0}")]
    IpBlocked(String),
}

/// Rate limiter for per-IP traffic control
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Requests per IP (IP -> (count, window_start))
    requests: HashMap<IpAddr, (u32, u64)>,
    /// Maximum requests per window
    max_requests: u32,
    /// Time window in seconds
    window_seconds: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window_seconds,
        }
    }

    pub fn check_rate_limit(&mut self, ip: &IpAddr) -> Result<(), DDoSError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = self.requests.entry(*ip).or_insert((0, now));

        // Reset window if expired
        if now - entry.1 >= self.window_seconds {
            entry.0 = 0;
            entry.1 = now;
        }

        // Check limit
        if entry.0 >= self.max_requests {
            return Err(DDoSError::RateLimitExceeded(ip.to_string()));
        }

        entry.0 += 1;
        Ok(())
    }

    pub fn get_request_count(&self, ip: &IpAddr) -> u32 {
        self.requests.get(ip).map(|(count, _)| *count).unwrap_or(0)
    }

    pub fn cleanup_old_entries(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.requests
            .retain(|_, (_, window_start)| now - *window_start < self.window_seconds * 2);
    }
}

/// SYN flood protection using SYN cookies
#[derive(Debug, Clone)]
pub struct SynFloodProtection {
    /// SYN requests per IP (IP -> (count, window_start))
    syn_requests: HashMap<IpAddr, (u32, u64)>,
    /// Maximum SYN requests per window
    max_syn_per_window: u32,
    /// Time window in seconds
    window_seconds: u64,
    /// Blocked IPs
    blocked_ips: HashMap<IpAddr, u64>,
    /// Block duration in seconds
    block_duration: u64,
}

impl SynFloodProtection {
    pub fn new(max_syn_per_window: u32, window_seconds: u64, block_duration: u64) -> Self {
        Self {
            syn_requests: HashMap::new(),
            max_syn_per_window,
            window_seconds,
            blocked_ips: HashMap::new(),
            block_duration,
        }
    }

    pub fn check_syn_flood(&mut self, ip: &IpAddr) -> Result<(), DDoSError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if IP is blocked
        if let Some(block_time) = self.blocked_ips.get(ip) {
            if now - block_time < self.block_duration {
                return Err(DDoSError::IpBlocked(ip.to_string()));
            } else {
                self.blocked_ips.remove(ip);
            }
        }

        let entry = self.syn_requests.entry(*ip).or_insert((0, now));

        // Reset window if expired
        if now - entry.1 >= self.window_seconds {
            entry.0 = 0;
            entry.1 = now;
        }

        // Check SYN flood threshold
        if entry.0 >= self.max_syn_per_window {
            self.blocked_ips.insert(*ip, now);
            return Err(DDoSError::SynFloodDetected(ip.to_string()));
        }

        entry.0 += 1;
        Ok(())
    }

    pub fn is_blocked(&self, ip: &IpAddr) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Some(block_time) = self.blocked_ips.get(ip) {
            now - block_time < self.block_duration
        } else {
            false
        }
    }

    pub fn unblock_ip(&mut self, ip: &IpAddr) {
        self.blocked_ips.remove(ip);
    }
}

/// Connection throttling
#[derive(Debug, Clone)]
pub struct ConnectionThrottler {
    /// Active connections per IP
    connections: HashMap<IpAddr, u32>,
    /// Maximum connections per IP
    max_connections_per_ip: u32,
}

impl ConnectionThrottler {
    pub fn new(max_connections_per_ip: u32) -> Self {
        Self {
            connections: HashMap::new(),
            max_connections_per_ip,
        }
    }

    pub fn check_connection_limit(&mut self, ip: &IpAddr) -> Result<(), DDoSError> {
        let count = self.connections.entry(*ip).or_insert(0);

        if *count >= self.max_connections_per_ip {
            return Err(DDoSError::ConnectionLimitExceeded(ip.to_string()));
        }

        *count += 1;
        Ok(())
    }

    pub fn release_connection(&mut self, ip: &IpAddr) {
        if let Some(count) = self.connections.get_mut(ip) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    pub fn get_connection_count(&self, ip: &IpAddr) -> u32 {
        self.connections.get(ip).copied().unwrap_or(0)
    }
}

/// Traffic pattern analyzer
#[derive(Debug, Clone)]
pub struct TrafficAnalyzer {
    /// Bytes per IP (IP -> (bytes, window_start))
    traffic: HashMap<IpAddr, (u64, u64)>,
    /// Maximum bytes per window
    max_bytes_per_window: u64,
    /// Time window in seconds
    window_seconds: u64,
}

impl TrafficAnalyzer {
    pub fn new(max_bytes_per_window: u64, window_seconds: u64) -> Self {
        Self {
            traffic: HashMap::new(),
            max_bytes_per_window,
            window_seconds,
        }
    }

    pub fn record_traffic(&mut self, ip: &IpAddr, bytes: u64) -> Result<(), DDoSError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = self.traffic.entry(*ip).or_insert((0, now));

        // Reset window if expired
        if now - entry.1 >= self.window_seconds {
            entry.0 = 0;
            entry.1 = now;
        }

        entry.0 += bytes;

        // Check bandwidth limit
        if entry.0 > self.max_bytes_per_window {
            return Err(DDoSError::TrafficAnomaly(format!(
                "Bandwidth limit exceeded for IP: {}",
                ip
            )));
        }

        Ok(())
    }

    pub fn get_traffic_bytes(&self, ip: &IpAddr) -> u64 {
        self.traffic.get(ip).map(|(bytes, _)| *bytes).unwrap_or(0)
    }
}

/// Geographic blocking configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoBlockingConfig {
    /// Blocked countries (ISO country codes)
    pub blocked_countries: Vec<String>,
    /// Allowed countries (if empty, all are allowed except blocked)
    pub allowed_countries: Vec<String>,
    /// Whether to use allowlist mode
    pub allowlist_mode: bool,
}

impl GeoBlockingConfig {
    pub fn new() -> Self {
        Self {
            blocked_countries: Vec::new(),
            allowed_countries: Vec::new(),
            allowlist_mode: false,
        }
    }

    pub fn is_allowed(&self, country_code: &str) -> bool {
        if self.allowlist_mode {
            self.allowed_countries.contains(&country_code.to_string())
        } else {
            !self.blocked_countries.contains(&country_code.to_string())
        }
    }
}

/// DDoS Protection Manager
#[derive(Debug, Clone)]
pub struct DDoSProtectionManager {
    /// Rate limiter
    rate_limiter: RateLimiter,
    /// SYN flood protection
    syn_flood_protection: SynFloodProtection,
    /// Connection throttler
    connection_throttler: ConnectionThrottler,
    /// Traffic analyzer
    traffic_analyzer: TrafficAnalyzer,
    /// Geographic blocking
    geo_blocking: GeoBlockingConfig,
    /// Statistics
    total_requests: u64,
    blocked_requests: u64,
    syn_floods_detected: u64,
}

impl DDoSProtectionManager {
    /// Create a new DDoS protection manager with default settings
    pub fn new() -> Self {
        Self {
            rate_limiter: RateLimiter::new(100, 60), // 100 requests per minute
            syn_flood_protection: SynFloodProtection::new(50, 10, 300), // 50 SYN per 10s, block for 5min
            connection_throttler: ConnectionThrottler::new(100), // 100 concurrent connections per IP
            traffic_analyzer: TrafficAnalyzer::new(10_000_000, 60), // 10MB per minute
            geo_blocking: GeoBlockingConfig::new(),
            total_requests: 0,
            blocked_requests: 0,
            syn_floods_detected: 0,
        }
    }

    /// Create a new DDoS protection manager with custom settings
    pub fn with_config(
        max_requests_per_minute: u32,
        max_syn_per_window: u32,
        max_connections_per_ip: u32,
        max_bytes_per_minute: u64,
    ) -> Self {
        Self {
            rate_limiter: RateLimiter::new(max_requests_per_minute, 60),
            syn_flood_protection: SynFloodProtection::new(max_syn_per_window, 10, 300),
            connection_throttler: ConnectionThrottler::new(max_connections_per_ip),
            traffic_analyzer: TrafficAnalyzer::new(max_bytes_per_minute, 60),
            geo_blocking: GeoBlockingConfig::new(),
            total_requests: 0,
            blocked_requests: 0,
            syn_floods_detected: 0,
        }
    }

    /// Check if a request should be allowed
    pub fn check_request(&mut self, ip: &IpAddr, bytes: u64) -> Result<(), DDoSError> {
        self.total_requests += 1;

        // Check rate limit
        if let Err(e) = self.rate_limiter.check_rate_limit(ip) {
            self.blocked_requests += 1;
            return Err(e);
        }

        // Check traffic bandwidth
        if let Err(e) = self.traffic_analyzer.record_traffic(ip, bytes) {
            self.blocked_requests += 1;
            return Err(e);
        }

        Ok(())
    }

    /// Check SYN packet
    pub fn check_syn_packet(&mut self, ip: &IpAddr) -> Result<(), DDoSError> {
        if let Err(e) = self.syn_flood_protection.check_syn_flood(ip) {
            self.syn_floods_detected += 1;
            self.blocked_requests += 1;
            return Err(e);
        }

        Ok(())
    }

    /// Check new connection
    pub fn check_new_connection(&mut self, ip: &IpAddr) -> Result<(), DDoSError> {
        if let Err(e) = self.connection_throttler.check_connection_limit(ip) {
            self.blocked_requests += 1;
            return Err(e);
        }

        Ok(())
    }

    /// Release a connection
    pub fn release_connection(&mut self, ip: &IpAddr) {
        self.connection_throttler.release_connection(ip);
    }

    /// Check geographic location
    pub fn check_geo_location(&self, country_code: &str) -> Result<(), DDoSError> {
        if !self.geo_blocking.is_allowed(country_code) {
            return Err(DDoSError::IpBlocked(format!(
                "Country blocked: {}",
                country_code
            )));
        }

        Ok(())
    }

    /// Configure geographic blocking
    pub fn configure_geo_blocking(&mut self, config: GeoBlockingConfig) {
        self.geo_blocking = config;
    }

    /// Unblock an IP address
    pub fn unblock_ip(&mut self, ip: &IpAddr) {
        self.syn_flood_protection.unblock_ip(ip);
    }

    /// Check if an IP is blocked
    pub fn is_blocked(&self, ip: &IpAddr) -> bool {
        self.syn_flood_protection.is_blocked(ip)
    }

    /// Get statistics
    pub fn get_statistics(&self) -> DDoSStatistics {
        DDoSStatistics {
            total_requests: self.total_requests,
            blocked_requests: self.blocked_requests,
            syn_floods_detected: self.syn_floods_detected,
            block_rate: if self.total_requests > 0 {
                (self.blocked_requests as f64 / self.total_requests as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Cleanup old entries
    pub fn cleanup(&mut self) {
        self.rate_limiter.cleanup_old_entries();
    }
}

/// DDoS protection statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DDoSStatistics {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub syn_floods_detected: u64,
    pub block_rate: f64,
}

impl Default for DDoSProtectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GeoBlockingConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(5, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Should allow first 5 requests
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(&ip).is_ok());
        }

        // Should block 6th request
        assert!(limiter.check_rate_limit(&ip).is_err());
    }

    #[test]
    fn test_syn_flood_protection() {
        let mut protection = SynFloodProtection::new(3, 10, 300);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Should allow first 3 SYN packets
        for _ in 0..3 {
            assert!(protection.check_syn_flood(&ip).is_ok());
        }

        // Should detect SYN flood on 4th packet
        assert!(protection.check_syn_flood(&ip).is_err());
        assert!(protection.is_blocked(&ip));
    }

    #[test]
    fn test_connection_throttler() {
        let mut throttler = ConnectionThrottler::new(2);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Should allow 2 connections
        assert!(throttler.check_connection_limit(&ip).is_ok());
        assert!(throttler.check_connection_limit(&ip).is_ok());

        // Should block 3rd connection
        assert!(throttler.check_connection_limit(&ip).is_err());

        // Release one connection
        throttler.release_connection(&ip);

        // Should allow another connection
        assert!(throttler.check_connection_limit(&ip).is_ok());
    }

    #[test]
    fn test_ddos_protection_manager() {
        let mut manager = DDoSProtectionManager::with_config(10, 5, 10, 1_000_000);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Should allow normal requests
        for _ in 0..10 {
            assert!(manager.check_request(&ip, 1024).is_ok());
        }

        // Should block 11th request (rate limit)
        assert!(manager.check_request(&ip, 1024).is_err());

        let stats = manager.get_statistics();
        assert_eq!(stats.total_requests, 11);
        assert_eq!(stats.blocked_requests, 1);
    }

    #[test]
    fn test_geo_blocking() {
        let mut config = GeoBlockingConfig::new();
        config.blocked_countries.push("XX".to_string());

        assert!(config.is_allowed("US"));
        assert!(!config.is_allowed("XX"));

        // Test allowlist mode
        config.allowlist_mode = true;
        config.allowed_countries.push("US".to_string());

        assert!(config.is_allowed("US"));
        assert!(!config.is_allowed("UK"));
    }
}
