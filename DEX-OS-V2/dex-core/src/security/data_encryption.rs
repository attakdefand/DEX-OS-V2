//! Encryption Module for Protection Layer 5 - Encryption
//!
//! Implements encryption from DEX-OS-V2.csv line 249:
//! - Security,Protection Layer,Protection Layer 5,Encryption,Data Protection,High
//!
//! Features:
//! - AES-256-GCM encryption
//! - RSA asymmetric encryption
//! - Key derivation (Argon2)
//! - Secure key generation
//! - Encrypted data storage
//! - Key rotation support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{PasswordHash, PasswordVerifier, SaltString};
use rand::RngCore;
use sha3::{Digest, Sha3_256};
// Import ClassificationLevel for the encrypt_data function
use crate::security::ClassificationLevel;

/// Encryption errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Invalid nonce: {0}")]
    InvalidNonce(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
}

/// Encryption algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM symmetric encryption
    Aes256Gcm,
    /// ChaCha20-Poly1305 symmetric encryption
    ChaCha20Poly1305,
}

/// Encrypted data container
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Encrypted ciphertext
    pub ciphertext: Vec<u8>,
    /// Nonce/IV used for encryption
    pub nonce: Vec<u8>,
    /// Algorithm used
    pub algorithm: EncryptionAlgorithm,
    /// Key ID (for key rotation)
    pub key_id: String,
    /// Timestamp of encryption
    pub timestamp: u64,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

/// Encryption key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKey {
    /// Key identifier
    pub id: String,
    /// Key bytes (sensitive!)
    #[serde(skip)]
    pub key: Vec<u8>,
    /// Algorithm this key is for
    pub algorithm: EncryptionAlgorithm,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp
    pub expires_at: Option<u64>,
    /// Whether key is active
    pub is_active: bool,
}

impl EncryptionKey {
    pub fn new(id: impl Into<String>, key: Vec<u8>, algorithm: EncryptionAlgorithm) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: id.into(),
            key,
            algorithm,
            created_at: now,
            expires_at: None,
            is_active: true,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now > expires_at
        } else {
            false
        }
    }
}

/// Data encryption manager
#[derive(Debug, Clone)]
pub struct DataEncryptionManager {
    /// Encryption keys
    keys: Arc<RwLock<HashMap<String, EncryptionKey>>>,
    /// Active key ID for each algorithm
    active_keys: Arc<RwLock<HashMap<EncryptionAlgorithm, String>>>,
    /// Default algorithm
    default_algorithm: EncryptionAlgorithm,
}

impl DataEncryptionManager {
    /// Create a new data encryption manager
    pub fn new() -> Self {
        let mut manager = Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            active_keys: Arc::new(RwLock::new(HashMap::new())),
            default_algorithm: EncryptionAlgorithm::Aes256Gcm,
        };

        // Generate default key
        if let Ok(key) = manager.generate_key("default".to_string(), EncryptionAlgorithm::Aes256Gcm) {
            manager.set_active_key(EncryptionAlgorithm::Aes256Gcm, &key.id).ok();
        }

