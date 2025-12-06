//! IoT wallet runtime for WASM integration (DEX-OS-V2.csv row 5)
//!
//! Implements: `5,Core Components,WASM Runtime,Runtime,IoT Wallet,Internet of Things Integration,Medium {Security: Layer 20 - Internet of Things (IoT) Security}`
//!
//! Focus areas for Layer 20:
//! - Device authentication via signed challenges
//! - Network segmentation enforcement per device
//! - Heartbeat-based liveness monitoring with risk scoring
//! - Audit trail for IoT-specific security events
//! - Short-lived sessions with replay protection

use crate::types::TraderId;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const DEFAULT_SESSION_TTL_SECS: u64 = 900; // 15 minutes
const DEFAULT_CHALLENGE_TTL_SECS: u64 = 300; // 5 minutes
const DEFAULT_HEARTBEAT_TIMEOUT_SECS: u64 = 600; // 10 minutes
const PUBLIC_KEY_LENGTH: usize = 32;
const SIGNATURE_LENGTH: usize = 64;

/// IoT network segment used for micro-segmentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum IoTSegment {
    Payments,
    Control,
    Telemetry,
    Guest,
}

impl Display for IoTSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            IoTSegment::Payments => "payments",
            IoTSegment::Control => "control",
            IoTSegment::Telemetry => "telemetry",
            IoTSegment::Guest => "guest",
        };
        f.write_str(value)
    }
}

impl FromStr for IoTSegment {
    type Err = IoTWalletError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "payments" => Ok(IoTSegment::Payments),
            "control" => Ok(IoTSegment::Control),
            "telemetry" => Ok(IoTSegment::Telemetry),
            "guest" => Ok(IoTSegment::Guest),
            _ => Err(IoTWalletError::InvalidSegment),
        }
    }
}

/// Risk levels for IoT devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoTRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl IoTRiskLevel {
    fn elevate(self, other: IoTRiskLevel) -> IoTRiskLevel {
        match (self, other) {
            (IoTRiskLevel::Critical, _) | (_, IoTRiskLevel::Critical) => IoTRiskLevel::Critical,
            (IoTRiskLevel::High, _) | (_, IoTRiskLevel::High) => IoTRiskLevel::High,
            (IoTRiskLevel::Medium, _) | (_, IoTRiskLevel::Medium) => IoTRiskLevel::Medium,
            _ => IoTRiskLevel::Low,
        }
    }
}

/// IoT device profile tracked by the runtime
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IoTDeviceProfile {
    pub device_id: String,
    pub owner: TraderId,
    pub public_key: Vec<u8>,
    pub firmware_version: String,
    pub allowed_segments: Vec<IoTSegment>,
    pub allowed_operations: HashSet<String>,
    pub last_attestation: Option<u64>,
    pub last_heartbeat: Option<u64>,
}

impl IoTDeviceProfile {
    /// Create a new device profile with explicit allowlists
    pub fn new(
        device_id: String,
        owner: TraderId,
        public_key: Vec<u8>,
        firmware_version: String,
        allowed_segments: Vec<IoTSegment>,
        allowed_operations: HashSet<String>,
    ) -> Self {
        Self {
            device_id,
            owner,
            public_key,
            firmware_version,
            allowed_segments,
            allowed_operations,
            last_attestation: None,
            last_heartbeat: None,
        }
    }
}

/// Authentication challenge issued to an IoT device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthChallenge {
    pub device_id: String,
    pub nonce: Vec<u8>,
    pub issued_at: u64,
    pub expires_at: u64,
    pub signing_message: Vec<u8>,
}

/// Short-lived authenticated session for a device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IoTSession {
    pub token: String,
    pub device_id: String,
    pub segment: IoTSegment,
    pub expires_at: u64,
    pub last_validated_at: u64,
}

/// Risk report for a device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IoTRiskAssessment {
    pub device_id: String,
    pub risk_level: IoTRiskLevel,
    pub reasons: Vec<String>,
}

/// Heartbeat status emitted by a device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IoTDeviceStatus {
    pub device_id: String,
    pub last_heartbeat: u64,
    pub firmware_version: String,
    pub risk_level: IoTRiskLevel,
}

/// Security events recorded for IoT flows
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IoTSecurityEvent {
    pub device_id: String,
    pub description: String,
    pub severity: IoTRiskLevel,
    pub timestamp: u64,
}

