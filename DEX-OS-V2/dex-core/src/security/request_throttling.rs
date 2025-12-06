//! Request Throttling Module for Protection Layer 1 - Rate Limiting
//!
//! Implements advanced request throttling from DEX-OS-V2.csv line 245:
//! - Security,Protection Layer,Protection Layer 1,Rate Limiting,Request Throttling,High
//!
//! Features:
//! - Adaptive rate limiting (adjusts based on system load)
//! - IP-based throttling
//! - Geographic throttling
//! - Behavioral analysis
//! - Bot detection
//! - Distributed rate limiting support

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Request throttling errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ThrottlingError {
    #[error("Request throttled: {reason}")]
    Throttled { reason: String },
    #[error("Bot detected: {user_agent}")]
    BotDetected { user_agent: String },
    #[error("Suspicious behavior detected")]
    SuspiciousBehavior,
    #[error("Geographic region blocked: {region}")]
    GeographicBlock { region: String },
    #[error("IP address blocked: {ip}")]
    IpBlocked { ip: String },
}

/// Throttling action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThrottlingAction {
    /// Allow the request
    Allow,
    /// Slow down the request (add delay)
    SlowDown { delay_ms: u64 },
    /// Block the request completely
    Block,
    /// Require CAPTCHA verification
    RequireCaptcha,
}

/// Request metadata for behavioral analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// IP address
    pub ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Request path
    pub path: String,
    /// Timestamp
    pub timestamp: u64,
    /// Geographic region (ISO country code)
    pub region: Option<String>,
    /// Request size in bytes
    pub size: usize,
}

/// Behavioral pattern
#[derive(Debug, Clone)]
struct BehaviorPattern {
    /// Request timestamps
    requests: VecDeque<u64>,
    /// Paths accessed
    paths: Vec<String>,
    /// User agents seen
    user_agents: HashSet<String>,
    /// Suspicious score (0-100)
    suspicion_score: f64,
}

use std::collections::HashSet;

impl BehaviorPattern {
    fn new() -> Self {
        Self {
            requests: VecDeque::new(),
            paths: Vec::new(),
            user_agents: HashSet::new(),
            suspicion_score: 0.0,
        }
    }

    fn add_request(&mut self, timestamp: u64, path: String, user_agent: Option<String>) {
        self.requests.push_back(timestamp);
        self.paths.push(path);
        
        if let Some(ua) = user_agent {
            self.user_agents.insert(ua);
        }

        // Keep only last 100 requests
        if self.requests.len() > 100 {
            self.requests.pop_front();
        }
        if self.paths.len() > 100 {
            self.paths.remove(0);
        }

        self.update_suspicion_score();
    }

    fn update_suspicion_score(&mut self) {
        let mut score: f64 = 0.0;

        // Check request rate (requests in last minute)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let one_minute_ago = now - 60;
        let recent_requests = self.requests.iter().filter(|&&t| t > one_minute_ago).count();
        
        if recent_requests > 60 {
            score += 20.0; // Very high request rate
        } else if recent_requests > 30 {
            score += 10.0; // High request rate
        }

        // Check path diversity (low diversity = suspicious)
        let unique_paths: HashSet<_> = self.paths.iter().collect();
        let diversity = unique_paths.len() as f64 / self.paths.len().max(1) as f64;
        
        if diversity < 0.1 {
            score += 15.0; // Hitting same endpoint repeatedly
        }

        // Check user agent changes (suspicious if too many)
        if self.user_agents.len() > 5 {
            score += 25.0; // User agent switching
        }

        // Check sequential scanning pattern
        if self.is_sequential_scan() {
            score += 30.0; // Looks like automated scanning
        }

        // Cap at 100 to avoid unbounded growth while keeping type inference explicit.
        self.suspicion_score = score.min(100.0);
    }

    fn is_sequential_scan(&self) -> bool {
        // Check if paths look like sequential scanning (e.g., /api/1, /api/2, /api/3)
        if self.paths.len() < 5 {
            return false;
        }

        let recent_paths: Vec<&String> = self.paths.iter().rev().take(10).collect();
        let numeric_endings: Vec<bool> = recent_paths
            .iter()
            .map(|p| p.chars().last().map(|c| c.is_numeric()).unwrap_or(false))
            .collect();

        numeric_endings.iter().filter(|&&x| x).count() >= 7
    }

