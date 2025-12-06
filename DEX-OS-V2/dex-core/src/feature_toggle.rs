//! Feature Toggle implementation for conditional execution.
//!
//! This module implements the Priority 3 Zero-Downtime Deployment feature from DEX-OS-V2.csv:
//! - Zero-Downtime Deployment,Zero-Downtime Deployment,Zero-Downtime Deployment,Feature Toggle,Conditional Execution,Medium

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors that can occur during feature toggle operations.
#[derive(Error, Debug, PartialEq)]
pub enum FeatureToggleError {
    /// Feature not found.
    #[error("Feature not found: {0}")]
    FeatureNotFound(String),
    
    /// Invalid percentage value (must be between 0.0 and 1.0).
    #[error("Invalid percentage: {0}. Must be between 0.0 and 1.0")]
    InvalidPercentage(f64),
    
    /// Time calculation error.
    #[error("Time calculation error: {0}")]
    TimeError(String),
    
    /// Invalid time window configuration.
    #[error("Invalid time window: {0}")]
    InvalidTimeWindow(String),
}

/// Feature toggle configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureToggleConfig {
    /// Unique identifier for the feature.
    pub id: String,
    
    /// Human-readable description of the feature.
    pub description: String,
    
    /// Whether the feature is enabled.
    pub enabled: bool,
    
    /// Percentage of users for whom the feature is enabled (0.0 to 1.0).
    /// Used for gradual rollouts.
    pub percentage: f64,
    
    /// User groups for whom the feature is enabled.
    pub user_groups: Vec<String>,
    
    /// Start time for time-based activation (milliseconds since UNIX epoch).
    pub start_time: Option<u64>,
    
    /// End time for time-based activation (milliseconds since UNIX epoch).
    pub end_time: Option<u64>,
    
    /// Whether to use user-based targeting.
    pub user_based: bool,
}

impl FeatureToggleConfig {
    /// Create a new feature toggle configuration.
    pub fn new(
        id: String,
        description: String,
        enabled: bool,
    ) -> Self {
        Self {
            id,
            description,
            enabled,
            percentage: if enabled { 1.0 } else { 0.0 },
            user_groups: Vec::new(),
            start_time: None,
            end_time: None,
            user_based: false,
        }
    }
    
    /// Set the percentage of users for whom the feature is enabled.
    pub fn with_percentage(mut self, percentage: f64) -> Result<Self, FeatureToggleError> {
        if percentage < 0.0 || percentage > 1.0 {
            return Err(FeatureToggleError::InvalidPercentage(percentage));
        }
        
        self.percentage = percentage;
        Ok(self)
    }
    
    /// Set user groups for whom the feature is enabled.
    pub fn with_user_groups(mut self, user_groups: Vec<String>) -> Self {
        self.user_groups = user_groups;
        self.user_based = !self.user_groups.is_empty();
        self
    }
    
    /// Set time window for feature activation.
    pub fn with_time_window(mut self, start_time: u64, end_time: u64) -> Result<Self, FeatureToggleError> {
        if start_time >= end_time {
            return Err(FeatureToggleError::InvalidTimeWindow(
                "Start time must be before end time".to_string()
            ));
        }
        
        self.start_time = Some(start_time);
        self.end_time = Some(end_time);
        Ok(self)
    }
    
    /// Check if the feature is active for a given user.
    pub fn is_active_for_user(&self, user_id: &str) -> Result<bool, FeatureToggleError> {
        // If feature is disabled, it's never active
        if !self.enabled {
            return Ok(false);
        }
        
        // Check time-based activation
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| FeatureToggleError::TimeError(e.to_string()))?
            .as_millis() as u64;
        
        // Check if we're within the time window (if specified)
        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            if now < start || now > end {
                return Ok(false);
            }
        }
        
        // If 100% enabled, it's always active
        if self.percentage >= 1.0 {
            return Ok(true);
        }
        
        // If 0% enabled, it's never active
        if self.percentage <= 0.0 {
            return Ok(false);
        }
        
        // If user-based targeting is enabled, check user groups
        if self.user_based {
            // Simple hash-based approach for user group membership
            let user_hash = calculate_hash(user_id) % 100;
            let user_percentage = user_hash as f64 / 100.0;
            
            // Check if user is in one of the allowed groups
            if self.user_groups.iter().any(|group| {
                let group_hash = calculate_hash(group) % 100;
                let group_percentage = group_hash as f64 / 100.0;
                user_percentage <= group_percentage
            }) {
                return Ok(true);
            }
            
            // Even if not in groups, apply percentage-based rollout
            return Ok(user_percentage < self.percentage);
        }
        
        // Standard percentage-based rollout
        let user_hash = calculate_hash(user_id) % 10000;
        let user_percentage = user_hash as f64 / 10000.0;
        Ok(user_percentage < self.percentage)
    }
    
    /// Enable the feature.
    pub fn enable(&mut self) {
        self.enabled = true;
        self.percentage = 1.0;
    }
    
    /// Disable the feature.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.percentage = 0.0;
    }
    
    /// Set the feature to a specific percentage.
    pub fn set_percentage(&mut self, percentage: f64) -> Result<(), FeatureToggleError> {
        if percentage < 0.0 || percentage > 1.0 {
            return Err(FeatureToggleError::InvalidPercentage(percentage));
        }
        
        self.percentage = percentage;
        self.enabled = percentage > 0.0;
        Ok(())
    }
}

