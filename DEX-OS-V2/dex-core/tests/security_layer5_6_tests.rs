use dex_core::security::{
    SecurityManager, 
    ClassificationLevel,
};
use dex_core::security::data_encryption::{
    DataEncryptionManager,
};
use dex_core::security::token_validation::{
    TokenManager,
    TokenValidationError,
};
use std::thread;
use std::time::Duration;

#[test]
fn test_sha3_256_hashing() {
    let encryption_manager = DataEncryptionManager::new();
    let data = b"sensitive data";
    let hash = encryption_manager.hash_data(data);
    
    assert_eq!(hash.len(), 32); // SHA3-256 produces 32 bytes
    
    let hash2 = encryption_manager.hash_data(data);
    assert_eq!(hash, hash2); // Deterministic
    
    let data2 = b"different data";
    let hash3 = encryption_manager.hash_data(data2);
    assert_ne!(hash, hash3);
}

#[test]
fn test_password_hashing_argon2() {
    let encryption_manager = DataEncryptionManager::new();
    let password = "secure_password_123";
    
    // Test hash_password (returns PHC string)
    let hash_result = encryption_manager.hash_password(password);
    assert!(hash_result.is_ok());
    let hash = hash_result.unwrap();
    
    // Verify correct password
    assert!(encryption_manager.verify_password(password, &hash));
    
    // Verify incorrect password
    assert!(!encryption_manager.verify_password("wrong_password", &hash));
}

#[test]
fn test_aes_gcm_encryption() {
    let mut security_manager = SecurityManager::new();
    let encryption_manager = DataEncryptionManager::new();
    let data = b"secret message";
    let classification = ClassificationLevel::Confidential;
    
    // First classify the data
    security_manager.classify_data(
        "test_data".to_string(),
        classification.clone(),
        "owner".to_string(),
        vec![],
    );
    
    // Encrypt
    let encrypted_result = encryption_manager.encrypt_data(data, classification.clone());
    assert!(encrypted_result.is_ok());
    let encrypted = encrypted_result.unwrap();
    
    assert_ne!(encrypted.ciphertext, data);
    // Note: The encrypt_data method doesn't store classification in metadata,
    // so we can't check that here. The classification is managed separately by SecurityManager.
    
    // Decrypt
    let decrypted_result = encryption_manager.decrypt_data(&encrypted);
    assert!(decrypted_result.is_ok());
    let decrypted = decrypted_result.unwrap();
    
    assert_eq!(decrypted, data);
}

#[test]
fn test_jwt_token_management() {
    let token_manager = TokenManager::new("dex-os", "dex-client");
    let user_id = "user_123";
    let roles = vec!["admin".to_string(), "trader".to_string()];
    let ttl = 60; // 1 minute
    
    // Create Token
    let token = token_manager.create_token(user_id, roles.clone(), ttl);
    assert!(!token.is_empty());
    assert_eq!(token.split('.').count(), 3);
    
    // Validate Token
    let validator = token_manager.get_validator();
    let claims_result = validator.validate_token(&token);
    assert!(claims_result.is_ok());
    let claims = claims_result.unwrap();
    
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.roles, roles);
    
    // Test Expiration
    let expired_token = token_manager.create_token(user_id, roles, 0);
    thread::sleep(Duration::from_secs(1));
    let expired_result = validator.validate_token(&expired_token);
    assert_eq!(expired_result.err(), Some(TokenValidationError::Expired));
}