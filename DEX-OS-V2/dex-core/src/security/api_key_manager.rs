//! API Key Manager for Security Layer 4 - API & Gateway Security
//!
//! Manages API key generation, validation, rotation, and lifecycle.
//! From DEX-OS-V2.csv line 238:
//! - Security,Security Layer,Security Layer 4,API & Gateway Security,API Protection,High

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// API key manager error types
#[derive(Debug, Error, Clone, PartialEq)]
pub enum APIKeyError {
    #[error("API key not found: {0}")]
    KeyNotFound(String),
    #[error("API key already exists: {0}")]
    KeyAlreadyExists(String),
    #[error("API key expired: {0}")]
    KeyExpired(String),
    #[error("API key disabled: {0}")]
    KeyDisabled(String),
    #[error("Invalid API key format: {0}")]
    InvalidKeyFormat(String),
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
}

/// API Key structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APIKey {
    /// Unique key ID
    pub id: String,
    /// SHA-256 hash of the actual key (for security)
    pub key_hash: String,
    /// Client ID associated with this key
    pub client_id: String,
    /// Scopes/permissions for this key
    pub scopes: Vec<String>,
    /// Key creation timestamp
    pub created_at: u64,
    /// Key expiration timestamp (None = never expires)
    pub expires_at: Option<u64>,
    /// Last time the key was used
    pub last_used: Option<u64>,
    /// Whether the key is enabled
    pub enabled: bool,
    /// Key metadata
    pub metadata: HashMap<String, String>,
}

impl APIKey {
    pub fn new(
        id: String,
        key_hash: String,
        client_id: String,
        scopes: Vec<String>,
        expires_at: Option<u64>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            key_hash,
            client_id,
            scopes,
            created_at: now,
            expires_at,
            last_used: None,
            enabled: true,
            metadata: HashMap::new(),
        }
    }

    /// Check if the key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now >= expires_at
        } else {
            false
        }
    }

    /// Check if the key has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(&scope.to_string())
    }

    /// Update last used timestamp
    pub fn update_last_used(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_used = Some(now);
    }
}

/// API Key Manager
#[derive(Debug, Clone)]
pub struct APIKeyManager {
    /// API keys indexed by key ID
    keys: Arc<RwLock<HashMap<String, APIKey>>>,
    /// Key hash to key ID mapping (for fast lookup)
    key_index: Arc<RwLock<HashMap<String, String>>>,
    /// Default key expiration in seconds (None = never expires)
    default_expiration: Option<u64>,
}

