//! Keeper service for health monitoring
//!
//! This module implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Core Trading,Keeper,Keeper,Health Check,Service Monitoring,Medium

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Keeper service for monitoring system health
#[derive(Debug, Clone)]
pub struct KeeperService {
    /// Health checks for various services
    health_checks: HashMap<String, ServiceHealth>,
    /// Alert configurations
    alert_configs: HashMap<String, AlertConfig>,
    /// Recent health events
    events: Vec<HealthEvent>,
    /// Maximum number of events to store
    max_events: usize,
}

/// Health status of a service
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Service identifier
    pub service_id: String,
    /// Current health status
    pub status: HealthStatus,
    /// Last check timestamp
    pub last_check: u64,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
    /// Error message if unhealthy
    pub error_message: Option<String>,
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Configuration for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Service identifier
    pub service_id: String,
    /// Threshold for response time alert (milliseconds)
    pub response_time_threshold_ms: Option<u64>,
    /// Threshold for error rate (0.0 to 1.0)
    pub error_rate_threshold: Option<f64>,
    /// Alert recipients
    pub recipients: Vec<String>,
    /// Whether alerts are enabled
    pub enabled: bool,
}

/// Health event for auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEvent {
    /// Event identifier
    pub id: String,
    /// Service identifier
    pub service_id: String,
    /// Event timestamp
    pub timestamp: u64,
    /// Previous health status
    pub previous_status: HealthStatus,
    /// New health status
    pub new_status: HealthStatus,
    /// Event description
    pub description: String,
}

impl KeeperService {
    /// Create a new keeper service
    pub fn new(max_events: usize) -> Self {
        Self {
            health_checks: HashMap::new(),
            alert_configs: HashMap::new(),
            events: Vec::new(),
            max_events,
        }
    }

    /// Register a service for health monitoring
    pub fn register_service(&mut self, service_id: String) {
        let health = ServiceHealth {
            service_id: service_id.clone(),
            status: HealthStatus::Unknown,
            last_check: 0,
            response_time_ms: None,
            error_message: None,
            metrics: HashMap::new(),
        };

        self.health_checks.insert(service_id, health);
    }

    /// Report health check result
    pub fn report_health(
        &mut self,
        service_id: String,
        status: HealthStatus,
        response_time_ms: Option<u64>,
        error_message: Option<String>,
        metrics: HashMap<String, f64>,
    ) -> Result<(), KeeperError> {
        if !self.health_checks.contains_key(&service_id) {
            return Err(KeeperError::ServiceNotRegistered);
        }

        let mut change_context: Option<(HealthStatus, HealthStatus, String)> = None;

        let health_snapshot = {
            let health = self.health_checks.get_mut(&service_id).unwrap();
            let previous_status = health.status.clone();

            health.status = status;
            health.last_check = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            health.response_time_ms = response_time_ms;
            health.error_message = error_message;
            health.metrics = metrics;

            if previous_status != health.status {
                let description = format!(
                    "Health status changed from {:?} to {:?}",
                    previous_status, health.status
                );
                change_context = Some((previous_status, health.status.clone(), description));
            }

            health.clone()
        };

        if let Some((previous_status, new_status, description)) = change_context {
            self.log_health_event(&service_id, &previous_status, &new_status, &description);
        }

        self.check_and_send_alerts(&service_id, &health_snapshot)?;

        Ok(())
    }

    /// Get health status for a service
    pub fn get_service_health(&self, service_id: &str) -> Option<&ServiceHealth> {
        self.health_checks.get(service_id)
    }

    /// Get health status for all services
    pub fn get_all_health(&self) -> Vec<&ServiceHealth> {
        self.health_checks.values().collect()
    }

    /// Configure alerts for a service
    pub fn configure_alerts(&mut self, config: AlertConfig) {
        self.alert_configs.insert(config.service_id.clone(), config);
    }

    /// Get alert configuration for a service
    pub fn get_alert_config(&self, service_id: &str) -> Option<&AlertConfig> {
        self.alert_configs.get(service_id)
    }

    /// Get recent health events
    pub fn get_recent_events(&self, count: usize) -> Vec<&HealthEvent> {
        let skip = if self.events.len() > count {
            self.events.len() - count
        } else {
            0
        };
        self.events.iter().skip(skip).collect()
    }

    /// Log a health event
    fn log_health_event(
        &mut self,
        service_id: &str,
        previous_status: &HealthStatus,
        new_status: &HealthStatus,
        description: &str,
    ) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let event_id = format!("event_{}_{}", service_id, timestamp);

        let event = HealthEvent {
            id: event_id,
            service_id: service_id.to_string(),
            timestamp,
            previous_status: previous_status.clone(),
            new_status: new_status.clone(),
            description: description.to_string(),
        };

        self.events.push(event);

        // Trim events if we exceed max_events
        if self.events.len() > self.max_events {
            self.events.drain(0..(self.events.len() - self.max_events));
        }
    }

    /// Check if alerts need to be sent and send them
    fn check_and_send_alerts(
        &self,
        service_id: &str,
        health: &ServiceHealth,
    ) -> Result<(), KeeperError> {
        if let Some(config) = self.alert_configs.get(service_id) {
            if !config.enabled {
                return Ok(());
            }

            let mut alerts_needed = Vec::new();

            // Check response time threshold
            if let (Some(threshold), Some(response_time)) =
                (config.response_time_threshold_ms, health.response_time_ms)
            {
                if response_time > threshold {
                    alerts_needed.push(format!(
                        "Response time {}ms exceeds threshold {}ms",
                        response_time, threshold
                    ));
                }
            }

            // Check for unhealthy status
            if health.status == HealthStatus::Unhealthy {
                alerts_needed.push("Service is unhealthy".to_string());
            } else if health.status == HealthStatus::Degraded {
                alerts_needed.push("Service is degraded".to_string());
            }

            // Send alerts if needed
            if !alerts_needed.is_empty() {
                self.send_alerts(service_id, &config.recipients, alerts_needed)?;
            }
        }

        Ok(())
    }

    /// Send alerts to recipients (simulated)
    fn send_alerts(
        &self,
        service_id: &str,
        recipients: &[String],
        alerts: Vec<String>,
    ) -> Result<(), KeeperError> {
        // In a real implementation, this would send actual alerts
        // For now, we'll just log that alerts would be sent
        println!(
            "ALERT for service {}: {} - Recipients: {:?}",
            service_id,
            alerts.join(", "),
            recipients
        );

        Ok(())
    }
}

impl Default for KeeperService {
    fn default() -> Self {
        Self::new(1000) // Default to storing 1000 events
    }
}

/// Errors that can occur during keeper operations
#[derive(Debug, Error)]
pub enum KeeperError {
    #[error("Service not registered")]
    ServiceNotRegistered,
    #[error("Failed to send alerts")]
    AlertSendFailed,
}
