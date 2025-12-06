//! Tests for SRE Patterns features (Canary Releases, Chaos Engineering, Handling Overload).
//!
//! This module provides full validation of the Priority 3 SRE Patterns features from DEX-OS-V2.csv:
//! - SRE Patterns,SRE Patterns,SRE Patterns,Canary Releases,Traffic Splitting,Medium
//! - SRE Patterns,SRE Patterns,SRE Patterns,Chaos Engineering,Failure Injection,Medium
//! - SRE Patterns,SRE Patterns,SRE Patterns,Handling Overload,Rate Limiting,Medium

use dex_core::canary_release::{CanaryConfig, CanaryManager, RolloutStep};
use dex_core::chaos_engineering::{ChaosExperiment, ChaosManager, FailureType};
use dex_core::rate_limiting::{RateLimitConfig, TieredRateLimiter};

/// Test Canary Release basic functionality
#[test]
fn test_canary_release_basic_functionality() {
    let config = CanaryConfig::new(
        "test-canary".to_string(),
        "Test canary release".to_string(),
        0.5, // 50% traffic
        3600, // 1 hour
    ).unwrap();
    
    assert_eq!(config.id, "test-canary");
    assert_eq!(config.traffic_percentage, 0.5);
    assert_eq!(config.duration_seconds, 3600);
    assert!(config.is_active().unwrap());
}

/// Test Canary Release with gradual rollout
#[test]
fn test_canary_release_gradual_rollout() {
    let steps = vec![
        RolloutStep {
            traffic_percentage: 0.1,
            duration_seconds: 1800, // 30 minutes
        },
        RolloutStep {
            traffic_percentage: 0.3,
            duration_seconds: 1800, // 30 minutes
        },
        RolloutStep {
            traffic_percentage: 0.5,
            duration_seconds: 1800, // 30 minutes
        },
    ];
    
    let config = CanaryConfig::new(
        "gradual-canary".to_string(),
        "Gradual canary release".to_string(),
        0.5, // Final percentage
        5400, // 1.5 hours total
    ).unwrap().with_gradual_rollout(steps).unwrap();
    
    assert!(config.gradual_rollout);
    assert_eq!(config.rollout_steps.len(), 3);
    assert_eq!(config.rollout_steps[0].traffic_percentage, 0.1);
    assert_eq!(config.rollout_steps[1].traffic_percentage, 0.3);
    assert_eq!(config.rollout_steps[2].traffic_percentage, 0.5);
}

/// Test Canary Manager functionality
#[test]
fn test_canary_manager_functionality() {
    let mut manager = CanaryManager::new();
    
    let config = CanaryConfig::new(
        "manager-test".to_string(),
        "Manager test canary".to_string(),
        0.2, // 20% traffic
        1800, // 30 minutes
    ).unwrap();
    
    assert!(manager.register_canary(config).is_ok());
    assert!(manager.get_canary("manager-test").is_ok());
    assert_eq!(manager.get_canary("manager-test").unwrap().traffic_percentage, 0.2);
    
    // Test routing decision (with 20% traffic, most should be false)
    let mut canary_count = 0;
    let mut total_count = 0;
    
    for _ in 0..1000 {
        if manager.should_route_to_canary("manager-test").unwrap() {
            canary_count += 1;
        }
        total_count += 1;
    }
    
    // Should be approximately 20% (allowing for some variance due to randomness)
    let percentage = canary_count as f64 / total_count as f64;
    assert!(percentage > 0.15 && percentage < 0.25, "Expected ~20%, got {:.2}%", percentage * 100.0);
    
    // Test removal
    assert!(manager.remove_canary("manager-test").is_ok());
    assert!(manager.get_canary("manager-test").is_err());
}

/// Test Chaos Engineering basic functionality
#[test]
fn test_chaos_engineering_basic_functionality() {
    let experiment = ChaosExperiment::new(
        "test-experiment".to_string(),
        "Test chaos experiment".to_string(),
        "api-service".to_string(),
        FailureType::Error {
            status_code: 500,
            message: "Internal Server Error".to_string(),
        },
        0.3, // 30% failure rate
        3600, // 1 hour
    ).unwrap();
    
    assert_eq!(experiment.id, "test-experiment");
    assert_eq!(experiment.target, "api-service");
    assert_eq!(experiment.failure_rate, 0.3);
    assert_eq!(experiment.duration_seconds, 3600);
    assert!(experiment.active);
    assert!(experiment.is_active().unwrap());
}

/// Test Chaos Engineering with latency injection
#[test]
fn test_chaos_engineering_latency_injection() {
    let experiment = ChaosExperiment::new(
        "latency-experiment".to_string(),
        "Latency injection experiment".to_string(),
        "slow-service".to_string(),
        FailureType::Latency {
            min_ms: 100,
            max_ms: 500,
        },
        1.0, // 100% failure rate for testing
        1800, // 30 minutes
    ).unwrap();
    
    // With 100% failure rate, should always apply latency
    for _ in 0..10 {
        assert!(experiment.should_apply_failure().unwrap());
        let delay = experiment.apply_latency();
        assert!(delay.is_some());
        let delay = delay.unwrap();
        assert!(delay >= std::time::Duration::from_millis(100));
        assert!(delay <= std::time::Duration::from_millis(500));
    }
}

