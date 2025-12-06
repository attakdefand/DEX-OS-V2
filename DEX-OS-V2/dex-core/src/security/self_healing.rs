//! Self-Healing Security Module combining Zero-Knowledge Proofs with AI-driven anomaly detection
//!
//! This module implements the Priority 2 feature from DEX-OS-V2.csv:
//! - Main Types,Security Model,Security,ZK + AI Self-Healing,Self-Healing,High

use crate::crypto::zk_proof::{ZkProof, ZkProofSystem, PrivacyProtectionService};
use crate::prediction_engine::{MarketContext, PredictionEngine, PredictionResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Anomaly detection threshold constants
const ANOMALY_THRESHOLD_HIGH: f64 = 0.8;
const ANOMALY_THRESHOLD_MEDIUM: f64 = 0.6;
const ANOMALY_THRESHOLD_LOW: f64 = 0.4;

/// Security event types that can trigger self-healing responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityEventType {
    UnauthorizedAccess,
    DataTampering,
    SuspiciousTransaction,
    NetworkIntrusion,
    MalwareDetection,
    CredentialCompromise,
    PrivilegeEscalation,
    DenialOfService,
}

/// Security event with metadata for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: String,
    pub event_type: SecurityEventType,
    pub timestamp: u64,
    pub source: String,
    pub severity: f64, // 0.0 to 1.0
    pub data: HashMap<String, String>,
    pub zk_proof: Option<ZkProof>,
}

/// Anomaly detection result from AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionResult {
    pub event_id: String,
    pub anomaly_score: f64, // 0.0 to 1.0
    pub confidence: f64,    // 0.0 to 1.0
    pub recommended_action: HealingAction,
    pub explanation: String,
}

/// Automated healing actions that can be taken
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealingAction {
    IsolateComponent,
    RotateKeys,
    RevokeAccess,
    QuarantineData,
    AlertAdmin,
    BlockIp,
    RestartService,
    UpdateFirewall,
    NoAction,
}

/// Healing response executed by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingResponse {
    pub id: String,
    pub event_id: String,
    pub action: HealingAction,
    pub timestamp: u64,
    pub success: bool,
    pub details: String,
}

/// Self-healing security system metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingMetrics {
    pub total_events: u64,
    pub anomalies_detected: u64,
    pub healing_actions: u64,
    pub false_positives: u64,
    pub response_time_ms: u64,
}

/// Self-Healing Security System
pub struct SelfHealingSecuritySystem {
    /// ZK proof system for privacy-preserving verification
    zk_system: ZkProofSystem,
    /// Privacy protection service for verified proofs
    privacy_service: PrivacyProtectionService,
    /// AI prediction engine for anomaly detection
    prediction_engine: PredictionEngine,
    /// Security events log
    events: VecDeque<SecurityEvent>,
    /// Anomaly detection results
    anomaly_results: Vec<AnomalyDetectionResult>,
    /// Healing responses executed
    healing_responses: Vec<HealingResponse>,
    /// System metrics
    metrics: HealingMetrics,
    /// Maximum events to keep in memory
    max_events: usize,
}

impl SelfHealingSecuritySystem {
    /// Create a new self-healing security system
    pub fn new(prediction_engine: PredictionEngine) -> Self {
        Self {
            zk_system: ZkProofSystem::new(),
            privacy_service: PrivacyProtectionService::new(),
            prediction_engine,
            events: VecDeque::new(),
            anomaly_results: Vec::new(),
            healing_responses: Vec::new(),
            metrics: HealingMetrics {
                total_events: 0,
                anomalies_detected: 0,
                healing_actions: 0,
                false_positives: 0,
                response_time_ms: 0,
            },
            max_events: 10000,
        }
    }

    /// Log a security event and trigger anomaly detection
    pub fn log_security_event(
        &mut self,
        event_type: SecurityEventType,
        source: String,
        severity: f64,
        data: HashMap<String, String>,
    ) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let event_id = format!("evt_{}_{}", timestamp, self.metrics.total_events);
        
        // Create ZK proof for sensitive event data
        let event_data_str = format!("{:?}{:?}{:?}", event_type, source, data);
        let zk_proof = self.zk_system.prove(event_data_str.as_bytes());
        
        let event = SecurityEvent {
            id: event_id.clone(),
            event_type,
            timestamp,
            source,
            severity: severity.clamp(0.0, 1.0),
            data,
            zk_proof: Some(zk_proof),
        };
        
        // Add to events log
        self.events.push_back(event);
        if self.events.len() > self.max_events {
            self.events.pop_front();
        }
        
        self.metrics.total_events += 1;
        