/// Errors returned by the IoT wallet runtime
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IoTWalletError {
    #[error("device already registered")]
    DeviceAlreadyRegistered,
    #[error("device not registered")]
    DeviceNotRegistered,
    #[error("device public key is invalid")]
    InvalidPublicKey,
    #[error("device public key has an invalid length")]
    InvalidPublicKeyLength,
    #[error("challenge expired")]
    ChallengeExpired,
    #[error("challenge not found")]
    ChallengeNotFound,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("session expired")]
    SessionExpired,
    #[error("session not found")]
    SessionNotFound,
    #[error("segment violation")]
    SegmentViolation,
    #[error("operation not allowed for device")]
    OperationNotAllowed,
    #[error("device risk level too high")]
    HighRiskDevice,
    #[error("no network segment assigned for device")]
    MissingSegment,
    #[error("no allowed operations configured for device")]
    MissingOperations,
    #[error("invalid segment value")]
    InvalidSegment,
}

/// Runtime responsible for securing IoT wallet interactions
#[derive(Debug, Default)]
pub struct IoTWalletRuntime {
    devices: HashMap<String, IoTDeviceProfile>,
    challenges: HashMap<String, AuthChallenge>,
    sessions: HashMap<String, IoTSession>,
    audit_log: Vec<IoTSecurityEvent>,
    failed_auth_attempts: HashMap<String, u32>,
    session_ttl_secs: u64,
    challenge_ttl_secs: u64,
    heartbeat_timeout_secs: u64,
}

impl IoTWalletRuntime {
    /// Create a runtime with default security windows
    pub fn new() -> Self {
        Self::with_security_windows(
            DEFAULT_SESSION_TTL_SECS,
            DEFAULT_CHALLENGE_TTL_SECS,
            DEFAULT_HEARTBEAT_TIMEOUT_SECS,
        )
    }

    /// Create a runtime with custom TTLs (primarily for testing)
    pub fn with_security_windows(
        session_ttl_secs: u64,
        challenge_ttl_secs: u64,
        heartbeat_timeout_secs: u64,
    ) -> Self {
        Self {
            devices: HashMap::new(),
            challenges: HashMap::new(),
            sessions: HashMap::new(),
            audit_log: Vec::new(),
            failed_auth_attempts: HashMap::new(),
            session_ttl_secs,
            challenge_ttl_secs,
            heartbeat_timeout_secs,
        }
    }

    /// Register a new IoT device with strict allowlists
    pub fn register_device(&mut self, profile: IoTDeviceProfile) -> Result<(), IoTWalletError> {
        if self.devices.contains_key(&profile.device_id) {
            return Err(IoTWalletError::DeviceAlreadyRegistered);
        }

        if profile.allowed_segments.is_empty() {
            return Err(IoTWalletError::MissingSegment);
        }

        if profile.allowed_operations.is_empty() {
            return Err(IoTWalletError::MissingOperations);
        }

        // Validate the public key length before storing
        if profile.public_key.len() != PUBLIC_KEY_LENGTH {
            return Err(IoTWalletError::InvalidPublicKeyLength);
        }

        // Ensure the key material parses correctly
        VerifyingKey::from_bytes(
            profile
                .public_key
                .as_slice()
                .try_into()
                .map_err(|_| IoTWalletError::InvalidPublicKeyLength)?,
        )
        .map_err(|_| IoTWalletError::InvalidPublicKey)?;

        self.devices
            .insert(profile.device_id.clone(), profile.clone());

        self.log_event(
            &profile.device_id,
            "Device registered with IoT runtime".to_string(),
            IoTRiskLevel::Low,
        );

        Ok(())
    }

    /// Issue a signed challenge to a device for authentication
    pub fn issue_challenge(&mut self, device_id: &str) -> Result<AuthChallenge, IoTWalletError> {
        let profile = self
            .devices
            .get(device_id)
            .ok_or(IoTWalletError::DeviceNotRegistered)?;

        let mut nonce = vec![0u8; 32];
        OsRng.fill_bytes(&mut nonce);
        let issued_at = Self::now();
        let expires_at = issued_at + self.challenge_ttl_secs;
        let signing_message = Self::compose_challenge_message(device_id, &nonce, expires_at);

        let challenge = AuthChallenge {
            device_id: profile.device_id.clone(),
            nonce,
            issued_at,
            expires_at,
            signing_message,
        };

        self.challenges
            .insert(profile.device_id.clone(), challenge.clone());

        self.log_event(
            device_id,
            "Challenge issued for IoT device authentication".to_string(),
            IoTRiskLevel::Low,
        );

        Ok(challenge)
    }

