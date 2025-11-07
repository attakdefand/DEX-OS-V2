//! Security features implementation for the DEX-OS core engine
//!
//! This module implements various Priority 3 security features from DEX-OS-V2.csv:
//! - Security,Security,Security,Digital Signatures,Evidence Integrity,Medium
//! - Security,Security,Security,Hash Map,Data Classification,Medium
//! - Security,Security,Security,B+ Tree,Certificate Management,Medium
//! - Security,Security,Security,Hash Map,Key Rotation,Medium
//! - Security,Security,Security,Regular Expressions,PII Detection,Medium
//! - Security,Orderbook,Orderbook,Event Logging,Security Auditing,Medium

use crate::types::{TokenId, TraderId};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use thiserror::Error;

/// Security manager for the DEX-OS core engine
#[derive(Debug, Clone)]
pub struct SecurityManager {
    /// Digital signatures for evidence integrity
    signatures: HashMap<String, DigitalSignature>,
    /// Data classification system
    data_classification: HashMap<String, DataClassification>,
    /// Certificate management using B+ tree concept
    certificates: CertificateManager,
    /// Key rotation system
    key_rotation: KeyRotationManager,
    /// PII detection system
    pii_detector: PIIDetector,
    /// Event logging for security auditing
    event_logger: EventLogger,
}

/// Digital signature for evidence integrity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigitalSignature {
    /// The data that was signed
    pub data_hash: Vec<u8>,
    /// The signature
    pub signature: Vec<u8>,
    /// Public key used for verification
    pub public_key: Vec<u8>,
    /// Timestamp of signing
    pub timestamp: u64,
}

/// Data classification levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClassificationLevel {
    Public,
    Internal,
    Confidential,
    Secret,
    TopSecret,
}

/// Data classification information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataClassification {
    /// Classification level
    pub level: ClassificationLevel,
    /// Owner of the data
    pub owner: TraderId,
    /// Access control list
    pub acl: Vec<TraderId>,
    /// Timestamp of classification
    pub timestamp: u64,
}

/// Certificate manager (simplified B+ tree implementation)
#[derive(Debug, Clone)]
pub struct CertificateManager {
    /// Certificates stored in a hash map for simplicity
    /// In a full implementation, this would be a B+ tree
    certificates: HashMap<String, Certificate>,
}

/// Digital certificate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Certificate {
    /// Certificate identifier
    pub id: String,
    /// Certificate data
    pub data: Vec<u8>,
    /// Issuer of the certificate
    pub issuer: String,
    /// Validity period
    pub valid_from: u64,
    pub valid_to: u64,
    /// Signature of the certificate
    pub signature: Vec<u8>,
    /// Whether the certificate is revoked
    pub revoked: bool,
}

/// Key rotation manager
#[derive(Debug, Clone)]
pub struct KeyRotationManager {
    /// Current keys
    current_keys: HashMap<String, KeyPair>,
    /// Key rotation history
    rotation_history: HashMap<String, Vec<KeyPair>>,
    /// Rotation period in seconds
    rotation_period: u64,
}

/// Key pair for cryptography
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyPair {
    /// Public key
    pub public_key: Vec<u8>,
    /// Private key (encrypted)
    pub private_key: Vec<u8>,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp
    pub expires_at: Option<u64>,
}

/// PII (Personally Identifiable Information) detector
#[derive(Debug, Clone)]
pub struct PIIDetector {
    /// Patterns for detecting PII
    patterns: Vec<PIIPattern>,
}

/// PII pattern for detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PIIPattern {
    /// Name of the pattern
    pub name: String,
    /// Regular expression pattern
    pub pattern: String,
    /// Compiled regex
    #[serde(skip)]
    pub regex: Option<Regex>,
}

/// Event logger for security auditing
#[derive(Debug, Clone)]
pub struct EventLogger {
    /// Security events
    events: Vec<SecurityEvent>,
    /// Maximum number of events to store
    max_events: usize,
}

/// Security event for auditing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityEvent {
    /// Event identifier
    pub id: String,
    /// Event type
    pub event_type: EventType,
    /// Description of the event
    pub description: String,
    /// User associated with the event
    pub user: Option<TraderId>,
    /// Timestamp of the event
    pub timestamp: u64,
    /// Additional data
    pub data: HashMap<String, String>,
}

