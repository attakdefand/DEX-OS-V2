//! Tests for Biometric DID functionality from DEX-OS-V2.csv line 214
//!
//! This file tests the Biometric DID implementation for Identity subtype.
//! The tests verify that users can be identified and authenticated using DIDs with biometric data.

use dex_core::identity::{IdentityManager, QuantumSecureCrypto};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Biometric DID creation
    /// This test verifies that a Biometric DID can be created with appropriate properties
    #[test]
    fn test_biometric_did_creation() {
        let mut identity_manager = IdentityManager::new();
        let user_id = "biometric_user_123".to_string();
        
        // Create DID for user
        let did_result = identity_manager.create_did(&user_id);
        assert!(did_result.is_ok(), "Failed to create Biometric DID");
        
        let did = did_result.unwrap();
        assert_eq!(did.id, user_id);
        assert_eq!(did.document.public_keys.len(), 1);
        assert_eq!(did.document.public_keys[0].key_type, "Dilithium");
        
        // Check that DID was stored
        assert!(identity_manager.get_did(&user_id).is_some());
        
        println!("✓ Biometric DID creation test passed");
    }

    /// Test Biometric data registration
    /// This test verifies that biometric data can be registered with a DID
    #[test]
    fn test_biometric_data_registration() {
        let mut identity_manager = IdentityManager::new();
        let user_id = "biometric_user_456".to_string();
        
        // Create DID first
        assert!(identity_manager.create_did(&user_id).is_ok());
        
        // Register biometric data
        let fingerprint_data = b"user_fingerprint_data_789";
        let registration_result = identity_manager.register_biometric(&user_id, "fingerprint", fingerprint_data);
        assert!(registration_result.is_ok(), "Failed to register biometric data");
        
        println!("✓ Biometric data registration test passed");
    }

    /// Test Biometric verification
    /// This test verifies that biometric data can be verified against registered templates
    #[test]
    fn test_biometric_verification() {
        let mut identity_manager = IdentityManager::new();
        let user_id = "biometric_user_789".to_string();
        
        // Create DID first
        assert!(identity_manager.create_did(&user_id).is_ok());
        
        // Register biometric data
        let fingerprint_data = b"user_fingerprint_template_abc";
        assert!(identity_manager.register_biometric(&user_id, "fingerprint", fingerprint_data).is_ok());
        
        // Verify correct biometric data
        let verification_result = identity_manager.verify_biometric(&user_id, "fingerprint", fingerprint_data);
        assert!(verification_result.is_ok(), "Failed to verify biometric data");
        assert!(verification_result.unwrap(), "Biometric verification should pass for correct data");
        
        // Verify incorrect biometric data (should fail)
        let incorrect_data = b"different_fingerprint_template";
        let incorrect_verification = identity_manager.verify_biometric(&user_id, "fingerprint", incorrect_data);
        assert!(incorrect_verification.is_ok(), "Verification should complete without error");
        assert!(!incorrect_verification.unwrap(), "Biometric verification should fail for incorrect data");
        
        println!("✓ Biometric verification test passed");
    }

    /// Test Biometric DID with multiple biometric types
    /// This test verifies that a single DID can handle multiple types of biometric data
    #[test]
    fn test_biometric_did_multiple_types() {
        let mut identity_manager = IdentityManager::new();
        let user_id = "multi_biometric_user".to_string();
        
        // Create DID first
        assert!(identity_manager.create_did(&user_id).is_ok());
        
        // Register multiple biometric types
        let fingerprint_data = b"user_fingerprint_data";
        let face_data = b"user_face_data";
        let iris_data = b"user_iris_data";
        
        assert!(identity_manager.register_biometric(&user_id, "fingerprint", fingerprint_data).is_ok());
        assert!(identity_manager.register_biometric(&user_id, "face", face_data).is_ok());
        assert!(identity_manager.register_biometric(&user_id, "iris", iris_data).is_ok());
        
        // Verify all biometric types
        assert!(identity_manager.verify_biometric(&user_id, "fingerprint", fingerprint_data).unwrap());
        assert!(identity_manager.verify_biometric(&user_id, "face", face_data).unwrap());
        assert!(identity_manager.verify_biometric(&user_id, "iris", iris_data).unwrap());
        
        println!("✓ Biometric DID multiple types test passed");
    }

    /// Test Biometric DID integration with quantum-secure crypto
    /// This test verifies that biometric data is properly secured using quantum-resistant cryptography
    #[test]
    fn test_biometric_did_quantum_secure() {
        // Test biometric hashing
        let biometric_template1 = b"face_scan_template_alpha";
        let biometric_template2 = b"face_scan_template_beta";
        
        let hash1 = QuantumSecureCrypto::hash_biometric(biometric_template1);
        let hash2 = QuantumSecureCrypto::hash_biometric(biometric_template2);
        
        // Hashes should be 32 bytes (SHA3-256)
        assert_eq!(hash1.len(), 32, "Biometric hash should be 32 bytes");
        assert_eq!(hash2.len(), 32, "Biometric hash should be 32 bytes");
        
        // Different inputs should produce different hashes
        assert_ne!(
            hash1, hash2,
            "Different biometric templates should produce different hashes"
        );
        
        // Same input should produce same hash
        let hash1_again = QuantumSecureCrypto::hash_biometric(biometric_template1);
        assert_eq!(
            hash1, hash1_again,
            "Same biometric template should produce same hash"
        );
        
        println!("✓ Biometric DID quantum-secure crypto test passed");
    }

    /// Test Biometric DID error handling
    /// This test verifies proper error handling for biometric operations
    #[test]
    fn test_biometric_did_error_handling() {
        let mut identity_manager = IdentityManager::new();
        let user_id = "error_test_user".to_string();
        let non_existent_user = "non_existent_user".to_string();
        
        // Try to register biometric data for non-existent DID
        let fingerprint_data = b"test_fingerprint_data";
        let registration_result = identity_manager.register_biometric(&non_existent_user, "fingerprint", fingerprint_data);
        assert!(registration_result.is_err(), "Should fail for non-existent DID");
        
        // Try to verify biometric data for non-existent DID
        let verification_result = identity_manager.verify_biometric(&non_existent_user, "fingerprint", fingerprint_data);
        assert!(verification_result.is_err(), "Should fail for non-existent DID");
        
        // Try to verify non-registered biometric type
        assert!(identity_manager.create_did(&user_id).is_ok());
        let verification_result = identity_manager.verify_biometric(&user_id, "fingerprint", fingerprint_data);
        assert!(verification_result.is_err(), "Should fail for non-registered biometric type");
        
        println!("✓ Biometric DID error handling test passed");
    }
}