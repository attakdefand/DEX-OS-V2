//! Canary Release implementation for traffic splitting.
//!
//! This module implements the Priority 3 SRE Patterns feature from DEX-OS-V2.csv:
//! - SRE Patterns,SRE Patterns,SRE Patterns,Canary Releases,Traffic Splitting,Medium

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors that can occur during canary release operations.
#[derive(Error, Debug, PartialEq)]
pub enum CanaryError {
    /// Invalid traffic percentage (must be between 0.0 and 1.0).
    #[error("Invalid traffic percentage: {0}. Must be between 0.0 and 1.0")]
    InvalidTrafficPercentage(f64),
    
    /// Invalid duration value.
    #[error("Invalid duration: {0}. Must be positive")]
    InvalidDuration(u64),
    
    /// Canary release not found.
    #[error("Canary release not found: {0}")]
    CanaryNotFound(String),
    
    /// Time calculation error.
    #[error("Time calculation error: {0}")]
    TimeError(String),
    
    /// Invalid step configuration.
    #[error("Invalid step configuration: {0}")]
    InvalidStep(String),
}

/// Canary release configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanaryConfig {
    /// Unique identifier for the canary release.
    pub id: String,
    
    /// Human-readable description of the canary release.
    pub description: String,
    
    /// Percentage of traffic to send to the canary version (0.0 to 1.0).
    pub traffic_percentage: f64,
    
    /// Duration of the canary release in seconds.
    pub duration_seconds: u64,
    
    /// Start time of the canary release (milliseconds since UNIX epoch).
    pub start_time: u64,
    
    /// End time of the canary release (milliseconds since UNIX epoch).
    pub end_time: u64,
    
    /// Whether to use gradual rollout.
    pub gradual_rollout: bool,
    
    /// Steps for gradual rollout (if enabled).
    pub rollout_steps: Vec<RolloutStep>,
}

/// A step in a gradual rollout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RolloutStep {
    /// Traffic percentage for this step.
    pub traffic_percentage: f64,
    
    /// Duration of this step in seconds.
    pub duration_seconds: u64,
}

impl CanaryConfig {
    /// Create a new canary release configuration.
    pub fn new(
        id: String,
        description: String,
        traffic_percentage: f64,
        duration_seconds: u64,
    ) -> Result<Self, CanaryError> {
        if traffic_percentage < 0.0 || traffic_percentage > 1.0 {
            return Err(CanaryError::InvalidTrafficPercentage(traffic_percentage));
        }
        
        if duration_seconds == 0 {
            return Err(CanaryError::InvalidDuration(duration_seconds));
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CanaryError::TimeError(e.to_string()))?
            .as_millis() as u64;
        
        let end_time = now + (duration_seconds * 1000);
        
        Ok(Self {
            id,
            description,
            traffic_percentage,
            duration_seconds,
            start_time: now,
            end_time,
            gradual_rollout: false,
            rollout_steps: Vec::new(),
        })
    }
    
    /// Enable gradual rollout with specified steps.
    pub fn with_gradual_rollout(mut self, steps: Vec<RolloutStep>) -> Result<Self, CanaryError> {
        // Validate steps
        for step in &steps {
            if step.traffic_percentage < 0.0 || step.traffic_percentage > 1.0 {
                return Err(CanaryError::InvalidStep(
                    format!("Invalid traffic percentage: {}", step.traffic_percentage)
                ));
            }
            
            if step.duration_seconds == 0 {
                return Err(CanaryError::InvalidStep(
                    format!("Invalid duration: {}", step.duration_seconds)
                ));
            }
        }
        
        self.gradual_rollout = true;
        self.rollout_steps = steps;
        Ok(self)
    }
    
    /// Check if the canary release is active.
    pub fn is_active(&self) -> Result<bool, CanaryError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CanaryError::TimeError(e.to_string()))?
            .as_millis() as u64;
        
        Ok(now >= self.start_time && now <= self.end_time)
    }
    
    /// Get the current traffic percentage based on time and rollout configuration.
    pub fn current_traffic_percentage(&self) -> Result<f64, CanaryError> {
        if !self.is_active()? {
            return Ok(0.0);
        }
        
        if !self.gradual_rollout {
            return Ok(self.traffic_percentage);
        }
        
        // For gradual rollout, calculate current percentage based on elapsed time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CanaryError::TimeError(e.to_string()))?
            .as_millis() as u64;
        
        let elapsed = now.saturating_sub(self.start_time);
        let total_duration: u64 = self.rollout_steps.iter().map(|s| s.duration_seconds).sum();
        
        if total_duration == 0 {
            return Ok(self.traffic_percentage);
        }
        
        let elapsed_ratio = elapsed as f64 / (total_duration * 1000) as f64;
        
        // Find the current step
        let mut cumulative_time = 0u64;
        for step in &self.rollout_steps {
            cumulative_time += step.duration_seconds * 1000;
            if elapsed <= cumulative_time {
                return Ok(step.traffic_percentage);
            }
        }
        
        // If we've passed all steps, return the final percentage
        Ok(self.rollout_steps.last().map(|s| s.traffic_percentage).unwrap_or(self.traffic_percentage))
    }
}

/// Canary release manager for handling traffic splitting.
pub struct CanaryManager {
    /// Active canary releases.
    canaries: HashMap<String, CanaryConfig>,
}

impl CanaryManager {
    /// Create a new canary manager.
    pub fn new() -> Self {
        Self {
            canaries: HashMap::new(),
        }
    }
    