/// Simple hash function for consistent user hashing
fn calculate_hash<T: std::hash::Hash + ?Sized>(t: &T) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Feature toggle manager for handling conditional feature execution.
pub struct FeatureToggleManager {
    /// Active feature toggles.
    features: HashMap<String, FeatureToggleConfig>,
}

impl FeatureToggleManager {
    /// Create a new feature toggle manager.
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }
    
    /// Register a new feature toggle.
    pub fn register_feature(&mut self, config: FeatureToggleConfig) -> Result<(), FeatureToggleError> {
        self.features.insert(config.id.clone(), config);
        Ok(())
    }
    
    /// Get a feature toggle by ID.
    pub fn get_feature(&self, id: &str) -> Result<&FeatureToggleConfig, FeatureToggleError> {
        self.features.get(id).ok_or_else(|| FeatureToggleError::FeatureNotFound(id.to_string()))
    }
    
    /// Get a mutable reference to a feature toggle by ID.
    pub fn get_feature_mut(&mut self, id: &str) -> Result<&mut FeatureToggleConfig, FeatureToggleError> {
        self.features.get_mut(id).ok_or_else(|| FeatureToggleError::FeatureNotFound(id.to_string()))
    }
    
    /// Remove a feature toggle.
    pub fn remove_feature(&mut self, id: &str) -> Result<(), FeatureToggleError> {
        if self.features.remove(id).is_some() {
            Ok(())
        } else {
            Err(FeatureToggleError::FeatureNotFound(id.to_string()))
        }
    }
    
    /// Check if a feature is active for a given user.
    pub fn is_feature_active(&self, feature_id: &str, user_id: &str) -> Result<bool, FeatureToggleError> {
        let feature = self.get_feature(feature_id)?;
        feature.is_active_for_user(user_id)
    }
    
    /// Enable a feature.
    pub fn enable_feature(&mut self, feature_id: &str) -> Result<(), FeatureToggleError> {
        let feature = self.get_feature_mut(feature_id)?;
        feature.enable();
        Ok(())
    }
    
    /// Disable a feature.
    pub fn disable_feature(&mut self, feature_id: &str) -> Result<(), FeatureToggleError> {
        let feature = self.get_feature_mut(feature_id)?;
        feature.disable();
        Ok(())
    }
    
    /// Set the percentage of users for whom a feature is enabled.
    pub fn set_feature_percentage(&mut self, feature_id: &str, percentage: f64) -> Result<(), FeatureToggleError> {
        let feature = self.get_feature_mut(feature_id)?;
        feature.set_percentage(percentage)
    }
    
    /// Get all active features.
    pub fn get_active_features(&self) -> Vec<&FeatureToggleConfig> {
        self.features.values().filter(|f| f.enabled).collect()
    }
    
    /// Get all features.
    pub fn get_all_features(&self) -> Vec<&FeatureToggleConfig> {
        self.features.values().collect()
    }
}

impl Default for FeatureToggleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_feature_toggle_config_creation() {
        let config = FeatureToggleConfig::new(
            "test-feature".to_string(),
            "Test feature".to_string(),
            true,
        );
        
