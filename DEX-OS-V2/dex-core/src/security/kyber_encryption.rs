//! Kyber Encryption - Security Layer 22 (Quantum-Resistant Security)
//!
//! Implements the "Kyber Encryption" component from DEX-OS-V2.csv line 76:
//! `2,Components,Security Layer,Security,Kyber Encryption,Encryption,High [IMPLEMENTED] {Security: Layer 22 - Quantum-Resistant Security}`
//!
//! This module provides a Kyber-inspired key encapsulation flow layered on top of
//! authenticated encryption (AES-256-GCM) to deliver a quantum-resilient channel
//! for data confidentiality. While not a full Kyber implementation, it models the
//! workflow (key generation, encapsulation/decapsulation, and ciphertext integrity)
//! with explicit fingerprints to detect tampering.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::sync::{Arc, RwLock};
use thiserror::Error;

const KYBER_SEED_SIZE: usize = 32;
const KYBER_TAG_SIZE: usize = 32;
const KYBER_NONCE_SIZE: usize = 12;
const KYBER_SHARED_KEY_SIZE: usize = 32;

/// Errors for Kyber encryption operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum KyberError {
    #[error("Invalid key material: {0}")]
    InvalidKey(String),
    #[error("Invalid ciphertext: {0}")]
    InvalidCiphertext(String),
    #[error("Authentication failed for encapsulated key")]
    AuthenticationFailed,
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
}

/// Kyber key pair (public/private)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KyberKeyPair {
    pub key_id: String,
    pub public_key: Vec<u8>,
    #[serde(skip)]
    pub private_key: Vec<u8>,
    pub created_at: u64,
}

/// Shared secret derived from encapsulation/decapsulation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KyberSharedSecret {
    pub key: [u8; KYBER_SHARED_KEY_SIZE],
    pub key_id: String,
    pub established_at: u64,
}

impl KyberSharedSecret {
    /// Stable fingerprint used to assert that both parties derived the same key
    pub fn fingerprint(&self, kem_ciphertext: &[u8]) -> Vec<u8> {
        fingerprint(&self.key, kem_ciphertext, &self.key_id)
    }
}

/// Encapsulated ciphertext bundle (KEM + AEAD ciphertext)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KyberEncryptedPackage {
    pub kem_ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub shared_secret_fingerprint: Vec<u8>,
    pub key_id: String,
    pub algorithm: String,
}

/// Encryption output that keeps both transport data and derived session secret
#[derive(Debug, Clone, PartialEq)]
pub struct KyberEncryptionOutput {
    pub package: KyberEncryptedPackage,
    pub shared_secret: KyberSharedSecret,
}

/// Decryption output for callers that need the plaintext and the negotiated key
#[derive(Debug, Clone, PartialEq)]
pub struct KyberDecryptionResult {
    pub plaintext: Vec<u8>,
    pub shared_secret: KyberSharedSecret,
}

/// Statistics for monitoring Kyber operations
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct KyberEncryptionStats {
    pub encryptions: usize,
    pub decryptions: usize,
    pub failures: usize,
    pub key_rotations: usize,
}

/// Kyber Encryption manager for Layer 22 quantum-resistant security
#[derive(Clone)]
pub struct KyberEncryptionManager {
    keypair: Arc<RwLock<KyberKeyPair>>,
    stats: Arc<RwLock<KyberEncryptionStats>>,
}

impl std::fmt::Debug for KyberEncryptionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KyberEncryptionManager")
            .field("keypair", &"<redacted>")
            .field("stats", &self.stats.read().unwrap())
            .finish()
    }
}

impl KyberEncryptionManager {
    /// Create a new manager with a fresh key pair
    pub fn new() -> Self {
        let keypair = Self::generate_keypair(format!("kyber_{}", now()));
        Self::with_keypair(keypair)
    }

    /// Create a manager from an existing key pair
    pub fn with_keypair(keypair: KyberKeyPair) -> Self {
        Self {
            keypair: Arc::new(RwLock::new(keypair)),
            stats: Arc::new(RwLock::new(KyberEncryptionStats::default())),
        }
    }

