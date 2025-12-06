//! Rolling Update implementation for incremental replacement.
//!
//! This module implements the Priority 3 Zero-Downtime Deployment feature from DEX-OS-V2.csv:
//! - Zero-Downtime Deployment,Zero-Downtime Deployment,Zero-Downtime Deployment,Rolling Update,Incremental Replacement,Medium

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors that can occur during rolling update operations.
#[derive(Error, Debug, PartialEq)]
pub enum RollingUpdateError {
    /// Invalid batch size (must be positive).
    #[error("Invalid batch size: {0}. Must be positive")]
    InvalidBatchSize(u32),
    
    /// Invalid delay value.
    #[error("Invalid delay: {0}. Must be positive")]
    InvalidDelay(u64),
    
    /// Rolling update not found.
    #[error("Rolling update not found: {0}")]
    RollingUpdateNotFound(String),
    
    /// Time calculation error.
    #[error("Time calculation error: {0}")]
    TimeError(String),
    
    /// Invalid step configuration.
    #[error("Invalid step configuration: {0}")]
    InvalidStep(String),
    
    /// Rolling update already completed.
    #[error("Rolling update already completed: {0}")]
    AlreadyCompleted(String),
}

/// Rolling update configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RollingUpdateConfig {
    /// Unique identifier for the rolling update.
    pub id: String,
    
    /// Human-readable description of the rolling update.
    pub description: String,
    
    /// Total number of instances to update.
    pub total_instances: u32,
    
    /// Number of instances to update in each batch.
    pub batch_size: u32,
    
    /// Delay between batches in seconds.
    pub batch_delay_seconds: u64,
    
    /// Start time of the rolling update (milliseconds since UNIX epoch).
    pub start_time: u64,
    
    /// End time of the rolling update (milliseconds since UNIX epoch).
    pub end_time: u64,
    
    /// Current batch being processed.
    pub current_batch: u32,
    
    /// Number of successfully updated instances.
    pub updated_instances: u32,
    
    /// Whether the rolling update is completed.
    pub completed: bool,
}

impl RollingUpdateConfig {
    /// Create a new rolling update configuration.
    pub fn new(
        id: String,
        description: String,
        total_instances: u32,
        batch_size: u32,
        mut batch_delay_seconds: u64,
    ) -> Result<Self, RollingUpdateError> {
        if batch_size == 0 {
            return Err(RollingUpdateError::InvalidBatchSize(batch_size));
        }
        
        if batch_delay_seconds == 0 {
            // Default to 30 seconds if not specified
            batch_delay_seconds = 30;
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RollingUpdateError::TimeError(e.to_string()))?
            .as_millis() as u64;
        
        // Estimate end time based on number of batches and delays
        let num_batches = (total_instances + batch_size - 1) / batch_size; // Ceiling division
        let estimated_duration = if num_batches > 1 {
            (num_batches as u64 - 1) * batch_delay_seconds
        } else {
            0u64
        };
        
        let end_time = now + (estimated_duration * 1000u64);
        
        Ok(Self {
            id,
            description,
            total_instances,
            batch_size,
            batch_delay_seconds,
            start_time: now,
            end_time,
            current_batch: 0,
            updated_instances: 0,
            completed: false,
        })
    }
    
    /// Check if the rolling update is active.
    pub fn is_active(&self) -> Result<bool, RollingUpdateError> {
        if self.completed {
            return Ok(false);
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RollingUpdateError::TimeError(e.to_string()))?
            .as_millis() as u64;
        
        Ok(now >= self.start_time && now <= self.end_time)
    }
    
    /// Get the number of instances to update in the current batch.
    pub fn current_batch_size(&self) -> u32 {
        if self.completed {
            return 0;
        }
        
        let remaining = self.total_instances - self.updated_instances;
        std::cmp::min(self.batch_size, remaining)
    }
    
