//! Chaos Engineering implementation for failure injection.
//!
//! This module implements the Priority 3 SRE Patterns feature from DEX-OS-V2.csv:
//! - SRE Patterns,SRE Patterns,SRE Patterns,Chaos Engineering,Failure Injection,Medium

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use rand::Rng;

/// Errors that can occur during chaos engineering operations.
#[derive(Error, Debug, PartialEq)]
pub enum ChaosError {
    /// Invalid failure rate (must be between 0.0 and 1.0).
    #[error("Invalid failure rate: {0}. Must be between 0.0 and 1.0")]
    InvalidFailureRate(f64),
    
    /// Invalid duration value.
    #[error("Invalid duration: {0}. Must be positive")]
    InvalidDuration(u64),
    
    /// Chaos experiment not found.
    #[error("Chaos experiment not found: {0}")]
    ExperimentNotFound(String),
    
    /// Time calculation error.
    #[error("Time calculation error: {0}")]
    TimeError(String),
    
    /// Invalid experiment configuration.
    #[error("Invalid experiment configuration: {0}")]
    InvalidConfiguration(String),
}

/// Types of failures that can be injected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FailureType {
    /// Simulate service latency.
    Latency {
        /// Minimum delay in milliseconds.
        min_ms: u64,
        /// Maximum delay in milliseconds.
        max_ms: u64,
    },
    
    /// Simulate service errors.
    Error {
        /// HTTP status code to return.
        status_code: u16,
        /// Error message to return.
        message: String,
    },
    
    /// Simulate service unavailability.
    Unavailable,
    
    /// Simulate memory pressure.
    MemoryPressure {
        /// Amount of memory to consume in MB.
        mb_to_consume: u64,
    },
    
    /// Simulate CPU pressure.
    CpuPressure {
        /// Percentage of CPU to consume (0-100).
        percentage: u8,
    },
}

/// Chaos experiment configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChaosExperiment {
    /// Unique identifier for the experiment.
    pub id: String,
    
    /// Human-readable description of the experiment.
    pub description: String,
    
    /// Target service or component to affect.
    pub target: String,
    
    /// Type of failure to inject.
    pub failure_type: FailureType,
    
    /// Probability of applying the failure (0.0 to 1.0).
    pub failure_rate: f64,
    
    /// Duration of the experiment in seconds.
    pub duration_seconds: u64,
    
    /// Start time of the experiment (milliseconds since UNIX epoch).
    pub start_time: u64,
    
    /// End time of the experiment (milliseconds since UNIX epoch).
    pub end_time: u64,
    
    /// Whether the experiment is currently active.
    pub active: bool,
}

impl ChaosExperiment {
    /// Create a new chaos experiment.
    pub fn new(
        id: String,
        description: String,
        target: String,
        failure_type: FailureType,
        failure_rate: f64,
        duration_seconds: u64,
    ) -> Result<Self, ChaosError> {
        if failure_rate < 0.0 || failure_rate > 1.0 {
            return Err(ChaosError::InvalidFailureRate(failure_rate));
        }
        
        if duration_seconds == 0 {
            return Err(ChaosError::InvalidDuration(duration_seconds));
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ChaosError::TimeError(e.to_string()))?
            .as_millis() as u64;
        
        let end_time = now + (duration_seconds * 1000);
        
        Ok(Self {
            id,
            description,
            target,
            failure_type,
            failure_rate,
            duration_seconds,
            start_time: now,
            end_time,
            active: true,
        })
    }
    
    /// Check if the experiment is active.
    pub fn is_active(&self) -> Result<bool, ChaosError> {
        if !self.active {
            return Ok(false);
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ChaosError::TimeError(e.to_string()))?
            .as_millis() as u64;
        
        Ok(now >= self.start_time && now <= self.end_time)
    }
    
    /// Apply the chaos experiment to a request.
    /// Returns true if the failure should be applied.
    pub fn should_apply_failure(&self) -> Result<bool, ChaosError> {
        if !self.is_active()? {
            return Ok(false);
        }
        
        let random_value = rand::random::<f64>();
        Ok(random_value < self.failure_rate)
    }
    
    /// Apply latency failure.
    pub fn apply_latency(&self) -> Option<Duration> {
        if let FailureType::Latency { min_ms, max_ms } = self.failure_type {
            let mut rng = rand::thread_rng();
            let delay_ms = rng.gen_range(min_ms..=max_ms);
            Some(Duration::from_millis(delay_ms))
        } else {
            None
        }
    }
    