    /// Verify the device's response and mint a short-lived session
    pub fn verify_challenge_response(
        &mut self,
        device_id: &str,
        signature: Vec<u8>,
    ) -> Result<IoTSession, IoTWalletError> {
        let profile = self
            .devices
            .get(device_id)
            .ok_or(IoTWalletError::DeviceNotRegistered)?
            .clone();

        let challenge = self
            .challenges
            .get(device_id)
            .ok_or(IoTWalletError::ChallengeNotFound)?
            .clone();

        let now = Self::now();
        if now > challenge.expires_at {
            self.challenges.remove(device_id);
            return Err(IoTWalletError::ChallengeExpired);
        }

        self.verify_signature(&profile, &challenge, &signature)?;

        // Reset failed attempts after a successful attestation
        self.failed_auth_attempts.insert(device_id.to_string(), 0);

        let session = self.create_session(&profile, &challenge, now);
        self.challenges.remove(device_id);

        if let Some(stored_profile) = self.devices.get_mut(device_id) {
            stored_profile.last_attestation = Some(now);
        }

        self.log_event(
            device_id,
            "IoT device authenticated".to_string(),
            IoTRiskLevel::Low,
        );

        Ok(session)
    }

    /// Validate session, segment, and risk posture before allowing a device operation
    pub fn authorize_operation(
        &mut self,
        device_id: &str,
        session_token: &str,
        segment: IoTSegment,
        operation: &str,
    ) -> Result<(), IoTWalletError> {
        let profile = self
            .devices
            .get(device_id)
            .ok_or(IoTWalletError::DeviceNotRegistered)?
            .clone();

        let now = Self::now();
        let session_segment = {
            let session = self
                .sessions
                .get_mut(session_token)
                .ok_or(IoTWalletError::SessionNotFound)?;

            if session.device_id != device_id {
                return Err(IoTWalletError::SessionNotFound);
            }

            if now > session.expires_at {
                self.sessions.remove(session_token);
                return Err(IoTWalletError::SessionExpired);
            }

            session.segment
        };

        let risk = self.assess_risk(device_id)?;
        if matches!(risk.risk_level, IoTRiskLevel::High | IoTRiskLevel::Critical) {
            return Err(IoTWalletError::HighRiskDevice);
        }

        if !profile.allowed_segments.contains(&segment) || session_segment != segment {
            return Err(IoTWalletError::SegmentViolation);
        }

        if !profile.allowed_operations.contains(operation) {
            return Err(IoTWalletError::OperationNotAllowed);
        }

        if let Some(session) = self.sessions.get_mut(session_token) {
            session.last_validated_at = now;
        }

        self.log_event(
            device_id,
            format!("Authorized IoT operation '{operation}' on {segment}"),
            IoTRiskLevel::Low,
        );

        Ok(())
    }

    /// Record a heartbeat and update the device status
    pub fn record_heartbeat(
        &mut self,
        device_id: &str,
        timestamp: u64,
    ) -> Result<IoTDeviceStatus, IoTWalletError> {
        let firmware_version = {
            let profile = self
                .devices
                .get_mut(device_id)
                .ok_or(IoTWalletError::DeviceNotRegistered)?;

            profile.last_heartbeat = Some(timestamp);
            profile.firmware_version.clone()
        };
        let risk = self.assess_risk(device_id)?;

        let status = IoTDeviceStatus {
            device_id: device_id.to_string(),
            last_heartbeat: timestamp,
            firmware_version,
            risk_level: risk.risk_level,
        };

        self.log_event(
            device_id,
            "Heartbeat recorded for IoT device".to_string(),
            risk.risk_level,
        );

        Ok(status)
    }