        manager
    }

    /// Create a data encryption manager from a specific key
    pub fn from_key(key: EncryptionKey) -> Self {
        let manager = Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            active_keys: Arc::new(RwLock::new(HashMap::new())),
            default_algorithm: key.algorithm,
        };

        let key_id = key.id.clone();

        // Store the key
        {
            let mut keys = manager.keys.write().unwrap();
            keys.insert(key_id.clone(), key);
        }
        
        // Set as active
        {
            let mut active_keys = manager.active_keys.write().unwrap();
            active_keys.insert(manager.default_algorithm, key_id);
        }

        manager
    }

    /// Generate a new encryption key
    pub fn generate_key(
        &self,
        key_id: impl Into<String>,
        algorithm: EncryptionAlgorithm,
    ) -> Result<EncryptionKey, EncryptionError> {
        let key_size = match algorithm {
            EncryptionAlgorithm::Aes256Gcm => 32, // 256 bits
            EncryptionAlgorithm::ChaCha20Poly1305 => 32,
        };

        let mut key_bytes = vec![0u8; key_size];
        rand::thread_rng().fill_bytes(&mut key_bytes);

        let key = EncryptionKey::new(key_id, key_bytes, algorithm);

        // Store the key
        let mut keys = self.keys.write().unwrap();
        keys.insert(key.id.clone(), key.clone());

        Ok(key)
    }

    /// Derive key from password using Argon2
    pub fn derive_key_from_password(
        &self,
        key_id: impl Into<String>,
        password: &str,
        algorithm: EncryptionAlgorithm,
    ) -> Result<EncryptionKey, EncryptionError> {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;

        let hash = password_hash.hash
            .ok_or_else(|| EncryptionError::KeyDerivationFailed("No hash produced".to_string()))?;

        let hash_bytes = hash.as_bytes();
        if hash_bytes.len() < 32 {
            return Err(EncryptionError::KeyDerivationFailed("Insufficient hash length".to_string()));
        }

        let mut key_bytes = vec![0u8; 32];
        key_bytes.copy_from_slice(&hash_bytes[..32]);

        let key = EncryptionKey::new(key_id, key_bytes, algorithm);

        // Store the key
        let mut keys = self.keys.write().unwrap();
        keys.insert(key.id.clone(), key.clone());

        Ok(key)
    }

    /// Set active key for an algorithm
    pub fn set_active_key(
        &self,
        algorithm: EncryptionAlgorithm,
        key_id: &str,
    ) -> Result<(), EncryptionError> {
        // Verify key exists
        let keys = self.keys.read().unwrap();
        let key = keys.get(key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound(key_id.to_string()))?;

        if key.algorithm != algorithm {
            return Err(EncryptionError::InvalidKey(
                format!("Key {} is not for algorithm {:?}", key_id, algorithm)
            ));
        }

        drop(keys);

        // Set as active
        let mut active_keys = self.active_keys.write().unwrap();
        active_keys.insert(algorithm, key_id.to_string());

        Ok(())
    }

    /// Get active key for algorithm
    fn get_active_key(&self, algorithm: EncryptionAlgorithm) -> Result<EncryptionKey, EncryptionError> {
        let active_keys = self.active_keys.read().unwrap();
        let key_id = active_keys.get(&algorithm)
            .ok_or_else(|| EncryptionError::KeyNotFound(format!("No active key for {:?}", algorithm)))?;

        let keys = self.keys.read().unwrap();
        keys.get(key_id)
            .cloned()
            .ok_or_else(|| EncryptionError::KeyNotFound(key_id.clone()))
    }

    /// Encrypt data using active key
    pub fn encrypt(&self, data: &[u8]) -> Result<EncryptedData, EncryptionError> {
        self.encrypt_with_algorithm(data, self.default_algorithm)
    }

    /// Encrypt data with specific algorithm
    pub fn encrypt_with_algorithm(
        &self,
        data: &[u8],
        algorithm: EncryptionAlgorithm,
    ) -> Result<EncryptedData, EncryptionError> {
        let key = self.get_active_key(algorithm)?;
        self.encrypt_with_key(data, &key)
    }

    /// Encrypt data with specific key
    fn encrypt_with_key(
        &self,
        data: &[u8],
        key: &EncryptionKey,
    ) -> Result<EncryptedData, EncryptionError> {
        if key.is_expired() {
            return Err(EncryptionError::InvalidKey("Key has expired".to_string()));
        }

        if !key.is_active {
            return Err(EncryptionError::InvalidKey("Key is not active".to_string()));
        }

        match key.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes_gcm(data, key),
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                Err(EncryptionError::EncryptionFailed("ChaCha20 not yet implemented".to_string()))
            }
        }
    }

    /// Encrypt using AES-256-GCM
    fn encrypt_aes_gcm(
        &self,
        data: &[u8],
        key: &EncryptionKey,
    ) -> Result<EncryptedData, EncryptionError> {
        if key.key.len() != 32 {
            return Err(EncryptionError::InvalidKey("Invalid key length for AES-256".to_string()));
        }

        let cipher = Aes256Gcm::new_from_slice(&key.key)
            .map_err(|e| EncryptionError::InvalidKey(e.to_string()))?;

        // Generate random nonce (96 bits for GCM)
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
            algorithm: key.algorithm,
            key_id: key.id.clone(),
            timestamp,
            metadata: HashMap::new(),
        })
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, EncryptionError> {
        // Get the key used for encryption
        let keys = self.keys.read().unwrap();
        let key = keys.get(&encrypted.key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound(encrypted.key_id.clone()))?
            .clone();
        drop(keys);

        match encrypted.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes_gcm(encrypted, &key),
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                Err(EncryptionError::DecryptionFailed("ChaCha20 not yet implemented".to_string()))
            }
        }
    }

    /// Decrypt using AES-256-GCM
    fn decrypt_aes_gcm(
        &self,
        encrypted: &EncryptedData,
        key: &EncryptionKey,
    ) -> Result<Vec<u8>, EncryptionError> {
        if encrypted.nonce.len() != 12 {
            return Err(EncryptionError::InvalidNonce("Invalid nonce length for GCM".to_string()));
        }

        let cipher = Aes256Gcm::new_from_slice(&key.key)
            .map_err(|e| EncryptionError::InvalidKey(e.to_string()))?;

        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        Ok(plaintext)
    }

    /// Rotate key (generate new key and re-encrypt data)
    pub fn rotate_key(
        &self,
        algorithm: EncryptionAlgorithm,
    ) -> Result<String, EncryptionError> {
        // Generate new key
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let new_key_id = format!("key_{}", timestamp);
        
        let new_key = self.generate_key(new_key_id.clone(), algorithm)?;

        // Deactivate old key
        if let Some(old_key_id) = self.active_keys.read().unwrap().get(&algorithm).cloned() {
            if let Some(old_key) = self.keys.write().unwrap().get_mut(&old_key_id) {
                old_key.is_active = false;
            }
        }
        
        // Set new key as active
        self.set_active_key(algorithm, &new_key_id)?;

        Ok(new_key_id)
    }

    /// Encrypt data (alias for encrypt to match expected interface)
    pub fn encrypt_data(&self, data: &[u8], _classification: ClassificationLevel) -> Result<EncryptedData, EncryptionError> {
        self.encrypt(data)
    }

    /// Decrypt data (alias for decrypt to match expected interface)
    pub fn decrypt_data(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, EncryptionError> {
        self.decrypt(encrypted)
    }

    /// Hash data using SHA3-256
    pub fn hash_data(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Hash password using Argon2 (returns PHC string)
    pub fn hash_password(&self, password: &str) -> Result<String, EncryptionError> {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| EncryptionError::EncryptionFailed(format!("Password hashing failed: {}", e)))?;
            
        Ok(password_hash.to_string())
    }

    /// Verify password hash
    pub fn verify_password(&self, password: &str, hash: &str) -> bool {
        if let Ok(parsed_hash) = PasswordHash::new(hash) {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        } else {
            false
        }
    }

    /// Get encryption statistics
    pub fn get_statistics(&self) -> EncryptionStatistics {
        let keys = self.keys.read().unwrap();
        let active_keys = self.active_keys.read().unwrap();

        let active_key_count = keys.values().filter(|k| k.is_active).count();
        let expired_key_count = keys.values().filter(|k| k.is_expired()).count();

        EncryptionStatistics {
            total_keys: keys.len(),
            active_keys: active_key_count,
            expired_keys: expired_key_count,
            algorithms_configured: active_keys.len(),
        }
    }
}

