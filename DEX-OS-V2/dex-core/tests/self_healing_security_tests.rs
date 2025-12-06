//! Integration tests for the Self-Healing Security System

use dex_core::security::self_healing::{
    SelfHealingSecuritySystem, SecurityEventType, HealingAction
};
use dex_core::prediction_engine::{
    PredictionEngine, AggregationStrategy, TransformerPredictor, ReinforcementLearningPredictor
};
use std::collections::HashMap;

fn create_test_prediction_engine() -> PredictionEngine {
    let models: Vec<Box<dyn dex_core::prediction_engine::Predictor>> = vec![
        Box::new(TransformerPredictor::new("transformer", 1.0, 42)),
        Box::new(ReinforcementLearningPredictor::new("rl", 24)),
    ];
    PredictionEngine::new(models, AggregationStrategy::ConfidenceWeighted)
}

#[test]
fn test_self_healing_system_full_lifecycle() {
    let engine = create_test_prediction_engine();
    let mut system = SelfHealingSecuritySystem::new(engine);
    
    // Test 1: System creation
    assert_eq!(system.get_metrics().total_events, 0);
    assert_eq!(system.get_metrics().anomalies_detected, 0);
    assert_eq!(system.get_metrics().healing_actions, 0);
    
    // Test 2: Log security events
    let mut data = HashMap::new();
    data.insert("user_id".to_string(), "user123".to_string());
    data.insert("resource".to_string(), "database".to_string());
    data.insert("action".to_string(), "read".to_string());
    
    let event_id1 = system.log_security_event(
        SecurityEventType::UnauthorizedAccess,
        "web_server_1".to_string(),
        0.9, // High severity
        data.clone(),
    );
    
    assert!(!event_id1.is_empty());
    assert_eq!(system.get_metrics().total_events, 1);
    
    // Test 3: Verify event with ZK proof
    assert!(system.verify_event(&event_id1));
    
    // Test 4: Check anomaly detection
    let anomaly_result = system.get_anomaly_result(&event_id1);
    assert!(anomaly_result.is_some());
    
    // Test 5: Check healing response
    let healing_response = system.get_healing_response(&event_id1);
    assert!(healing_response.is_some());
    
    // Test 6: Log more events to test pattern detection
    for i in 0..5 {
        let mut event_data = data.clone();
        event_data.insert("attempt".to_string(), format!("{}", i));
        
        system.log_security_event(
            SecurityEventType::UnauthorizedAccess,
            format!("web_server_{}", i),
            0.8, // High severity
            event_data,
        );
    }
    
    assert_eq!(system.get_metrics().total_events, 6);
    assert!(system.get_metrics().anomalies_detected >= 1);
    assert!(system.get_metrics().healing_actions >= 1);
    
    // Test 7: Get recent events and anomalies
    let recent_events = system.get_recent_events(3);
    assert_eq!(recent_events.len(), 3);
    
    let recent_anomalies = system.get_recent_anomalies(3);
    assert!(!recent_anomalies.is_empty());
}

#[test]
fn test_different_security_event_types() {
    let engine = create_test_prediction_engine();
    let mut system = SelfHealingSecuritySystem::new(engine);
    
    let mut data = HashMap::new();
    data.insert("source_ip".to_string(), "192.168.1.100".to_string());
    data.insert("target_resource".to_string(), "api_endpoint".to_string());
    
    // Test different security event types
    let event_types = vec![
        SecurityEventType::UnauthorizedAccess,
        SecurityEventType::DataTampering,
        SecurityEventType::SuspiciousTransaction,
        SecurityEventType::NetworkIntrusion,
        SecurityEventType::MalwareDetection,
        SecurityEventType::CredentialCompromise,
        SecurityEventType::PrivilegeEscalation,
        SecurityEventType::DenialOfService,
    ];
    
    for (i, event_type) in event_types.iter().enumerate() {
        let mut event_data = data.clone();
        event_data.insert("event_index".to_string(), format!("{}", i));
        
        let event_id = system.log_security_event(
            event_type.clone(),
            format!("source_{}", i),
            0.7, // Medium-high severity
            event_data,
        );
        
        assert!(!event_id.is_empty());
        assert!(system.verify_event(&event_id));
    }
    
    assert_eq!(system.get_metrics().total_events, 8);
}