    /// Produce a risk assessment using heartbeat, attestation, and failure counters
    pub fn assess_risk(&self, device_id: &str) -> Result<IoTRiskAssessment, IoTWalletError> {
        let profile = self
            .devices
            .get(device_id)
            .ok_or(IoTWalletError::DeviceNotRegistered)?;

        let now = Self::now();
        let mut reasons = Vec::new();
        let mut risk = IoTRiskLevel::Low;

        match profile.last_heartbeat {
            Some(last) if now.saturating_sub(last) > self.heartbeat_timeout_secs => {
                reasons.push("Heartbeat is stale".to_string());
                risk = risk.elevate(IoTRiskLevel::Medium);
            }
            None => {
                reasons.push("No heartbeat received yet".to_string());
                risk = risk.elevate(IoTRiskLevel::Medium);
            }
            _ => {}
        }

        if profile
            .last_attestation
            .map(|att| now.saturating_sub(att) > self.session_ttl_secs * 2)
            .unwrap_or(true)
        {
            reasons.push("Attestation is stale or missing".to_string());
            risk = risk.elevate(IoTRiskLevel::High);
        }

        if let Some(failures) = self.failed_auth_attempts.get(device_id) {
            if *failures >= 3 {
                reasons.push(format!("Repeated authentication failures ({failures})"));
                risk = risk.elevate(IoTRiskLevel::High);
            }
        }

        // Elevated risk if the device belongs to sensitive segments
        if profile.allowed_segments.contains(&IoTSegment::Control) {
            risk = risk.elevate(IoTRiskLevel::Medium);
            reasons.push("Device has control-plane permissions".to_string());
        }

        Ok(IoTRiskAssessment {
            device_id: device_id.to_string(),
            risk_level: risk,
            reasons,
        })
    }

    /// Get recent audit events (useful for observability pipelines)
    pub fn audit_events(&self) -> &[IoTSecurityEvent] {
        &self.audit_log
    }

    fn create_session(
        &mut self,
        profile: &IoTDeviceProfile,
        challenge: &AuthChallenge,
        now: u64,
    ) -> IoTSession {
        let token = Self::derive_session_token(&profile.device_id, &challenge.nonce, now);
        let segment = *profile
            .allowed_segments
            .get(0)
            .unwrap_or(&IoTSegment::Guest);

        let session = IoTSession {
            token: token.clone(),
            device_id: profile.device_id.clone(),
            segment,
            expires_at: now + self.session_ttl_secs,
            last_validated_at: now,
        };

        self.sessions.insert(token.clone(), session.clone());
        session
    }

    fn compose_challenge_message(device_id: &str, nonce: &[u8], expires_at: u64) -> Vec<u8> {
        let mut message = Vec::with_capacity(nonce.len() + device_id.len() + 8);
        message.extend_from_slice(nonce);
        message.extend_from_slice(device_id.as_bytes());
        message.extend_from_slice(&expires_at.to_be_bytes());
        message
    }

    fn verify_signature(
        &mut self,
        profile: &IoTDeviceProfile,
        challenge: &AuthChallenge,
        signature: &[u8],
    ) -> Result<(), IoTWalletError> {
        let key_bytes: [u8; PUBLIC_KEY_LENGTH] = profile
            .public_key
            .as_slice()
            .try_into()
            .map_err(|_| IoTWalletError::InvalidPublicKeyLength)?;

        let verifying_key =
            VerifyingKey::from_bytes(&key_bytes).map_err(|_| IoTWalletError::InvalidPublicKey)?;

        let signature_bytes: [u8; Signature::BYTE_SIZE] = signature
            .try_into()
            .map_err(|_| IoTWalletError::InvalidSignature)?;

        let signature = Signature::from_bytes(&signature_bytes);

        verifying_key
            .verify(&challenge.signing_message, &signature)
            .map_err(|_| {
                let counter = self
                    .failed_auth_attempts
                    .entry(profile.device_id.clone())
                    .or_insert(0);
                *counter += 1;
                IoTWalletError::InvalidSignature
            })
    }

    fn derive_session_token(device_id: &str, nonce: &[u8], issued_at: u64) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(device_id.as_bytes());
        hasher.update(nonce);
        hasher.update(&issued_at.to_be_bytes());
        let digest = hasher.finalize();

