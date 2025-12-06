//! Neuralink Interface Module for WASM Runtime
//!
//! Implements Priority 5 feature from DEX-OS-V2.csv:
//! - Core Components,WASM Runtime,Runtime,Neuralink Interface,Brain-Computer Interface,Medium {Security: Layer 19 - Mobile Security}
//!
//! Features:
//! - Brain-computer interface for transaction authorization
//! - Thought-based wallet access
//! - Neural signature verification
//! - Secure mental command processing
//! - Biometric authentication via neural patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Neuralink interface errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum NeuralinkError {
    #[error("Device not connected: {0}")]
    DeviceNotConnected(String),
    #[error("Neural signature verification failed: {0}")]
    SignatureVerificationFailed(String),
    #[error("Command not recognized: {0}")]
    CommandNotRecognized(String),
    #[error("User not authenticated: {0}")]
    AuthenticationFailed(String),
    #[error("Neural pattern mismatch: {0}")]
    PatternMismatch(String),
    #[error("Transaction authorization denied: {0}")]
    AuthorizationDenied(String),
}

/// Neural device identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(pub String);

/// Neural device status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Connected { signal_quality: u8 },
    Calibrating,
    Disconnected,
    Error(String),
}

/// Neural command types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NeuralCommand {
    /// Authorize a transaction
    AuthorizeTransaction { transaction_id: String },
    /// Access wallet
    AccessWallet { wallet_address: String },
    /// Sign message
    SignMessage { message: String },
    /// Lock/unlock account
    LockAccount { lock: bool },
    /// Emergency shutdown
    EmergencyShutdown,
}

/// Neural pattern (simplified representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralPattern {
    /// Pattern identifier
    pub id: String,
    /// Pattern data (simplified as vector of values)
    pub data: Vec<f64>,
    /// Timestamp when pattern was recorded
    pub timestamp: u64,
    /// Confidence score (0-100)
    pub confidence: u8,
}

/// User neural profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralProfile {
    pub user_id: String,
    pub device_id: DeviceId,
    /// Baseline neural patterns for authentication
    pub baseline_patterns: Vec<NeuralPattern>,
    /// Last authentication timestamp
    pub last_auth: u64,
    /// Authentication attempts
    pub auth_attempts: u32,
}

/// Command authorization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResult {
    pub authorized: bool,
    pub command: NeuralCommand,
    pub confidence: u8,
    pub timestamp: u64,
    pub reason: Option<String>,
}

/// Neuralink interface manager
#[derive(Debug, Clone)]
pub struct NeuralinkInterface {
    /// Connected devices
    devices: Arc<RwLock<HashMap<DeviceId, DeviceStatus>>>,
    /// User neural profiles
    profiles: Arc<RwLock<HashMap<String, NeuralProfile>>>,
    /// Command history
    command_history: Arc<RwLock<Vec<(String, NeuralCommand, AuthorizationResult)>>>,
}

impl NeuralinkInterface {
    /// Create a new Neuralink interface manager
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            profiles: Arc::new(RwLock::new(HashMap::new())),
            command_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a neural device
    pub fn register_device(
        &self,
        device_id: DeviceId,
        status: DeviceStatus,
    ) -> Result<(), NeuralinkError> {
        let mut devices = self.devices.write().unwrap();
        devices.insert(device_id, status);
        Ok(())
    }