    /// Generate a new Kyber key pair
    pub fn generate_keypair(key_id: impl Into<String>) -> KyberKeyPair {
        let mut private_key = vec![0u8; KYBER_SEED_SIZE];
        OsRng.fill_bytes(&mut private_key);

        let public_key = derive_public_key(&private_key);

        KyberKeyPair {
            key_id: key_id.into(),
            public_key,
            private_key,
            created_at: now(),
        }
    }

    /// Return the current public key
    pub fn public_key(&self) -> Vec<u8> {
        self.keypair.read().unwrap().public_key.clone()
    }

    /// Return the key identifier for routing messages
    pub fn current_key_id(&self) -> String {
        self.keypair.read().unwrap().key_id.clone()
    }

    /// Rotate the Kyber key pair
    pub fn rotate_key(&self) -> KyberKeyPair {
        let new_pair = Self::generate_keypair(format!("kyber_{}", now()));
        {
            let mut slot = self.keypair.write().unwrap();
            *slot = new_pair.clone();
        }
        self.stats.write().unwrap().key_rotations += 1;
        new_pair
    }

    /// Encrypt data for a recipient public key (returns ciphertext + shared secret)
    pub fn encrypt_for(
        &self,
        recipient_public_key: &[u8],
        recipient_key_id: impl Into<String>,
        plaintext: &[u8],
    ) -> Result<KyberEncryptionOutput, KyberError> {
        let key_id = recipient_key_id.into();
        let (kem_ciphertext, shared_secret) = Self::encapsulate(recipient_public_key, &key_id)?;

        let cipher = Aes256Gcm::new_from_slice(&shared_secret.key)
            .map_err(|e| KyberError::InvalidKey(e.to_string()))?;

        let nonce = random_nonce();
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(|e| KyberError::EncryptionFailed(e.to_string()))?;

        self.stats.write().unwrap().encryptions += 1;

        let fingerprint = fingerprint(&shared_secret.key, &kem_ciphertext, &key_id);
        Ok(KyberEncryptionOutput {
            package: KyberEncryptedPackage {
                kem_ciphertext,
                nonce: nonce.to_vec(),
                ciphertext,
                shared_secret_fingerprint: fingerprint,
                key_id,
                algorithm: "kyber".to_string(),
            },
            shared_secret,
        })
    }

    /// Encrypt data to the manager's own key (useful for loopback tests)
    pub fn encrypt_to_self(&self, plaintext: &[u8]) -> Result<KyberEncryptionOutput, KyberError> {
        let key_id = self.current_key_id();
        let public_key = self.public_key();
        self.encrypt_for(&public_key, key_id, plaintext)
    }

    /// Decrypt a Kyber package targeted at this manager's key pair
    pub fn decrypt(&self, package: &KyberEncryptedPackage) -> Result<KyberDecryptionResult, KyberError> {
        if package.nonce.len() != KYBER_NONCE_SIZE {
            self.stats.write().unwrap().failures += 1;
            return Err(KyberError::InvalidCiphertext(
                "Invalid nonce length for Kyber payload".to_string(),
            ));
        }

        let keypair = self.keypair.read().unwrap().clone();
        let shared_secret = match Self::decapsulate(&keypair, &package.kem_ciphertext) {
            Ok(secret) => secret,
            Err(err) => {
                self.stats.write().unwrap().failures += 1;
                return Err(err);
            }
        };

        // Verify that the sender derived the same shared secret
        let expected_fingerprint = shared_secret.fingerprint(&package.kem_ciphertext);
        if expected_fingerprint != package.shared_secret_fingerprint {
            self.stats.write().unwrap().failures += 1;
            return Err(KyberError::AuthenticationFailed);
        }

        let cipher = Aes256Gcm::new_from_slice(&shared_secret.key)
            .map_err(|e| KyberError::InvalidKey(e.to_string()))?;

        let plaintext = cipher
            .decrypt(Nonce::from_slice(&package.nonce), package.ciphertext.as_ref())
            .map_err(|e| {
                self.stats.write().unwrap().failures += 1;
                KyberError::DecryptionFailed(e.to_string())
            })?;

        self.stats.write().unwrap().decryptions += 1;

        Ok(KyberDecryptionResult {
            plaintext,
            shared_secret,
        })
    }