    /// Register a new canary release.
    pub fn register_canary(&mut self, config: CanaryConfig) -> Result<(), CanaryError> {
        self.canaries.insert(config.id.clone(), config);
        Ok(())
    }
    
    /// Get a canary release by ID.
    pub fn get_canary(&self, id: &str) -> Result<&CanaryConfig, CanaryError> {
        self.canaries.get(id).ok_or_else(|| CanaryError::CanaryNotFound(id.to_string()))
    }
    
    /// Remove a canary release.
    pub fn remove_canary(&mut self, id: &str) -> Result<(), CanaryError> {
        if self.canaries.remove(id).is_some() {
            Ok(())
        } else {
            Err(CanaryError::CanaryNotFound(id.to_string()))
        }
    }
    
    /// Check if a request should be routed to the canary version.
    pub fn should_route_to_canary(&self, canary_id: &str) -> Result<bool, CanaryError> {
        let canary = self.get_canary(canary_id)?;
        if !canary.is_active()? {
            return Ok(false);
        }
        
        let percentage = canary.current_traffic_percentage()?;
        let random_value = rand::random::<f64>();
        Ok(random_value < percentage)
    }
    
    /// Get all active canary releases.
    pub fn get_active_canaries(&self) -> Result<Vec<&CanaryConfig>, CanaryError> {
        let mut active = Vec::new();
        for canary in self.canaries.values() {
            if canary.is_active()? {
                active.push(canary);
            }
        }
        Ok(active)
    }
}

impl Default for CanaryManager {
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
    fn test_canary_config_creation() {
        let config = CanaryConfig::new(
            "test-canary".to_string(),
            "Test canary release".to_string(),
            0.1,
            3600,
        );
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.id, "test-canary");
        assert_eq!(config.description, "Test canary release");
        assert_eq!(config.traffic_percentage, 0.1);
        assert_eq!(config.duration_seconds, 3600);
        assert!(!config.gradual_rollout);
        assert!(config.rollout_steps.is_empty());
    }
    
    #[test]
    fn test_canary_config_invalid_percentage() {
        let config = CanaryConfig::new(
            "invalid".to_string(),
            "Invalid config".to_string(),
            1.5,
            3600,
        );
        assert_eq!(config, Err(CanaryError::InvalidTrafficPercentage(1.5)));
        
        let config = CanaryConfig::new(
            "invalid".to_string(),
            "Invalid config".to_string(),
            -0.1,
            3600,
        );
        assert_eq!(config, Err(CanaryError::InvalidTrafficPercentage(-0.1)));
    }
    
    #[test]
    fn test_canary_config_invalid_duration() {
        let config = CanaryConfig::new(
            "invalid".to_string(),
            "Invalid config".to_string(),
            0.1,
            0,
        );
        assert_eq!(config, Err(CanaryError::InvalidDuration(0)));
    }
    
    #[test]
    fn test_canary_config_with_gradual_rollout() {
        let steps = vec![
            RolloutStep {
                traffic_percentage: 0.05,
                duration_seconds: 1800,
            },
            RolloutStep {
                traffic_percentage: 0.1,
                duration_seconds: 1800,
            },
        ];
        
        let config = CanaryConfig::new(
            "gradual-canary".to_string(),
            "Gradual canary release".to_string(),
            0.1,
            3600,
        ).unwrap().with_gradual_rollout(steps);
        
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.gradual_rollout);
        assert_eq!(config.rollout_steps.len(), 2);
    }
    
    #[test]
    fn test_canary_config_invalid_rollout_steps() {
        let invalid_steps = vec![
            RolloutStep {
                traffic_percentage: 1.5, // Invalid percentage
                duration_seconds: 1800,
            },
        ];
        
        let config = CanaryConfig::new(
            "invalid-canary".to_string(),
            "Invalid canary release".to_string(),
            0.1,
            3600,
        ).unwrap().with_gradual_rollout(invalid_steps);
        
        assert!(config.is_err());
        assert_eq!(
            config.unwrap_err(),
            CanaryError::InvalidStep("Invalid traffic percentage: 1.5".to_string())
        );
    }
    
    #[test]
    fn test_canary_manager() {
        let mut manager = CanaryManager::new();
        
        let config = CanaryConfig::new(
            "manager-test".to_string(),
            "Manager test canary".to_string(),
            0.1,
            3600,
        ).unwrap();
        
        assert!(manager.register_canary(config).is_ok());
        assert!(manager.get_canary("manager-test").is_ok());
        assert_eq!(
            manager.get_canary("nonexistent"),
            Err(CanaryError::CanaryNotFound("nonexistent".to_string()))
        );
        
        assert!(manager.remove_canary("manager-test").is_ok());
        assert_eq!(
            manager.get_canary("manager-test"),
            Err(CanaryError::CanaryNotFound("manager-test".to_string()))
        );
    }
    
    #[test]
    fn test_canary_routing() {
        let mut manager = CanaryManager::new();
        
        let config = CanaryConfig::new(
            "routing-test".to_string(),
            "Routing test canary".to_string(),
            1.0, // 100% traffic to canary
            3600,
        ).unwrap();
        
        assert!(manager.register_canary(config).is_ok());
        
        // With 100% traffic to canary, should always route to canary
        for _ in 0..10 {
            assert!(manager.should_route_to_canary("routing-test").unwrap());
        }
    }
}