#[test]
fn test_healing_action_execution() {
    let engine = create_test_prediction_engine();
    let mut system = SelfHealingSecuritySystem::new(engine);
    
    let mut data = HashMap::new();
    data.insert("attacker_ip".to_string(), "10.0.0.1".to_string());
    data.insert("target_service".to_string(), "auth_service".to_string());
    
    let event_id = system.log_security_event(
        SecurityEventType::NetworkIntrusion,
        "network_monitor".to_string(),
        0.95, // Very high severity
        data,
    );
    
    // Check that appropriate healing action was taken
    let healing_response = system.get_healing_response(&event_id);
    assert!(healing_response.is_some());
    
    let response = healing_response.unwrap();
    assert_eq!(response.event_id, event_id);
    assert_eq!(response.action, HealingAction::IsolateComponent);
    assert!(response.success);
    assert!(!response.details.is_empty());
    
    assert_eq!(system.get_metrics().healing_actions, 1);
}

#[test]
fn test_low_severity_events() {
    let engine = create_test_prediction_engine();
    let mut system = SelfHealingSecuritySystem::new(engine);
    
    let mut data = HashMap::new();
    data.insert("user_id".to_string(), "user456".to_string());
    data.insert("resource".to_string(), "public_data".to_string());
    
    let event_id = system.log_security_event(
        SecurityEventType::UnauthorizedAccess,
        "web_server".to_string(),
        0.2, // Low severity
        data,
    );
    
    assert!(!event_id.is_empty());
    assert_eq!(system.get_metrics().total_events, 1);
    
    // For low severity events, we might not trigger healing actions
    // depending on the AI prediction, but we should still log the event
    assert!(system.verify_event(&event_id));
}

#[test]
fn test_metrics_and_performance() {
    let engine = create_test_prediction_engine();
    let mut system = SelfHealingSecuritySystem::new(engine);
    
    // Initial metrics check
    let initial_metrics = system.get_metrics();
    assert_eq!(initial_metrics.total_events, 0);
    assert_eq!(initial_metrics.anomalies_detected, 0);
    assert_eq!(initial_metrics.healing_actions, 0);
    assert_eq!(initial_metrics.false_positives, 0);
    
    // Log multiple events rapidly to test performance
    let mut data = HashMap::new();
    data.insert("test_suite".to_string(), "performance_test".to_string());
    
    let start_time = std::time::Instant::now();
    
    for i in 0..100 {
        let mut event_data = data.clone();
        event_data.insert("iteration".to_string(), format!("{}", i));
        
        system.log_security_event(
            SecurityEventType::SuspiciousTransaction,
            format!("service_{}", i % 5),
            0.5 + (i as f64 * 0.005), // Increasing severity
            event_data,
        );
    }
    
    let duration = start_time.elapsed();
    
    // Check final metrics
    let final_metrics = system.get_metrics();
    assert_eq!(final_metrics.total_events, 100);
    assert!(final_metrics.anomalies_detected >= 1);
    assert!(final_metrics.healing_actions >= 1);
    
    // Performance check - should complete in reasonable time
    assert!(duration.as_millis() < 5000); // Less than 5 seconds
    
    // Check recent events retrieval
    let recent_events = system.get_recent_events(10);
    assert_eq!(recent_events.len(), 10);
    
    let recent_anomalies = system.get_recent_anomalies(10);
    assert!(!recent_anomalies.is_empty());
}

#[test]
fn test_edge_cases_and_error_handling() {
    let engine = create_test_prediction_engine();
    let mut system = SelfHealingSecuritySystem::new(engine);
    
    // Test querying non-existent events
    assert!(!system.verify_event("non_existent_event"));
    assert!(system.get_anomaly_result("non_existent_event").is_none());
    assert!(system.get_healing_response("non_existent_event").is_none());
    
    // Test with empty data
    let empty_data = HashMap::new();
    let event_id = system.log_security_event(
        SecurityEventType::UnauthorizedAccess,
        "empty_data_source".to_string(),
        0.6,
        empty_data,
    );
    
    assert!(!event_id.is_empty());
    assert!(system.verify_event(&event_id));
    
    // Test with very high severity
    let mut high_severity_data = HashMap::new();
    high_severity_data.insert("critical".to_string(), "true".to_string());
    
    let critical_event_id = system.log_security_event(
        SecurityEventType::DenialOfService,
        "critical_system".to_string(),
        1.0, // Maximum severity
        high_severity_data,
    );
    
    assert!(!critical_event_id.is_empty());
    let critical_anomaly = system.get_anomaly_result(&critical_event_id);
    assert!(critical_anomaly.is_some());
    
    // Test with very low severity
    let mut low_severity_data = HashMap::new();
    low_severity_data.insert("benign".to_string(), "true".to_string());
    
    let low_event_id = system.log_security_event(
        SecurityEventType::UnauthorizedAccess,
        "low_priority_system".to_string(),
        0.0, // Minimum severity
        low_severity_data,
    );
    
    assert!(!low_event_id.is_empty());
}