    /// Apply error failure.
    pub fn apply_error(&self) -> Option<(u16, String)> {
        if let FailureType::Error { status_code, message } = &self.failure_type {
            Some((*status_code, message.clone()))
        } else {
            None
        }
    }
    
    /// Stop the experiment.
    pub fn stop(&mut self) {
        self.active = false;
    }
}

/// Chaos engineering manager for handling failure injection.
pub struct ChaosManager {
    /// Active chaos experiments.
    experiments: Arc<RwLock<HashMap<String, ChaosExperiment>>>,
}

impl ChaosManager {
    /// Create a new chaos manager.
    pub fn new() -> Self {
        Self {
            experiments: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a new chaos experiment.
    pub fn register_experiment(&self, experiment: ChaosExperiment) -> Result<(), ChaosError> {
        let mut experiments = self.experiments.write().unwrap();
        experiments.insert(experiment.id.clone(), experiment);
        Ok(())
    }
    
    /// Get a chaos experiment by ID.
    pub fn get_experiment(&self, id: &str) -> Result<ChaosExperiment, ChaosError> {
        let experiments = self.experiments.read().unwrap();
        experiments.get(id).cloned().ok_or_else(|| ChaosError::ExperimentNotFound(id.to_string()))
    }
    
    /// Remove a chaos experiment.
    pub fn remove_experiment(&self, id: &str) -> Result<(), ChaosError> {
        let mut experiments = self.experiments.write().unwrap();
        if experiments.remove(id).is_some() {
            Ok(())
        } else {
            Err(ChaosError::ExperimentNotFound(id.to_string()))
        }
    }
    
    /// Stop a chaos experiment.
    pub fn stop_experiment(&self, id: &str) -> Result<(), ChaosError> {
        let mut experiments = self.experiments.write().unwrap();
        if let Some(experiment) = experiments.get_mut(id) {
            experiment.stop();
            Ok(())
        } else {
            Err(ChaosError::ExperimentNotFound(id.to_string()))
        }
    }
    
    /// Get all active chaos experiments.
    pub fn get_active_experiments(&self) -> Result<Vec<ChaosExperiment>, ChaosError> {
        let experiments = self.experiments.read().unwrap();
        let mut active = Vec::new();
        for experiment in experiments.values() {
            if experiment.is_active()? {
                active.push(experiment.clone());
            }
        }
        Ok(active)
    }
    
    /// Check if any active experiments should apply failures to a target.
    /// Returns a list of experiments that should apply failures.
    pub fn get_applicable_experiments(&self, target: &str) -> Result<Vec<ChaosExperiment>, ChaosError> {
        let experiments = self.experiments.read().unwrap();
        let mut applicable = Vec::new();
        
        for experiment in experiments.values() {
            if experiment.target == target && experiment.is_active()? && experiment.should_apply_failure()? {
                applicable.push(experiment.clone());
            }
        }
        
        Ok(applicable)
    }
    
    /// Apply chaos to a target service.
    /// This method checks all active experiments for the target and applies any applicable failures.
    pub fn apply_chaos(&self, target: &str) -> Result<Option<ChaosAction>, ChaosError> {
        let applicable = self.get_applicable_experiments(target)?;
        
        // If multiple experiments apply, pick one randomly
        if !applicable.is_empty() {
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..applicable.len());
            let experiment = &applicable[index];
            
            match &experiment.failure_type {
                FailureType::Latency { .. } => {
                    if let Some(delay) = experiment.apply_latency() {
                        Ok(Some(ChaosAction::Latency(delay)))
                    } else {
                        Ok(None)
                    }
                }
                FailureType::Error { .. } => {
                    if let Some((status, message)) = experiment.apply_error() {
                        Ok(Some(ChaosAction::Error(status, message)))
                    } else {
                        Ok(None)
                    }
                }
                FailureType::Unavailable => {
                    Ok(Some(ChaosAction::Unavailable))
                }
                FailureType::MemoryPressure { .. } | FailureType::CpuPressure { .. } => {
                    // These are more complex to simulate in a test environment
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}

/// Actions that can be taken as a result of chaos engineering.
#[derive(Debug, Clone, PartialEq)]
pub enum ChaosAction {
    /// Add latency to the request.
    Latency(Duration),
    
    /// Return an error response.
    Error(u16, String),
    
    /// Make the service unavailable.
    Unavailable,
}

impl Default for ChaosManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_chaos_experiment_creation() {
        let experiment = ChaosExperiment::new(
            "test-experiment".to_string(),
            "Test experiment".to_string(),
            "api-service".to_string(),
            FailureType::Error {
                status_code: 500,
                message: "Internal Server Error".to_string(),
            },
            0.1,
            3600,
        );
        assert!(experiment.is_ok());
        
        let experiment = experiment.unwrap();
        assert_eq!(experiment.id, "test-experiment");
        assert_eq!(experiment.target, "api-service");
        assert_eq!(experiment.failure_rate, 0.1);
        assert_eq!(experiment.duration_seconds, 3600);
        assert!(experiment.active);
    }
    
    #[test]
    fn test_chaos_experiment_invalid_failure_rate() {
        let experiment = ChaosExperiment::new(
            "invalid".to_string(),
            "Invalid experiment".to_string(),
            "api-service".to_string(),
            FailureType::Unavailable,
            1.5,
            3600,
        );
        assert_eq!(experiment, Err(ChaosError::InvalidFailureRate(1.5)));
        
        let experiment = ChaosExperiment::new(
            "invalid".to_string(),
            "Invalid experiment".to_string(),
            "api-service".to_string(),
            FailureType::Unavailable,
            -0.1,
            3600,
        );
        assert_eq!(experiment, Err(ChaosError::InvalidFailureRate(-0.1)));
    }
    
    #[test]
    fn test_chaos_experiment_invalid_duration() {
        let experiment = ChaosExperiment::new(
            "invalid".to_string(),
            "Invalid experiment".to_string(),
            "api-service".to_string(),
            FailureType::Unavailable,
            0.1,
            0,
        );
        assert_eq!(experiment, Err(ChaosError::InvalidDuration(0)));
    }
    
    #[test]
    fn test_chaos_experiment_should_apply_failure() {
        let experiment = ChaosExperiment::new(
            "apply-test".to_string(),
            "Apply test experiment".to_string(),
            "api-service".to_string(),
            FailureType::Unavailable,
            1.0, // 100% failure rate
            3600,
        ).unwrap();
        
        // With 100% failure rate, should always apply
        for _ in 0..10 {
            assert!(experiment.should_apply_failure().unwrap());
        }
    }
    
    #[test]
    fn test_chaos_manager() {
        let manager = ChaosManager::new();
        
        let experiment = ChaosExperiment::new(
            "manager-test".to_string(),
            "Manager test experiment".to_string(),
            "api-service".to_string(),
            FailureType::Unavailable,
            0.1,
            3600,
        ).unwrap();
        
        assert!(manager.register_experiment(experiment).is_ok());
        assert!(manager.get_experiment("manager-test").is_ok());
        assert_eq!(
            manager.get_experiment("nonexistent"),
            Err(ChaosError::ExperimentNotFound("nonexistent".to_string()))
        );
        
        assert!(manager.stop_experiment("manager-test").is_ok());
        assert!(manager.remove_experiment("manager-test").is_ok());
        assert_eq!(
            manager.get_experiment("manager-test"),
            Err(ChaosError::ExperimentNotFound("manager-test".to_string()))
        );
    }
    
    #[test]
    fn test_chaos_action_application() {
        let manager = ChaosManager::new();
        
        let experiment = ChaosExperiment::new(
            "action-test".to_string(),
            "Action test experiment".to_string(),
            "test-service".to_string(),
            FailureType::Error {
                status_code: 503,
                message: "Service Unavailable".to_string(),
            },
            1.0, // 100% failure rate
            3600,
        ).unwrap();
        
        assert!(manager.register_experiment(experiment).is_ok());
        
        // Should always return an error action for this experiment
        for _ in 0..5 {
            let action = manager.apply_chaos("test-service").unwrap();
            assert!(matches!(action, Some(ChaosAction::Error(503, _))));
        }
    }
    
    #[test]
    fn test_chaos_latency_application() {
        let manager = ChaosManager::new();
        
        let experiment = ChaosExperiment::new(
            "latency-test".to_string(),
            "Latency test experiment".to_string(),
            "latency-service".to_string(),
            FailureType::Latency {
                min_ms: 100,
                max_ms: 500,
            },
            1.0, // 100% failure rate
            3600,
        ).unwrap();
        
        assert!(manager.register_experiment(experiment).is_ok());
        
        // Should always return a latency action for this experiment
        for _ in 0..5 {
            let action = manager.apply_chaos("latency-service").unwrap();
            assert!(matches!(action, Some(ChaosAction::Latency(_))));
            if let Some(ChaosAction::Latency(duration)) = action {
                assert!(duration >= Duration::from_millis(100));
                assert!(duration <= Duration::from_millis(500));
            }
        }
    }
}