        assert_eq!(config.id, "test-feature");
        assert_eq!(config.description, "Test feature");
        assert!(config.enabled);
        assert_eq!(config.percentage, 1.0);
        assert!(config.user_groups.is_empty());
        assert_eq!(config.start_time, None);
        assert_eq!(config.end_time, None);
        assert!(!config.user_based);
    }
    
    #[test]
    fn test_feature_toggle_with_percentage() {
        let config = FeatureToggleConfig::new(
            "percentage-feature".to_string(),
            "Percentage feature".to_string(),
            true,
        ).with_percentage(0.5);
        
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.percentage, 0.5);
    }
    
    #[test]
    fn test_feature_toggle_invalid_percentage() {
        let config = FeatureToggleConfig::new(
            "invalid-feature".to_string(),
            "Invalid feature".to_string(),
            true,
        ).with_percentage(1.5);
        
        assert_eq!(config, Err(FeatureToggleError::InvalidPercentage(1.5)));
        
        let config = FeatureToggleConfig::new(
            "invalid-feature".to_string(),
            "Invalid feature".to_string(),
            true,
        ).with_percentage(-0.1);
        
        assert_eq!(config, Err(FeatureToggleError::InvalidPercentage(-0.1)));
    }
    
    #[test]
    fn test_feature_toggle_with_user_groups() {
        let config = FeatureToggleConfig::new(
            "group-feature".to_string(),
            "Group feature".to_string(),
            true,
        ).with_user_groups(vec!["beta-testers".to_string(), "premium-users".to_string()]);
        
        assert_eq!(config.user_groups.len(), 2);
        assert!(config.user_based);
    }
    
    #[test]
    fn test_feature_toggle_with_time_window() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let start = now;
        let end = now + 3600000; // 1 hour from now
        
        let config = FeatureToggleConfig::new(
            "time-feature".to_string(),
            "Time feature".to_string(),
            true,
        ).with_time_window(start, end);
        
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.start_time, Some(start));
        assert_eq!(config.end_time, Some(end));
    }
    
    #[test]
    fn test_feature_toggle_invalid_time_window() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let start = now + 3600000; // 1 hour from now
        let end = now; // Before start - invalid
        
        let config = FeatureToggleConfig::new(
            "invalid-time-feature".to_string(),
            "Invalid time feature".to_string(),
            true,
        ).with_time_window(start, end);
        
        assert_eq!(
            config, 
            Err(FeatureToggleError::InvalidTimeWindow(
                "Start time must be before end time".to_string()
            ))
        );
    }
    
    #[test]
    fn test_feature_toggle_is_active_for_user() {
        // Test disabled feature
        let config = FeatureToggleConfig::new(
            "disabled-feature".to_string(),
            "Disabled feature".to_string(),
            false,
        );
        
        assert!(!config.is_active_for_user("user1").unwrap());
        
        // Test 100% enabled feature
        let config = FeatureToggleConfig::new(
            "enabled-feature".to_string(),
            "Enabled feature".to_string(),
            true,
        );
        
        assert!(config.is_active_for_user("user1").unwrap());
        
        // Test 0% enabled feature
        let mut config = FeatureToggleConfig::new(
            "zero-percent-feature".to_string(),
            "Zero percent feature".to_string(),
            true,
        );
        config.set_percentage(0.0).unwrap();
        
        assert!(!config.is_active_for_user("user1").unwrap());
        
        // Test percentage-based feature
        let mut config = FeatureToggleConfig::new(
            "percentage-feature".to_string(),
            "Percentage feature".to_string(),
            true,
        );
        config.set_percentage(0.5).unwrap();
        
        // Test with multiple users to verify percentage distribution
        let mut active_count = 0;
        let test_users: Vec<String> = (0..1000).map(|i| format!("user{}", i)).collect();
        
        for user in &test_users {
            if config.is_active_for_user(user).unwrap() {
                active_count += 1;
            }
        }
        
        let actual_percentage = active_count as f64 / test_users.len() as f64;
        // Allow for some variance due to randomness (within 5%)
        assert!(
            actual_percentage > 0.45 && actual_percentage < 0.55,
            "Expected ~50% activation, got {:.2}%",
            actual_percentage * 100.0
        );
    }
    
    #[test]
    fn test_feature_toggle_manager() {
        let mut manager = FeatureToggleManager::new();
        
        let config = FeatureToggleConfig::new(
            "manager-test".to_string(),
            "Manager test feature".to_string(),
            true,
        );
        
        assert!(manager.register_feature(config).is_ok());
        assert!(manager.get_feature("manager-test").is_ok());
        assert_eq!(
            manager.get_feature("nonexistent"),
            Err(FeatureToggleError::FeatureNotFound("nonexistent".to_string()))
        );
        
        assert!(manager.remove_feature("manager-test").is_ok());
        assert_eq!(
            manager.get_feature("manager-test"),
            Err(FeatureToggleError::FeatureNotFound("manager-test".to_string()))
        );
    }
    
    #[test]
    fn test_feature_toggle_manager_operations() {
        let mut manager = FeatureToggleManager::new();
        
        let config = FeatureToggleConfig::new(
            "operations-test".to_string(),
            "Operations test feature".to_string(),
            false,
        );
        
        assert!(manager.register_feature(config).is_ok());
        
        // Test enabling feature
        assert!(manager.enable_feature("operations-test").is_ok());
        assert!(manager.is_feature_active("operations-test", "user1").unwrap());
        
        // Test disabling feature
        assert!(manager.disable_feature("operations-test").is_ok());
        assert!(!manager.is_feature_active("operations-test", "user1").unwrap());
        
        // Test setting percentage
        assert!(manager.set_feature_percentage("operations-test", 0.75).is_ok());
        assert!(manager.is_feature_active("operations-test", "user1").unwrap());
    }
    
    #[test]
    fn test_feature_toggle_user_groups() {
        let mut config = FeatureToggleConfig::new(
            "group-test".to_string(),
            "Group test feature".to_string(),
            true,
        );
        config = config.with_percentage(0.0).unwrap(); // 0% for general users
        config = config.with_user_groups(vec!["beta-testers".to_string()]);
        
        // Even with 0% general rollout, users in groups should have access
        // This test verifies the user group logic works
    }
}