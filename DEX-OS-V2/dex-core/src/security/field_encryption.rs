//! Field Encryption for Security Layer 5 - Data Security
//!
//! Field-level encryption for granular data protection.

use crate::security::data_encryption::DataEncryptionManager as EncryptionManager;
use crate::security::security_manager::SecurityError;
use crate::security::security_manager::ClassificationLevel;
use crate::security::data_encryption::EncryptedData;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct FieldEncryptionManager {
    /// Keys for specific fields
    field_keys: Arc<RwLock<HashMap<String, [u8; 32]>>>,
    /// Set of fields that should be encrypted
    encrypted_fields: Arc<RwLock<HashSet<String>>>,
}

impl FieldEncryptionManager {
    pub fn new() -> Self {
        Self {
            field_keys: Arc::new(RwLock::new(HashMap::new())),
            encrypted_fields: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Register a field for encryption with a specific key
    pub fn register_field(&self, field_name: String, key: [u8; 32]) {
        let mut keys = self.field_keys.write().unwrap();
        keys.insert(field_name.clone(), key);
        
        let mut fields = self.encrypted_fields.write().unwrap();
        fields.insert(field_name);
    }

    /// Encrypt a value for a specific field
    pub fn encrypt_field(&self, field_name: &str, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let keys = self.field_keys.read().unwrap();
        let key_bytes = keys.get(field_name).ok_or_else(|| {
            SecurityError::EncryptionError(format!("No key registered for field: {}", field_name))
        })?;

        // Create a temporary encryption key
        let key = crate::security::data_encryption::EncryptionKey::new(
            field_name.to_string(),
            key_bytes.to_vec(),
            crate::security::data_encryption::EncryptionAlgorithm::Aes256Gcm,
        );
        
        // Create a temporary encryption manager with the field's key
        let manager = EncryptionManager::from_key(key);
        
        // Encrypt the data (using default classification for field level)
        // Note: In a real system we might want to pass classification here too
        let encrypted = manager.encrypt_data(
            data, 
            ClassificationLevel::Confidential
        ).map_err(SecurityError::from)?;  // Convert EncryptionError to SecurityError
        // Serialize the EncryptedData to bytes for storage
        bincode::serialize(&encrypted).map_err(|e| {
            SecurityError::EncryptionError(format!("Serialization failed: {}", e))
        })
    }

    /// Decrypt a value for a specific field
    pub fn decrypt_field(&self, field_name: &str, encrypted_data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let keys = self.field_keys.read().unwrap();
        let key_bytes = keys.get(field_name).ok_or_else(|| {
            SecurityError::EncryptionError(format!("No key registered for field: {}", field_name))
        })?;

        // Create a temporary encryption key
        let key = crate::security::data_encryption::EncryptionKey::new(
            field_name.to_string(),
            key_bytes.to_vec(),
            crate::security::data_encryption::EncryptionAlgorithm::Aes256Gcm,
        );

        // Deserialize the EncryptedData
        let encrypted_struct: EncryptedData = bincode::deserialize(encrypted_data).map_err(|e| {
            SecurityError::EncryptionError(format!("Deserialization failed: {}", e))
        })?;
        
        // Create a temporary encryption manager with the field's key
        let manager = EncryptionManager::from_key(key);
        
        // Decrypt and convert EncryptionError to SecurityError
        manager.decrypt_data(&encrypted_struct).map_err(SecurityError::from)
    }

    /// Check if a field is registered for encryption
    pub fn is_encrypted_field(&self, field_name: &str) -> bool {
        let fields = self.encrypted_fields.read().unwrap();
        fields.contains(field_name)
    }
}

impl Default for FieldEncryptionManager {
    fn default() -> Self {
        Self::new()
    }
}
