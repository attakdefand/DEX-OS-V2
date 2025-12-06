//! Comprehensive test suite for Security Layer 5 - Data Security
//!
//! Tests all components of the data security system including:
//! - Data Classification (policies, labeling, access control)
//! - Field Encryption (key management, encryption/decryption)
//! - Tokenization (creation, detokenization, expiration)
//! - Key Rotation (generation, rotation, versioning)

use dex_core::security::{
    DataClassificationManager, 
    ClassificationLevel,
    FieldEncryptionManager,
    TokenizationManager,
    TokenDataType,
    DataKeyRotationManager,
};

use std::thread;
use std::time::Duration;

// ============================================================================
// DATA CLASSIFICATION TESTS
// ============================================================================

#[test]
fn test_classification_policies() {
    let manager = DataClassificationManager::new();
    
    // Check default policies
    let public_policy = manager.get_policy(&ClassificationLevel::Public).unwrap();
    assert!(!public_policy.encryption_required);
    
    let secret_policy = manager.get_policy(&ClassificationLevel::Secret).unwrap();
    assert!(secret_policy.encryption_required);
    assert!(secret_policy.access_roles.contains(&"admin".to_string()));
}

#[test]
fn test_data_labeling() {
    let manager = DataClassificationManager::new();
    
    let label = manager.classify_data(
        "user_123".to_string(),
        ClassificationLevel::Confidential,
        "system".to_string()
    );
    
    assert_eq!(label.id, "user_123");
    assert_eq!(label.classification, ClassificationLevel::Confidential);
    
    // Retrieve label
    let retrieved = manager.get_label("user_123").unwrap();
    assert_eq!(retrieved.classification, ClassificationLevel::Confidential);
}

#[test]
fn test_access_control() {
    let manager = DataClassificationManager::new();
    
    // Public data - everyone should access
    assert!(manager.check_access(&ClassificationLevel::Public, &vec![]));
    assert!(manager.check_access(&ClassificationLevel::Public, &vec!["admin".to_string()]));
    
    // Confidential data - only employees and managers
    assert!(manager.check_access(&ClassificationLevel::Confidential, &vec!["employee".to_string()]));
    assert!(manager.check_access(&ClassificationLevel::Confidential, &vec!["manager".to_string()]));
    assert!(!manager.check_access(&ClassificationLevel::Confidential, &vec!["intern".to_string()]));
    
    // Secret data - only admin and security officer
    assert!(manager.check_access(&ClassificationLevel::Secret, &vec!["admin".to_string()]));
    assert!(!manager.check_access(&ClassificationLevel::Secret, &vec!["employee".to_string()]));
}

#[test]
fn test_data_item_access() {
    let manager = DataClassificationManager::new();
    
    manager.classify_data(
        "doc_secret".to_string(),
        ClassificationLevel::Secret,
        "admin".to_string()
    );
    
    assert!(manager.check_data_access("doc_secret", &vec!["admin".to_string()]));
    assert!(!manager.check_data_access("doc_secret", &vec!["employee".to_string()]));
    
    // Non-existent data -> deny
    assert!(!manager.check_data_access("missing_doc", &vec!["admin".to_string()]));
}

// ============================================================================
// FIELD ENCRYPTION TESTS
// ============================================================================

#[test]
fn test_field_encryption_registration() {
    let manager = FieldEncryptionManager::new();
    let key = [0u8; 32]; // In real usage, use random key
    
    manager.register_field("ssn".to_string(), key);
    assert!(manager.is_encrypted_field("ssn"));
    assert!(!manager.is_encrypted_field("email"));
}

#[test]
fn test_field_encryption_decryption() {
    let manager = FieldEncryptionManager::new();
    let key = [1u8; 32]; // Dummy key
    
    manager.register_field("credit_card".to_string(), key);
    
    let data = b"1234-5678-9012-3456";
    
    // Encrypt
    let encrypted = manager.encrypt_field("credit_card", data).unwrap();
    assert_ne!(encrypted, data);
    
    // Decrypt
    let decrypted = manager.decrypt_field("credit_card", &encrypted).unwrap();
    assert_eq!(decrypted, data);
}

#[test]
fn test_field_encryption_missing_key() {
    let manager = FieldEncryptionManager::new();
    let data = b"test";
    
    // Try to encrypt unregistered field
    let result = manager.encrypt_field("unknown", data);
    assert!(result.is_err());
}

// ============================================================================
// TOKENIZATION TESTS
// ============================================================================

#[test]
fn test_tokenization_cycle() {
    let manager = TokenizationManager::new(None);
    let data = b"sensitive_info";
    
    // Tokenize
    let token = manager.tokenize(data, TokenDataType::Custom("secret".to_string()), None);
    assert!(token.starts_with("tok_"));
    
    // Detokenize
    let retrieved = manager.detokenize(&token).unwrap();
    assert_eq!(retrieved, data);
}

#[test]
fn test_token_expiration() {
    let manager = TokenizationManager::new(None);
    let data = b"temp_data";
    
    // Token with 1 second TTL (using small TTL for test might be flaky, but we can mock time or sleep)
    // Since we can't easily mock SystemTime in this setup, we'll rely on the logic being correct
    // or use a slightly longer sleep if needed.
    // Let's use a very short TTL and sleep.
    
    let token = manager.tokenize(data, TokenDataType::Custom("temp".to_string()), Some(1));
    
    // Should be valid immediately
    assert!(manager.detokenize(&token).is_some());
    
    // Wait for expiration
    thread::sleep(Duration::from_secs(2));
    
    // Should be invalid
    assert!(manager.detokenize(&token).is_none());
}

#[test]
fn test_token_cleanup() {
    let manager = TokenizationManager::new(None);
    
    // Create expired token
    manager.tokenize(b"expired", TokenDataType::Custom("test".to_string()), Some(1));
    
    // Create valid token
    manager.tokenize(b"valid", TokenDataType::Custom("test".to_string()), Some(3600));
    
    thread::sleep(Duration::from_secs(2));
    
    let cleaned = manager.cleanup_expired_tokens();
    assert_eq!(cleaned, 1);
    
    // One token should remain (internal implementation detail, but we can check via detokenize)
    // We can't easily check count without exposing it, but cleanup returning 1 confirms it found one.
}

#[test]
fn test_token_revocation() {
    let manager = TokenizationManager::new(None);
    let token = manager.tokenize(b"data", TokenDataType::Custom("test".to_string()), None);
    
    assert!(manager.revoke_token(&token));
    assert!(manager.detokenize(&token).is_none());
}

// ============================================================================
// KEY ROTATION TESTS
// ============================================================================

#[test]
fn test_key_generation() {
    let mut manager = DataKeyRotationManager::new(90);
    
    let key = manager.get_current_key();
    assert!(key.is_some());
    
    // Check key size
    assert_eq!(key.unwrap().len(), 32); // Key size is 32 bytes
}

#[test]
fn test_key_rotation() {
    let mut manager = DataKeyRotationManager::new(90);
    
    // Get initial key
    let key_v1 = *manager.get_current_key().unwrap();
    
    // Rotate key
    let new_version = manager.rotate_key();
    assert_eq!(new_version, 2); // Should be version 2
    
    let key_v2 = *manager.get_current_key().unwrap();
    
    assert_ne!(key_v1, key_v2);
}

#[test]
fn test_rotation_schedule() {
    let manager = DataKeyRotationManager::new(90);
    
    // Newly created manager shouldn't need rotation for any user
    assert!(!manager.is_rotation_needed());
}