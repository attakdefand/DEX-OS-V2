//! Intrusion Detection and Prevention System (IDS/IPS) for DEX-OS Network Security
//!
//! Implements Security Layer 6 - Network & Infrastructure Security (Perimeter Defense)
//! Provides signature-based and anomaly-based threat detection with automatic prevention.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use thiserror::Error;

/// IDS/IPS error types
#[derive(Debug, Error, Clone, PartialEq)]
pub enum IDSError {
    #[error("Threat detected: {0}")]
    ThreatDetected(String),
    #[error("Signature not found: {0}")]
    SignatureNotFound(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Attack blocked: {0}")]
    AttackBlocked(String),
}

/// Threat severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Threat response actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThreatResponse {
    /// Log the threat only
    Log,
    /// Alert administrators
    Alert,
    /// Block the IP address
    Block,
    /// Block and alert
    BlockAndAlert,
    /// Drop the packet silently
    Drop,
}

/// Attack types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttackType {
    PortScan,
    SqlInjection,
    XssAttack,
    BufferOverflow,
    DDoS,
    BruteForce,
    MalwareDetected,
    SuspiciousActivity,
    UnauthorizedAccess,
    DataExfiltration,
}

/// Threat signature for pattern matching
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatSignature {
    /// Signature ID
    pub id: String,
    /// Signature name
    pub name: String,
    /// Attack type
    pub attack_type: AttackType,
    /// Pattern to match (simplified - in production would use regex or more complex matching)
    pub pattern: String,
    /// Severity level
    pub severity: ThreatSeverity,
    /// Response action
    pub response: ThreatResponse,
    /// Description
    pub description: String,
    /// Enabled flag
    pub enabled: bool,
    /// Creation timestamp
    pub created_at: u64,
}

impl ThreatSignature {
    pub fn new(
        id: String,
        name: String,
        attack_type: AttackType,
        pattern: String,
        severity: ThreatSeverity,
        response: ThreatResponse,
        description: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            name,
            attack_type,
            pattern,
            severity,
            response,
            description,
            enabled: true,
            created_at: now,
        }
    }

    pub fn matches(&self, data: &str) -> bool {
        if !self.enabled {
            return false;
        }
        data.contains(&self.pattern)
    }
}

/// Detected threat information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectedThreat {
    /// Threat ID
    pub id: String,
    /// Source IP
    pub source_ip: IpAddr,
    /// Signature that matched
    pub signature_id: String,
    /// Attack type
    pub attack_type: AttackType,
    /// Severity
    pub severity: ThreatSeverity,
    /// Response taken
    pub response: ThreatResponse,
    /// Detection timestamp
    pub detected_at: u64,
    /// Additional details
    pub details: String,
}

/// Anomaly detection baseline
#[derive(Debug, Clone)]
pub struct AnomalyBaseline {
    /// Average requests per minute
    avg_requests_per_minute: f64,
    /// Average packet size
    avg_packet_size: f64,
    /// Average connections per IP
    avg_connections_per_ip: f64,
    /// Standard deviation for requests
    std_dev_requests: f64,
    /// Anomaly threshold (number of standard deviations)
    threshold: f64,
}

impl AnomalyBaseline {
    pub fn new(threshold: f64) -> Self {
        Self {
            avg_requests_per_minute: 0.0,
            avg_packet_size: 0.0,
            avg_connections_per_ip: 0.0,
            std_dev_requests: 0.0,
            threshold,
        }
    }

    pub fn update(&mut self, requests_per_minute: f64, packet_size: f64, connections: f64) {
        // Simple exponential moving average
        let alpha = 0.1;
        self.avg_requests_per_minute = alpha * requests_per_minute + (1.0 - alpha) * self.avg_requests_per_minute;
        self.avg_packet_size = alpha * packet_size + (1.0 - alpha) * self.avg_packet_size;
        self.avg_connections_per_ip = alpha * connections + (1.0 - alpha) * self.avg_connections_per_ip;

        // Update standard deviation (simplified)
        let diff = requests_per_minute - self.avg_requests_per_minute;
        self.std_dev_requests = alpha * diff.abs() + (1.0 - alpha) * self.std_dev_requests;
    }

