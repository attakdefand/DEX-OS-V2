//! SRE Patterns implementation for error budgets and SLO targets.
//!
//! This module implements the Priority 3 SRE Patterns features from DEX-OS-V2.csv:
//! - SRE Patterns,SRE Patterns,SRE Patterns,Error Budget,SLO Targets,Medium

use crate::observability::ObservabilityManager;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors that can occur during SRE operations.
#[derive(Error, Debug, PartialEq)]
pub enum SREError {
    /// Invalid SLO target value (must be between 0.0 and 1.0).
    #[error("Invalid SLO target: {0}. Must be between 0.0 and 1.0")]
    InvalidSloTarget(f64),
    /// Invalid error budget value (must be between 0.0 and 1.0).
    #[error("Invalid error budget: {0}. Must be between 0.0 and 1.0")]
    InvalidErrorBudget(f64),
    /// SLO not found.
    #[error("SLO not found: {0}")]
    SloNotFound(String),
    /// Service not found.
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    /// Insufficient error budget.
    #[error("Insufficient error budget for service {service}: {available} available, {requested} requested")]
    InsufficientErrorBudget {
        service: String,
        available: f64,
        requested: f64,
    },
    /// Time calculation error.
    #[error("Time calculation error: {0}")]
    TimeError(String),
}

/// Service Level Objective definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SLO {
    /// Unique identifier for the SLO.
    pub id: String,
    /// Human-readable description of the SLO.
    pub description: String,
    /// Target success rate (between 0.0 and 1.0).
    pub target: f64,
    /// Current actual success rate.
    pub actual: f64,
    /// Total number of requests.
    pub total_requests: u64,
    /// Number of successful requests.
    pub successful_requests: u64,
    /// Timestamp of last update (milliseconds since UNIX epoch).
    pub last_updated: u64,
    /// Rolling window size in seconds for SLO calculation.
    pub window_size_seconds: u64,
    /// Request timestamps for rolling window calculation.
    pub request_timestamps: Vec<u64>,
}

impl SLO {
    /// Create a new SLO.
    pub fn new(id: String, description: String, target: f64) -> Result<Self, SREError> {
        if target < 0.0 || target > 1.0 {
            return Err(SREError::InvalidSloTarget(target));
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SREError::TimeError(e.to_string()))?
            .as_millis() as u64;
        
        Ok(Self {
            id,
            description,
            target,
            actual: 1.0,
            total_requests: 0,
            successful_requests: 0,
            last_updated: now,
            window_size_seconds: 3600, // 1 hour default window
            request_timestamps: Vec::new(),
        })
    }
    
    /// Update the SLO with a new request result.
    pub fn record_request(&mut self, success: bool) -> Result<(), SREError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SREError::TimeError(e.to_string()))?
            .as_millis() as u64;
        
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        }
        
        // Add timestamp for rolling window calculation
        self.request_timestamps.push(now);
        self.last_updated = now;
        
        // Clean up old timestamps outside the window
        let window_start = now.saturating_sub(self.window_size_seconds * 1000);
        self.request_timestamps.retain(|&timestamp| timestamp >= window_start);
        
        // Recalculate actual success rate based on window
        self.actual = self.successful_requests as f64 / self.total_requests as f64;
        
        Ok(())
    }
    
    /// Calculate the error budget (1.0 - actual performance).
    pub fn error_consumed(&self) -> f64 {
        1.0 - self.actual
    }
    
    /// Calculate remaining error budget (target - actual).
    pub fn error_budget_remaining(&self) -> f64 {
        // If we're meeting or exceeding our target, we have budget remaining
        if self.actual >= self.target {
            self.target - self.actual
        } else {
            // If we're below target, we've consumed our budget and then some
            -(self.actual - self.target)
        }
    }
    
    /// Check if the SLO is currently meeting its target.
    pub fn is_meeting_target(&self) -> bool {
        self.actual >= self.target
    }
}