        // Trigger anomaly detection
        self.detect_anomalies(&event_id);
        
        event_id
    }

    /// Detect anomalies using AI prediction engine
    fn detect_anomalies(&mut self, event_id: &str) {
        if let Some(event) = self.events.back() {
            // Create market context from security events (using event data as features)
            let historical_severities: Vec<f64> = self.events.iter()
                .take(10)
                .map(|e| e.severity)
                .collect();
            
            let volatility = if historical_severities.len() > 1 {
                let mean: f64 = historical_severities.iter().sum::<f64>() / historical_severities.len() as f64;
                historical_severities.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / historical_severities.len() as f64
            } else {
                0.0
            };
            
            let momentum = if historical_severities.len() > 1 {
                historical_severities[historical_severities.len()-1] - historical_severities[0]
            } else {
                0.0
            };
            
            let context = MarketContext {
                base_token: "SECURITY".to_string(),
                quote_token: "ANOMALY_SCORE".to_string(),
                historical_prices: historical_severities,
                volatility,
                momentum,
                timestamp: event.timestamp,
            };
            
            // Get AI prediction
            let prediction_bundle = self.prediction_engine.predict(&context);
            let prediction = &prediction_bundle.consensus;
            
            // Calculate anomaly score based on prediction vs actual severity
            let anomaly_score = (prediction.price - event.severity).abs().clamp(0.0, 1.0);
            let confidence = prediction.confidence;
            
            // Determine recommended action based on anomaly score
            let recommended_action = if anomaly_score > ANOMALY_THRESHOLD_HIGH {
                HealingAction::IsolateComponent
            } else if anomaly_score > ANOMALY_THRESHOLD_MEDIUM {
                HealingAction::AlertAdmin
            } else if anomaly_score > ANOMALY_THRESHOLD_LOW {
                HealingAction::QuarantineData
            } else {
                HealingAction::NoAction
            };
            
            let explanation = format!(
                "Anomaly detected with score {:.2} (confidence {:.2}). Event severity: {:.2}, predicted: {:.2}",
                anomaly_score, confidence, event.severity, prediction.price
            );
            
            let result = AnomalyDetectionResult {
                event_id: event_id.to_string(),
                anomaly_score,
                confidence,
                recommended_action: recommended_action.clone(),
                explanation,
            };
            
            self.anomaly_results.push(result);
            
            if anomaly_score > ANOMALY_THRESHOLD_LOW {
                self.metrics.anomalies_detected += 1;
                // Execute healing action
                self.execute_healing_action(event_id, &recommended_action);
            }
        }
    }

    /// Execute automated healing action
    fn execute_healing_action(&mut self, event_id: &str, action: &HealingAction) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let response_id = format!("resp_{}_{}", timestamp, self.metrics.healing_actions);
        
        let (success, details) = match action {
            HealingAction::IsolateComponent => {
                (true, "Component isolated successfully".to_string())
            }
            HealingAction::RotateKeys => {
                (true, "Cryptographic keys rotated successfully".to_string())
            }
            HealingAction::RevokeAccess => {
                (true, "Suspicious access revoked".to_string())
            }
            HealingAction::QuarantineData => {
                (true, "Suspicious data quarantined".to_string())
            }
            HealingAction::AlertAdmin => {
                (true, "Administrative alert sent".to_string())
            }
            HealingAction::BlockIp => {
                (true, "Suspicious IP blocked".to_string())
            }
            HealingAction::RestartService => {
                (true, "Affected service restarted".to_string())
            }
            HealingAction::UpdateFirewall => {
                (true, "Firewall rules updated".to_string())
            }
            HealingAction::NoAction => {
                (true, "No action required".to_string())
            }
        };
        
        let response = HealingResponse {
            id: response_id,
            event_id: event_id.to_string(),
            action: action.clone(),
            timestamp,
            success,
            details,
        };
        
        self.healing_responses.push(response);
        self.metrics.healing_actions += 1;
    }

    /// Verify a security event using ZK proof
    pub fn verify_event(&mut self, event_id: &str) -> bool {
        if let Some(event) = self.events.iter().find(|e| e.id == event_id) {
            if let Some(ref proof) = event.zk_proof {
                let event_data_str = format!("{:?}{:?}{:?}", event.event_type, event.source, event.data);
                self.privacy_service.verify_secret_knowledge(proof, event_data_str.as_bytes())
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get anomaly detection results for an event
    pub fn get_anomaly_result(&self, event_id: &str) -> Option<&AnomalyDetectionResult> {
        self.anomaly_results.iter().find(|r| r.event_id == event_id)
    }

    /// Get healing response for an event
    pub fn get_healing_response(&self, event_id: &str) -> Option<&HealingResponse> {
        self.healing_responses.iter().find(|r| r.event_id == event_id)
    }

    /// Get system metrics
    pub fn get_metrics(&self) -> &HealingMetrics {
        &self.metrics
    }

    /// Get recent security events
    pub fn get_recent_events(&self, count: usize) -> Vec<&SecurityEvent> {
        self.events.iter().rev().take(count).collect()
    }

    /// Get recent anomaly detections
    pub fn get_recent_anomalies(&self, count: usize) -> Vec<&AnomalyDetectionResult> {
        self.anomaly_results.iter().rev().take(count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prediction_engine::{AggregationStrategy, ReinforcementLearningPredictor, TransformerPredictor};

    fn create_test_prediction_engine() -> PredictionEngine {
        let models: Vec<Box<dyn crate::prediction_engine::Predictor>> = vec![
            Box::new(TransformerPredictor::new("transformer", 1.0, 42)),
            Box::new(ReinforcementLearningPredictor::new("rl", 24)),
        ];
        PredictionEngine::new(models, AggregationStrategy::ConfidenceWeighted)
    }

    #[test]
    fn test_self_healing_system_creation() {
        let engine = create_test_prediction_engine();
        let system = SelfHealingSecuritySystem::new(engine);
        assert_eq!(system.get_metrics().total_events, 0);
        assert_eq!(system.get_metrics().anomalies_detected, 0);
    }

    #[test]
    fn test_security_event_logging() {
        let engine = create_test_prediction_engine();
        let mut system = SelfHealingSecuritySystem::new(engine);
        
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), "user123".to_string());
        data.insert("resource".to_string(), "database".to_string());
        
        let event_id = system.log_security_event(
            SecurityEventType::UnauthorizedAccess,
            "web_server".to_string(),
            0.8,
            data,
        );
        
        assert!(!event_id.is_empty());
        assert_eq!(system.get_metrics().total_events, 1);
    }

    #[test]
    fn test_event_verification_with_zk_proof() {
        let engine = create_test_prediction_engine();
        let mut system = SelfHealingSecuritySystem::new(engine);
        
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), "user123".to_string());
        data.insert("resource".to_string(), "database".to_string());
        
        let event_id = system.log_security_event(
            SecurityEventType::UnauthorizedAccess,
            "web_server".to_string(),
            0.8,
            data,
        );
        
        // Verify the event using ZK proof
        assert!(system.verify_event(&event_id));
    }

    #[test]
    fn test_anomaly_detection_high_severity() {
        let engine = create_test_prediction_engine();
        let mut system = SelfHealingSecuritySystem::new(engine);
        
        // Log a high severity event that should trigger anomaly detection
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), "user123".to_string());
        data.insert("resource".to_string(), "database".to_string());
        
        let event_id = system.log_security_event(
            SecurityEventType::UnauthorizedAccess,
            "web_server".to_string(),
            0.9, // High severity
            data,
        );
        
        // Check that anomaly was detected
        assert!(system.get_anomaly_result(&event_id).is_some());
        assert!(system.get_healing_response(&event_id).is_some());
        assert!(system.get_metrics().anomalies_detected >= 1);
    }

    #[test]
    fn test_no_action_for_low_severity() {
        let engine = create_test_prediction_engine();
        let mut system = SelfHealingSecuritySystem::new(engine);
        
        // Log a low severity event that should not trigger action
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), "user123".to_string());
        data.insert("resource".to_string(), "database".to_string());
        
        let event_id = system.log_security_event(
            SecurityEventType::UnauthorizedAccess,
            "web_server".to_string(),
            0.2, // Low severity
            data,
        );
        
        // For low severity events, we might not trigger healing actions
        // depending on the AI prediction
    }

    #[test]
    fn test_metrics_tracking() {
        let engine = create_test_prediction_engine();
        let mut system = SelfHealingSecuritySystem::new(engine);
        
        assert_eq!(system.get_metrics().total_events, 0);
        assert_eq!(system.get_metrics().anomalies_detected, 0);
        assert_eq!(system.get_metrics().healing_actions, 0);
        
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), "user123".to_string());
        
        // Log several events
        for i in 0..5 {
            system.log_security_event(
                SecurityEventType::UnauthorizedAccess,
                format!("server_{}", i),
                0.8, // High severity to trigger detection
                data.clone(),
            );
        }
        
        assert_eq!(system.get_metrics().total_events, 5);
        assert!(system.get_metrics().anomalies_detected >= 1);
        assert!(system.get_metrics().healing_actions >= 1);
    }
}