/// Types of security events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventType {
    LoginAttempt,
    Transaction,
    GovernanceProposal,
    SecurityAlert,
    SystemEvent,
    AuditTrail,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
            data_classification: HashMap::new(),
            certificates: CertificateManager::new(),
            key_rotation: KeyRotationManager::new(86400), // Rotate daily
            pii_detector: PIIDetector::new(),
            event_logger: EventLogger::new(10000), // Store up to 10,000 events
        }
    }

    /// Sign data for evidence integrity
    pub fn sign_data(&mut self, data: &[u8], private_key: &[u8], public_key: &[u8]) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        let data_hash = hasher.finalize().to_vec();

        // In a real implementation, this would use actual cryptographic signatures
        // For now, we'll simulate with a hash of the data and private key
        let mut signature_hasher = Sha3_256::new();
        signature_hasher.update(&data_hash);
        signature_hasher.update(private_key);
        let signature = signature_hasher.finalize().to_vec();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature_id = format!("sig_{}", timestamp);

        let digital_signature = DigitalSignature {
            data_hash,
            signature,
            public_key: public_key.to_vec(),
            timestamp,
        };

        self.signatures
            .insert(signature_id.clone(), digital_signature);
        signature_id
    }

    /// Verify a digital signature
    pub fn verify_signature(&self, signature_id: &str, data: &[u8]) -> bool {
        if let Some(stored_signature) = self.signatures.get(signature_id) {
            let mut hasher = Sha3_256::new();
            hasher.update(data);
            let data_hash = hasher.finalize().to_vec();

            // Compare data hashes
            if stored_signature.data_hash != data_hash {
                return false;
            }

            // In a real implementation, this would verify the cryptographic signature
            // For now, we'll simulate verification
            let mut signature_hasher = Sha3_256::new();
            signature_hasher.update(&data_hash);
            signature_hasher.update(&stored_signature.signature); // Using signature as private key for simulation
            let expected_signature = signature_hasher.finalize().to_vec();

            stored_signature.signature == expected_signature
        } else {
            false
        }
    }

    /// Classify data with a security level
    pub fn classify_data(
        &mut self,
        data_id: String,
        level: ClassificationLevel,
        owner: TraderId,
        acl: Vec<TraderId>,
    ) {
        let classification = DataClassification {
            level,
            owner,
            acl,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.data_classification.insert(data_id, classification);
    }

    /// Check if a user has access to classified data
    pub fn check_data_access(&self, data_id: &str, user: &TraderId) -> bool {
        if let Some(classification) = self.data_classification.get(data_id) {
            // Owner always has access
            if &classification.owner == user {
                return true;
            }

            // Check ACL
            classification.acl.contains(user)
        } else {
            // Data not classified, assume public access
            true
        }
    }

    /// Add a certificate
    pub fn add_certificate(&mut self, certificate: Certificate) -> Result<(), SecurityError> {
        self.certificates.add_certificate(certificate)
    }

    /// Get a certificate by ID
    pub fn get_certificate(&self, cert_id: &str) -> Option<&Certificate> {
        self.certificates.get_certificate(cert_id)
    }

    /// Revoke a certificate
    pub fn revoke_certificate(&mut self, cert_id: &str) -> Result<(), SecurityError> {
        self.certificates.revoke_certificate(cert_id)
    }

    /// Rotate keys for a user
    pub fn rotate_keys(&mut self, user_id: &str) -> Result<KeyPair, SecurityError> {
        self.key_rotation.rotate_keys(user_id)
    }

    /// Detect PII in text
    pub fn detect_pii(&self, text: &str) -> Vec<PIIDetectionResult> {
        self.pii_detector.detect(text)
    }

    /// Log a security event
    pub fn log_event(
        &mut self,
        event_type: EventType,
        description: String,
        user: Option<TraderId>,
        data: HashMap<String, String>,
    ) -> String {
        self.event_logger.log(event_type, description, user, data)
    }

    /// Get security events
    pub fn get_events(&self) -> &[SecurityEvent] {
        self.event_logger.get_events()
    }

    /// Get events of a specific type
    pub fn get_events_by_type(&self, event_type: EventType) -> Vec<&SecurityEvent> {
        self.event_logger.get_events_by_type(event_type)
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CertificateManager {
    /// Create a new certificate manager
    pub fn new() -> Self {
        Self {
            certificates: HashMap::new(),
        }
    }

    /// Add a certificate
    pub fn add_certificate(&mut self, certificate: Certificate) -> Result<(), SecurityError> {
        if self.certificates.contains_key(&certificate.id) {
            return Err(SecurityError::CertificateAlreadyExists);
        }

        self.certificates
            .insert(certificate.id.clone(), certificate);
        Ok(())
    }

    /// Get a certificate by ID
    pub fn get_certificate(&self, cert_id: &str) -> Option<&Certificate> {
        self.certificates.get(cert_id)
    }

    /// Revoke a certificate
    pub fn revoke_certificate(&mut self, cert_id: &str) -> Result<(), SecurityError> {
        if let Some(certificate) = self.certificates.get_mut(cert_id) {
            if certificate.revoked {
                return Err(SecurityError::CertificateAlreadyRevoked);
            }

            certificate.revoked = true;
            Ok(())
        } else {
            Err(SecurityError::CertificateNotFound)
        }
    }

    /// Check if a certificate is valid
    pub fn is_certificate_valid(&self, cert_id: &str) -> bool {
        if let Some(certificate) = self.certificates.get(cert_id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            !certificate.revoked && now >= certificate.valid_from && now <= certificate.valid_to
        } else {
            false
        }
    }
}

impl Default for CertificateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyRotationManager {
    /// Create a new key rotation manager
    pub fn new(rotation_period: u64) -> Self {
        Self {
            current_keys: HashMap::new(),
            rotation_history: HashMap::new(),
            rotation_period,
        }
    }

    /// Rotate keys for a user
    pub fn rotate_keys(&mut self, user_id: &str) -> Result<KeyPair, SecurityError> {
        // Generate new key pair (simulated)
        let public_key = vec![1u8; 32];
        let private_key = vec![2u8; 32];

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let new_keypair = KeyPair {
            public_key,
            private_key,
            created_at: now,
            expires_at: Some(now + self.rotation_period),
        };

        // Store old key in history if it exists
        if let Some(old_keypair) = self.current_keys.remove(user_id) {
            self.rotation_history
                .entry(user_id.to_string())
                .or_insert_with(Vec::new)
                .push(old_keypair);
        }

        // Store new key as current
        self.current_keys
            .insert(user_id.to_string(), new_keypair.clone());

        Ok(new_keypair)
    }

    /// Get current key for a user
    pub fn get_current_key(&self, user_id: &str) -> Option<&KeyPair> {
        self.current_keys.get(user_id)
    }

    /// Get key rotation history for a user
    pub fn get_rotation_history(&self, user_id: &str) -> Option<&Vec<KeyPair>> {
        self.rotation_history.get(user_id)
    }
}

impl Default for KeyRotationManager {
    fn default() -> Self {
        Self::new(86400) // Daily rotation
    }
}

impl PIIDetector {
    /// Create a new PII detector with default patterns
    pub fn new() -> Self {
        let patterns = vec![
            PIIPattern {
                name: "Email".to_string(),
                pattern: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),
                regex: None,
            },
            PIIPattern {
                name: "Phone".to_string(),
                pattern: r"(\+\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}".to_string(),
                regex: None,
            },
            PIIPattern {
                name: "SSN".to_string(),
                pattern: r"\d{3}-?\d{2}-?\d{4}".to_string(),
                regex: None,
            },
        ];

        Self { patterns }
    }

    /// Detect PII in text
    pub fn detect(&self, text: &str) -> Vec<PIIDetectionResult> {
        let mut results = Vec::new();

        for pattern in &self.patterns {
            // Compile regex if not already compiled
            let regex = Regex::new(&pattern.pattern)
                .unwrap_or_else(|_| panic!("Invalid regex pattern: {}", pattern.pattern));

            // Find matches
            for mat in regex.find_iter(text) {
                results.push(PIIDetectionResult {
                    pattern_name: pattern.name.clone(),
                    matched_text: mat.as_str().to_string(),
                    start: mat.start(),
                    end: mat.end(),
                });
            }
        }

        results
    }
}

impl Default for PIIDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of PII detection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PIIDetectionResult {
    /// Name of the pattern that matched
    pub pattern_name: String,
    /// The text that matched
    pub matched_text: String,
    /// Start position of match
    pub start: usize,
    /// End position of match
    pub end: usize,
}

impl EventLogger {
    /// Create a new event logger
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
        }
    }

    /// Log a security event
    pub fn log(
        &mut self,
        event_type: EventType,
        description: String,
        user: Option<TraderId>,
        data: HashMap<String, String>,
    ) -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let event_id = format!("event_{}", timestamp);

        let event = SecurityEvent {
            id: event_id.clone(),
            event_type,
            description,
            user,
            timestamp,
            data,
        };

        self.events.push(event);

        // Trim events if we exceed max_events
        if self.events.len() > self.max_events {
            self.events.drain(0..(self.events.len() - self.max_events));
        }

        event_id
    }

    /// Get all security events
    pub fn get_events(&self) -> &[SecurityEvent] {
        &self.events
    }

    /// Get events of a specific type
    pub fn get_events_by_type(&self, event_type: EventType) -> Vec<&SecurityEvent> {
        self.events
            .iter()
            .filter(|event| event.event_type == event_type)
            .collect()
    }

    /// Get events for a specific user
    pub fn get_events_for_user(&self, user: &TraderId) -> Vec<&SecurityEvent> {
        self.events
            .iter()
            .filter(|event| event.user.as_ref() == Some(user))
            .collect()
    }
}