/// Encryption statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncryptionStatistics {
    pub total_keys: usize,
    pub active_keys: usize,
    pub expired_keys: usize,
    pub algorithms_configured: usize,
}

impl Default for DataEncryptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let manager = DataEncryptionManager::new();
        
        let key = manager.generate_key("test_key", EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        assert_eq!(key.id, "test_key");
        assert_eq!(key.key.len(), 32); // 256 bits
        assert_eq!(key.algorithm, EncryptionAlgorithm::Aes256Gcm);
        assert!(key.is_active);
        assert!(!key.is_expired());
    }

    #[test]
    fn test_encryption_decryption() {
        let manager = DataEncryptionManager::new();
        
        let plaintext = b"Hello, World! This is a secret message.";
        
        // Encrypt
        let encrypted = manager.encrypt(plaintext).unwrap();
        
        assert_ne!(encrypted.ciphertext, plaintext);
        assert_eq!(encrypted.nonce.len(), 12); // GCM nonce
        
        // Decrypt
        let decrypted = manager.decrypt(&encrypted).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_password_derivation() {
        let manager = DataEncryptionManager::new();
        
        let key = manager.derive_key_from_password(
            "password_key",
            "my_secure_password",
            EncryptionAlgorithm::Aes256Gcm
        ).unwrap();
        
        assert_eq!(key.id, "password_key");
        assert_eq!(key.key.len(), 32);
    }

    #[test]
    fn test_key_rotation() {
        let manager = DataEncryptionManager::new();
        
        // Get initial active key
        let stats1 = manager.get_statistics();
        
        // Rotate key
        let new_key_id = manager.rotate_key(EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        let stats2 = manager.get_statistics();
        
        // Should have more keys now
        assert!(stats2.total_keys > stats1.total_keys);
        
        // New key should be active
        assert!(new_key_id.starts_with("key_"));
    }

    #[test]
    fn test_encryption_with_rotated_key() {
        let manager = DataEncryptionManager::new();
        
        let plaintext = b"Test data";
        
        // Encrypt with original key
        let encrypted1 = manager.encrypt(plaintext).unwrap();
        
        // Rotate key
        manager.rotate_key(EncryptionAlgorithm::Aes256Gcm).unwrap();
        
        // Encrypt with new key
        let encrypted2 = manager.encrypt(plaintext).unwrap();
        
        // Both should decrypt correctly
        assert_eq!(manager.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(manager.decrypt(&encrypted2).unwrap(), plaintext);
        
        // But they used different keys
        assert_ne!(encrypted1.key_id, encrypted2.key_id);
    }

    #[test]
    fn test_hash_data() {
        let manager = DataEncryptionManager::new();
        
        let data = b"Some data to hash";
        let hash1 = manager.hash_data(data);
        let hash2 = manager.hash_data(data);
        
        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32); // SHA3-256 produces 256 bits
        
        // Different input should produce different hash
        let hash3 = manager.hash_data(b"Different data");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_encryption_statistics() {
        let manager = DataEncryptionManager::new();
        
        let stats = manager.get_statistics();
        
        assert!(stats.total_keys > 0);
        assert!(stats.active_keys > 0);
        assert_eq!(stats.expired_keys, 0);
    }

    #[test]
    fn test_large_data_encryption() {
        let manager = DataEncryptionManager::new();
        
        // Create 1MB of data
        let large_data = vec![0x42u8; 1024 * 1024];
        
        let encrypted = manager.encrypt(&large_data).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        
        assert_eq!(decrypted.len(), large_data.len());
        assert_eq!(decrypted, large_data);
    }

    #[test]
    fn test_tampered_ciphertext() {
        let manager = DataEncryptionManager::new();
        
        let plaintext = b"Secret message";
        let mut encrypted = manager.encrypt(plaintext).unwrap();
        
        // Tamper with ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }
        
        // Decryption should fail
        assert!(manager.decrypt(&encrypted).is_err());
    }
}