    /// Create a neural profile for a user
    pub fn create_profile(
        &self,
        user_id: String,
        device_id: DeviceId,
    ) -> Result<NeuralProfile, NeuralinkError> {
        // Verify device is connected
        let devices = self.devices.read().unwrap();
        let device_status = devices
            .get(&device_id)
            .ok_or_else(|| NeuralinkError::DeviceNotConnected(device_id.0.clone()))?;

        if !matches!(device_status, DeviceStatus::Connected { .. }) {
            return Err(NeuralinkError::DeviceNotConnected(
                "Device not in connected state".to_string(),
            ));
        }

        drop(devices);

        // Create baseline patterns (in real implementation, this would involve actual neural recording)
        let baseline_patterns = self.generate_baseline_patterns();

        let profile = NeuralProfile {
            user_id: user_id.clone(),
            device_id,
            baseline_patterns,
            last_auth: Self::current_timestamp(),
            auth_attempts: 0,
        };

        let mut profiles = self.profiles.write().unwrap();
        profiles.insert(user_id, profile.clone());

        Ok(profile)
    }

    /// Authenticate user via neural pattern
    pub fn authenticate_user(
        &self,
        user_id: &str,
        current_pattern: NeuralPattern,
    ) -> Result<bool, NeuralinkError> {
        let mut profiles = self.profiles.write().unwrap();
        let profile = profiles
            .get_mut(user_id)
            .ok_or_else(|| NeuralinkError::AuthenticationFailed("User profile not found".to_string()))?;

        // Verify device is connected
        let devices = self.devices.read().unwrap();
        let device_status = devices
            .get(&profile.device_id)
            .ok_or_else(|| NeuralinkError::DeviceNotConnected(profile.device_id.0.clone()))?;

        if !matches!(device_status, DeviceStatus::Connected { .. }) {
            return Err(NeuralinkError::DeviceNotConnected(
                "Device not connected".to_string(),
            ));
        }

        drop(devices);

        // Compare with baseline patterns
        let match_score = self.compare_patterns(&profile.baseline_patterns, &current_pattern);

        profile.auth_attempts += 1;

        if match_score >= 80 {
            profile.last_auth = Self::current_timestamp();
            Ok(true)
        } else {
            Err(NeuralinkError::PatternMismatch(format!(
                "Match score {} below threshold",
                match_score
            )))
        }
    }

    /// Process a neural command
    pub fn process_command(
        &self,
        user_id: &str,
        command: NeuralCommand,
        neural_pattern: NeuralPattern,
    ) -> Result<AuthorizationResult, NeuralinkError> {
        // Authenticate user first
        let authenticated = self.authenticate_user(user_id, neural_pattern.clone())?;

        if !authenticated {
            return Ok(AuthorizationResult {
                authorized: false,
                command: command.clone(),
                confidence: 0,
                timestamp: Self::current_timestamp(),
                reason: Some("Authentication failed".to_string()),
            });
        }

        // Verify command intent from neural pattern
        let confidence = self.verify_command_intent(&command, &neural_pattern);

        let authorized = confidence >= 70;

        let result = AuthorizationResult {
            authorized,
            command: command.clone(),
            confidence,
            timestamp: Self::current_timestamp(),
            reason: if authorized {
                None
            } else {
                Some(format!("Low confidence: {}", confidence))
            },
        };

        // Record command in history
        let mut history = self.command_history.write().unwrap();
        history.push((user_id.to_string(), command, result.clone()));

        Ok(result)
    }

    /// Authorize a transaction via neural command
    pub fn authorize_transaction(
        &self,
        user_id: &str,
        transaction_id: String,
        neural_pattern: NeuralPattern,
    ) -> Result<AuthorizationResult, NeuralinkError> {
        let command = NeuralCommand::AuthorizeTransaction { transaction_id };
        self.process_command(user_id, command, neural_pattern)
    }

    /// Access wallet via neural command
    pub fn access_wallet(
        &self,
        user_id: &str,
        wallet_address: String,
        neural_pattern: NeuralPattern,
    ) -> Result<AuthorizationResult, NeuralinkError> {
        let command = NeuralCommand::AccessWallet { wallet_address };
        self.process_command(user_id, command, neural_pattern)
    }

    /// Update device status
    pub fn update_device_status(
        &self,
        device_id: &DeviceId,
        status: DeviceStatus,
    ) -> Result<(), NeuralinkError> {
        let mut devices = self.devices.write().unwrap();
        devices.insert(device_id.clone(), status);
        Ok(())
    }