    pub fn is_anomalous(&self, requests_per_minute: f64) -> bool {
        if self.std_dev_requests == 0.0 {
            return false;
        }
        let z_score = (requests_per_minute - self.avg_requests_per_minute).abs() / self.std_dev_requests;
        z_score > self.threshold
    }
}

/// Intrusion Detection System
#[derive(Debug, Clone)]
pub struct IntrusionDetectionSystem {
    /// Threat signatures database
    signatures: HashMap<String, ThreatSignature>,
    /// Detected threats
    detected_threats: Vec<DetectedThreat>,
    /// Anomaly detection baseline
    anomaly_baseline: AnomalyBaseline,
    /// Statistics
    total_inspections: u64,
    threats_detected: u64,
    false_positives: u64,
}

impl IntrusionDetectionSystem {
    pub fn new() -> Self {
        let mut ids = Self {
            signatures: HashMap::new(),
            detected_threats: Vec::new(),
            anomaly_baseline: AnomalyBaseline::new(3.0), // 3 standard deviations
            total_inspections: 0,
            threats_detected: 0,
            false_positives: 0,
        };

        // Add default signatures
        ids.load_default_signatures();
        ids
    }

    fn load_default_signatures(&mut self) {
        // SQL Injection signatures
        let _ = self.add_signature(ThreatSignature::new(
            "sql_001".to_string(),
            "SQL Injection - UNION".to_string(),
            AttackType::SqlInjection,
            "UNION SELECT".to_string(),
            ThreatSeverity::High,
            ThreatResponse::BlockAndAlert,
            "Detects UNION-based SQL injection attempts".to_string(),
        ));

        let _ = self.add_signature(ThreatSignature::new(
            "sql_002".to_string(),
            "SQL Injection - OR 1=1".to_string(),
            AttackType::SqlInjection,
            "OR 1=1".to_string(),
            ThreatSeverity::High,
            ThreatResponse::BlockAndAlert,
            "Detects OR-based SQL injection attempts".to_string(),
        ));

        // XSS signatures
        let _ = self.add_signature(ThreatSignature::new(
            "xss_001".to_string(),
            "XSS - Script Tag".to_string(),
            AttackType::XssAttack,
            "<script>".to_string(),
            ThreatSeverity::Medium,
            ThreatResponse::BlockAndAlert,
            "Detects script tag injection".to_string(),
        ));

        // Port scan detection
        let _ = self.add_signature(ThreatSignature::new(
            "scan_001".to_string(),
            "Port Scan".to_string(),
            AttackType::PortScan,
            "PORTSCAN".to_string(),
            ThreatSeverity::Medium,
            ThreatResponse::Alert,
            "Detects port scanning activity".to_string(),
        ));

        // Brute force detection
        let _ = self.add_signature(ThreatSignature::new(
            "brute_001".to_string(),
            "Brute Force Attack".to_string(),
            AttackType::BruteForce,
            "BRUTEFORCE".to_string(),
            ThreatSeverity::High,
            ThreatResponse::Block,
            "Detects brute force login attempts".to_string(),
        ));
    }

    pub fn add_signature(&mut self, signature: ThreatSignature) -> Result<(), IDSError> {
        if self.signatures.contains_key(&signature.id) {
            return Err(IDSError::InvalidSignature(format!(
                "Signature already exists: {}",
                signature.id
            )));
        }

        self.signatures.insert(signature.id.clone(), signature);
        Ok(())
    }

    pub fn remove_signature(&mut self, signature_id: &str) -> Result<(), IDSError> {
        self.signatures
            .remove(signature_id)
            .ok_or_else(|| IDSError::SignatureNotFound(signature_id.to_string()))?;
        Ok(())
    }