    /// Get the delay before the next batch.
    pub fn next_batch_delay(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.batch_delay_seconds)
    }
    
    /// Mark instances in the current batch as updated.
    pub fn mark_batch_updated(&mut self, instances_updated: u32) -> Result<(), RollingUpdateError> {
        if self.completed {
            return Err(RollingUpdateError::AlreadyCompleted(self.id.clone()));
        }
        
        self.updated_instances += instances_updated;
        self.current_batch += 1;
        
        if self.updated_instances >= self.total_instances {
            self.completed = true;
            self.updated_instances = self.total_instances; // Cap at total
            
            // Update end time to now
            self.end_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| RollingUpdateError::TimeError(e.to_string()))?
                .as_millis() as u64;
        }
        
        Ok(())
    }
    
    /// Get progress percentage.
    pub fn progress_percentage(&self) -> f64 {
        if self.total_instances == 0 {
            return 100.0;
        }
        
        (self.updated_instances as f64 / self.total_instances as f64) * 100.0
    }
    
    /// Get estimated completion time.
    pub fn estimated_completion_time(&self) -> Result<u64, RollingUpdateError> {
        if self.completed {
            return Ok(self.end_time);
        }
        
        let remaining_instances = self.total_instances - self.updated_instances;
        if remaining_instances == 0 {
            return Ok(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| RollingUpdateError::TimeError(e.to_string()))?
                .as_millis() as u64);
        }
        
        let remaining_batches = (remaining_instances + self.batch_size - 1) / self.batch_size;
        let estimated_delay = if remaining_batches > 1 {
            (remaining_batches as u64 - 1) * self.batch_delay_seconds
        } else {
            0u64
        };
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RollingUpdateError::TimeError(e.to_string()))?
            .as_millis() as u64;
        
        Ok(now + (estimated_delay * 1000u64))
    }
}

/// Rolling update manager for handling incremental replacements.
pub struct RollingUpdateManager {
    /// Active rolling updates.
    updates: HashMap<String, RollingUpdateConfig>,
}

impl RollingUpdateManager {
    /// Create a new rolling update manager.
    pub fn new() -> Self {
        Self {
            updates: HashMap::new(),
        }
    }
    
    /// Register a new rolling update.
    pub fn register_update(&mut self, config: RollingUpdateConfig) -> Result<(), RollingUpdateError> {
        self.updates.insert(config.id.clone(), config);
        Ok(())
    }
    
    /// Get a rolling update by ID.
    pub fn get_update(&self, id: &str) -> Result<&RollingUpdateConfig, RollingUpdateError> {
        self.updates.get(id).ok_or_else(|| RollingUpdateError::RollingUpdateNotFound(id.to_string()))
    }
    
    /// Get a mutable reference to a rolling update by ID.
    pub fn get_update_mut(&mut self, id: &str) -> Result<&mut RollingUpdateConfig, RollingUpdateError> {
        self.updates.get_mut(id).ok_or_else(|| RollingUpdateError::RollingUpdateNotFound(id.to_string()))
    }
    
    /// Remove a rolling update.
    pub fn remove_update(&mut self, id: &str) -> Result<(), RollingUpdateError> {
        if self.updates.remove(id).is_some() {
            Ok(())
        } else {
            Err(RollingUpdateError::RollingUpdateNotFound(id.to_string()))
        }
    }
    
    /// Start processing the next batch of a rolling update.
    pub fn process_next_batch(&mut self, update_id: &str, instances_updated: u32) -> Result<(), RollingUpdateError> {
        let update = self.get_update_mut(update_id)?;
        update.mark_batch_updated(instances_updated)
    }
    
    /// Get all active rolling updates.
    pub fn get_active_updates(&self) -> Result<Vec<&RollingUpdateConfig>, RollingUpdateError> {
        let mut active = Vec::new();
        for update in self.updates.values() {
            if update.is_active()? && !update.completed {
                active.push(update);
            }
        }
        Ok(active)
    }
    
    /// Get all completed rolling updates.
    pub fn get_completed_updates(&self) -> Vec<&RollingUpdateConfig> {
        self.updates.values().filter(|u| u.completed).collect()
    }
}