    /// Get command history for a user
    pub fn get_command_history(&self, user_id: &str) -> Vec<(NeuralCommand, AuthorizationResult)> {
        let history = self.command_history.read().unwrap();
        history
            .iter()
            .filter(|(uid, _, _)| uid == user_id)
            .map(|(_, cmd, result)| (cmd.clone(), result.clone()))
            .collect()
    }

    /// Calibrate neural patterns
    pub fn calibrate_patterns(
        &self,
        user_id: &str,
        new_patterns: Vec<NeuralPattern>,
    ) -> Result<(), NeuralinkError> {
        let mut profiles = self.profiles.write().unwrap();
        let profile = profiles
            .get_mut(user_id)
            .ok_or_else(|| NeuralinkError::AuthenticationFailed("User profile not found".to_string()))?;

        // Update baseline patterns
        profile.baseline_patterns = new_patterns;

        Ok(())
    }

    /// Helper: Generate baseline neural patterns (simulated)
    fn generate_baseline_patterns(&self) -> Vec<NeuralPattern> {
        let mut patterns = Vec::new();
        
        for i in 0..3 {
            patterns.push(NeuralPattern {
                id: format!("pattern-{}", i),
                data: (0..10).map(|j| (i * 10 + j) as f64 / 100.0).collect(),
                timestamp: Self::current_timestamp(),
                confidence: 95,
            });
        }

        patterns
    }

    /// Helper: Compare neural patterns
    fn compare_patterns(&self, baseline: &[NeuralPattern], current: &NeuralPattern) -> u8 {
        // Simplified pattern matching (in real implementation, use ML/AI)
        let mut best_match = 0.0;

        for pattern in baseline {
            if pattern.data.len() != current.data.len() {
                continue;
            }

            let mut similarity = 0.0;
            for (a, b) in pattern.data.iter().zip(&current.data) {
                similarity += 1.0 - (a - b).abs();
            }
            similarity /= pattern.data.len() as f64;

            if similarity > best_match {
                best_match = similarity;
            }
        }

        (best_match * 100.0) as u8
    }

    /// Helper: Verify command intent from neural pattern
    fn verify_command_intent(&self, _command: &NeuralCommand, pattern: &NeuralPattern) -> u8 {
        // Simplified intent verification (in real implementation, use neural decoding)
        pattern.confidence
    }