impl APIKeyManager {
    /// Create a new API key manager
    pub fn new(default_expiration: Option<u64>) -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            key_index: Arc::new(RwLock::new(HashMap::new())),
            default_expiration,
        }
    }

    /// Generate a new API key
    pub fn generate_key(
        &self,
        client_id: String,
        scopes: Vec<String>,
        expires_at: Option<u64>,
    ) -> Result<(String, String), APIKeyError> {
        // Generate random key
        let key = self.generate_random_key();
        let key_hash = self.hash_key(&key);
        let key_id = format!("key_{}", uuid::Uuid::new_v4());

        // Calculate expiration
        let final_expiration = expires_at.or_else(|| {
            self.default_expiration.map(|duration| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + duration
            })
        });

        // Create API key
        let api_key = APIKey::new(key_id.clone(), key_hash.clone(), client_id, scopes, final_expiration);

        // Store key
        let mut keys = self.keys.write().unwrap();
        if keys.contains_key(&key_id) {
            return Err(APIKeyError::KeyAlreadyExists(key_id));
        }
        keys.insert(key_id.clone(), api_key);

        // Update index
        let mut index = self.key_index.write().unwrap();
        index.insert(key_hash, key_id.clone());

        Ok((key_id, key))
    }

    /// Validate an API key
    pub fn validate_key(&self, key: &str) -> Result<APIKey, APIKeyError> {
        let key_hash = self.hash_key(key);

        // Find key ID from hash
        let index = self.key_index.read().unwrap();
        let key_id = index
            .get(&key_hash)
            .ok_or_else(|| APIKeyError::KeyNotFound("Invalid key".to_string()))?;

        // Get API key
        let mut keys = self.keys.write().unwrap();
        let mut api_key = keys
            .get_mut(key_id)
            .ok_or_else(|| APIKeyError::KeyNotFound(key_id.clone()))?
            .clone();

        // Check if enabled
        if !api_key.enabled {
            return Err(APIKeyError::KeyDisabled(key_id.clone()));
        }

        // Check if expired
        if api_key.is_expired() {
            return Err(APIKeyError::KeyExpired(key_id.clone()));
        }

        // Update last used
        api_key.update_last_used();
        keys.insert(key_id.clone(), api_key.clone());

        Ok(api_key)
    }

    /// Validate key and check scope
    pub fn validate_key_with_scope(&self, key: &str, required_scope: &str) -> Result<APIKey, APIKeyError> {
        let api_key = self.validate_key(key)?;

        if !api_key.has_scope(required_scope) {
            return Err(APIKeyError::InsufficientPermissions(format!(
                "Key does not have scope: {}",
                required_scope
            )));
        }

        Ok(api_key)
    }

    /// Revoke an API key
    pub fn revoke_key(&self, key_id: &str) -> Result<(), APIKeyError> {
        let mut keys = self.keys.write().unwrap();
        let api_key = keys
            .get_mut(key_id)
            .ok_or_else(|| APIKeyError::KeyNotFound(key_id.to_string()))?;

        api_key.enabled = false;
        Ok(())
    }

    /// Delete an API key
    pub fn delete_key(&self, key_id: &str) -> Result<(), APIKeyError> {
        let mut keys = self.keys.write().unwrap();
        let api_key = keys
            .remove(key_id)
            .ok_or_else(|| APIKeyError::KeyNotFound(key_id.to_string()))?;

        // Remove from index
        let mut index = self.key_index.write().unwrap();
        index.remove(&api_key.key_hash);

        Ok(())
    }

    /// Get an API key by ID
    pub fn get_key(&self, key_id: &str) -> Result<APIKey, APIKeyError> {
        let keys = self.keys.read().unwrap();
        keys.get(key_id)
            .cloned()
            .ok_or_else(|| APIKeyError::KeyNotFound(key_id.to_string()))
    }

    /// List all keys for a client
    pub fn list_client_keys(&self, client_id: &str) -> Vec<APIKey> {
        let keys = self.keys.read().unwrap();
        keys.values()
            .filter(|k| k.client_id == client_id)
            .cloned()
            .collect()
    }

    /// Rotate a key (generate new key, disable old one)
    pub fn rotate_key(&self, key_id: &str) -> Result<(String, String), APIKeyError> {
        let old_key = self.get_key(key_id)?;

        // Generate new key with same properties
        let (new_key_id, new_key) = self.generate_key(
            old_key.client_id.clone(),
            old_key.scopes.clone(),
            old_key.expires_at,
        )?;

        // Disable old key
        self.revoke_key(key_id)?;

        Ok((new_key_id, new_key))
    }

    /// Cleanup expired keys
    pub fn cleanup_expired_keys(&self) -> usize {
        let mut keys = self.keys.write().unwrap();
        let mut index = self.key_index.write().unwrap();

        let expired_keys: Vec<String> = keys
            .iter()
            .filter(|(_, k)| k.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        for key_id in &expired_keys {
            if let Some(key) = keys.remove(key_id) {
                index.remove(&key.key_hash);
            }
        }

        expired_keys.len()
    }

    /// Get statistics
    pub fn get_statistics(&self) -> APIKeyStatistics {
        let keys = self.keys.read().unwrap();

        let total_keys = keys.len();
        let active_keys = keys.values().filter(|k| k.enabled && !k.is_expired()).count();
        let expired_keys = keys.values().filter(|k| k.is_expired()).count();
        let disabled_keys = keys.values().filter(|k| !k.enabled).count();

        APIKeyStatistics {
            total_keys,
            active_keys,
            expired_keys,
            disabled_keys,
        }
    }

    // Helper methods

    fn generate_random_key(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const KEY_LEN: usize = 32;

        let mut rng = rand::thread_rng();
        let key: String = (0..KEY_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        format!("dex_{}", key)
    }

    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(key.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }
}

/// API key statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APIKeyStatistics {
    pub total_keys: usize,
    pub active_keys: usize,
    pub expired_keys: usize,
    pub disabled_keys: usize,
}