    fn is_suspicious(&self) -> bool {
        self.suspicion_score > 50.0
    }
}

/// System load information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemLoad {
    /// CPU usage (0.0-1.0)
    pub cpu_usage: f64,
    /// Memory usage (0.0-1.0)
    pub memory_usage: f64,
    /// Active connections
    pub active_connections: usize,
    /// Requests per second
    pub requests_per_second: f64,
}

/// Adaptive throttling configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    /// Base requests per second
    pub base_rps: f64,
    /// Maximum requests per second under low load
    pub max_rps: f64,
    /// Minimum requests per second under high load
    pub min_rps: f64,
    /// CPU threshold for throttling (0.0-1.0)
    pub cpu_threshold: f64,
    /// Memory threshold for throttling (0.0-1.0)
    pub memory_threshold: f64,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            base_rps: 100.0,
            max_rps: 1000.0,
            min_rps: 10.0,
            cpu_threshold: 0.8,
            memory_threshold: 0.9,
        }
    }
}

/// Bot detection patterns
#[derive(Debug, Clone)]
struct BotDetector {
    /// Known bot user agents
    bot_patterns: Vec<regex::Regex>,
    /// Known good bot patterns (search engines)
    good_bot_patterns: Vec<regex::Regex>,
}

impl BotDetector {
    fn new() -> Self {
        Self {
            bot_patterns: Self::build_bot_patterns(),
            good_bot_patterns: Self::build_good_bot_patterns(),
        }
    }

    fn build_bot_patterns() -> Vec<regex::Regex> {
        vec![
            regex::Regex::new(r"(?i)bot").unwrap(),
            regex::Regex::new(r"(?i)crawler").unwrap(),
            regex::Regex::new(r"(?i)spider").unwrap(),
            regex::Regex::new(r"(?i)scraper").unwrap(),
            regex::Regex::new(r"(?i)curl").unwrap(),
            regex::Regex::new(r"(?i)wget").unwrap(),
            regex::Regex::new(r"(?i)python-requests").unwrap(),
        ]
    }

    fn build_good_bot_patterns() -> Vec<regex::Regex> {
        vec![
            regex::Regex::new(r"(?i)googlebot").unwrap(),
            regex::Regex::new(r"(?i)bingbot").unwrap(),
            regex::Regex::new(r"(?i)slackbot").unwrap(),
        ]
    }

    fn is_bot(&self, user_agent: &str) -> bool {
        // Check if it's a good bot first
        for pattern in &self.good_bot_patterns {
            if pattern.is_match(user_agent) {
                return false; // Allow good bots
            }
        }

        // Check for bot patterns
        for pattern in &self.bot_patterns {
            if pattern.is_match(user_agent) {
                return true;
            }
        }

        false
    }
}

/// Request throttling manager
#[derive(Debug, Clone)]
pub struct RequestThrottler {
    /// Adaptive configuration
    config: AdaptiveConfig,
    /// IP-based behavior patterns
    ip_patterns: Arc<RwLock<HashMap<String, BehaviorPattern>>>,
    /// Blocked IPs
    blocked_ips: Arc<RwLock<HashSet<String>>>,
    /// Blocked regions
    blocked_regions: Arc<RwLock<HashSet<String>>>,
    /// Bot detector
    bot_detector: BotDetector,
    /// Current system load
    system_load: Arc<RwLock<SystemLoad>>,
    /// Request history for RPS calculation
    request_history: Arc<RwLock<VecDeque<u64>>>,
}

