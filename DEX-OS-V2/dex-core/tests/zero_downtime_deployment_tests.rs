//! Tests for Zero-Downtime Deployment features (Rolling Update, Feature Toggle).
//!
//! This module provides full validation of the Priority 3 Zero-Downtime Deployment features from DEX-OS-V2.csv:
//! - Zero-Downtime Deployment,Zero-Downtime Deployment,Zero-Downtime Deployment,Rolling Update,Incremental Replacement,Medium
//! - Zero-Downtime Deployment,Zero-Downtime Deployment,Zero-Downtime Deployment,Feature Toggle,Conditional Execution,Medium

use dex_core::rolling_update::{RollingUpdateConfig, RollingUpdateManager, RollingUpdateError};
use dex_core::feature_toggle::{FeatureToggleConfig, FeatureToggleManager, FeatureToggleError};

/// Test Rolling Update basic functionality
#[test]
fn test_rolling_update_basic_functionality() {
    let config = RollingUpdateConfig::new(
        "test-update".to_string(),
        "Test rolling update".to_string(),
        10, // 10 instances
        2,  // 2 instances per batch
        30, // 30 seconds delay
    ).unwrap();
    
    assert_eq!(config.id, "test-update");
    assert_eq!(config.total_instances, 10);
    assert_eq!(config.batch_size, 2);
    assert_eq!(config.batch_delay_seconds, 30);
    assert_eq!(config.current_batch, 0);
    assert_eq!(config.updated_instances, 0);
    assert!(!config.completed);
    assert!(config.is_active().unwrap());
}

/// Test Rolling Update with batch processing
#[test]
fn test_rolling_update_batch_processing() {
    let mut manager = RollingUpdateManager::new();
    
    let config = RollingUpdateConfig::new(
        "batch-update".to_string(),
        "Batch processing update".to_string(),
        8, // 8 instances
        3, // 3 instances per batch
        15, // 15 seconds delay
    ).unwrap();
    
    assert!(manager.register_update(config).is_ok());
    
    // Process batches
    assert!(manager.process_next_batch("batch-update", 3).is_ok());
    let update = manager.get_update("batch-update").unwrap();
    assert_eq!(update.updated_instances, 3);
    assert_eq!(update.current_batch, 1);
    assert!(!update.completed);
    assert_eq!(update.progress_percentage(), 37.5);
    
    assert!(manager.process_next_batch("batch-update", 3).is_ok());
    let update = manager.get_update("batch-update").unwrap();
    assert_eq!(update.updated_instances, 6);
    assert_eq!(update.current_batch, 2);
    assert!(!update.completed);
    assert_eq!(update.progress_percentage(), 75.0);
    
    assert!(manager.process_next_batch("batch-update", 2).is_ok());
    let update = manager.get_update("batch-update").unwrap();
    assert_eq!(update.updated_instances, 8);
    assert_eq!(update.current_batch, 3);
    assert!(update.completed);
    assert_eq!(update.progress_percentage(), 100.0);
}

/// Test Rolling Update error handling
#[test]
fn test_rolling_update_error_handling() {
    // Test invalid batch size
    assert_eq!(
        RollingUpdateConfig::new(
            "invalid".to_string(),
            "Invalid update".to_string(),
            10,
            0, // Invalid batch size
            30,
        ),
        Err(RollingUpdateError::InvalidBatchSize(0))
    );
    
    let mut manager = RollingUpdateManager::new();
    
    // Test processing non-existent update
    assert_eq!(
        manager.process_next_batch("nonexistent", 1),
        Err(RollingUpdateError::RollingUpdateNotFound("nonexistent".to_string()))
    );
    
    // Test removing non-existent update
    assert_eq!(
        manager.remove_update("nonexistent"),
        Err(RollingUpdateError::RollingUpdateNotFound("nonexistent".to_string()))
    );
}

/// Test Feature Toggle basic functionality
#[test]
fn test_feature_toggle_basic_functionality() {
    let config = FeatureToggleConfig::new(
        "test-feature".to_string(),
        "Test feature".to_string(),
        true, // Enabled
    );
    
    assert_eq!(config.id, "test-feature");
    assert_eq!(config.description, "Test feature");
    assert!(config.enabled);
    assert_eq!(config.percentage, 1.0);
    assert!(config.user_groups.is_empty());
    
    // Test that enabled feature is active for users
    assert!(config.is_active_for_user("user1").unwrap());
}