/// Test Chaos Manager functionality
#[test]
fn test_chaos_manager_functionality() {
    let manager = ChaosManager::new();
    
    let experiment = ChaosExperiment::new(
        "manager-test".to_string(),
        "Manager test experiment".to_string(),
        "test-service".to_string(),
        FailureType::Unavailable,
        1.0, // 100% failure rate for testing
        1800, // 30 minutes
    ).unwrap();
    
    assert!(manager.register_experiment(experiment).is_ok());
    assert!(manager.get_experiment("manager-test").is_ok());
    
    // Test applying chaos
    let action = manager.apply_chaos("test-service").unwrap();
    assert!(matches!(action, Some(dex_core::chaos_engineering::ChaosAction::Unavailable)));
    
    // Test stopping experiment
    assert!(manager.stop_experiment("manager-test").is_ok());
    let experiment = manager.get_experiment("manager-test").unwrap();
    assert!(!experiment.active);
    
    // After stopping, should not apply chaos
    let action = manager.apply_chaos("test-service").unwrap();
    assert!(action.is_none());
    
    // Test removal
    assert!(manager.remove_experiment("manager-test").is_ok());
    assert!(manager.get_experiment("manager-test").is_err());
}

/// Test Handling Overload with Tiered Rate Limiting
#[test]
fn test_handling_overload_tiered_rate_limiting() {
    let global_config = RateLimitConfig::new(100, 50).unwrap(); // 100 capacity, 50 refill rate
    let default_config = RateLimitConfig::new(10, 5).unwrap();  // 10 capacity, 5 refill rate
    let limiter = TieredRateLimiter::new(global_config, default_config);
    
    // Should allow requests within limits
    for _ in 0..10 {
        assert!(limiter.check("user1").unwrap());
    }
    
    // Should reject when limit is exceeded
    assert!(limiter.check("user1").is_err());
    
    // Different user should have separate limits
    for _ in 0..10 {
        assert!(limiter.check("user2").unwrap());
    }
    
    // Test custom key limits
    let custom_config = RateLimitConfig::new(20, 10).unwrap(); // Higher limits
    assert!(limiter.set_key_limit("high-priority", custom_config).is_ok());
    
    // Should allow more requests for high-priority key
    for _ in 0..20 {
        assert!(limiter.check("high-priority").unwrap());
    }
    
    // Should reject when limit is exceeded
    assert!(limiter.check("high-priority").is_err());
}

/// Test error handling for invalid configurations
#[test]
fn test_sre_patterns_error_handling() {
    // Test invalid canary configuration
    assert!(CanaryConfig::new(
        "invalid".to_string(),
        "Invalid canary".to_string(),
        1.5, // Invalid percentage
        3600,
    ).is_err());
    
    // Test invalid chaos experiment configuration
    assert!(ChaosExperiment::new(
        "invalid".to_string(),
        "Invalid experiment".to_string(),
        "test-service".to_string(),
        FailureType::Unavailable,
        1.5, // Invalid failure rate
        3600,
    ).is_err());
    
    // Test invalid rate limit configuration
    assert!(RateLimitConfig::new(0, 10).is_err());
    assert!(RateLimitConfig::new(10, 0).is_err());
}

/// Test integration between SRE Patterns components
#[test]
fn test_sre_patterns_integration() {
    // Create a canary release
    let mut canary_manager = CanaryManager::new();
    let canary_config = CanaryConfig::new(
        "integration-canary".to_string(),
        "Integration test canary".to_string(),
        0.1, // 10% traffic
        3600, // 1 hour
    ).unwrap();
    assert!(canary_manager.register_canary(canary_config).is_ok());
    
    // Create a chaos experiment
    let chaos_manager = ChaosManager::new();
    let chaos_experiment = ChaosExperiment::new(
        "integration-chaos".to_string(),
        "Integration test chaos".to_string(),
        "integration-service".to_string(),
        FailureType::Error {
            status_code: 503,
            message: "Service Unavailable".to_string(),
        },
        0.05, // 5% failure rate
        3600, // 1 hour
    ).unwrap();
    assert!(chaos_manager.register_experiment(chaos_experiment).is_ok());
    
    // Create rate limiting
    let global_config = RateLimitConfig::new(1000, 100).unwrap();
    let default_config = RateLimitConfig::new(100, 10).unwrap();
    let rate_limiter = TieredRateLimiter::new(global_config, default_config);
    
    // Simulate handling requests with all three SRE patterns
    let mut canary_routed = 0;
    let mut chaos_applied = 0;
    let mut requests_handled = 0;
    
    for i in 0..1000 {
        let user_key = format!("user-{}", i % 100); // 100 different users
        
        // Check rate limiting
        if rate_limiter.check(&user_key).is_err() {
            continue; // Rate limited, skip request
        }
        
        // Check canary routing
        let route_to_canary = canary_manager.should_route_to_canary("integration-canary").unwrap_or(false);
        if route_to_canary {
            canary_routed += 1;
        }
        
        // Apply chaos engineering
        let chaos_action = chaos_manager.apply_chaos("integration-service").unwrap_or(None);
        if chaos_action.is_some() {
            chaos_applied += 1;
        }
        
        requests_handled += 1;
    }
    
    // Verify approximate percentages
    let canary_percentage = canary_routed as f64 / requests_handled as f64;
    let chaos_percentage = chaos_applied as f64 / requests_handled as f64;
    
    // Allow for some variance due to randomness
    assert!(canary_percentage > 0.05 && canary_percentage < 0.15, 
            "Expected ~10% canary routing, got {:.2}%", canary_percentage * 100.0);
    assert!(chaos_percentage > 0.01 && chaos_percentage < 0.09, 
            "Expected ~5% chaos application, got {:.2}%", chaos_percentage * 100.0);
}