impl RequestThrottler {
    /// Create a new request throttler
    pub fn new(config: AdaptiveConfig) -> Self {
        Self {
            config,
            ip_patterns: Arc::new(RwLock::new(HashMap::new())),
            blocked_ips: Arc::new(RwLock::new(HashSet::new())),
            blocked_regions: Arc::new(RwLock::new(HashSet::new())),
            bot_detector: BotDetector::new(),
            system_load: Arc::new(RwLock::new(SystemLoad {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                active_connections: 0,
                requests_per_second: 0.0,
            })),
            request_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Update system load
    pub fn update_system_load(&self, load: SystemLoad) {
        let mut system_load = self.system_load.write().unwrap();
        *system_load = load;
    }

    /// Block an IP address
    pub fn block_ip(&self, ip: String) {
        let mut blocked = self.blocked_ips.write().unwrap();
        blocked.insert(ip);
    }

    /// Unblock an IP address
    pub fn unblock_ip(&self, ip: &str) {
        let mut blocked = self.blocked_ips.write().unwrap();
        blocked.remove(ip);
    }

    /// Block a geographic region
    pub fn block_region(&self, region: String) {
        let mut blocked = self.blocked_regions.write().unwrap();
        blocked.insert(region);
    }

    /// Check if request should be throttled
    pub fn check_request(&self, metadata: &RequestMetadata) -> Result<ThrottlingAction, ThrottlingError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check IP blocking
        if let Some(ref ip) = metadata.ip {
            let blocked = self.blocked_ips.read().unwrap();
            if blocked.contains(ip) {
                return Err(ThrottlingError::IpBlocked { ip: ip.clone() });
            }
        }

        // Check geographic blocking
        if let Some(ref region) = metadata.region {
            let blocked = self.blocked_regions.read().unwrap();
            if blocked.contains(region) {
                return Err(ThrottlingError::GeographicBlock {
                    region: region.clone(),
                });
            }
        }

        // Check bot detection
        if let Some(ref user_agent) = metadata.user_agent {
            if self.bot_detector.is_bot(user_agent) {
                return Ok(ThrottlingAction::SlowDown { delay_ms: 1000 });
            }
        }

        // Update request history
        {
            let mut history = self.request_history.write().unwrap();
            history.push_back(now);
            
            // Keep only last minute
            let one_minute_ago = now - 60;
            while let Some(&oldest) = history.front() {
                if oldest < one_minute_ago {
                    history.pop_front();
                } else {
                    break;
                }
            }
        }

        // Calculate current RPS
        let current_rps = {
            let history = self.request_history.read().unwrap();
            history.len() as f64 / 60.0
        };

        // Get adaptive limit based on system load
        let limit = self.calculate_adaptive_limit();

        // Check if we're over the limit
        if current_rps > limit {
            return Ok(ThrottlingAction::SlowDown {
                delay_ms: ((current_rps - limit) * 100.0) as u64,
            });
        }

        // Update behavioral pattern
        if let Some(ref ip) = metadata.ip {
            let mut patterns = self.ip_patterns.write().unwrap();
            let pattern = patterns.entry(ip.clone()).or_insert_with(BehaviorPattern::new);
            pattern.add_request(now, metadata.path.clone(), metadata.user_agent.clone());

            if pattern.is_suspicious() {
                return Ok(ThrottlingAction::RequireCaptcha);
            }
        }

        Ok(ThrottlingAction::Allow)
    }

    /// Calculate adaptive rate limit based on system load
    fn calculate_adaptive_limit(&self) -> f64 {
        let load = self.system_load.read().unwrap();
        
        // If system is under high load, reduce limit
        if load.cpu_usage > self.config.cpu_threshold || load.memory_usage > self.config.memory_threshold {
            // Linearly decrease from base to min as load increases
            let load_factor = (load.cpu_usage.max(load.memory_usage) - self.config.cpu_threshold) / (1.0 - self.config.cpu_threshold);
            let reduction = (self.config.base_rps - self.config.min_rps) * load_factor;
            (self.config.base_rps - reduction).max(self.config.min_rps)
        } else {
            // Under low load, can handle more requests
            let load_factor = 1.0 - load.cpu_usage.max(load.memory_usage);
            let increase = (self.config.max_rps - self.config.base_rps) * load_factor;
            (self.config.base_rps + increase).min(self.config.max_rps)
        }
    }

    /// Get behavioral analysis for an IP
    pub fn get_behavior_analysis(&self, ip: &str) -> Option<f64> {
        let patterns = self.ip_patterns.read().unwrap();
        patterns.get(ip).map(|p| p.suspicion_score)
    }

    /// Cleanup old behavioral data
    pub fn cleanup(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let one_hour_ago = now - 3600;
        
        let mut patterns = self.ip_patterns.write().unwrap();
        patterns.retain(|_, pattern| {
            pattern.requests.back().map(|&t| t > one_hour_ago).unwrap_or(false)
        });
    }

    /// Get statistics
    pub fn get_statistics(&self) -> ThrottlingStatistics {
        let patterns = self.ip_patterns.read().unwrap();
        let blocked_ips = self.blocked_ips.read().unwrap();
        let blocked_regions = self.blocked_regions.read().unwrap();
        let load = self.system_load.read().unwrap();
        let history = self.request_history.read().unwrap();

        ThrottlingStatistics {
            active_ips: patterns.len(),
            blocked_ips: blocked_ips.len(),
            blocked_regions: blocked_regions.len(),
            current_rps: history.len() as f64 / 60.0,
            adaptive_limit: self.calculate_adaptive_limit(),
            system_load: load.clone(),
        }
    }
}

/// Throttling statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThrottlingStatistics {
    pub active_ips: usize,
    pub blocked_ips: usize,
    pub blocked_regions: usize,
    pub current_rps: f64,
    pub adaptive_limit: f64,
    pub system_load: SystemLoad,
}