/// Test Feature Toggle with percentage rollout
#[test]
fn test_feature_toggle_percentage_rollout() {
    let mut config = FeatureToggleConfig::new(
        "percentage-feature".to_string(),
        "Percentage feature".to_string(),
        true,
    );
    
    // Set to 30% rollout
    assert!(config.set_percentage(0.3).is_ok());
    assert_eq!(config.percentage, 0.3);
    
    // Test with multiple users to verify percentage distribution
    let mut active_count = 0;
    let test_users: Vec<String> = (0..1000).map(|i| format!("user{}", i)).collect();
    
    for user in &test_users {
        if config.is_active_for_user(user).unwrap() {
            active_count += 1;
        }
    }
    
    let actual_percentage = active_count as f64 / test_users.len() as f64;
    // Allow for some variance due to hashing (within 5%)
    assert!(
        actual_percentage > 0.25 && actual_percentage < 0.35,
        "Expected ~30% activation, got {:.2}%",
        actual_percentage * 100.0
    );
}

/// Test Feature Toggle enable/disable operations
#[test]
fn test_feature_toggle_enable_disable() {
    let mut manager = FeatureToggleManager::new();
    
    let config = FeatureToggleConfig::new(
        "toggle-test".to_string(),
        "Toggle test feature".to_string(),
        false, // Initially disabled
    );
    
    assert!(manager.register_feature(config).is_ok());
    
    // Test that disabled feature is not active
    assert!(!manager.is_feature_active("toggle-test", "user1").unwrap());
    
    // Enable the feature
    assert!(manager.enable_feature("toggle-test").is_ok());
    assert!(manager.is_feature_active("toggle-test", "user1").unwrap());
    
    // Disable the feature
    assert!(manager.disable_feature("toggle-test").is_ok());
    assert!(!manager.is_feature_active("toggle-test", "user1").unwrap());
}

/// Test Feature Toggle error handling
#[test]
fn test_feature_toggle_error_handling() {
    // Test invalid percentage values
    assert_eq!(
        FeatureToggleConfig::new("invalid".to_string(), "Invalid feature".to_string(), true)
            .with_percentage(1.5),
        Err(FeatureToggleError::InvalidPercentage(1.5))
    );
    
    assert_eq!(
        FeatureToggleConfig::new("invalid".to_string(), "Invalid feature".to_string(), true)
            .with_percentage(-0.1),
        Err(FeatureToggleError::InvalidPercentage(-0.1))
    );
    
    let mut manager = FeatureToggleManager::new();
    
    // Test operations on non-existent feature
    assert_eq!(
        manager.is_feature_active("nonexistent", "user1"),
        Err(FeatureToggleError::FeatureNotFound("nonexistent".to_string()))
    );
    
    assert_eq!(
        manager.enable_feature("nonexistent"),
        Err(FeatureToggleError::FeatureNotFound("nonexistent".to_string()))
    );
    
    assert_eq!(
        manager.disable_feature("nonexistent"),
        Err(FeatureToggleError::FeatureNotFound("nonexistent".to_string()))
    );
    
    assert_eq!(
        manager.set_feature_percentage("nonexistent", 0.5),
        Err(FeatureToggleError::FeatureNotFound("nonexistent".to_string()))
    );
}

/// Test integration between Rolling Update and Feature Toggle
#[test]
fn test_zero_downtime_deployment_integration() {
    // Create rolling update manager
    let mut rolling_update_manager = RollingUpdateManager::new();
    
    // Create feature toggle manager
    let mut feature_toggle_manager = FeatureToggleManager::new();
    
    // Register a rolling update
    let update_config = RollingUpdateConfig::new(
        "integration-update".to_string(),
        "Integration test update".to_string(),
        6, // 6 instances
        2, // 2 instances per batch
        10, // 10 seconds delay
    ).unwrap();
    
    assert!(rolling_update_manager.register_update(update_config).is_ok());
    
    // Register a feature toggle
    let feature_config = FeatureToggleConfig::new(
        "integration-feature".to_string(),
        "Integration test feature".to_string(),
        false, // Initially disabled
    );
    
    assert!(feature_toggle_manager.register_feature(feature_config).is_ok());
    
    // Simulate deployment process
    let mut deployment_complete = false;
    let mut batch_count = 0;
    
    while !deployment_complete {
        batch_count += 1;
        
        // Process next batch of rolling update
        let instances_in_batch = if batch_count <= 3 { 2 } else { 0 };
        if instances_in_batch > 0 {
            assert!(rolling_update_manager.process_next_batch("integration-update", instances_in_batch).is_ok());
        }
        
        // Check if deployment is complete
        let update = rolling_update_manager.get_update("integration-update").unwrap();
        if update.completed {
            deployment_complete = true;
            
            // Enable the new feature after successful deployment
            assert!(feature_toggle_manager.enable_feature("integration-feature").is_ok());
        }
        
        // Verify feature state during deployment
        if deployment_complete {
            // After deployment, feature should be enabled
            assert!(feature_toggle_manager.is_feature_active("integration-feature", "user1").unwrap());
        } else {
            // During deployment, feature should be disabled
            assert!(!feature_toggle_manager.is_feature_active("integration-feature", "user1").unwrap());
        }
    }
    
    // Verify final state
    let update = rolling_update_manager.get_update("integration-update").unwrap();
    assert!(update.completed);
    assert_eq!(update.updated_instances, 6);
    assert_eq!(update.current_batch, 3);
    
    let feature = feature_toggle_manager.get_feature("integration-feature").unwrap();
    assert!(feature.enabled);
    assert_eq!(feature.percentage, 1.0);
}