impl Default for APIKeyManager {
    fn default() -> Self {
        Self::new(Some(365 * 24 * 3600)) // 1 year default expiration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_creation() {
        let manager = APIKeyManager::new(Some(3600));
        let (key_id, key) = manager
            .generate_key("client1".to_string(), vec!["read".to_string()], None)
            .unwrap();

        assert!(key_id.starts_with("key_"));
        assert!(key.starts_with("dex_"));
    }

    #[test]
    fn test_api_key_validation() {
        let manager = APIKeyManager::new(None);
        let (_, key) = manager
            .generate_key("client1".to_string(), vec!["read".to_string()], None)
            .unwrap();

        let api_key = manager.validate_key(&key).unwrap();
        assert_eq!(api_key.client_id, "client1");
        assert!(api_key.has_scope("read"));
    }

    #[test]
    fn test_api_key_expiration() {
        let manager = APIKeyManager::new(None);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create key that expires in the past
        let (_, key) = manager
            .generate_key("client1".to_string(), vec!["read".to_string()], Some(now - 1))
            .unwrap();

        // Should fail validation
        let result = manager.validate_key(&key);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), APIKeyError::KeyExpired(_)));
    }

    #[test]
    fn test_api_key_revocation() {
        let manager = APIKeyManager::new(None);
        let (key_id, key) = manager
            .generate_key("client1".to_string(), vec!["read".to_string()], None)
            .unwrap();

        // Revoke key
        manager.revoke_key(&key_id).unwrap();

        // Should fail validation
        let result = manager.validate_key(&key);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), APIKeyError::KeyDisabled(_)));
    }

    #[test]
    fn test_api_key_scope_validation() {
        let manager = APIKeyManager::new(None);
        let (_, key) = manager
            .generate_key("client1".to_string(), vec!["read".to_string()], None)
            .unwrap();

        // Should succeed for "read" scope
        assert!(manager.validate_key_with_scope(&key, "read").is_ok());

        // Should fail for "write" scope
        let result = manager.validate_key_with_scope(&key, "write");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            APIKeyError::InsufficientPermissions(_)
        ));
    }

    #[test]
    fn test_api_key_rotation() {
        let manager = APIKeyManager::new(None);
        let (old_key_id, old_key) = manager
            .generate_key("client1".to_string(), vec!["read".to_string()], None)
            .unwrap();

        // Rotate key
        let (new_key_id, new_key) = manager.rotate_key(&old_key_id).unwrap();

        // Old key should be disabled
        let result = manager.validate_key(&old_key);
        assert!(result.is_err());

        // New key should work
        let api_key = manager.validate_key(&new_key).unwrap();
        assert_eq!(api_key.id, new_key_id);
        assert_eq!(api_key.client_id, "client1");
    }

    #[test]
    fn test_list_client_keys() {
        let manager = APIKeyManager::new(None);

        manager
            .generate_key("client1".to_string(), vec!["read".to_string()], None)
            .unwrap();
        manager
            .generate_key("client1".to_string(), vec!["write".to_string()], None)
            .unwrap();
        manager
            .generate_key("client2".to_string(), vec!["read".to_string()], None)
            .unwrap();

        let client1_keys = manager.list_client_keys("client1");
        assert_eq!(client1_keys.len(), 2);

        let client2_keys = manager.list_client_keys("client2");
        assert_eq!(client2_keys.len(), 1);
    }

    #[test]
    fn test_cleanup_expired_keys() {
        let manager = APIKeyManager::new(None);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create expired key
        manager
            .generate_key("client1".to_string(), vec!["read".to_string()], Some(now - 1))
            .unwrap();

        // Create valid key
        manager
            .generate_key("client2".to_string(), vec!["read".to_string()], None)
            .unwrap();

        let cleaned = manager.cleanup_expired_keys();
        assert_eq!(cleaned, 1);

        let stats = manager.get_statistics();
        assert_eq!(stats.total_keys, 1);
    }
}