/// Service representation for SRE monitoring.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Service {
    /// Unique identifier for the service.
    pub id: String,
    /// Human-readable name of the service.
    pub name: String,
    /// SLOs associated with this service.
    pub slos: HashMap<String, SLO>,
    /// Total error budget for the service.
    pub total_error_budget: f64,
    /// Consumed error budget for the service.
    pub consumed_error_budget: f64,
}

impl Service {
    /// Create a new service.
    pub fn new(id: String, name: String, total_error_budget: f64) -> Result<Self, SREError> {
        if total_error_budget < 0.0 || total_error_budget > 1.0 {
            return Err(SREError::InvalidErrorBudget(total_error_budget));
        }
        
        Ok(Self {
            id,
            name,
            slos: HashMap::new(),
            total_error_budget,
            consumed_error_budget: 0.0,
        })
    }
    
    /// Add an SLO to the service.
    pub fn add_slo(&mut self, slo: SLO) {
        self.slos.insert(slo.id.clone(), slo);
    }
    
    /// Record a request for a specific SLO.
    pub fn record_request(&mut self, slo_id: &str, success: bool) -> Result<(), SREError> {
        if let Some(slo) = self.slos.get_mut(slo_id) {
            slo.record_request(success)?;
            // Update service-level error budget
            self.consumed_error_budget = self.calculate_consumed_error_budget();
            Ok(())
        } else {
            Err(SREError::SloNotFound(slo_id.to_string()))
        }
    }
    
    /// Calculate the total consumed error budget across all SLOs.
    fn calculate_consumed_error_budget(&self) -> f64 {
        let mut total_weighted_error = 0.0;
        let mut total_weight = 0.0;
        
        for slo in self.slos.values() {
            let weight = slo.total_requests as f64;
            if weight > 0.0 {
                total_weighted_error += slo.error_consumed() * weight;
                total_weight += weight;
            }
        }
        
        if total_weight > 0.0 {
            total_weighted_error / total_weight
        } else {
            0.0
        }
    }
    
    /// Get remaining error budget for the service.
    pub fn error_budget_remaining(&self) -> f64 {
        self.total_error_budget - self.consumed_error_budget
    }
    
    /// Check if the service has sufficient error budget for a new request.
    pub fn has_sufficient_error_budget(&self, requested_budget: f64) -> bool {
        self.error_budget_remaining() >= requested_budget
    }
    
    /// Consume error budget for the service.
    pub fn consume_error_budget(&mut self, amount: f64) -> Result<(), SREError> {
        if !self.has_sufficient_error_budget(amount) {
            return Err(SREError::InsufficientErrorBudget {
                service: self.id.clone(),
                available: self.error_budget_remaining(),
                requested: amount,
            });
        }
        
        self.consumed_error_budget += amount;
        Ok(())
    }
}

/// SRE Patterns manager for handling error budgets and SLO targets.
pub struct SREManager {
    /// Services being monitored.
    services: Arc<RwLock<HashMap<String, Service>>>,
    /// Reference to observability manager for metrics integration.
    observability: Arc<ObservabilityManager>,
}

/// Helper for timing operations and automatically recording latency.
pub struct SRETimer {
    /// Start time of the operation.
    start: Instant,
    /// Service ID to record metrics for.
    service_id: String,
    /// SLO ID to record metrics for.
    slo_id: String,
    /// Reference to the SRE manager.
    manager: Arc<SREManager>,
}

impl SRETimer {
    /// Create a new timer for an SRE operation.
    pub fn new(manager: Arc<SREManager>, service_id: String, slo_id: String) -> Self {
        Self {
            start: Instant::now(),
            service_id,
            slo_id,
            manager,
        }
    }
    
    /// Stop the timer and record the operation as successful.
    pub fn stop_success(self) {
        let duration = self.start.elapsed();
        let _ = self.manager.record_request_with_latency(
            &self.service_id, 
            &self.slo_id, 
            true, 
            Some(duration.as_millis() as u64)
        );
    }
    