impl Default for EventLogger {
    fn default() -> Self {
        Self::new(1000) // Default to 1000 events
    }
}

/// Errors that can occur during security operations
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Certificate already exists")]
    CertificateAlreadyExists,
    #[error("Certificate not found")]
    CertificateNotFound,
    #[error("Certificate already revoked")]
    CertificateAlreadyRevoked,
    #[error("Key rotation failed")]
    KeyRotationFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_manager_creation() {
        let manager = SecurityManager::new();
        assert!(manager.signatures.is_empty());
        assert!(manager.data_classification.is_empty());
    }

    #[test]
    fn test_sign_and_verify_data() {
        let mut manager = SecurityManager::new();
        let data = b"important data to sign";
        let private_key = b"private_key";
        let public_key = b"public_key";

        let signature_id = manager.sign_data(data, private_key, public_key);
        assert!(!signature_id.is_empty());

        // Verify correct data
        assert!(manager.verify_signature(&signature_id, data));

        // Verify incorrect data
        assert!(!manager.verify_signature(&signature_id, b"different data"));
    }

    #[test]
    fn test_data_classification() {
        let mut manager = SecurityManager::new();
        let owner = "owner".to_string();
        let user1 = "user1".to_string();
        let user2 = "user2".to_string();

        // Classify data
        manager.classify_data(
            "data1".to_string(),
            ClassificationLevel::Confidential,
            owner.clone(),
            vec![user1.clone()],
        );

        // Check access
        assert!(manager.check_data_access("data1", &owner)); // Owner has access
        assert!(manager.check_data_access("data1", &user1)); // ACL user has access
        assert!(!manager.check_data_access("data1", &user2)); // Other user doesn't have access

        // Public data (not classified) should be accessible
        assert!(manager.check_data_access("public_data", &user1));
    }

    #[test]
    fn test_certificate_management() {
        let mut manager = SecurityManager::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let certificate = Certificate {
            id: "cert1".to_string(),
            data: vec![1, 2, 3, 4],
            issuer: "CA".to_string(),
            valid_from: now - 1000,
            valid_to: now + 1000,
            signature: vec![5, 6, 7, 8],
            revoked: false,
        };

        // Add certificate
        assert!(manager.add_certificate(certificate.clone()).is_ok());

        // Get certificate
        let retrieved = manager.get_certificate("cert1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &certificate);

        // Revoke certificate
        assert!(manager.revoke_certificate("cert1").is_ok());

        // Try to revoke again
        assert!(manager.revoke_certificate("cert1").is_err());
    }

    #[test]
    fn test_key_rotation() {
        let mut manager = SecurityManager::new();
        let user_id = "user1";

        // Rotate keys
        let result = manager.rotate_keys(user_id);
        assert!(result.is_ok());

        let keypair1 = result.unwrap();

        // Rotate keys again
        let result = manager.rotate_keys(user_id);
        assert!(result.is_ok());

        let keypair2 = result.unwrap();
        assert_ne!(keypair1, keypair2);

        // Check current key
        let current = manager.key_rotation.get_current_key(user_id);
        assert!(current.is_some());
        assert_eq!(current.unwrap(), &keypair2);
    }

    #[test]
    fn test_pii_detection() {
        let manager = SecurityManager::new();

        let text = "Contact me at john.doe@example.com or call 555-123-4567";
        let results = manager.detect_pii(text);

        assert_eq!(results.len(), 2);

        // Check email detection
        let email_result = &results[0];
        assert_eq!(email_result.pattern_name, "Email");
        assert_eq!(email_result.matched_text, "john.doe@example.com");

        // Check phone detection
        let phone_result = &results[1];
        assert_eq!(phone_result.pattern_name, "Phone");
        assert_eq!(phone_result.matched_text, "555-123-4567");
    }

    #[test]
    fn test_event_logging() {
        let mut manager = SecurityManager::new();
        let user = "user1".to_string();

        // Log an event
        let mut data = HashMap::new();
        data.insert("ip".to_string(), "192.168.1.1".to_string());
        data.insert("location".to_string(), "New York".to_string());

        let event_id = manager.log_event(
            EventType::LoginAttempt,
            "User login attempt".to_string(),
            Some(user.clone()),
            data,
        );

        assert!(!event_id.is_empty());

        // Check events
        let events = manager.get_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type, EventType::LoginAttempt);
        assert_eq!(event.user, Some(user.clone()));
        assert_eq!(event.data.len(), 2);

        // Check events by type
        let login_events = manager.get_events_by_type(EventType::LoginAttempt);
        assert_eq!(login_events.len(), 1);

        // Check events for user
        let user_events = manager.event_logger.get_events_for_user(&user);
        assert_eq!(user_events.len(), 1);
    }
}
