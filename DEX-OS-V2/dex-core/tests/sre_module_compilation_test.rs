//! Simple test to verify that our new SRE modules can be imported and compiled.

#[test]
fn test_sre_module_imports() {
    // This test simply verifies that the modules can be imported without compilation errors
    let _canary = dex_core::canary_release::CanaryManager::new();
    let _chaos = dex_core::chaos_engineering::ChaosExperiment::new(
        "test".to_string(),
        "Test experiment".to_string(),
        "test-service".to_string(),
        dex_core::chaos_engineering::FailureType::Unavailable,
        0.1,
        3600,
    );
    let _rate_limit = dex_core::rate_limiting::RateLimiter::new(
        dex_core::rate_limiting::RateLimitConfig::new(100, 10).unwrap()
    );
    
    // If we can compile this test, it means our modules are syntactically correct
    assert!(true);
}