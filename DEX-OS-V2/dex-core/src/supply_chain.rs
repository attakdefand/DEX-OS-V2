//! Supply Chain Security Features Implementation
//!
//! This module implements the Priority 3 supply chain security features:
//! - Supply Chain,Supply Chain,Supply Chain,B+ Tree,Artifact Registry,Medium
//! - Supply Chain,Supply Chain,Supply Chain,Hash Map,Signature Verification,Medium

use crate::database::BPlusTree;
use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use thiserror::Error;

/// Supply Chain Security Manager
#[derive(Debug, Clone)]
pub struct SupplyChainManager {
    /// Artifact registry using B+ tree for efficient storage and retrieval
    artifact_registry: ArtifactRegistry,
    /// Signature verification system using hash map for efficient lookup
    signature_verifier: SignatureVerifier,
}

/// Artifact registry using B+ tree for efficient storage and retrieval
#[derive(Debug, Clone)]
pub struct ArtifactRegistry {
    /// B+ tree storing artifacts by their ID
    artifacts: BPlusTree<String, Artifact>,
}

/// Digital artifact in the supply chain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique identifier for the artifact
    pub id: String,
    /// Name of the artifact
    pub name: String,
    /// Version of the artifact
    pub version: String,
    /// Hash of the artifact content
    pub content_hash: Vec<u8>,
    /// Path to the artifact
    pub path: String,
    /// Timestamp of artifact creation
    pub created_at: u64,
    /// Creator of the artifact
    pub creator: String,
    /// Certificate for the artifact
    pub certificate_id: String,
    /// Metadata associated with the artifact
    pub metadata: HashMap<String, String>,
}

/// Signature verifier using hash map for efficient lookup
#[derive(Debug, Clone)]
pub struct SignatureVerifier {
    /// Hash map storing signatures by artifact ID for fast verification
    signatures: HashMap<String, ArtifactSignature>,
}

/// Artifact signature for verification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactSignature {
    /// ID of the artifact being signed
    pub artifact_id: String,
    /// Signature data
    pub signature: Vec<u8>,
    /// Public key used for verification
    pub public_key: Vec<u8>,
    /// Timestamp of signing
    pub timestamp: u64,
    /// Signer information
    pub signer: String,
}

/// Supply chain security errors
#[derive(Error, Debug, PartialEq)]
pub enum SupplyChainError {
    /// Artifact not found in registry
    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),
    /// Signature verification failed
    #[error("Signature verification failed for artifact: {0}")]
    SignatureVerificationFailed(String),
    /// Artifact already exists in registry
    #[error("Artifact already exists: {0}")]
    ArtifactAlreadyExists(String),
    /// Invalid certificate
    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),
}

impl SupplyChainManager {
    /// Create a new supply chain manager
    pub fn new() -> Self {
        Self {
            artifact_registry: ArtifactRegistry::new(),
            signature_verifier: SignatureVerifier::new(),
        }
    }

    /// Register a new artifact in the registry
    pub fn register_artifact(&mut self, artifact: Artifact) -> Result<(), SupplyChainError> {
        self.artifact_registry.register_artifact(artifact)
    }

    /// Get an artifact by ID
    pub fn get_artifact(&self, artifact_id: &str) -> Result<Artifact, SupplyChainError> {
        self.artifact_registry.get_artifact(artifact_id)
    }

    /// Verify an artifact exists in the registry
    pub fn verify_artifact_exists(&self, artifact_id: &str) -> bool {
        self.artifact_registry.artifact_exists(artifact_id)
    }

    /// Add a signature for an artifact
    pub fn add_signature(&mut self, signature: ArtifactSignature) -> Result<(), SupplyChainError> {
        self.signature_verifier.add_signature(signature)
    }

    /// Verify an artifact's signature against its stored content hash
    pub fn verify_signature(&self, artifact_id: &str) -> Result<bool, SupplyChainError> {
        let artifact = self.get_artifact(artifact_id)?;
        self.signature_verifier
            .verify_signature_for_artifact(artifact_id, &artifact)
    }

    /// Get artifact signature
    pub fn get_signature(&self, artifact_id: &str) -> Option<&ArtifactSignature> {
        self.signature_verifier.get_signature(artifact_id)
    }
}

impl Default for SupplyChainManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactRegistry {
    /// Create a new artifact registry
    pub fn new() -> Self {
        Self {
            artifacts: BPlusTree::new(4), // B+ tree with order 4
        }
    }

    /// Register a new artifact in the registry
    pub fn register_artifact(&mut self, artifact: Artifact) -> Result<(), SupplyChainError> {
        if self.artifacts.contains_key(&artifact.id) {
            return Err(SupplyChainError::ArtifactAlreadyExists(artifact.id));
        }

        self.artifacts.insert(artifact.id.clone(), artifact);
        Ok(())
    }

    /// Get an artifact by ID
    pub fn get_artifact(&self, artifact_id: &str) -> Result<Artifact, SupplyChainError> {
        self.artifacts
            .get(&artifact_id.to_string())
            .ok_or_else(|| SupplyChainError::ArtifactNotFound(artifact_id.to_string()))
    }

    /// Check if an artifact exists in the registry
    pub fn artifact_exists(&self, artifact_id: &str) -> bool {
        self.artifacts.contains_key(&artifact_id.to_string())
    }
}

