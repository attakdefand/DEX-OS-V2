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
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::{rngs::OsRng, RngCore};
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
    /// Timestamp of last rotation
    last_rotation: u64,
    /// Key rotation policy
    rotation_policy: RotationPolicy,
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
    /// Key algorithm
    pub algorithm: String,
    /// Key usage
    pub usage: KeyUsage,
}

/// Key usage types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyUsage {
    /// Key used for signing
    Signing,
    /// Key used for encryption
    Encryption,
    /// Key used for both signing and encryption
    Both,
}

/// Key rotation policy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RotationPolicy {
    /// Minimum rotation period in seconds
    pub min_rotation_period: u64,
    /// Maximum rotation period in seconds
    pub max_rotation_period: u64,
    /// Whether to automatically rotate keys
    pub auto_rotate: bool,
    /// Key algorithms allowed
    pub allowed_algorithms: Vec<String>,
}

impl Default for KeyUsage {
    fn default() -> Self {
        KeyUsage::Signing
    }
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            min_rotation_period: 3600,  // 1 hour
            max_rotation_period: 86400, // 1 day
            auto_rotate: true,
            allowed_algorithms: vec!["Ed25519".to_string()],
        }
    }
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
    /// Evidence for the event (for integrity verification)
    pub evidence: Option<Vec<u8>>,
    /// Severity level of the event
    pub severity: SeverityLevel,
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
    /// New event types for comprehensive security auditing
    AccessViolation,
    DataModification,
    KeyRotation,
    CertificateIssued,
    CertificateRevoked,
    PIIDetected,
    PolicyViolation,
    ConfigurationChange,
}