impl Default for RollingUpdateManager {
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
    fn test_rolling_update_config_creation() {
        let config = RollingUpdateConfig::new(
            "test-update".to_string(),
            "Test rolling update".to_string(),
            10,
            2,
            30,
        );
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.id, "test-update");
        assert_eq!(config.description, "Test rolling update");
        assert_eq!(config.total_instances, 10);
        assert_eq!(config.batch_size, 2);
        assert_eq!(config.batch_delay_seconds, 30);
        assert_eq!(config.current_batch, 0);
        assert_eq!(config.updated_instances, 0);
        assert!(!config.completed);
        assert!(config.start_time > 0);
        assert!(config.end_time > config.start_time);
    }
    
    #[test]
    fn test_rolling_update_config_invalid_batch_size() {
        let config = RollingUpdateConfig::new(
            "invalid".to_string(),
            "Invalid config".to_string(),
            10,
            0, // Invalid batch size
            30,
        );
        assert_eq!(config, Err(RollingUpdateError::InvalidBatchSize(0)));
    }
    
    #[test]
    fn test_rolling_update_current_batch_size() {
        let mut config = RollingUpdateConfig::new(
            "batch-test".to_string(),
            "Batch size test".to_string(),
            10,
            3,
            30,
        ).unwrap();
        
        assert_eq!(config.current_batch_size(), 3);
        
        // Mark 5 instances as updated
        config.mark_batch_updated(5).unwrap();
        assert_eq!(config.current_batch_size(), 3); // Still 3 in batch
        
        // Mark 3 more instances as updated
        config.mark_batch_updated(3).unwrap();
        assert_eq!(config.current_batch_size(), 2); // Only 2 remaining
        
        // Mark final 2 instances as updated
        config.mark_batch_updated(2).unwrap();
        assert_eq!(config.current_batch_size(), 0); // Completed
        assert!(config.completed);
    }
    
    #[test]
    fn test_rolling_update_progress() {
        let mut config = RollingUpdateConfig::new(
            "progress-test".to_string(),
            "Progress test".to_string(),
            100,
            10,
            30,
        ).unwrap();
        
        assert_eq!(config.progress_percentage(), 0.0);
        
        config.mark_batch_updated(25).unwrap();
        assert_eq!(config.progress_percentage(), 25.0);
        
        config.mark_batch_updated(50).unwrap();
        assert_eq!(config.progress_percentage(), 75.0);
        
        config.mark_batch_updated(25).unwrap();
        assert_eq!(config.progress_percentage(), 100.0);
        assert!(config.completed);
    }
    
    #[test]
    fn test_rolling_update_manager() {
        let mut manager = RollingUpdateManager::new();
        
        let config = RollingUpdateConfig::new(
            "manager-test".to_string(),
            "Manager test update".to_string(),
            5,
            1,
            10,
        ).unwrap();
        
        assert!(manager.register_update(config).is_ok());
        assert!(manager.get_update("manager-test").is_ok());
        assert_eq!(
            manager.get_update("nonexistent"),
            Err(RollingUpdateError::RollingUpdateNotFound("nonexistent".to_string()))
        );
        
        assert!(manager.remove_update("manager-test").is_ok());
        assert_eq!(
            manager.get_update("manager-test"),
            Err(RollingUpdateError::RollingUpdateNotFound("manager-test".to_string()))
        );
    }
    
    #[test]
    fn test_rolling_update_processing() {
        let mut manager = RollingUpdateManager::new();
        
        let config = RollingUpdateConfig::new(
            "processing-test".to_string(),
            "Processing test update".to_string(),
            6,
            2,
            5,
        ).unwrap();
        
        assert!(manager.register_update(config).is_ok());
        
        // Process first batch
        assert!(manager.process_next_batch("processing-test", 2).is_ok());
        let update = manager.get_update("processing-test").unwrap();
        assert_eq!(update.updated_instances, 2);
        assert_eq!(update.current_batch, 1);
        assert!(!update.completed);
        
        // Process second batch
        assert!(manager.process_next_batch("processing-test", 2).is_ok());
        let update = manager.get_update("processing-test").unwrap();
        assert_eq!(update.updated_instances, 4);
        assert_eq!(update.current_batch, 2);
        assert!(!update.completed);
        
        // Process final batch
        assert!(manager.process_next_batch("processing-test", 2).is_ok());
        let update = manager.get_update("processing-test").unwrap();
        assert_eq!(update.updated_instances, 6);
        assert_eq!(update.current_batch, 3);
        assert!(update.completed);
    }
    
    #[test]
    fn test_rolling_update_already_completed() {
        let mut manager = RollingUpdateManager::new();
        
        let mut config = RollingUpdateConfig::new(
            "completed-test".to_string(),
            "Completed test update".to_string(),
            2,
            1,
            5,
        ).unwrap();
        
        // Mark as completed manually
        config.mark_batch_updated(2).unwrap();
        assert!(config.completed);
        
        assert!(manager.register_update(config).is_ok());
        
        // Try to process a batch on completed update
        assert_eq!(
            manager.process_next_batch("completed-test", 1),
            Err(RollingUpdateError::AlreadyCompleted("completed-test".to_string()))
        );
    }
}