use dex_core::security::{
    ring_buffer::RingBuffer,
    whitelist_blacklist::WhitelistBlacklistManager,
    data_sanitization::DataValidator,
    whitelist_blacklist::ListError,
    token_validation::{TokenValidationError, TokenManager},
};

use std::thread;
use std::time::Duration;

#[test]
fn test_ring_buffer_functionality() {
    let mut rb = RingBuffer::new(3);
    
    // Test empty
    assert!(rb.is_empty());
    assert_eq!(rb.len(), 0);
    assert_eq!(rb.pop(), None);

    // Test fill
    rb.push(1);
    rb.push(2);
    rb.push(3);
    assert!(rb.is_full());
    assert_eq!(rb.len(), 3);
    assert_eq!(rb.peek(), Some(&1));

    // Test overflow (should drop 1)
    rb.push(4);
    assert_eq!(rb.len(), 3);
    assert_eq!(rb.peek(), Some(&2));

    // Test pop order
    assert_eq!(rb.pop(), Some(2));
    assert_eq!(rb.pop(), Some(3));
    assert_eq!(rb.pop(), Some(4));
    assert_eq!(rb.pop(), None);
}

#[test]
fn test_whitelist_blacklist_logic() {
    let manager = WhitelistBlacklistManager::new();

    // IP Blacklist
    manager.add_to_blacklist("ip", "1.2.3.4").unwrap();
    assert!(!manager.is_ip_allowed("1.2.3.4"));
    assert!(manager.is_ip_allowed("1.2.3.5")); // Default allow

    // User Whitelist
    manager.add_to_whitelist("user", "admin").unwrap();
    assert!(manager.is_user_allowed("admin"));
    assert!(!manager.is_user_allowed("guest")); // Not in whitelist implies block if whitelist exists
    
    // Conflict check
    assert_eq!(
        manager.add_to_blacklist("user", "admin"),
        Err(ListError::Conflict)
    );
}

#[test]
fn test_token_validation_flow() {
    // Use TokenManager to create tokens, then get validator to validate them
    let manager = TokenManager::new("dex-os", "dex-client");
    let validator = manager.get_validator();
    
    // Create valid token using the manager
    let token = manager.create_token("user1", vec!["admin".to_string()], 3600);
    let claims = validator.validate_token(&token).expect("Token should be valid");
    
    assert_eq!(claims.sub, "user1");
    assert_eq!(claims.iss, "dex-os");
    assert!(claims.roles.contains(&"admin".to_string()));

    // Create expired token
    let short_token = manager.create_token("user2", vec![], 1);
    thread::sleep(Duration::from_secs(2));
    
    let result = validator.validate_token(&short_token);
    assert_eq!(result.err(), Some(TokenValidationError::Expired));
}

#[test]
fn test_data_validation_strict() {
    let validator = DataValidator::new();

    // Email
    assert!(validator.validate_email("test@example.com"));
    assert!(!validator.validate_email("invalid-email"));

    // URL
    assert!(validator.validate_url("https://example.com"));
    assert!(!validator.validate_url("javascript:alert(1)"));

    // Password Strength
    assert!(validator.validate_password_strength("StrongP@ss1")); // 11 chars, Upper, Lower, Digit, Special
    assert!(!validator.validate_password_strength("weak")); // Too short
    assert!(!validator.validate_password_strength("onlylowercase1")); // No upper/special
    
    // UUID
    assert!(validator.validate_uuid("123e4567-e89b-12d3-a456-426614174000"));
    assert!(!validator.validate_uuid("invalid-uuid"));
}

#[test]
fn test_rate_limiting_concept_with_ring_buffer() {
    // This test demonstrates how a RingBuffer can be used for rate limiting (storing timestamps)
    let mut request_timestamps = RingBuffer::new(5); // Window size 5
    let now = 1000;

    // Add 5 requests
    for i in 0..5 {
        request_timestamps.push(now + i);
    }

    assert!(request_timestamps.is_full());

    // 6th request comes in
    // In a real rate limiter, we'd check if the oldest timestamp is within the window.
    // If it is, we block. If it's old, we pop it and push new.
    
    let _oldest = request_timestamps.peek().unwrap();
    let _window_size = 10;
    
    // If oldest is within window (now + 0 vs now + 6), it depends on the logic.
    // Here we just verify the buffer holds the last N requests.
    
    request_timestamps.push(now + 100);
    assert_eq!(request_timestamps.peek(), Some(&(now + 1))); // Oldest (now+0) was dropped
}
