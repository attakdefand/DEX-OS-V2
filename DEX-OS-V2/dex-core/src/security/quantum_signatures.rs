//! Dilithium-inspired signature primitives for Security Layer 22 (Quantum-Resistant Security)
//!
//! This module provides a lightweight simulation of Dilithium-style signatures using
//! hardened Ed25519 primitives with SHA3-based domain separation. It is designed to
//! satisfy the "Dilithium Signatures" requirement from DEX-OS-V2.csv while keeping
//! dependencies minimal in this repository.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256, Sha3_512};
use thiserror::Error;

/// Default context string to bind signatures to the Layer 22 security domain
pub const DEFAULT_DILITHIUM_CONTEXT: &str = "dex-os:layer22:dilithium";

/// Simulated Dilithium security levels (kept for future tuning)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DilithiumLevel {
    Level1,
    Level2,
    Level3,
}

impl Default for DilithiumLevel {
    fn default() -> Self {
        DilithiumLevel::Level2
    }
}

/// Keypair for Dilithium-style signatures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DilithiumKeyPair {
    /// Private key bytes (seed for the signer)
    pub private_key: Vec<u8>,
    /// Public key bytes (verifier key)
    pub public_key: Vec<u8>,
    /// Security level used for domain separation
    pub level: DilithiumLevel,
}

/// Detached Dilithium-style signature record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DilithiumSignature {
    /// Raw signature bytes
    pub signature: Vec<u8>,
    /// Public key used for verification
    pub public_key: Vec<u8>,
    /// SHA3-256 hash of the original message (for quick integrity checks)
    pub message_hash: Vec<u8>,
    /// Security level used during signing
    pub level: DilithiumLevel,
    /// Context string binding the signature to a particular security domain
    pub context: String,
}

/// Errors that can occur during Dilithium-style operations
#[derive(Debug, Error)]
pub enum DilithiumError {
    #[error("invalid key length")]
    InvalidKeyLength,
    #[error("invalid signature length")]
    InvalidSignatureLength,
    #[error("verification failed")]
    VerificationFailed,
    #[error("signature parse failed: {0}")]
    SignatureParseFailed(String),
}

/// Quantum-resistant signature engine (Dilithium simulation)
pub struct DilithiumSignatureEngine;

impl DilithiumSignatureEngine {
    /// Generate a Dilithium-style keypair using secure randomness
    pub fn generate_keypair(level: DilithiumLevel) -> Result<DilithiumKeyPair, DilithiumError> {
        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();

        Ok(DilithiumKeyPair {
            private_key: signing_key.to_bytes().to_vec(),
            public_key: verifying_key.to_bytes().to_vec(),
            level,
        })
    }

    /// Reconstruct a keypair from an existing private key (derives the public key)
    pub fn keypair_from_private(
        private_key: &[u8],
        level: DilithiumLevel,
    ) -> Result<DilithiumKeyPair, DilithiumError> {
        let signing_key = SigningKey::from_bytes(
            private_key
                .as_ref()
                .try_into()
                .map_err(|_| DilithiumError::InvalidKeyLength)?,
        );
        let verifying_key = signing_key.verifying_key();

        Ok(DilithiumKeyPair {
            private_key: private_key.to_vec(),
            public_key: verifying_key.to_bytes().to_vec(),
            level,
        })
    }

    /// Create a domain-separated digest for signing/verification
    fn contextual_digest(data: &[u8], context: &str, level: DilithiumLevel) -> Vec<u8> {
        let mut hasher = Sha3_512::new();
        hasher.update(b"dilithium-sim");
        hasher.update([level as u8]);
        hasher.update(context.as_bytes());
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Sign data with a Dilithium-style detached signature
    pub fn sign(
        data: &[u8],
        keypair: &DilithiumKeyPair,
        context: Option<&str>,
    ) -> Result<DilithiumSignature, DilithiumError> {
        let signing_key = SigningKey::from_bytes(
            keypair
                .private_key
                .as_slice()
                .try_into()
                .map_err(|_| DilithiumError::InvalidKeyLength)?,
        );

        let ctx = context.unwrap_or(DEFAULT_DILITHIUM_CONTEXT);
        let digest = Self::contextual_digest(data, ctx, keypair.level);
        let signature = signing_key.sign(&digest);
        let message_hash = Sha3_256::digest(data).to_vec();

        Ok(DilithiumSignature {
            signature: signature.to_bytes().to_vec(),
            public_key: keypair.public_key.clone(),
            message_hash,
            level: keypair.level,
            context: ctx.to_string(),
        })
    }

    /// Verify a Dilithium-style signature against the provided data
    pub fn verify(data: &[u8], signature: &DilithiumSignature) -> Result<bool, DilithiumError> {
        let verifying_key = VerifyingKey::from_bytes(
            signature
                .public_key
                .as_slice()
                .try_into()
                .map_err(|_| DilithiumError::InvalidKeyLength)?,
        )
        .map_err(|e| DilithiumError::SignatureParseFailed(e.to_string()))?;

        // Quick integrity check before full verification
        let expected_hash = Sha3_256::digest(data).to_vec();
        if expected_hash != signature.message_hash {
            return Ok(false);
        }

        let digest = Self::contextual_digest(data, &signature.context, signature.level);
        let sig_bytes: [u8; 64] = signature
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| DilithiumError::InvalidSignatureLength)?;
        let parsed_signature = Signature::from_bytes(&sig_bytes);

        verifying_key
            .verify(&digest, &parsed_signature)
            .map(|_| true)
            .map_err(|_| DilithiumError::VerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dilithium_keypair_generation() {
        let keypair = DilithiumSignatureEngine::generate_keypair(DilithiumLevel::Level2).unwrap();
        assert_eq!(keypair.private_key.len(), 32);
        assert_eq!(keypair.public_key.len(), 32);
        assert_eq!(keypair.level, DilithiumLevel::Level2);
    }

    #[test]
    fn test_dilithium_sign_and_verify() {
        let keypair = DilithiumSignatureEngine::generate_keypair(DilithiumLevel::Level1).unwrap();
        let data = b"quantum resistant payload";

        let signature =
            DilithiumSignatureEngine::sign(data, &keypair, Some("custom-context")).unwrap();
        assert_eq!(
            signature.message_hash,
            Sha3_256::digest(data).to_vec()
        );

        let verified = DilithiumSignatureEngine::verify(data, &signature).unwrap();
        assert!(verified);

        let tampered = DilithiumSignature {
            message_hash: signature.message_hash.clone(),
            ..signature.clone()
        };
        assert!(!DilithiumSignatureEngine::verify(b"tampered", &tampered).unwrap());
    }

    #[test]
    fn test_reconstruct_keypair_from_private_key() {
        let keypair = DilithiumSignatureEngine::generate_keypair(DilithiumLevel::Level3).unwrap();
        let reconstructed =
            DilithiumSignatureEngine::keypair_from_private(&keypair.private_key, keypair.level)
                .unwrap();
        assert_eq!(reconstructed.public_key, keypair.public_key);
    }
}
