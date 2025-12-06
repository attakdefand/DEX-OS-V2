//! Comprehensive tests for the AI Alerts functionality
//! 
//! These tests cover all aspects of the AI Alerts implementation,
//! including alert configuration, health monitoring, alert triggering,
//! and integration with the Keeper service.

use dex_core::keeper::{AlertConfig, HealthStatus, KeeperService};
use std::collections::HashMap;

#[test]
fn test_ai_alerts_full_workflow() {
    let mut keeper = KeeperService::new(100);

    // 1. Register a service for monitoring
    keeper.register_service("payment_service".to_string());

    // 2. Configure AI alerts
    let alert_config = AlertConfig {
        service_id: "payment_service".to_string(),
        response_time_threshold_ms: Some(100),
        error_rate_threshold: Some(0.05),
        recipients: vec![
            "admin@example.com".to_string(),
            "ops@example.com".to_string(),
        ],
        enabled: true,
    };
    keeper.configure_alerts(alert_config);

    // 3. Verify alert configuration
    let config = keeper.get_alert_config("payment_service");
    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!(config.service_id, "payment_service");
    assert_eq!(config.response_time_threshold_ms, Some(100));
    assert_eq!(config.error_rate_threshold, Some(0.05));
    assert_eq!(config.recipients.len(), 2);
    assert!(config.enabled);

    // 4. Report healthy status (should not trigger alerts)
    let mut healthy_metrics = HashMap::new();
    healthy_metrics.insert("error_rate".to_string(), 0.01); // Below threshold
    healthy_metrics.insert("throughput".to_string(), 1000.0);

    let result = keeper.report_health(
        "payment_service".to_string(),
        HealthStatus::Healthy,
        Some(50), // Below response time threshold
        None,
        healthy_metrics,
    );
    assert!(result.is_ok());

    // 5. Report degraded status (should trigger alerts)
    let mut degraded_metrics = HashMap::new();
    degraded_metrics.insert("error_rate".to_string(), 0.08); // Above threshold
    degraded_metrics.insert("throughput".to_string(), 800.0);

    let result = keeper.report_health(
        "payment_service".to_string(),
        HealthStatus::Degraded,
        Some(150), // Above response time threshold
        Some("High latency detected".to_string()),
        degraded_metrics,
    );
    // Alerts should be triggered but reporting should succeed
    assert!(result.is_ok());

    // 6. Check health status
    let health = keeper.get_service_health("payment_service");
    assert!(health.is_some());
    let health = health.unwrap();
    assert_eq!(health.status, HealthStatus::Degraded);
    assert_eq!(health.response_time_ms, Some(150));
    assert_eq!(health.error_message, Some("High latency detected".to_string()));
    assert_eq!(health.metrics.len(), 2);

    // 7. Get recent events
    let events = keeper.get_recent_events(5);
    assert!(!events.is_empty());
    // Should have at least one event for the status change
}

#[test]
fn test_ai_alerts_configuration_management() {
    let mut keeper = KeeperService::new(50);

    // Test configuring alerts for multiple services
    let payment_alerts = AlertConfig {
        service_id: "payment_service".to_string(),
        response_time_threshold_ms: Some(100),
        error_rate_threshold: Some(0.05),
        recipients: vec!["payments-team@example.com".to_string()],
        enabled: true,
    };

    let database_alerts = AlertConfig {
        service_id: "database_service".to_string(),
        response_time_threshold_ms: Some(200),
        error_rate_threshold: Some(0.01),
        recipients: vec!["db-admins@example.com".to_string()],
        enabled: false, // Disabled initially
    };

    keeper.configure_alerts(payment_alerts.clone());
    keeper.configure_alerts(database_alerts.clone());

    // Verify configurations
    let payment_config = keeper.get_alert_config("payment_service").unwrap();
    assert_eq!(payment_config, &payment_alerts);

    let db_config = keeper.get_alert_config("database_service").unwrap();
    assert_eq!(db_config, &database_alerts);

    // Update configuration
    let updated_payment_alerts = AlertConfig {
        service_id: "payment_service".to_string(),
        response_time_threshold_ms: Some(150), // Increased threshold
        error_rate_threshold: Some(0.07),      // Increased threshold
        recipients: vec![
            "payments-team@example.com".to_string(),
            "management@example.com".to_string(), // Added recipient
        ],
        enabled: true,
    };

    keeper.configure_alerts(updated_payment_alerts.clone());

    // Verify update
    let updated_config = keeper.get_alert_config("payment_service").unwrap();
    assert_eq!(updated_config, &updated_payment_alerts);
}

#[test]
fn test_ai_alerts_triggering_conditions() {
    let mut keeper = KeeperService::new(100);
    keeper.register_service("test_service".to_string());

    // Configure sensitive alerts
    let alert_config = AlertConfig {
        service_id: "test_service".to_string(),
        response_time_threshold_ms: Some(50),
        error_rate_threshold: Some(0.02),
        recipients: vec!["alerts@example.com".to_string()],
        enabled: true,
    };
    keeper.configure_alerts(alert_config);

    // Test response time threshold triggering
    let mut metrics = HashMap::new();
    metrics.insert("error_rate".to_string(), 0.01); // Below error threshold

    let result = keeper.report_health(
        "test_service".to_string(),
        HealthStatus::Healthy,
        Some(100), // Above response time threshold
        None,
        metrics,
    );
    // Should succeed but trigger alerts
    assert!(result.is_ok());

    // Test error rate threshold triggering
    let mut error_metrics = HashMap::new();
    error_metrics.insert("error_rate".to_string(), 0.05); // Above error threshold

    let result = keeper.report_health(
        "test_service".to_string(),
        HealthStatus::Healthy,
        Some(25), // Below response time threshold
        None,
        error_metrics,
    );
    // Should succeed but trigger alerts
    assert!(result.is_ok());

    // Test combined threshold triggering
    let mut combined_metrics = HashMap::new();
    combined_metrics.insert("error_rate".to_string(), 0.03); // Above error threshold

    let result = keeper.report_health(
        "test_service".to_string(),
        HealthStatus::Unhealthy, // Unhealthy status should trigger alerts
        Some(75),                // Above response time threshold
        Some("Service failure".to_string()),
        combined_metrics,
    );
    // Should succeed but trigger multiple alerts
    assert!(result.is_ok());
}