/// Severity levels for security events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SeverityLevel {
    Info,
    Warning,
    Error,
    Critical,
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

    /// Sign data for evidence integrity using Ed25519
    pub fn sign_data(&mut self, data: &[u8], private_key: &[u8], public_key: &[u8]) -> String {
        // Create signing key from private key bytes
        let signing_key = SigningKey::from_bytes(private_key.as_ref().try_into().unwrap());

        // Sign the data
        let signature: Signature = signing_key.sign(data);

        let mut hasher = Sha3_256::new();
        hasher.update(data);
        let data_hash = hasher.finalize().to_vec();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let signature_id = format!("sig_{}", timestamp);

        let digital_signature = DigitalSignature {
            data_hash,
            signature: signature.to_bytes().to_vec(),
            public_key: public_key.to_vec(),
            timestamp,
        };

        self.signatures
            .insert(signature_id.clone(), digital_signature);
        signature_id
    }

    /// Verify a digital signature using Ed25519
    pub fn verify_signature(&self, signature_id: &str, data: &[u8]) -> bool {
        if let Some(stored_signature) = self.signatures.get(signature_id) {
            let mut hasher = Sha3_256::new();
            hasher.update(data);
            let data_hash = hasher.finalize().to_vec();

            // Compare data hashes
            if stored_signature.data_hash != data_hash {
                return false;
            }

            // Verify the cryptographic signature
            let verifying_key = VerifyingKey::from_bytes(
                stored_signature.public_key.as_slice().try_into().unwrap(),
            );
            if verifying_key.is_err() {
                return false;
            }

            let verifying_key = verifying_key.unwrap();
            let signature =
                Signature::from_bytes(stored_signature.signature.as_slice().try_into().unwrap());

            verifying_key.verify(data, &signature).is_ok()
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

    /// Update access control list for classified data
    pub fn update_data_acl(
        &mut self,
        data_id: &str,
        acl: Vec<TraderId>,
    ) -> Result<(), SecurityError> {
        if let Some(classification) = self.data_classification.get_mut(data_id) {
            classification.acl = acl;
            Ok(())
        } else {
            Err(SecurityError::DataNotClassified)
        }
    }

    /// Add a user to the access control list for classified data
    pub fn add_user_to_acl(&mut self, data_id: &str, user: TraderId) -> Result<(), SecurityError> {
        if let Some(classification) = self.data_classification.get_mut(data_id) {
            if !classification.acl.contains(&user) {
                classification.acl.push(user);
            }
            Ok(())
        } else {
            Err(SecurityError::DataNotClassified)
        }
    }

    /// Remove a user from the access control list for classified data
    pub fn remove_user_from_acl(
        &mut self,
        data_id: &str,
        user: &TraderId,
    ) -> Result<(), SecurityError> {
        if let Some(classification) = self.data_classification.get_mut(data_id) {
            classification.acl.retain(|u| u != user);
            Ok(())
        } else {
            Err(SecurityError::DataNotClassified)
        }
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

    /// Check whether a certificate is currently valid
    pub fn is_certificate_valid(&self, cert_id: &str) -> bool {
        self.certificates.is_certificate_valid(cert_id)
    }

    /// Revoke a certificate
    pub fn revoke_certificate(&mut self, cert_id: &str) -> Result<(), SecurityError> {
        self.certificates.revoke_certificate(cert_id)
    }

    /// Rotate keys for a user
    pub fn rotate_keys(&mut self, user_id: &str) -> Result<KeyPair, SecurityError> {
        self.key_rotation.rotate_keys(user_id)
    }

    /// Inspect rotation history for a user (used for audit/reporting).
    pub fn key_rotation_history(&self, user_id: &str) -> Option<&Vec<KeyPair>> {
        self.key_rotation.get_rotation_history(user_id)
    }

    /// Detect PII in text
    pub fn detect_pii(&self, text: &str) -> Vec<PIIDetectionResult> {
        self.pii_detector.detect(text)
    }

    /// Log a security event with evidence
    pub fn log_event(
        &mut self,
        event_type: EventType,
        description: String,
        user: Option<TraderId>,
        data: HashMap<String, String>,
        evidence: Option<Vec<u8>>,
        severity: SeverityLevel,
    ) -> String {
        self.event_logger
            .log(event_type, description, user, data, evidence, severity)
    }

    /// Log a security event with default severity (Info)
    pub fn log_event_simple(
        &mut self,
        event_type: EventType,
        description: String,
        user: Option<TraderId>,
        data: HashMap<String, String>,
    ) -> String {
        self.log_event(
            event_type,
            description,
            user,
            data,
            None,
            SeverityLevel::Info,
        )
    }

    /// Get security events
    pub fn get_events(&self) -> &[SecurityEvent] {
        self.event_logger.get_events()
    }

    /// Get events of a specific type
    pub fn get_events_by_type(&self, event_type: EventType) -> Vec<&SecurityEvent> {
        self.event_logger.get_events_by_type(event_type)
    }

    /// Get events with a specific severity level
    pub fn get_events_by_severity(&self, severity: SeverityLevel) -> Vec<&SecurityEvent> {
        self.event_logger.get_events_by_severity(severity)
    }

    /// Get events for a specific user
    pub fn get_events_for_user(&self, user: &TraderId) -> Vec<&SecurityEvent> {
        self.event_logger.get_events_for_user(user)
    }

    /// Get events within a time range
    pub fn get_events_in_time_range(&self, start: u64, end: u64) -> Vec<&SecurityEvent> {
        self.event_logger.get_events_in_time_range(start, end)
    }

    /// Generate a new Ed25519 key pair for digital signatures
    pub fn generate_key_pair() -> (Vec<u8>, Vec<u8>) {
        let mut csprng = OsRng;
        let mut seed = [0u8; 32];
        csprng.fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();

        (
            signing_key.to_bytes().to_vec(),
            verifying_key.to_bytes().to_vec(),
        )
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DigitalSignature {
    /// Create a new digital signature
    pub fn new(data_hash: Vec<u8>, signature: Vec<u8>, public_key: Vec<u8>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            data_hash,
            signature,
            public_key,
            timestamp,
        }
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            current_keys: HashMap::new(),
            rotation_history: HashMap::new(),
            rotation_period,
            last_rotation: now,
            rotation_policy: RotationPolicy::default(),
        }
    }

    /// Rotate keys for a user with proper cryptographic implementation
    pub fn rotate_keys(&mut self, user_id: &str) -> Result<KeyPair, SecurityError> {
        // Generate new Ed25519 key pair
        let (private_key, public_key) = SecurityManager::generate_key_pair();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let new_keypair = KeyPair {
            public_key,
            private_key,
            created_at: now,
            expires_at: Some(now + self.rotation_period),
            algorithm: "Ed25519".to_string(),
            usage: KeyUsage::Signing,
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

        self.last_rotation = now;

        Ok(new_keypair)
    }

    /// Rotate keys for a user with specific algorithm
    pub fn rotate_keys_with_algorithm(
        &mut self,
        user_id: &str,
        algorithm: &str,
    ) -> Result<KeyPair, SecurityError> {
        let (private_key, public_key) = match algorithm {
            "Ed25519" => SecurityManager::generate_key_pair(),
            _ => return Err(SecurityError::KeyRotationFailed),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let key_usage = match algorithm {
            "Ed25519" => KeyUsage::Signing,
            _ => KeyUsage::Encryption,
        };

        let new_keypair = KeyPair {
            public_key,
            private_key,
            created_at: now,
            expires_at: Some(now + self.rotation_period),
            algorithm: algorithm.to_string(),
            usage: key_usage,
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

        self.last_rotation = now;

        Ok(new_keypair)
    }

    /// Automatic key rotation based on policy
    pub fn auto_rotate_keys(&mut self) -> Result<Vec<(String, KeyPair)>, SecurityError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if it's time for rotation based on policy
        if now < self.last_rotation + self.rotation_period {
            return Ok(Vec::new());
        }

        let mut rotated_keys = Vec::new();

        // Rotate all keys that need rotation
        let user_ids: Vec<String> = self.current_keys.keys().cloned().collect();

        for user_id in user_ids {
            if let Ok(new_key) = self.rotate_keys(&user_id) {
                rotated_keys.push((user_id, new_key));
            }
        }

        Ok(rotated_keys)
    }

    /// Get current key for a user
    pub fn get_current_key(&self, user_id: &str) -> Option<&KeyPair> {
        self.current_keys.get(user_id)
    }

    /// Get key rotation history for a user
    pub fn get_rotation_history(&self, user_id: &str) -> Option<&Vec<KeyPair>> {
        self.rotation_history.get(user_id)
    }

    /// Check if key rotation is needed for a user
    pub fn is_rotation_needed(&self, user_id: &str) -> bool {
        if let Some(keypair) = self.current_keys.get(user_id) {
            if let Some(expires_at) = keypair.expires_at {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return now >= expires_at;
            }
        }
        false
    }

    /// Set rotation policy
    pub fn set_rotation_policy(&mut self, policy: RotationPolicy) {
        self.rotation_policy = policy;
    }

    /// Get rotation policy
    pub fn get_rotation_policy(&self) -> &RotationPolicy {
        &self.rotation_policy
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
            PIIPattern {
                name: "CreditCard".to_string(),
                pattern: r"\b(?:\d{4}[-\s]?){3}\d{4}\b|\b\d{16}\b".to_string(),
                regex: None,
            },
            PIIPattern {
                name: "IPAddress".to_string(),
                pattern: r"\b(?:\d{1,3}\.){3}\d{1,3}\b".to_string(),
                regex: None,
            },
            PIIPattern {
                name: "DOB".to_string(),
                pattern: r"\b(0[1-9]|1[0-2])[-/.](0[1-9]|[12]\d|3[01])[-/.](19|20)\d{2}\b"
                    .to_string(),
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

    /// Log a security event with all metadata
    pub fn log(
        &mut self,
        event_type: EventType,
        description: String,
        user: Option<TraderId>,
        data: HashMap<String, String>,
        evidence: Option<Vec<u8>>,
        severity: SeverityLevel,
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
            evidence,
            severity,
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

    /// Get events with a specific severity level
    pub fn get_events_by_severity(&self, severity: SeverityLevel) -> Vec<&SecurityEvent> {
        self.events
            .iter()
            .filter(|event| event.severity == severity)
            .collect()
    }

    /// Get events for a specific user
    pub fn get_events_for_user(&self, user: &TraderId) -> Vec<&SecurityEvent> {
        self.events
            .iter()
            .filter(|event| event.user.as_ref() == Some(user))
            .collect()
    }

    /// Get events within a time range
    pub fn get_events_in_time_range(&self, start: u64, end: u64) -> Vec<&SecurityEvent> {
        self.events
            .iter()
            .filter(|event| event.timestamp >= start && event.timestamp <= end)
            .collect()
    }

    /// Export events in a structured format for analysis
    pub fn export_events(&self) -> String {
        serde_json::to_string_pretty(&self.events).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get event statistics
    pub fn get_event_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();

        // Count by event type
        for event in &self.events {
            let count = stats
                .entry(format!("type_{:?}", event.event_type))
                .or_insert(0);
            *count += 1;

            let count = stats
                .entry(format!("severity_{:?}", event.severity))
                .or_insert(0);
            *count += 1;
        }

        stats
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
    #[error("Data not classified")]
    DataNotClassified,
    #[error("Invalid key algorithm")]
    InvalidKeyAlgorithm,
    #[error("Key expired")]
    KeyExpired,
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
    fn test_digital_signature() {
        let mut manager = SecurityManager::new();

        // Generate a key pair
        let (private_key, public_key) = SecurityManager::generate_key_pair();

        // Sign some data
        let data = b"test data for signing";
        let signature_id = manager.sign_data(data, &private_key, &public_key);

        // Verify the signature
        assert!(manager.verify_signature(&signature_id, data));

        // Verify with incorrect data should fail
        assert!(!manager.verify_signature(&signature_id, b"different data"));
    }

    #[test]
    fn test_sign_and_verify_data() {
        let mut manager = SecurityManager::new();
        let data = b"important data to sign";
        let (private_key, public_key) = SecurityManager::generate_key_pair();

        let signature_id = manager.sign_data(data, &private_key, &public_key);
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
    fn test_data_acl_management() {
        let mut manager = SecurityManager::new();
        let owner = "owner".to_string();
        let user1 = "user1".to_string();
        let user2 = "user2".to_string();
        let user3 = "user3".to_string();

        // Classify data
        manager.classify_data(
            "data1".to_string(),
            ClassificationLevel::Confidential,
            owner.clone(),
            vec![user1.clone()],
        );

        // Add user to ACL
        assert!(manager.add_user_to_acl("data1", user2.clone()).is_ok());
        assert!(manager.check_data_access("data1", &user2));

        // Remove user from ACL
        assert!(manager.remove_user_from_acl("data1", &user1).is_ok());
        assert!(!manager.check_data_access("data1", &user1));

        // Update entire ACL
        assert!(manager
            .update_data_acl("data1", vec![user3.clone()])
            .is_ok());
        assert!(manager.check_data_access("data1", &user3));
        assert!(!manager.check_data_access("data1", &user2));

        // Try to modify non-existent data
        assert!(manager
            .add_user_to_acl("nonexistent", user1.clone())
            .is_err());
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

        // Test algorithm-specific rotation
        let result = manager
            .key_rotation
            .rotate_keys_with_algorithm(user_id, "Ed25519");
        assert!(result.is_ok());

        // Test rotation needed
        assert!(!manager.key_rotation.is_rotation_needed(user_id));
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
            None,
            SeverityLevel::Info,
        );

        assert!(!event_id.is_empty());

        // Check events
        let events = manager.get_events();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.event_type, EventType::LoginAttempt);
        assert_eq!(event.user, Some(user.clone()));
        assert_eq!(event.data.len(), 2);
        assert_eq!(event.severity, SeverityLevel::Info);

        // Check events by type
        let login_events = manager.get_events_by_type(EventType::LoginAttempt);
        assert_eq!(login_events.len(), 1);

        // Check events for user
        let user_events = manager.event_logger.get_events_for_user(&user);
        assert_eq!(user_events.len(), 1);

        // Check events by severity
        let info_events = manager.get_events_by_severity(SeverityLevel::Info);
        assert_eq!(info_events.len(), 1);
    }

    #[test]
    fn test_comprehensive_event_logging() {
        let mut manager = SecurityManager::new();
        let user = "user1".to_string();

        // Log different types of events
        let mut data = HashMap::new();
        data.insert("resource".to_string(), "sensitive_data".to_string());

        // Log access violation
        manager.log_event(
            EventType::AccessViolation,
            "Unauthorized access attempt".to_string(),
            Some(user.clone()),
            data.clone(),
            Some(b"evidence_data".to_vec()),
            SeverityLevel::Error,
        );

        // Log key rotation
        manager.log_event(
            EventType::KeyRotation,
            "User key rotated".to_string(),
            Some(user.clone()),
            HashMap::new(),
            None,
            SeverityLevel::Info,
        );

        // Check events
        let events = manager.get_events();
        assert_eq!(events.len(), 2);

        // Check events by severity
        let error_events = manager.get_events_by_severity(SeverityLevel::Error);
        assert_eq!(error_events.len(), 1);

        let info_events = manager.get_events_by_severity(SeverityLevel::Info);
        assert_eq!(info_events.len(), 1);

        // Check events by type
        let access_violation_events = manager.get_events_by_type(EventType::AccessViolation);
        assert_eq!(access_violation_events.len(), 1);

        let key_rotation_events = manager.get_events_by_type(EventType::KeyRotation);
        assert_eq!(key_rotation_events.len(), 1);
    }
}