impl Default for ArtifactRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SignatureVerifier {
    /// Create a new signature verifier
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
        }
    }

    /// Add a signature for an artifact
    pub fn add_signature(&mut self, signature: ArtifactSignature) -> Result<(), SupplyChainError> {
        self.signatures
            .insert(signature.artifact_id.clone(), signature);
        Ok(())
    }

    /// Verify an artifact's signature using Ed25519 over the artifact's content hash
    pub fn verify_signature_for_artifact(
        &self,
        artifact_id: &str,
        artifact: &Artifact,
    ) -> Result<bool, SupplyChainError> {
        let signature_entry = self.signatures.get(artifact_id).ok_or_else(|| {
            SupplyChainError::SignatureVerificationFailed(artifact_id.to_string())
        })?;

        let signature_bytes: [u8; 64] = signature_entry
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| SupplyChainError::SignatureVerificationFailed(artifact_id.to_string()))?;
        let signature = Ed25519Signature::from_bytes(&signature_bytes);

        let public_key_bytes: [u8; 32] = signature_entry
            .public_key
            .as_slice()
            .try_into()
            .map_err(|_| SupplyChainError::SignatureVerificationFailed(artifact_id.to_string()))?;
        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes).map_err(|_| {
            SupplyChainError::SignatureVerificationFailed(artifact_id.to_string())
        })?;

        verifying_key
            .verify(&artifact.content_hash, &signature)
            .map_err(|_| {
                SupplyChainError::SignatureVerificationFailed(artifact_id.to_string())
            })?;

        Ok(true)
    }

    /// Get artifact signature
    pub fn get_signature(&self, artifact_id: &str) -> Option<&ArtifactSignature> {
        self.signatures.get(artifact_id)
    }
}

impl Default for SignatureVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;
    use sha3::{Digest, Sha3_256};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_artifact_registry() {
        let mut manager = SupplyChainManager::new();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let artifact = Artifact {
            id: "artifact_1".to_string(),
            name: "test_component".to_string(),
            version: "1.0.0".to_string(),
            content_hash: vec![1, 2, 3, 4, 5],
            path: "/path/to/component".to_string(),
            created_at: now,
            creator: "build_system".to_string(),
            certificate_id: "cert_1".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("build_id".to_string(), "build_123".to_string());
                map.insert(
                    "source_repo".to_string(),
                    "https://github.com/test/repo".to_string(),
                );
                map
            },
        };

        // Register artifact
        assert!(manager.register_artifact(artifact.clone()).is_ok());

        // Try to register the same artifact again (should fail)
        assert_eq!(
            manager.register_artifact(artifact.clone()),
            Err(SupplyChainError::ArtifactAlreadyExists(
                "artifact_1".to_string()
            ))
        );

        // Get artifact
        let retrieved_artifact = manager.get_artifact("artifact_1").unwrap();
        assert_eq!(retrieved_artifact, artifact);

        // Verify artifact exists
        assert!(manager.verify_artifact_exists("artifact_1"));
        assert!(!manager.verify_artifact_exists("nonexistent_artifact"));

        // Try to get nonexistent artifact
        assert_eq!(
            manager.get_artifact("nonexistent_artifact"),
            Err(SupplyChainError::ArtifactNotFound(
                "nonexistent_artifact".to_string()
            ))
        );
    }

    #[test]
    fn test_signature_verification() {
        let mut manager = SupplyChainManager::new();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let content_hash = Sha3_256::digest(b"artifact content").to_vec();

        let artifact = Artifact {
            id: "artifact_1".to_string(),
            name: "test_component".to_string(),
            version: "1.0.0".to_string(),
            content_hash,
            path: "/path/to/component".to_string(),
            created_at: now,
            creator: "build_system".to_string(),
            certificate_id: "cert_1".to_string(),
            metadata: HashMap::new(),
        };

        manager.register_artifact(artifact.clone()).unwrap();

        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        let signature = signing_key.sign(&artifact.content_hash);

        let signature_entry = ArtifactSignature {
            artifact_id: artifact.id.clone(),
            signature: signature.to_bytes().to_vec(),
            public_key: verifying_key.to_bytes().to_vec(),
            timestamp: now,
            signer: "trusted_signer".to_string(),
        };

        // Add signature
        assert!(manager.add_signature(signature_entry.clone()).is_ok());

        // Verify signature exists
        let retrieved_signature = manager.get_signature("artifact_1").unwrap();
        assert_eq!(retrieved_signature, &signature_entry);

        // Verify signature
        assert!(manager.verify_signature("artifact_1").unwrap());

        // Replace with an invalid public key to force verification failure
        let bad_signing_key = SigningKey::generate(&mut rng);
        let bad_signature_entry = ArtifactSignature {
            artifact_id: artifact.id.clone(),
            signature: signature.to_bytes().to_vec(),
            public_key: bad_signing_key.verifying_key().to_bytes().to_vec(),
            timestamp: now,
            signer: "untrusted_signer".to_string(),
        };
        manager.add_signature(bad_signature_entry).unwrap();

        assert_eq!(
            manager.verify_signature("artifact_1"),
            Err(SupplyChainError::SignatureVerificationFailed(
                "artifact_1".to_string()
            ))
        );

        // Try to verify signature for nonexistent artifact
        assert_eq!(
            manager.verify_signature("nonexistent_artifact"),
            Err(SupplyChainError::ArtifactNotFound(
                "nonexistent_artifact".to_string()
            ))
        );
    }
}