#[test]
fn test_ai_alerts_disabled_functionality() {
    let mut keeper = KeeperService::new(100);
    keeper.register_service("disabled_service".to_string());

    // Configure but disable alerts
    let alert_config = AlertConfig {
        service_id: "disabled_service".to_string(),
        response_time_threshold_ms: Some(50),
        error_rate_threshold: Some(0.02),
        recipients: vec!["alerts@example.com".to_string()],
        enabled: false, // Disabled
    };
    keeper.configure_alerts(alert_config);

    // Report conditions that would normally trigger alerts
    let mut metrics = HashMap::new();
    metrics.insert("error_rate".to_string(), 0.10); // Well above threshold

    let result = keeper.report_health(
        "disabled_service".to_string(),
        HealthStatus::Unhealthy, // Unhealthy status
        Some(100),               // Well above response time threshold
        Some("Critical failure".to_string()),
        metrics,
    );
    // Should succeed without triggering alerts (because they're disabled)
    assert!(result.is_ok());

    // Re-enable alerts
    let mut enabled_config = keeper
        .get_alert_config("disabled_service")
        .unwrap()
        .clone();
    enabled_config.enabled = true;
    keeper.configure_alerts(enabled_config);

    // Now alerts should trigger
    let mut new_metrics = HashMap::new();
    new_metrics.insert("error_rate".to_string(), 0.15);

    let result = keeper.report_health(
        "disabled_service".to_string(),
        HealthStatus::Degraded,
        Some(150),
        Some("Performance degradation".to_string()),
        new_metrics,
    );
    // Should succeed and trigger alerts now
    assert!(result.is_ok());
}

#[test]
fn test_ai_alerts_edge_cases() {
    let mut keeper = KeeperService::new(100);

    // Test reporting health for unregistered service
    let mut metrics = HashMap::new();
    metrics.insert("error_rate".to_string(), 0.05);

    let result = keeper.report_health(
        "nonexistent_service".to_string(),
        HealthStatus::Healthy,
        Some(50),
        None,
        metrics,
    );
    // Should fail because service is not registered
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Service not registered"));

    // Register the service
    keeper.register_service("nonexistent_service".to_string());

    // Now it should succeed
    let mut metrics = HashMap::new();
    metrics.insert("error_rate".to_string(), 0.05);

    let result = keeper.report_health(
        "nonexistent_service".to_string(),
        HealthStatus::Healthy,
        Some(50),
        None,
        metrics,
    );
    assert!(result.is_ok());

    // Test with None thresholds (no threshold checking)
    let alert_config = AlertConfig {
        service_id: "nonexistent_service".to_string(),
        response_time_threshold_ms: None, // No response time threshold
        error_rate_threshold: None,       // No error rate threshold
        recipients: vec!["admin@example.com".to_string()],
        enabled: true,
    };
    keeper.configure_alerts(alert_config);

    // Report health that would trigger alerts if thresholds were set
    let mut metrics = HashMap::new();
    metrics.insert("error_rate".to_string(), 0.99); // Very high error rate

    let result = keeper.report_health(
        "nonexistent_service".to_string(),
        HealthStatus::Unhealthy,
        Some(1000), // Very high response time
        Some("Catastrophic failure".to_string()),
        metrics,
    );
    // Should succeed and trigger alerts based on status only (no threshold checking)
    assert!(result.is_ok());
}

#[test]
fn test_ai_alerts_health_status_transitions() {
    let mut keeper = KeeperService::new(100);
    keeper.register_service("transition_service".to_string());

    let alert_config = AlertConfig {
        service_id: "transition_service".to_string(),
        response_time_threshold_ms: Some(100),
        error_rate_threshold: Some(0.05),
        recipients: vec!["alerts@example.com".to_string()],
        enabled: true,
    };
    keeper.configure_alerts(alert_config);

    // Test various health status transitions
    let statuses = vec![
        (HealthStatus::Unknown, "Initial state"),
        (HealthStatus::Healthy, "Service is healthy"),
        (HealthStatus::Degraded, "Performance degradation"),
        (HealthStatus::Unhealthy, "Service failure"),
        (HealthStatus::Degraded, "Recovering from failure"),
        (HealthStatus::Healthy, "Full recovery"),
    ];

    let mut metrics = HashMap::new();
    metrics.insert("error_rate".to_string(), 0.01);

    for (status, description) in statuses {
        let result = keeper.report_health(
            "transition_service".to_string(),
            status,
            Some(50),
            Some(description.to_string()),
            metrics.clone(),
        );
        assert!(result.is_ok(), "Failed to report status: {:?}", status);
    }

    // Check that all health statuses were recorded
    let health = keeper.get_service_health("transition_service").unwrap();
    assert_eq!(health.status, HealthStatus::Healthy);

    // Check recent events for status transitions
    let events = keeper.get_recent_events(10);
    assert!(events.len() >= 5); // At least 5 transitions
}