    pub fn inspect_data(&mut self, source_ip: IpAddr, data: &str) -> Option<DetectedThreat> {
        self.total_inspections += 1;

        // Check against all signatures
        for signature in self.signatures.values() {
            if signature.matches(data) {
                let threat = self.create_threat(source_ip, signature, data);
                self.detected_threats.push(threat.clone());
                self.threats_detected += 1;
                return Some(threat);
            }
        }

        None
    }

    pub fn check_anomaly(&mut self, requests_per_minute: f64, packet_size: f64, connections: f64) -> Option<DetectedThreat> {
        if self.anomaly_baseline.is_anomalous(requests_per_minute) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let threat = DetectedThreat {
                id: format!("anomaly_{}", now),
                source_ip: IpAddr::from([0, 0, 0, 0]),
                signature_id: "anomaly_detection".to_string(),
                attack_type: AttackType::SuspiciousActivity,
                severity: ThreatSeverity::Medium,
                response: ThreatResponse::Alert,
                detected_at: now,
                details: format!(
                    "Anomalous traffic detected: {} req/min (baseline: {:.2})",
                    requests_per_minute, self.anomaly_baseline.avg_requests_per_minute
                ),
            };

            self.detected_threats.push(threat.clone());
            self.threats_detected += 1;
            return Some(threat);
        }

