//! Comprehensive tests for the Rate Limiting functionality
//!
//! This file tests the API rate limiter implementation which provides request throttling
//! capabilities for the DEX-OS API security.

use dex_core::security::api_rate_limiter::{APIRateLimiter, RateLimit};

#[test]
fn test_global_rate_limiting() {
    // Create a rate limiter with a strict global limit
    let limiter = APIRateLimiter::new(RateLimit::new(Some(5), None, None, 0));
    
    // First 5 requests should be allowed
    for i in 0..5 {
        let result = limiter.check_request("client1", "/api/test");
        assert!(result.is_ok(), "Request {} should be allowed", i);
    }
    
    // 6th request should be blocked
    let result = limiter.check_request("client1", "/api/test");
    assert!(result.is_err(), "6th request should be blocked");
    if let Err(err) = result {
        assert!(format!("{:?}", err).contains("global"));
    }
}

#[test]
fn test_per_client_rate_limiting() {
    let limiter = APIRateLimiter::new(RateLimit::unlimited());
    
    // Set a limit for client1
    limiter.set_client_limit("client1".to_string(), RateLimit::new(Some(3), None, None, 0));
    
    // client1 should be limited to 3 requests
    for i in 0..3 {
        let result = limiter.check_request("client1", "/api/test");
        assert!(result.is_ok(), "Client1 request {} should be allowed", i);
    }
    
    // 4th request from client1 should be blocked
    let result = limiter.check_request("client1", "/api/test");
    assert!(result.is_err(), "4th request from client1 should be blocked");
    if let Err(err) = result {
        assert!(format!("{:?}", err).contains("client"));
    }
    
    // client2 should not be limited (no specific limit set)
    for i in 0..10 {
        let result = limiter.check_request("client2", "/api/test");
        assert!(result.is_ok(), "Client2 request {} should be allowed", i);
    }
}

#[test]
fn test_per_endpoint_rate_limiting() {
    let limiter = APIRateLimiter::new(RateLimit::unlimited());
    
    // Set a limit for a specific endpoint
    limiter.set_endpoint_limit("/api/restricted".to_string(), RateLimit::new(Some(2), None, None, 0));
    
    // The restricted endpoint should allow only 2 requests
    for i in 0..2 {
        let result = limiter.check_request("client1", "/api/restricted");
        assert!(result.is_ok(), "Restricted endpoint request {} should be allowed", i);
    }
    
    // 3rd request to the restricted endpoint should be blocked
    let result = limiter.check_request("client1", "/api/restricted");
    assert!(result.is_err(), "3rd request to restricted endpoint should be blocked");
    if let Err(err) = result {
        assert!(format!("{:?}", err).contains("endpoint"));
    }
    
    // Other endpoints should not be limited
    for i in 0..10 {
        let result = limiter.check_request("client1", "/api/unlimited");
        assert!(result.is_ok(), "Unrestricted endpoint request {} should be allowed", i);
    }
}

#[test]
fn test_burst_capacity() {
    // Create a rate limiter with 2 requests per second and burst capacity of 3
    let limiter = APIRateLimiter::new(RateLimit::new(Some(2), None, None, 3));
    
    // Should allow 5 requests immediately (2 regular + 3 burst)
    for i in 0..5 {
        let result = limiter.check_request("client1", "/api/test");
        assert!(result.is_ok(), "Burst request {} should be allowed", i);
    }
    
    // 6th request should be blocked
    let result = limiter.check_request("client1", "/api/test");
    assert!(result.is_err(), "6th request should be blocked");
}

#[test]
fn test_multiple_rate_limits() {
    let limiter = APIRateLimiter::new(RateLimit::new(Some(10), Some(30), None, 0));
    
    // Test per-second limit
    for i in 0..10 {
        let result = limiter.check_request("client1", "/api/test");
        assert!(result.is_ok(), "Per-second request {} should be allowed", i);
    }
    
    // 11th per-second request should be blocked
    let result = limiter.check_request("client1", "/api/test");
    assert!(result.is_err(), "11th per-second request should be blocked");
    
    // Test per-minute limit (this is a simplified test)
    // In a real scenario, we would need to simulate time passing
}

#[test]
fn test_rate_limit_statistics() {
    let limiter = APIRateLimiter::new(RateLimit::new(Some(2), None, None, 0));
    
    // Make some requests
    for _ in 0..3 {
        let _ = limiter.check_request("client1", "/api/test");
    }
    
    let stats = limiter.get_statistics();
    assert_eq!(stats.total_requests, 3);
    assert_eq!(stats.blocked_requests, 1);
    assert_eq!(stats.allowed_requests, 2);
    assert!(stats.block_rate > 0.0);
}

#[test]
fn test_remaining_requests() {
    let limiter = APIRateLimiter::new(RateLimit::new(Some(5), None, None, 1));
    
    // Make 3 requests
    for _ in 0..3 {
        let _ = limiter.check_request("client1", "/api/test");
    }
    
    let info = limiter.get_remaining_requests("client1");
    // Should have 3 remaining (5 limit + 1 burst - 3 used)
    assert_eq!(info.remaining_per_second, Some(3));
}

#[test]
fn test_unlimited_rate_limit() {
    let limiter = APIRateLimiter::new(RateLimit::unlimited());
    
    // Should allow many requests
    for i in 0..100 {
        let result = limiter.check_request("client1", "/api/test");
        assert!(result.is_ok(), "Unlimited request {} should be allowed", i);
    }
}

#[test]
fn test_permissive_rate_limit() {
    let limiter = APIRateLimiter::new(RateLimit::permissive());
    
    // Should allow many requests
    for i in 0..150 {
        let result = limiter.check_request("client1", "/api/test");
        assert!(result.is_ok(), "Permissive request {} should be allowed", i);
    }
}

#[test]
fn test_strict_rate_limit() {
    let limiter = APIRateLimiter::new(RateLimit::strict());
    
    // Should allow only 10 requests per second
    for i in 0..10 {
        let result = limiter.check_request("client1", "/api/test");
        assert!(result.is_ok(), "Strict request {} should be allowed", i);
    }
    
    // 11th request should be blocked
    let result = limiter.check_request("client1", "/api/test");
    assert!(result.is_err(), "11th strict request should be blocked");
}