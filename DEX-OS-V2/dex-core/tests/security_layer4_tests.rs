//! Comprehensive test suite for Security Layer 4 - API & Gateway Security
//!
//! Tests all components of the API security system including:
//! - API Rate Limiter (sliding window, per-client, per-endpoint)
//! - API Key Manager (generation, validation, rotation, scopes)
//! - CORS Policy (origin validation, preflight handling)
//! - API Gateway (routing, authentication, authorization)

use dex_core::security::*;

// ============================================================================
// API RATE LIMITER TESTS
// ============================================================================

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
fn test_rate_limiter_per_client_isolation() {
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

// ============================================================================
// API KEY MANAGER TESTS
// ============================================================================

#[test]
fn test_api_key_generation() {
    let manager = APIKeyManager::new(Some(3600));
    let (key_id, key) = manager
        .generate_key("client1".to_string(), vec!["read".to_string()], None)
        .unwrap();

    assert!(key_id.starts_with("key_"));
    assert!(key.starts_with("dex_"));
    assert_eq!(key.len(), 36); // "dex_" + 32 chars
}

#[test]
fn test_api_key_validation() {
    let manager = APIKeyManager::new(None);
    let (_, key) = manager
        .generate_key("client1".to_string(), vec!["read".to_string()], None)
        .unwrap();

    let api_key = manager.validate_key(&key).unwrap();
    assert_eq!(api_key.client_id, "client1");
    assert!(api_key.has_scope("read"));
    assert!(!api_key.has_scope("write"));
}

#[test]
fn test_api_key_expiration() {
    let manager = APIKeyManager::new(None);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create key that expires in the past
    let (_, key) = manager
        .generate_key("client1".to_string(), vec!["read".to_string()], Some(now - 1))
        .unwrap();

    // Should fail validation
    let result = manager.validate_key(&key);
    assert!(result.is_err());
}

#[test]
fn test_api_key_revocation() {
    let manager = APIKeyManager::new(None);
    let (key_id, key) = manager
        .generate_key("client1".to_string(), vec!["read".to_string()], None)
        .unwrap();

    // Revoke key
    manager.revoke_key(&key_id).unwrap();

    // Should fail validation
    let result = manager.validate_key(&key);
    assert!(result.is_err());
}

#[test]
fn test_api_key_scope_validation() {
    let manager = APIKeyManager::new(None);
    let (_, key) = manager
        .generate_key("client1".to_string(), vec!["read".to_string()], None)
        .unwrap();

    // Should succeed for "read" scope
    assert!(manager.validate_key_with_scope(&key, "read").is_ok());

    // Should fail for "write" scope
    let result = manager.validate_key_with_scope(&key, "write");
    assert!(result.is_err());
}

#[test]
fn test_api_key_rotation() {
    let manager = APIKeyManager::new(None);
    let (old_key_id, old_key) = manager
        .generate_key("client1".to_string(), vec!["read".to_string()], None)
        .unwrap();

    // Rotate key
    let (new_key_id, new_key) = manager.rotate_key(&old_key_id).unwrap();

    // Old key should be disabled
    assert!(manager.validate_key(&old_key).is_err());

    // New key should work
    let api_key = manager.validate_key(&new_key).unwrap();
    assert_eq!(api_key.id, new_key_id);
    assert_eq!(api_key.client_id, "client1");
}

// ============================================================================
// CORS POLICY TESTS
// ============================================================================

#[test]
fn test_cors_policy_permissive() {
    let policy = CORSPolicy::permissive();
    assert!(policy.is_origin_allowed("https://example.com"));
    assert!(policy.is_origin_allowed("https://any-origin.com"));
    assert!(policy.is_method_allowed(&HttpMethod::GET));
    assert!(policy.is_method_allowed(&HttpMethod::POST));
}

#[test]
fn test_cors_policy_strict() {
    let policy = CORSPolicy::strict(vec!["https://example.com".to_string()]);
    assert!(policy.is_origin_allowed("https://example.com"));
    assert!(!policy.is_origin_allowed("https://other.com"));
}

#[test]
fn test_cors_headers_generation() {
    let mut policy = CORSPolicy::new();
    policy.add_origin("https://example.com".to_string());
    policy.add_method(HttpMethod::GET);
    policy.add_header("Content-Type".to_string());

    let headers = policy.get_cors_headers(Some("https://example.com"));
    
    // Should have Access-Control-Allow-Origin
    assert!(headers.iter().any(|(k, v)| k == "Access-Control-Allow-Origin" && v == "https://example.com"));
    
    // Should have Access-Control-Allow-Methods
    assert!(headers.iter().any(|(k, _)| k == "Access-Control-Allow-Methods"));
}

#[test]
fn test_cors_preflight_success() {
    let mut policy = CORSPolicy::new();
    policy.add_origin("https://example.com".to_string());
    policy.add_method(HttpMethod::POST);
    policy.add_header("Content-Type".to_string());

    let result = policy.handle_preflight(
        "https://example.com",
        &HttpMethod::POST,
        &vec!["Content-Type".to_string()],
    );

    assert!(result.is_ok());
}

#[test]
fn test_cors_preflight_blocked() {
    let policy = CORSPolicy::strict(vec!["https://example.com".to_string()]);

    // Wrong origin
    let result = policy.handle_preflight(
        "https://malicious.com",
        &HttpMethod::GET,
        &vec![],
    );
    assert!(result.is_err());
}

// ============================================================================
// API GATEWAY INTEGRATION TESTS
// ============================================================================

#[test]
fn test_api_gateway_public_route() {
    use std::sync::Arc;
    
    let key_manager = Arc::new(APIKeyManager::default());
    let rate_limiter = Arc::new(APIRateLimiter::new(RateLimit::permissive()));
    let cors_policy = CORSPolicy::permissive();

    let mut gateway = APIGateway::new(key_manager, rate_limiter, cors_policy);

    let route = RouteConfig {
        path: "/api/public".to_string(),
        methods: vec![HttpMethod::GET],
        auth_required: false,
        required_scopes: vec![],
        rate_limit: None,
    };

    gateway.register_route(route);

    let result = gateway.process_request("/api/public", &HttpMethod::GET, None, None);
    assert!(result.is_ok());
}

#[test]
fn test_api_gateway_requires_auth() {
    use std::sync::Arc;
    
    let key_manager = Arc::new(APIKeyManager::default());
    let rate_limiter = Arc::new(APIRateLimiter::new(RateLimit::permissive()));
    let cors_policy = CORSPolicy::permissive();

    let mut gateway = APIGateway::new(key_manager, rate_limiter, cors_policy);

    let route = RouteConfig {
        path: "/api/private".to_string(),
        methods: vec![HttpMethod::GET],
        auth_required: true,
        required_scopes: vec![],
        rate_limit: None,
    };

    gateway.register_route(route);

    let result = gateway.process_request("/api/private", &HttpMethod::GET, None, None);
    assert!(result.is_err());
}

#[test]
fn test_api_gateway_with_valid_key() {
    use std::sync::Arc;
    
    let key_manager = Arc::new(APIKeyManager::default());
    let (_, key) = key_manager
        .generate_key("client1".to_string(), vec!["read".to_string()], None)
        .unwrap();

    let rate_limiter = Arc::new(APIRateLimiter::new(RateLimit::permissive()));
    let cors_policy = CORSPolicy::permissive();

    let mut gateway = APIGateway::new(key_manager, rate_limiter, cors_policy);

    let route = RouteConfig {
        path: "/api/private".to_string(),
        methods: vec![HttpMethod::GET],
        auth_required: true,
        required_scopes: vec!["read".to_string()],
        rate_limit: None,
    };

    gateway.register_route(route);

    let result = gateway.process_request("/api/private", &HttpMethod::GET, Some(&key), None);
    assert!(result.is_ok());
}

#[test]
fn test_api_gateway_insufficient_scope() {
    use std::sync::Arc;
    
    let key_manager = Arc::new(APIKeyManager::default());
    let (_, key) = key_manager
        .generate_key("client1".to_string(), vec!["read".to_string()], None)
        .unwrap();

    let rate_limiter = Arc::new(APIRateLimiter::new(RateLimit::permissive()));
    let cors_policy = CORSPolicy::permissive();

    let mut gateway = APIGateway::new(key_manager, rate_limiter, cors_policy);

    let route = RouteConfig {
        path: "/api/admin".to_string(),
        methods: vec![HttpMethod::POST],
        auth_required: true,
        required_scopes: vec!["admin".to_string()],
        rate_limit: None,
    };

    gateway.register_route(route);

    let result = gateway.process_request("/api/admin", &HttpMethod::POST, Some(&key), None);
    assert!(result.is_err());
}
