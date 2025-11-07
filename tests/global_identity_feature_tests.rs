//! Tests for the Global Identity features from DEX-OS-V2.csv lines 53-55
//!
//! Features tested:
//! - DID + Biometrics (Line 53)
//! - Self-Sovereign Identity (Line 54)
//! - Quantum-Secure Identity (Line 55)

use dex_core::identity::{IdentityManager, QuantumSecureCrypto};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test DID + Biometrics feature (Line 53)
    /// This test verifies that decentralized identifiers can be created and biometric data
    /// can be securely registered and verified for identity verification.
    #[test]
    fn test_did_plus_biometrics_feature() {
        let mut identity_manager = IdentityManager::new();
        let trader_id = "trader123".to_string();
        
        // Create DID with quantum-secure keys
        let did_result = identity_manager.create_did(&trader_id);
        assert!(did_result.is_ok(), "Failed to create DID");
        
        let did = did_result.unwrap();
        assert_eq!(did.id, trader_id);
        assert_eq!(did.document.public_keys.len(), 1);
        assert_eq!(did.document.public_keys[0].key_type, "Dilithium");
        
        // Register biometric data
        let fingerprint_data = b"user_fingerprint_template_12345";
        let registration_result = identity_manager.register_biometric(&trader_id, "fingerprint", fingerprint_data);
        assert!(registration_result.is_ok(), "Failed to register biometric data");
        
        // Verify correct biometric data
        let verification_result = identity_manager.verify_biometric(&trader_id, "fingerprint", fingerprint_data);
        assert!(verification_result.is_ok(), "Failed to verify biometric data");
        assert!(verification_result.unwrap(), "Biometric verification should pass for correct data");
        
        // Verify incorrect biometric data (should fail)
        let incorrect_data = b"different_fingerprint_template";
        let incorrect_verification = identity_manager.verify_biometric(&trader_id, "fingerprint", incorrect_data);
        assert!(incorrect_verification.is_ok(), "Verification should complete without error");
        assert!(!incorrect_verification.unwrap(), "Biometric verification should fail for incorrect data");
        
        println!("✓ DID + Biometrics feature test passed");
    }

    /// Test Self-Sovereign Identity feature (Line 54)
    /// This test verifies that users can create and control their own identities with
    /// encrypted personal data and verifiable credentials.
    #[test]
    fn test_self_sovereign_identity_feature() {
        let mut identity_manager = IdentityManager::new();
        let trader_id = "self_sovereign_user".to_string();
        
        // Create DID first
        assert!(identity_manager.create_did(&trader_id).is_ok(), "Failed to create DID");
        
        // Create self-sovereign identity with encrypted personal data
        let personal_data = b"encrypted_personal_information_data".to_vec();
        let identity_result = identity_manager.create_self_sovereign_identity(&trader_id, personal_data.clone());
        assert!(identity_result.is_ok(), "Failed to create self-sovereign identity");
        
        let identity = identity_result.unwrap();
        assert_eq!(identity.personal_data, personal_data);
        assert!(!identity.is_revoked);
        assert_eq!(identity.biometrics.len(), 0);
        assert_eq!(identity.verifiable_credentials.len(), 0);
        
        // Check that identity was stored and is valid
        assert!(identity_manager.get_identity(&trader_id).is_some(), "Identity should be stored");
        assert!(identity_manager.is_valid_identity(&trader_id), "Identity should be valid");
        
        // Add trusted issuer
        let issuer_did = "trusted_issuer_did".to_string();
        identity_manager.add_trusted_issuer(issuer_did.clone());
        
        // Create and add verifiable credential
        let mut credential_data = std::collections::HashMap::new();
        credential_data.insert("name".to_string(), "John Doe".to_string());
        credential_data.insert("kyc_level".to_string(), "verified".to_string());
        
        let credential = dex_core::identity::VerifiableCredential {
            id: "credential_001".to_string(),
            issuer: issuer_did,
            subject: trader_id.clone(),
            data: credential_data,
            issued_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            expires_at: None,
            signature: vec![1, 2, 3, 4, 5],
        };
        
        let credential_result = identity_manager.add_verifiable_credential(&trader_id, credential);
        assert!(credential_result.is_ok(), "Failed to add verifiable credential");
        
        // Check that credential was added
        let updated_identity = identity_manager.get_identity(&trader_id).unwrap();
        assert_eq!(updated_identity.verifiable_credentials.len(), 1);
        assert_eq!(updated_identity.verifiable_credentials[0].id, "credential_001");
        
        // Revoke identity
        assert!(identity_manager.revoke_identity(&trader_id).is_ok(), "Failed to revoke identity");
        assert!(!identity_manager.is_valid_identity(&trader_id), "Identity should be invalid after revocation");
        
        println!("✓ Self-Sovereign Identity feature test passed");
    }

    /// Test Quantum-Secure Identity feature (Line 55)
    /// This test verifies that quantum-resistant cryptographic operations are implemented
    /// for secure identity management.
    #[test]
    fn test_quantum_secure_identity_feature() {
        // Test Dilithium keypair generation
        let (private_key, public_key) = QuantumSecureCrypto::generate_dilithium_keypair();
        assert_eq!(private_key.len(), 32, "Private key should be 32 bytes");
        assert_eq!(public_key.len(), 32, "Public key should be 32 bytes");
        
        // Test quantum-secure signing
        let data_to_sign = b"critical_identity_data_for_signing";
        let signature = QuantumSecureCrypto::quantum_sign(&private_key, data_to_sign);
        assert!(!signature.is_empty(), "Signature should not be empty");
        
        // Test quantum-secure verification with correct data
        let verification_result = QuantumSecureCrypto::quantum_verify(&public_key, data_to_sign, &signature);
        assert!(verification_result, "Signature verification should pass for correct data");
        
        // Test quantum-secure verification with incorrect data
        let incorrect_data = b"different_data_for_verification";
        let incorrect_verification = QuantumSecureCrypto::quantum_verify(&public_key, incorrect_data, &signature);
        assert!(!incorrect_verification, "Signature verification should fail for incorrect data");
        
        // Test biometric hashing (part of quantum-secure identity)
        let biometric_template1 = b"face_scan_template_alpha";
        let biometric_template2 = b"face_scan_template_beta";
        
        let hash1 = QuantumSecureCrypto::hash_biometric(biometric_template1);
        let hash2 = QuantumSecureCrypto::hash_biometric(biometric_template2);
        
        // Hashes should be 32 bytes (SHA3-256)
        assert_eq!(hash1.len(), 32, "Biometric hash should be 32 bytes");
        assert_eq!(hash2.len(), 32, "Biometric hash should be 32 bytes");
        
        // Different inputs should produce different hashes
        assert_ne!(hash1, hash2, "Different biometric templates should produce different hashes");
        
        // Same input should produce same hash
        let hash1_again = QuantumSecureCrypto::hash_biometric(biometric_template1);
        assert_eq!(hash1, hash1_again, "Same biometric template should produce same hash");
        
        println!("✓ Quantum-Secure Identity feature test passed");
    }

    /// Test integration of all three Global Identity features
    /// This test verifies that all three features work together cohesively
    #[test]
    fn test_global_identity_features_integration() {
        let mut identity_manager = IdentityManager::new();
        let trader_id = "integrated_identity_user".to_string();
        
        // 1. Create DID with quantum-secure keys (Feature from Line 53 & 55)
        let did_result = identity_manager.create_did(&trader_id);
        assert!(did_result.is_ok(), "Failed to create quantum-secure DID");
        
        let did = did_result.unwrap();
        assert_eq!(did.document.public_keys[0].key_type, "Dilithium");
        
        // 2. Register biometric data (Feature from Line 53)
        let face_data = b"user_face_scan_data_98765";
        assert!(identity_manager.register_biometric(&trader_id, "face", face_data).is_ok());
        assert!(identity_manager.verify_biometric(&trader_id, "face", face_data).unwrap());
        
        // 3. Create self-sovereign identity (Feature from Line 54)
        let personal_data = b"user_encrypted_health_records".to_vec();
        let identity_result = identity_manager.create_self_sovereign_identity(&trader_id, personal_data);
        assert!(identity_result.is_ok(), "Failed to create self-sovereign identity");
        
        let identity = identity_result.unwrap();
        assert!(!identity.is_revoked);
        
        // 4. Verify all components work together
        assert!(identity_manager.get_did(&trader_id).is_some());
        assert!(identity_manager.get_identity(&trader_id).is_some());
        assert!(identity_manager.is_valid_identity(&trader_id));
        
        println!("✓ Global Identity features integration test passed");
    }
}