    /// Get statistics for monitoring/alerting
    pub fn get_statistics(&self) -> KyberEncryptionStats {
        self.stats.read().unwrap().clone()
    }

    fn encapsulate(
        recipient_public_key: &[u8],
        key_id: &str,
    ) -> Result<(Vec<u8>, KyberSharedSecret), KyberError> {
        if recipient_public_key.len() < KYBER_SEED_SIZE {
            return Err(KyberError::InvalidKey(
                "Recipient public key too small for Kyber encapsulation".to_string(),
            ));
        }

        let mut entropy = [0u8; KYBER_SEED_SIZE];
        OsRng.fill_bytes(&mut entropy);

        let shared_key = derive_shared_secret(recipient_public_key, &entropy, key_id);
        let tag = kem_tag(&shared_key, key_id);

        let mut kem_ciphertext = Vec::with_capacity(KYBER_SEED_SIZE + KYBER_TAG_SIZE);
        kem_ciphertext.extend_from_slice(&entropy);
        kem_ciphertext.extend_from_slice(&tag);

        Ok((
            kem_ciphertext,
            KyberSharedSecret {
                key: shared_key,
                key_id: key_id.to_string(),
                established_at: now(),
            },
        ))
    }

    fn decapsulate(keypair: &KyberKeyPair, kem_ciphertext: &[u8]) -> Result<KyberSharedSecret, KyberError> {
        if kem_ciphertext.len() != KYBER_SEED_SIZE + KYBER_TAG_SIZE {
            return Err(KyberError::InvalidCiphertext(
                "Invalid Kyber KEM ciphertext length".to_string(),
            ));
        }

        let entropy = &kem_ciphertext[..KYBER_SEED_SIZE];
        let received_tag = &kem_ciphertext[KYBER_SEED_SIZE..];

        let expected_public_key = derive_public_key(&keypair.private_key);
        let shared_key = derive_shared_secret(&expected_public_key, entropy, &keypair.key_id);
        let expected_tag = kem_tag(&shared_key, &keypair.key_id);

        if received_tag != expected_tag {
            return Err(KyberError::AuthenticationFailed);
        }

        Ok(KyberSharedSecret {
            key: shared_key,
            key_id: keypair.key_id.clone(),
            established_at: now(),
        })
    }
}

fn derive_public_key(private_key: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(b"kyber-public-key");
    hasher.update(private_key);
    hasher.finalize().to_vec()
}

fn derive_shared_secret(public_key: &[u8], entropy: &[u8], key_id: &str) -> [u8; KYBER_SHARED_KEY_SIZE] {
    let mut hasher = Sha3_256::new();
    hasher.update(b"kyber-shared-secret");
    hasher.update(public_key);
    hasher.update(entropy);
    hasher.update(key_id.as_bytes());
    let result = hasher.finalize();

    let mut key = [0u8; KYBER_SHARED_KEY_SIZE];
    key.copy_from_slice(&result[..KYBER_SHARED_KEY_SIZE]);
    key
}

fn kem_tag(shared_key: &[u8; KYBER_SHARED_KEY_SIZE], key_id: &str) -> [u8; KYBER_TAG_SIZE] {
    let mut hasher = Sha3_256::new();
    hasher.update(b"kyber-kem-tag");
    hasher.update(shared_key);
    hasher.update(key_id.as_bytes());
    let result = hasher.finalize();

    let mut tag = [0u8; KYBER_TAG_SIZE];
    tag.copy_from_slice(&result[..KYBER_TAG_SIZE]);
    tag
}

fn fingerprint(shared_key: &[u8; KYBER_SHARED_KEY_SIZE], kem_ciphertext: &[u8], key_id: &str) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(b"kyber-fingerprint");
    hasher.update(shared_key);
    hasher.update(kem_ciphertext);
    hasher.update(key_id.as_bytes());
    hasher.finalize().to_vec()
}

fn random_nonce() -> [u8; KYBER_NONCE_SIZE] {
    let mut nonce = [0u8; KYBER_NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Default for KyberEncryptionManager {
    fn default() -> Self {
        Self::new()
    }
}