/// Test Rolling Update progress tracking
#[test]
fn test_rolling_update_progress_tracking() {
    let mut config = RollingUpdateConfig::new(
        "progress-test".to_string(),
        "Progress tracking test".to_string(),
        100, // 100 instances
        10,  // 10 instances per batch
        30,  // 30 seconds delay
    ).unwrap();
    
    assert_eq!(config.progress_percentage(), 0.0);
    
    // Update 25 instances
    config.mark_batch_updated(25).unwrap();
    assert_eq!(config.progress_percentage(), 25.0);
    
    // Update 50 more instances
    config.mark_batch_updated(50).unwrap();
    assert_eq!(config.progress_percentage(), 75.0);
    
    // Update remaining 25 instances
    config.mark_batch_updated(25).unwrap();
    assert_eq!(config.progress_percentage(), 100.0);
    assert!(config.completed);
}

/// Test Feature Toggle with user groups
#[test]
fn test_feature_toggle_user_groups() {
    let config = FeatureToggleConfig::new(
        "group-feature".to_string(),
        "User group feature".to_string(),
        true,
    )
    .with_percentage(0.5) // 50% general rollout
    .unwrap()
    .with_user_groups(vec!["beta-testers".to_string(), "premium-users".to_string()]);
    
    // Test that the configuration is valid
    assert_eq!(config.user_groups.len(), 2);
    assert!(config.user_based);
    
    // Note: Detailed user group testing would require more complex mocking
    // For now, we verify the configuration is set correctly
}

/// Test Rolling Update edge cases
#[test]
fn test_rolling_update_edge_cases() {
    // Test single instance update
    let mut config = RollingUpdateConfig::new(
        "single-instance".to_string(),
        "Single instance update".to_string(),
        1, // 1 instance
        1, // 1 instance per batch
        5, // 5 seconds delay
    ).unwrap();
    
    assert_eq!(config.current_batch_size(), 1);
    assert!(config.is_active().unwrap());
    
    // Process the single instance
    config.mark_batch_updated(1).unwrap();
    assert_eq!(config.updated_instances, 1);
    assert!(config.completed);
    assert_eq!(config.current_batch_size(), 0);
    
    // Test zero instances (edge case)
    let config = RollingUpdateConfig::new(
        "zero-instances".to_string(),
        "Zero instances update".to_string(),
        0, // 0 instances
        1, // 1 instance per batch
        5, // 5 seconds delay
    ).unwrap();
    
    assert_eq!(config.current_batch_size(), 0);
    assert_eq!(config.progress_percentage(), 100.0); // Already complete
    assert!(config.completed);
}

/// Test Feature Toggle time-based activation
#[test]
fn test_feature_toggle_time_based_activation() {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let start = now - 1000; // Started 1 second ago
    let end = now + 3600000; // Ends in 1 hour
    
    let config = FeatureToggleConfig::new(
        "time-based-feature".to_string(),
        "Time-based feature".to_string(),
        true,
    )
    .with_time_window(start, end)
    .unwrap();
    
    // Feature should be active since we're within the time window
    assert!(config.is_active_for_user("user1").unwrap());
    
    // Test with a feature that hasn't started yet
    let future_start = now + 3600000; // Starts in 1 hour
    let future_end = now + 7200000;   // Ends in 2 hours
    
    let future_config = FeatureToggleConfig::new(
        "future-feature".to_string(),
        "Future feature".to_string(),
        true,
    )
    .with_time_window(future_start, future_end)
    .unwrap();
    
    // Feature should not be active since it hasn't started yet
    assert!(!future_config.is_active_for_user("user1").unwrap());
}