        digest.iter().map(|b| format!("{:02x}", b)).collect()
    }

    fn log_event(&mut self, device_id: &str, description: String, severity: IoTRiskLevel) {
        let event = IoTSecurityEvent {
            device_id: device_id.to_string(),
            description,
            severity,
            timestamp: Self::now(),
        };

        self.audit_log.push(event);

        // Keep the log bounded for predictable memory usage
        if self.audit_log.len() > 2048 {
            let surplus = self.audit_log.len() - 2048;
            self.audit_log.drain(0..surplus);
        }
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn demo_profile() -> (IoTDeviceProfile, SigningKey) {
        let signing_key = SigningKey::generate(&mut OsRng);
        let profile = IoTDeviceProfile::new(
            "device-1".to_string(),
            "owner-1".to_string(),
            signing_key.verifying_key().to_bytes().to_vec(),
            "1.0.0".to_string(),
            vec![IoTSegment::Payments, IoTSegment::Telemetry],
            HashSet::from(["payment.sign".to_string(), "heartbeat".to_string()]),
        );
        (profile, signing_key)
    }

    #[test]
    fn authenticates_device_and_authorizes_operation() {
        let (profile, signing_key) = demo_profile();
        let mut runtime = IoTWalletRuntime::new();
        runtime.register_device(profile.clone()).unwrap();

        let challenge = runtime.issue_challenge(&profile.device_id).unwrap();
        let signature = signing_key
            .sign(&challenge.signing_message)
            .to_bytes()
            .to_vec();

        let session = runtime
            .verify_challenge_response(&profile.device_id, signature)
            .unwrap();

        runtime
            .record_heartbeat(&profile.device_id, session.last_validated_at + 1)
            .unwrap();

        runtime
            .authorize_operation(
                &profile.device_id,
                &session.token,
                IoTSegment::Payments,
                "payment.sign",
            )
            .unwrap();
    }

    #[test]
    fn rejects_invalid_signature_and_tracks_failures() {
        let (profile, _) = demo_profile();
        let mut runtime = IoTWalletRuntime::new();
        runtime.register_device(profile.clone()).unwrap();

        let challenge = runtime.issue_challenge(&profile.device_id).unwrap();
        // Sign with a different key
        let rogue_key = SigningKey::generate(&mut OsRng);
        let bad_signature = rogue_key
            .sign(&challenge.signing_message)
            .to_bytes()
            .to_vec();

        let result = runtime.verify_challenge_response(&profile.device_id, bad_signature);
        assert!(matches!(result, Err(IoTWalletError::InvalidSignature)));

        // Risk should elevate due to failed attempts
        let risk = runtime.assess_risk(&profile.device_id).unwrap();
        assert!(matches!(
            risk.risk_level,
            IoTRiskLevel::High | IoTRiskLevel::Medium
        ));
    }

    #[test]
    fn enforces_segment_and_operation_allowlists() {
        let (profile, signing_key) = demo_profile();
        let mut runtime = IoTWalletRuntime::new();
        runtime.register_device(profile.clone()).unwrap();

        let challenge = runtime.issue_challenge(&profile.device_id).unwrap();
        let signature = signing_key
            .sign(&challenge.signing_message)
            .to_bytes()
            .to_vec();
        let session = runtime
            .verify_challenge_response(&profile.device_id, signature)
            .unwrap();

        // Segment violation
        let err = runtime
            .authorize_operation(
                &profile.device_id,
                &session.token,
                IoTSegment::Control,
                "payment.sign",
            )
            .unwrap_err();
        assert_eq!(err, IoTWalletError::SegmentViolation);

        // Operation violation
        let err = runtime
            .authorize_operation(
                &profile.device_id,
                &session.token,
                IoTSegment::Payments,
                "unapproved.op",
            )
            .unwrap_err();
        assert_eq!(err, IoTWalletError::OperationNotAllowed);
    }

    #[test]
    fn expires_challenges_and_sessions() {
        let (profile, signing_key) = demo_profile();
        let mut runtime = IoTWalletRuntime::with_security_windows(1, 1, 1);
        runtime.register_device(profile.clone()).unwrap();

        let mut challenge = runtime.issue_challenge(&profile.device_id).unwrap();

        // Force expiration
        challenge.expires_at = IoTWalletRuntime::now() - 1;
        runtime
            .challenges
            .insert(profile.device_id.clone(), challenge.clone());

        let signature = signing_key
            .sign(&challenge.signing_message)
            .to_bytes()
            .to_vec();
        let result = runtime.verify_challenge_response(&profile.device_id, signature);
        assert_eq!(result.unwrap_err(), IoTWalletError::ChallengeExpired);
    }
}