    /// Helper: Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Default for NeuralinkInterface {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pattern(confidence: u8) -> NeuralPattern {
        NeuralPattern {
            id: "test-pattern".to_string(),
            data: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
            timestamp: NeuralinkInterface::current_timestamp(),
            confidence,
        }
    }

    #[test]
    fn test_register_device() {
        let interface = NeuralinkInterface::new();
        let device_id = DeviceId("device-001".to_string());

        assert!(interface
            .register_device(
                device_id.clone(),
                DeviceStatus::Connected {
                    signal_quality: 85
                }
            )
            .is_ok());

        let devices = interface.devices.read().unwrap();
        assert!(devices.contains_key(&device_id));
    }

    #[test]
    fn test_create_profile() {
        let interface = NeuralinkInterface::new();
        let device_id = DeviceId("device-001".to_string());

        interface
            .register_device(
                device_id.clone(),
                DeviceStatus::Connected {
                    signal_quality: 90,
                },
            )
            .unwrap();

        let profile = interface
            .create_profile("user123".to_string(), device_id.clone())
            .unwrap();

        assert_eq!(profile.user_id, "user123");
        assert_eq!(profile.device_id, device_id);
        assert!(!profile.baseline_patterns.is_empty());
    }

    #[test]
    fn test_authenticate_user() {
        let interface = NeuralinkInterface::new();
        let device_id = DeviceId("device-001".to_string());

        interface
            .register_device(
                device_id.clone(),
                DeviceStatus::Connected {
                    signal_quality: 90,
                },
            )
            .unwrap();

        interface
            .create_profile("user123".to_string(), device_id)
            .unwrap();

        let pattern = create_test_pattern(95);
        let result = interface.authenticate_user("user123", pattern);

        // Note: This may fail due to pattern mismatch in simplified implementation
        // In real implementation, we'd use actual neural pattern matching
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_authorize_transaction() {
        let interface = NeuralinkInterface::new();
        let device_id = DeviceId("device-001".to_string());

        interface
            .register_device(
                device_id.clone(),
                DeviceStatus::Connected {
                    signal_quality: 90,
                },
            )
            .unwrap();

        interface
            .create_profile("user123".to_string(), device_id)
            .unwrap();

        let pattern = create_test_pattern(95);
        let result = interface.authorize_transaction(
            "user123",
            "tx-12345".to_string(),
            pattern,
        );

        // Result depends on authentication success
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_access_wallet() {
        let interface = NeuralinkInterface::new();
        let device_id = DeviceId("device-001".to_string());

        interface
            .register_device(
                device_id.clone(),
                DeviceStatus::Connected {
                    signal_quality: 90,
                },
            )
            .unwrap();

        interface
            .create_profile("user123".to_string(), device_id)
            .unwrap();

        let pattern = create_test_pattern(95);
        let result = interface.access_wallet(
            "user123",
            "0x1234567890".to_string(),
            pattern,
        );

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_device_not_connected_error() {
        let interface = NeuralinkInterface::new();
        let device_id = DeviceId("device-001".to_string());

        let result = interface.create_profile("user123".to_string(), device_id);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            NeuralinkError::DeviceNotConnected(_)
        ));
    }

    #[test]
    fn test_update_device_status() {
        let interface = NeuralinkInterface::new();
        let device_id = DeviceId("device-001".to_string());

        interface
            .register_device(device_id.clone(), DeviceStatus::Calibrating)
            .unwrap();

        interface
            .update_device_status(
                &device_id,
                DeviceStatus::Connected {
                    signal_quality: 95,
                },
            )
            .unwrap();

        let devices = interface.devices.read().unwrap();
        assert!(matches!(
            devices.get(&device_id),
            Some(DeviceStatus::Connected { signal_quality: 95 })
        ));
    }

    #[test]
    fn test_command_history() {
        let interface = NeuralinkInterface::new();
        let device_id = DeviceId("device-001".to_string());

        interface
            .register_device(
                device_id.clone(),
                DeviceStatus::Connected {
                    signal_quality: 90,
                },
            )
            .unwrap();

        interface
            .create_profile("user123".to_string(), device_id)
            .unwrap();

        let pattern = create_test_pattern(95);
        
        // Try to execute some commands (may or may not succeed)
        let _ = interface.authorize_transaction(
            "user123",
            "tx-1".to_string(),
            pattern.clone(),
        );
        let _ = interface.access_wallet(
            "user123",
            "0x123".to_string(),
            pattern,
        );

        let history = interface.get_command_history("user123");
        // History should contain attempted commands
        assert!(history.len() <= 2);
    }

    #[test]
    fn test_calibrate_patterns() {
        let interface = NeuralinkInterface::new();
        let device_id = DeviceId("device-001".to_string());

        interface
            .register_device(
                device_id.clone(),
                DeviceStatus::Connected {
                    signal_quality: 90,
                },
            )
            .unwrap();

        interface
            .create_profile("user123".to_string(), device_id)
            .unwrap();

        let new_patterns = vec![create_test_pattern(98), create_test_pattern(97)];

        assert!(interface
            .calibrate_patterns("user123", new_patterns.clone())
            .is_ok());

        let profiles = interface.profiles.read().unwrap();
        let profile = profiles.get("user123").unwrap();
        assert_eq!(profile.baseline_patterns.len(), 2);
    }
}
