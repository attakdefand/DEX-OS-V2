//! Tokenization for Security Layer 5 - Data Security
//!
//! Tokenization for sensitive data (PII, payment info).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenDataType {
    CreditCard,
    SSN,
    Email,
    PhoneNumber,
    Custom(String),
}

#[derive(Debug, Clone)]
struct TokenEntry {
    token: String,
    original_data: Vec<u8>,
    data_type: TokenDataType,
    created_at: u64,
    expires_at: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct TokenizationManager {
    token_vault: Arc<RwLock<HashMap<String, TokenEntry>>>,
    default_ttl: Option<u64>,
}

impl TokenizationManager {
    pub fn new(default_ttl: Option<u64>) -> Self {
        Self {
            token_vault: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// Tokenize data with optional custom TTL
    pub fn tokenize(&self, data: &[u8], data_type: TokenDataType, ttl: Option<u64>) -> String {
        let token = format!("tok_{}", uuid::Uuid::new_v4());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires_at = ttl.or(self.default_ttl).map(|duration| now + duration);

        let entry = TokenEntry {
            token: token.clone(),
            original_data: data.to_vec(),
            data_type,
            created_at: now,
            expires_at,
        };

        let mut vault = self.token_vault.write().unwrap();
        vault.insert(token.clone(), entry);

        token
    }

    /// Retrieve original data from token if valid
    pub fn detokenize(&self, token: &str) -> Option<Vec<u8>> {
        let vault = self.token_vault.read().unwrap();
        
        if let Some(entry) = vault.get(token) {
            // Check expiration
            if let Some(expires_at) = entry.expires_at {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                if now >= expires_at {
                    return None;
                }
            }
            
            Some(entry.original_data.clone())
        } else {
            None
        }
    }

    /// Remove a token manually
    pub fn revoke_token(&self, token: &str) -> bool {
        let mut vault = self.token_vault.write().unwrap();
        vault.remove(token).is_some()
    }

    /// Cleanup expired tokens
    pub fn cleanup_expired_tokens(&self) -> usize {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut vault = self.token_vault.write().unwrap();
        let initial_len = vault.len();
        
        vault.retain(|_, entry| {
            if let Some(expires_at) = entry.expires_at {
                now < expires_at
            } else {
                true
            }
        });

        initial_len - vault.len()
    }
}

impl Default for TokenizationManager {
    fn default() -> Self {
        Self::new(Some(3600)) // Default 1 hour TTL
    }
}