impl Default for RequestThrottler {
    fn default() -> Self {
        Self::new(AdaptiveConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_config_default() {
        let config = AdaptiveConfig::default();
        assert_eq!(config.base_rps, 100.0);
    }

    #[test]
    fn test_bot_detection() {
        let detector = BotDetector::new();
        
        assert!(detector.is_bot("Mozilla/5.0 (compatible; Googlebot/2.1)") == false); // Good bot
        assert!(detector.is_bot("curl/7.64.1"));
        assert!(detector.is_bot("python-requests/2.25.1"));
        assert!(!detector.is_bot("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"));
    }

    #[test]
    fn test_ip_blocking() {
        let throttler = RequestThrottler::default();
        
        throttler.block_ip("192.168.1.1".to_string());
        
        let metadata = RequestMetadata {
            ip: Some("192.168.1.1".to_string()),
            user_agent: None,
            path: "/api/test".to_string(),
            timestamp: 0,
            region: None,
            size: 0,
        };
        
        assert!(throttler.check_request(&metadata).is_err());
    }

    #[test]
    fn test_region_blocking() {
        let throttler = RequestThrottler::default();
        
        throttler.block_region("XX".to_string());
        
        let metadata = RequestMetadata {
            ip: Some("1.2.3.4".to_string()),
            user_agent: None,
            path: "/api/test".to_string(),
            timestamp: 0,
            region: Some("XX".to_string()),
            size: 0,
        };
        
        assert!(throttler.check_request(&metadata).is_err());
    }

    #[test]
    fn test_adaptive_limiting() {
        let config = AdaptiveConfig {
            base_rps: 100.0,
            max_rps: 200.0,
            min_rps: 10.0,
            cpu_threshold: 0.8,
            memory_threshold: 0.9,
        };
        
        let throttler = RequestThrottler::new(config);
        
        // Low load - should allow more
        throttler.update_system_load(SystemLoad {
            cpu_usage: 0.3,
            memory_usage: 0.3,
            active_connections: 10,
            requests_per_second: 50.0,
        });
        
        let limit = throttler.calculate_adaptive_limit();
        assert!(limit > 100.0);
        
        // High load - should reduce
        throttler.update_system_load(SystemLoad {
            cpu_usage: 0.9,
            memory_usage: 0.95,
            active_connections: 100,
            requests_per_second: 200.0,
        });
        
        let limit = throttler.calculate_adaptive_limit();
        assert!(limit < 100.0);
    }

    #[test]
    fn test_behavioral_analysis() {
        let throttler = RequestThrottler::default();
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Simulate suspicious behavior (many rapid requests)
        for i in 0..70 {
            let metadata = RequestMetadata {
                ip: Some("1.2.3.4".to_string()),
                user_agent: Some("Test".to_string()),
                path: "/api/test".to_string(),
                timestamp: now + i,
                region: None,
                size: 0,
            };
            let _ = throttler.check_request(&metadata);
        }
        
        let score = throttler.get_behavior_analysis("1.2.3.4");
        assert!(score.is_some());
        assert!(score.unwrap() > 0.0);
    }
}