        // Update baseline
        self.anomaly_baseline.update(requests_per_minute, packet_size, connections);
        None
    }

    pub fn get_detected_threats(&self) -> &[DetectedThreat] {
        &self.detected_threats
    }

    pub fn get_threats_by_severity(&self, severity: ThreatSeverity) -> Vec<&DetectedThreat> {
        self.detected_threats
            .iter()
            .filter(|t| t.severity == severity)
            .collect()
    }

    pub fn get_statistics(&self) -> IDSStatistics {
        IDSStatistics {
            total_signatures: self.signatures.len(),
            enabled_signatures: self.signatures.values().filter(|s| s.enabled).count(),
            total_inspections: self.total_inspections,
            threats_detected: self.threats_detected,
            false_positives: self.false_positives,
            detection_rate: if self.total_inspections > 0 {
                (self.threats_detected as f64 / self.total_inspections as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    fn create_threat(&self, source_ip: IpAddr, signature: &ThreatSignature, data: &str) -> DetectedThreat {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        DetectedThreat {
            id: format!("threat_{}_{}", signature.id, now),
            source_ip,
            signature_id: signature.id.clone(),
            attack_type: signature.attack_type.clone(),
            severity: signature.severity.clone(),
            response: signature.response.clone(),
            detected_at: now,
            details: format!("Matched signature: {} in data: {}", signature.name, data),
        }
    }
}

/// Intrusion Prevention System
#[derive(Debug, Clone)]
pub struct IntrusionPreventionSystem {
    /// IDS for detection
    ids: IntrusionDetectionSystem,
    /// Blocked IPs
    blocked_ips: HashMap<IpAddr, u64>,
    /// Block duration in seconds
    block_duration: u64,
    /// Auto-block enabled
    auto_block_enabled: bool,
    /// Statistics
    threats_blocked: u64,
}

impl IntrusionPreventionSystem {
    pub fn new() -> Self {
        Self {
            ids: IntrusionDetectionSystem::new(),
            blocked_ips: HashMap::new(),
            block_duration: 3600, // 1 hour
            auto_block_enabled: true,
            threats_blocked: 0,
        }
    }

    pub fn inspect_and_prevent(&mut self, source_ip: IpAddr, data: &str) -> Result<(), IDSError> {
        // Check if IP is already blocked
        if self.is_blocked(&source_ip) {
            return Err(IDSError::AttackBlocked(format!("IP blocked: {}", source_ip)));
        }

        // Inspect data
        if let Some(threat) = self.ids.inspect_data(source_ip, data) {
            // Take action based on response
            match threat.response {
                ThreatResponse::Block | ThreatResponse::BlockAndAlert => {
                    if self.auto_block_enabled {
                        self.block_ip(source_ip);
                        self.threats_blocked += 1;
                    }
                    return Err(IDSError::ThreatDetected(threat.details));
                }
                ThreatResponse::Drop => {
                    self.threats_blocked += 1;
                    return Err(IDSError::ThreatDetected(threat.details));
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn block_ip(&mut self, ip: IpAddr) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.blocked_ips.insert(ip, now);
    }

    pub fn unblock_ip(&mut self, ip: &IpAddr) {
        self.blocked_ips.remove(ip);
    }

    pub fn is_blocked(&self, ip: &IpAddr) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Some(block_time) = self.blocked_ips.get(ip) {
            now - block_time < self.block_duration
        } else {
            false
        }
    }

    pub fn get_ids(&self) -> &IntrusionDetectionSystem {
        &self.ids
    }

    pub fn get_ids_mut(&mut self) -> &mut IntrusionDetectionSystem {
        &mut self.ids
    }

    pub fn get_statistics(&self) -> IPSStatistics {
        let ids_stats = self.ids.get_statistics();
        IPSStatistics {
            ids_statistics: ids_stats,
            threats_blocked: self.threats_blocked,
            blocked_ips: self.blocked_ips.len(),
            auto_block_enabled: self.auto_block_enabled,
        }
    }

    pub fn cleanup_blocked_ips(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.blocked_ips
            .retain(|_, block_time| now - *block_time < self.block_duration);
    }
}

/// IDS statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IDSStatistics {
    pub total_signatures: usize,
    pub enabled_signatures: usize,
    pub total_inspections: u64,
    pub threats_detected: u64,
    pub false_positives: u64,
    pub detection_rate: f64,
}

/// IPS statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IPSStatistics {
    pub ids_statistics: IDSStatistics,
    pub threats_blocked: u64,
    pub blocked_ips: usize,
    pub auto_block_enabled: bool,
}

impl Default for IntrusionDetectionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for IntrusionPreventionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_threat_signature_matching() {
        let signature = ThreatSignature::new(
            "test_001".to_string(),
            "Test Signature".to_string(),
            AttackType::SqlInjection,
            "UNION SELECT".to_string(),
            ThreatSeverity::High,
            ThreatResponse::Block,
            "Test".to_string(),
        );

        assert!(signature.matches("SELECT * FROM users UNION SELECT * FROM passwords"));
        assert!(!signature.matches("SELECT * FROM users"));
    }

    #[test]
    fn test_ids_detection() {
        let mut ids = IntrusionDetectionSystem::new();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        let threat = ids.inspect_data(ip, "SELECT * UNION SELECT * FROM passwords");
        assert!(threat.is_some());

        let stats = ids.get_statistics();
        assert_eq!(stats.threats_detected, 1);
    }

    #[test]
    fn test_ips_blocking() {
        let mut ips = IntrusionPreventionSystem::new();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // Should detect and block SQL injection
        let result = ips.inspect_and_prevent(ip, "SELECT * UNION SELECT * FROM passwords");
        assert!(result.is_err());
        assert!(ips.is_blocked(&ip));

        // Subsequent requests should be blocked
        let result = ips.inspect_and_prevent(ip, "SELECT * FROM users");
        assert!(result.is_err());
    }

    #[test]
    fn test_anomaly_detection() {
        let mut ids = IntrusionDetectionSystem::new();

        // Build baseline
        for _ in 0..10 {
            ids.anomaly_baseline.update(100.0, 1024.0, 10.0);
        }

        // Normal traffic should not trigger
        assert!(ids.check_anomaly(100.0, 1024.0, 10.0).is_none());

        // Anomalous traffic should trigger
        assert!(ids.check_anomaly(1000.0, 1024.0, 10.0).is_some());
    }
}
