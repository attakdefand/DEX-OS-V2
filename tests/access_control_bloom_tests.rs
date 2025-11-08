//! Tests for Access Control using Bloom Filter

use dex_core::security::{SecurityManager, ClassificationLevel};

#[test]
fn test_access_control_with_bloom_filter() {
    let mut manager = SecurityManager::new();
    let owner = "owner".to_string();
    let user1 = "user1".to_string();
    let user2 = "user2".to_string();
    let user3 = "user3".to_string();

    // Classify data
    manager.classify_data(
        "data1".to_string(),
        ClassificationLevel::Confidential,
        owner.clone(),
        vec![user1.clone(), user2.clone()],
    );

    // Check access using Bloom filter optimization
    assert!(manager.check_data_access("data1", &owner)); // Owner has access
    assert!(manager.check_data_access("data1", &user1)); // ACL user has access
    assert!(manager.check_data_access("data1", &user2)); // ACL user has access
    assert!(!manager.check_data_access("data1", &user3)); // Other user doesn't have access

    // Add user to ACL
    assert!(manager.add_user_to_acl("data1", user3.clone()).is_ok());
    assert!(manager.check_data_access("data1", &user3)); // Now user3 has access

    // Public data (not classified) should be accessible
    assert!(manager.check_data_access("public_data", &user1));
}

#[test]
fn test_bloom_filter_performance_optimization() {
    let mut manager = SecurityManager::new();
    let owner = "owner".to_string();
    let user1 = "user1".to_string();
    
    // Add many users to the access control system
    for i in 0..1000 {
        manager.classify_data(
            format!("data_{}", i),
            ClassificationLevel::Confidential,
            owner.clone(),
            vec![format!("user_{}", i)],
        );
    }
    
    // Check that a user not in any ACL is quickly rejected by the Bloom filter
    let non_user = "non_existent_user".to_string();
    assert!(!manager.check_data_access("data_1", &non_user));
    
    // Check that actual users are correctly identified
    assert!(manager.check_data_access("data_1", &user1));
}