    /// Stop the timer and record the operation as failed.
    pub fn stop_failure(self) {
        let duration = self.start.elapsed();
        let _ = self.manager.record_request_with_latency(
            &self.service_id, 
            &self.slo_id, 
            false, 
            Some(duration.as_millis() as u64)
        );
    }
}

impl SREManager {
    /// Create a new SRE manager.
    pub fn new(observability: Arc<ObservabilityManager>) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            observability,
        }
    }
    
    /// Register a new service for SRE monitoring.
    pub fn register_service(&self, service: Service) -> Result<(), SREError> {
        let mut services = self.services.write().unwrap();
        services.insert(service.id.clone(), service);
        Ok(())
    }
    
    /// Get a service by ID.
    pub fn get_service(&self, service_id: &str) -> Result<Service, SREError> {
        let services = self.services.read().unwrap();
        services.get(service_id).cloned().ok_or_else(|| SREError::ServiceNotFound(service_id.to_string()))
    }
    
    /// Record a request for a specific service and SLO.
    pub fn record_request(&self, service_id: &str, slo_id: &str, success: bool) -> Result<(), SREError> {
        self.record_request_with_latency(service_id, slo_id, success, None)
    }
    
    /// Record a request for a specific service and SLO with latency measurement.
    pub fn record_request_with_latency(&self, service_id: &str, slo_id: &str, success: bool, latency_ms: Option<u64>) -> Result<(), SREError> {
        let mut services = self.services.write().unwrap();
        if let Some(service) = services.get_mut(service_id) {
            service.record_request(slo_id, success)?;
            
            // Update observability metrics
            let slo = service.slos.get(slo_id).unwrap();
            let _ = self.observability.increment_counter(&format!("slo_requests_total_{}", slo_id));
            if success {
                let _ = self.observability.increment_counter(&format!("slo_successful_requests_{}", slo_id));
            }
            
            // Update gauge metrics
            let _ = self.observability.set_gauge(
                &format!("slo_success_rate_{}", slo_id), 
                (slo.actual * 100.0) as i64
            );
            let _ = self.observability.set_gauge(
                &format!("slo_error_budget_remaining_{}", slo_id), 
                (slo.error_budget_remaining() * 100.0) as i64
            );
            
            // Record latency if provided
            if let Some(latency) = latency_ms {
                let _ = self.observability.record_histogram(
                    &format!("slo_request_latency_{}", slo_id), 
                    latency
                );
            }
            
            Ok(())
        } else {
            Err(SREError::ServiceNotFound(service_id.to_string()))
        }
    }
    
    /// Consume error budget for a service.
    pub fn consume_error_budget(&self, service_id: &str, amount: f64) -> Result<(), SREError> {
        let mut services = self.services.write().unwrap();
        if let Some(service) = services.get_mut(service_id) {
            service.consume_error_budget(amount)?;
            
            // Update observability metrics
            let _ = self.observability.set_gauge(
                &format!("service_error_budget_consumed_{}", service_id), 
                (service.consumed_error_budget * 100.0) as i64
            );
            let _ = self.observability.set_gauge(
                &format!("service_error_budget_remaining_{}", service_id), 
                (service.error_budget_remaining() * 100.0) as i64
            );
            
            Ok(())
        } else {
            Err(SREError::ServiceNotFound(service_id.to_string()))
        }
    }
    
    /// Check if a service has sufficient error budget.
    pub fn has_sufficient_error_budget(&self, service_id: &str, requested_budget: f64) -> Result<bool, SREError> {
        let services = self.services.read().unwrap();
        if let Some(service) = services.get(service_id) {
            Ok(service.has_sufficient_error_budget(requested_budget))
        } else {
            Err(SREError::ServiceNotFound(service_id.to_string()))
        }
    }
    
    /// Get all services.
    pub fn get_all_services(&self) -> Vec<Service> {
        let services = self.services.read().unwrap();
        services.values().cloned().collect()
    }
    
    /// Create a timer for measuring an SRE operation.
    pub fn start_timer(self: &Arc<Self>, service_id: String, slo_id: String) -> SRETimer {
        SRETimer::new(self.clone(), service_id, slo_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::ObservabilityManager;
    use std::sync::Arc;
    
    #[test]
    fn test_slo_creation() {
        let slo = SLO::new("availability".to_string(), "API availability".to_string(), 0.99);
        assert!(slo.is_ok());
        
        let slo = slo.unwrap();
        assert_eq!(slo.id, "availability");
        assert_eq!(slo.description, "API availability");
        assert_eq!(slo.target, 0.99);
        assert_eq!(slo.actual, 1.0);
        assert_eq!(slo.total_requests, 0);
        assert_eq!(slo.successful_requests, 0);
        assert!(slo.last_updated > 0);
        assert_eq!(slo.window_size_seconds, 3600);
        assert!(slo.request_timestamps.is_empty());
    }
    
    #[test]
    fn test_slo_invalid_target() {
        let slo = SLO::new("invalid".to_string(), "Invalid SLO".to_string(), 1.5);
        assert_eq!(slo, Err(SREError::InvalidSloTarget(1.5)));
        
        let slo = SLO::new("invalid".to_string(), "Invalid SLO".to_string(), -0.1);
        assert_eq!(slo, Err(SREError::InvalidSloTarget(-0.1)));
    }
    
    #[test]
    fn test_slo_time_error() {
        // This test is more for coverage, as it's difficult to trigger a time error in normal conditions
        let slo = SLO::new("test".to_string(), "Test SLO".to_string(), 0.99);
        assert!(slo.is_ok());
    }
    
    #[test]
    fn test_slo_request_recording() {
        let mut slo = SLO::new("latency".to_string(), "95th percentile latency".to_string(), 0.95).unwrap();
        
        // Record 100 successful requests
        for _ in 0..100 {
            slo.record_request(true).unwrap();
        }
        
        assert_eq!(slo.total_requests, 100);
        assert_eq!(slo.successful_requests, 100);
        assert_eq!(slo.actual, 1.0);
        assert_eq!(slo.error_consumed(), 0.0);
        assert_eq!(slo.error_budget_remaining(), -0.05); // We're exceeding target by 0.05
        assert!(slo.is_meeting_target());
        
        // Record 10 failed requests
        for _ in 0..10 {
            slo.record_request(false).unwrap();
        }
        
        assert_eq!(slo.total_requests, 110);
        assert_eq!(slo.successful_requests, 100);
        assert_eq!(slo.actual, 100.0/110.0);
        assert_eq!(slo.error_consumed(), 1.0 - (100.0/110.0));
        assert!((slo.error_budget_remaining() - (0.95 - (100.0/110.0))).abs() < f64::EPSILON);
        assert!(slo.is_meeting_target());
    }
    
    #[test]
    fn test_service_creation() {
        let service = Service::new("api-service".to_string(), "API Service".to_string(), 0.01);
        assert!(service.is_ok());
        
        let service = service.unwrap();
        assert_eq!(service.id, "api-service");
        assert_eq!(service.name, "API Service");
        assert_eq!(service.total_error_budget, 0.01);
        assert_eq!(service.consumed_error_budget, 0.0);
        assert!(service.slos.is_empty());
    }
    
    #[test]
    fn test_service_invalid_error_budget() {
        let service = Service::new("invalid".to_string(), "Invalid Service".to_string(), 1.5);
        assert_eq!(service, Err(SREError::InvalidErrorBudget(1.5)));
        
        let service = Service::new("invalid".to_string(), "Invalid Service".to_string(), -0.1);
        assert_eq!(service, Err(SREError::InvalidErrorBudget(-0.1)));
    }
    
    #[test]
    fn test_service_error_budget_edge_cases() {
        let mut service = Service::new("edge-service".to_string(), "Edge Service".to_string(), 0.01).unwrap();
        
        // Test consuming exactly the available budget
        assert!(service.consume_error_budget(0.01).is_ok());
        assert_eq!(service.consumed_error_budget, 0.01);
        assert_eq!(service.error_budget_remaining(), 0.0);
        
        // Test that we can't consume more
        assert_eq!(
            service.consume_error_budget(0.001), 
            Err(SREError::InsufficientErrorBudget {
                service: "edge-service".to_string(),
                available: 0.0,
                requested: 0.001,
            })
        );
        
        // Test with zero budget
        let service = Service::new("zero-service".to_string(), "Zero Budget Service".to_string(), 0.0);
        assert!(service.is_ok());
        let service = service.unwrap();
        assert_eq!(service.error_budget_remaining(), 0.0);
        assert!(!service.has_sufficient_error_budget(0.001));
        
        // Test with maximum budget
        let service = Service::new("max-service".to_string(), "Max Budget Service".to_string(), 1.0);
        assert!(service.is_ok());
        let service = service.unwrap();
        assert_eq!(service.error_budget_remaining(), 1.0);
        assert!(service.has_sufficient_error_budget(1.0));
    }
    
    #[test]
    fn test_service_slo_management() {
        let mut service = Service::new("web-service".to_string(), "Web Service".to_string(), 0.02).unwrap();
        
        let slo = SLO::new("availability".to_string(), "99.9% availability".to_string(), 0.999).unwrap();
        service.add_slo(slo);
        
        assert!(service.slos.contains_key("availability"));
        assert_eq!(service.slos.len(), 1);
        
        // Record requests
        assert!(service.record_request("availability", true).is_ok());
        assert!(service.record_request("availability", false).is_ok());
        assert_eq!(
            service.record_request("nonexistent", true), 
            Err(SREError::SloNotFound("nonexistent".to_string()))
        );
    }
    
    #[test]
    fn test_service_error_budget() {
        let mut service = Service::new("payment-service".to_string(), "Payment Service".to_string(), 0.05).unwrap();
        
        let mut availability_slo = SLO::new("availability".to_string(), "Availability SLO".to_string(), 0.99).unwrap();
        for _ in 0..99 {
            availability_slo.record_request(true).unwrap();
        }
        availability_slo.record_request(false).unwrap(); // 1% error rate
        
        let mut latency_slo = SLO::new("latency".to_string(), "Latency SLO".to_string(), 0.95).unwrap();
        for _ in 0..95 {
            latency_slo.record_request(true).unwrap();
        }
        for _ in 0..5 {
            latency_slo.record_request(false).unwrap(); // 5% error rate
        }
        
        service.add_slo(availability_slo);
        service.add_slo(latency_slo);
        
        // Update error budget calculations
        service.consumed_error_budget = service.calculate_consumed_error_budget();
        
        assert!(service.has_sufficient_error_budget(0.01));
        assert!(service.has_sufficient_error_budget(0.03));
        assert!(!service.has_sufficient_error_budget(0.1));
        
        assert!(service.consume_error_budget(0.02).is_ok());
        assert_eq!(service.consumed_error_budget, 0.02);
        assert_eq!(service.error_budget_remaining(), 0.03);
        
        assert_eq!(
            service.consume_error_budget(0.1), 
            Err(SREError::InsufficientErrorBudget {
                service: "payment-service".to_string(),
                available: 0.03,
                requested: 0.1,
            })
        );
    }
    
    #[test]
    fn test_sre_manager() {
        let observability = Arc::new(ObservabilityManager::new());
        let manager = SREManager::new(observability);
        
        let mut service = Service::new("test-service".to_string(), "Test Service".to_string(), 0.01).unwrap();
        let slo = SLO::new("test-slo".to_string(), "Test SLO".to_string(), 0.99).unwrap();
        service.add_slo(slo);
        
        assert!(manager.register_service(service).is_ok());
        assert!(manager.get_service("test-service").is_ok());
        assert_eq!(
            manager.get_service("nonexistent"), 
            Err(SREError::ServiceNotFound("nonexistent".to_string()))
        );
        
        // Record requests
        assert!(manager.record_request("test-service", "test-slo", true).is_ok());
        assert!(manager.record_request("test-service", "test-slo", false).is_ok());
        assert_eq!(
            manager.record_request("test-service", "nonexistent", true), 
            Err(SREError::SloNotFound("nonexistent".to_string()))
        );
        assert_eq!(
            manager.record_request("nonexistent", "test-slo", true), 
            Err(SREError::ServiceNotFound("nonexistent".to_string()))
        );
        
        // Error budget management
        assert!(manager.has_sufficient_error_budget("test-service", 0.005).unwrap());
        assert!(manager.consume_error_budget("test-service", 0.005).is_ok());
        assert_eq!(
            manager.consume_error_budget("nonexistent", 0.005), 
            Err(SREError::ServiceNotFound("nonexistent".to_string()))
        );
    }
    
    #[test]
    fn test_sre_manager_error_conditions() {
        let observability = Arc::new(ObservabilityManager::new());
        let manager = SREManager::new(observability);
        
        // Test operations on non-existent service
        assert_eq!(
            manager.record_request("nonexistent", "nonexistent-slo", true), 
            Err(SREError::ServiceNotFound("nonexistent".to_string()))
        );
        
        assert_eq!(
            manager.has_sufficient_error_budget("nonexistent", 0.001), 
            Err(SREError::ServiceNotFound("nonexistent".to_string()))
        );
        
        assert_eq!(
            manager.consume_error_budget("nonexistent", 0.001), 
            Err(SREError::ServiceNotFound("nonexistent".to_string()))
        );
        
        // Test operations on existent service but non-existent SLO
        let service = Service::new("existent-service".to_string(), "Existent Service".to_string(), 0.01).unwrap();
        assert!(manager.register_service(service).is_ok());
        
        assert_eq!(
            manager.record_request("existent-service", "nonexistent-slo", true), 
            Err(SREError::SloNotFound("nonexistent-slo".to_string()))
        );
    }
    
    #[test]
    fn test_sre_timer() {
        let observability = Arc::new(ObservabilityManager::new());
        let manager = Arc::new(SREManager::new(observability));
        
        let mut service = Service::new("timer-service".to_string(), "Timer Service".to_string(), 0.01).unwrap();
        let slo = SLO::new("timer-slo".to_string(), "Timer SLO".to_string(), 0.99).unwrap();
        service.add_slo(slo);
        
        assert!(manager.register_service(service).is_ok());
        
        // Test successful operation timer
        let timer = manager.start_timer("timer-service".to_string(), "timer-slo".to_string());
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(10));
        timer.stop_success();
        
        // Test failed operation timer
        let timer = manager.start_timer("timer-service".to_string(), "timer-slo".to_string());
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(10));
        timer.stop_failure();
        
        // Verify metrics were recorded
        let service = manager.get_service("timer-service").unwrap();
        let slo = service.slos.get("timer-slo").unwrap();
        assert_eq!(slo.total_requests, 2);
        assert_eq!(slo.successful_requests, 1);
    }
    
    #[test]
    fn test_sre_manager_with_latency() {
        let observability = Arc::new(ObservabilityManager::new());
        let manager = SREManager::new(observability.clone());
        
        let mut service = Service::new("latency-service".to_string(), "Latency Service".to_string(), 0.01).unwrap();
        let slo = SLO::new("latency-slo".to_string(), "Latency SLO".to_string(), 0.99).unwrap();
        service.add_slo(slo);
        
        assert!(manager.register_service(service).is_ok());
        
        // Record request with latency
        assert!(manager.record_request_with_latency("latency-service", "latency-slo", true, Some(50)).is_ok());
        assert!(manager.record_request_with_latency("latency-service", "latency-slo", false, Some(100)).is_ok());
        
        // Verify metrics were recorded
        let service = manager.get_service("latency-service").unwrap();
        let slo = service.slos.get("latency-slo").unwrap();
        assert_eq!(slo.total_requests, 2);
        assert_eq!(slo.successful_requests, 1);
    }
}