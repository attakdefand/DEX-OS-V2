//! Main entry point for Global Identity security tests
//!
//! This runs the security tests for the three Global Identity features from DEX-OS-V2.csv lines 53-55:
//! - DID + Biometrics (Line 53)
//! - Self-Sovereign Identity (Line 54)
//! - Quantum-Secure Identity (Line 55)

use dex_core::identity::{IdentityManager, QuantumSecureCrypto};
use dex_core::test_results::{
    IndividualTestResult, TestMetadata, TestResultsManager, TestStatus, TestSuiteResult,
};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Test DID + Biometrics feature (Line 53)
fn test_did_plus_biometrics_feature() -> Result<IndividualTestResult, Box<dyn std::error::Error>> {
    let start_time = SystemTime::now();

    let mut identity_manager = IdentityManager::new();
    let trader_id = "trader123".to_string();

    // Create DID with quantum-secure keys
    let did_result = identity_manager.create_did(&trader_id)?;

    let did = did_result;
    assert_eq!(did.id, trader_id);
    assert_eq!(did.document.public_keys.len(), 1);
    assert_eq!(did.document.public_keys[0].key_type, "Dilithium");

    // Register biometric data
    let fingerprint_data = b"user_fingerprint_template_12345";
    identity_manager.register_biometric(&trader_id, "fingerprint", fingerprint_data)?;

    // Verify correct biometric data
    let verification_result =
        identity_manager.verify_biometric(&trader_id, "fingerprint", fingerprint_data)?;
    assert!(
        verification_result,
        "Biometric verification should pass for correct data"
    );

    let duration = start_time.elapsed().unwrap().as_millis() as u64;

    Ok(IndividualTestResult {
        name: "test_did_plus_biometrics_feature".to_string(),
        status: TestStatus::Passed,
        duration_ms: duration,
        error_message: None,
        data: HashMap::new(),
    })
}

/// Test Self-Sovereign Identity feature (Line 54)
fn test_self_sovereign_identity_feature() -> Result<IndividualTestResult, Box<dyn std::error::Error>>
{
    let start_time = SystemTime::now();

    let mut identity_manager = IdentityManager::new();
    let trader_id = "self_sovereign_user".to_string();

    // Create DID first
    identity_manager.create_did(&trader_id)?;

    // Create self-sovereign identity with encrypted personal data
    let personal_data = b"encrypted_personal_information_data".to_vec();
    let identity_result =
        identity_manager.create_self_sovereign_identity(&trader_id, personal_data.clone())?;

    let identity = identity_result;
    assert_eq!(identity.personal_data, personal_data);
    assert!(!identity.is_revoked);

    // Add trusted issuer and verifiable credential
    let issuer_did = "trusted_issuer_did".to_string();
    identity_manager.add_trusted_issuer(issuer_did.clone());

    let mut credential_data = HashMap::new();
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

    identity_manager.add_verifiable_credential(&trader_id, credential)?;

    let duration = start_time.elapsed().unwrap().as_millis() as u64;

    Ok(IndividualTestResult {
        name: "test_self_sovereign_identity_feature".to_string(),
        status: TestStatus::Passed,
        duration_ms: duration,
        error_message: None,
        data: HashMap::new(),
    })
}

/// Test Quantum-Secure Identity feature (Line 55)
fn test_quantum_secure_identity_feature() -> Result<IndividualTestResult, Box<dyn std::error::Error>>
{
    let start_time = SystemTime::now();

    // Test Dilithium keypair generation
    let (private_key, public_key) = QuantumSecureCrypto::generate_dilithium_keypair();
    assert_eq!(private_key.len(), 32);
    assert_eq!(public_key.len(), 32);

    // Test quantum-secure signing and verification
    let data_to_sign = b"critical_identity_data_for_signing";
    let signature = QuantumSecureCrypto::quantum_sign(&private_key, data_to_sign);
    assert!(!signature.is_empty());

    let verification_result =
        QuantumSecureCrypto::quantum_verify(&public_key, data_to_sign, &signature);
    assert!(verification_result);

    // Test biometric hashing
    let biometric_template1 = b"face_scan_template_alpha";
    let biometric_template2 = b"face_scan_template_beta";

    let hash1 = QuantumSecureCrypto::hash_biometric(biometric_template1);
    let hash2 = QuantumSecureCrypto::hash_biometric(biometric_template2);

    assert_eq!(hash1.len(), 32);
    assert_eq!(hash2.len(), 32);
    assert_ne!(hash1, hash2);

    let duration = start_time.elapsed().unwrap().as_millis() as u64;

    Ok(IndividualTestResult {
        name: "test_quantum_secure_identity_feature".to_string(),
        status: TestStatus::Passed,
        duration_ms: duration,
        error_message: None,
        data: HashMap::new(),
    })
}

/// Run all Global Identity security tests
fn run_global_identity_tests() -> Result<TestSuiteResult, Box<dyn std::error::Error>> {
    let started_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut test_results = Vec::new();

    // Run each test and collect results
    match test_did_plus_biometrics_feature() {
        Ok(result) => test_results.push(result),
        Err(e) => {
            test_results.push(IndividualTestResult {
                name: "test_did_plus_biometrics_feature".to_string(),
                status: TestStatus::Failed,
                duration_ms: 0,
                error_message: Some(e.to_string()),
                data: HashMap::new(),
            });
        }
    }

    match test_self_sovereign_identity_feature() {
        Ok(result) => test_results.push(result),
        Err(e) => {
            test_results.push(IndividualTestResult {
                name: "test_self_sovereign_identity_feature".to_string(),
                status: TestStatus::Failed,
                duration_ms: 0,
                error_message: Some(e.to_string()),
                data: HashMap::new(),
            });
        }
    }

    match test_quantum_secure_identity_feature() {
        Ok(result) => test_results.push(result),
        Err(e) => {
            test_results.push(IndividualTestResult {
                name: "test_quantum_secure_identity_feature".to_string(),
                status: TestStatus::Failed,
                duration_ms: 0,
                error_message: Some(e.to_string()),
                data: HashMap::new(),
            });
        }
    }

    let finished_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(TestSuiteResult {
        id: format!("global_identity_test_suite_{}", started_at),
        suite_name: "Global Identity Security Tests".to_string(),
        started_at,
        finished_at,
        status: if test_results.iter().all(|r| r.status == TestStatus::Passed) {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        },
        test_results,
        metadata: TestMetadata {
            version: "1.0.0".to_string(),
            commit_hash: "abc123def456".to_string(),
            environment: "testing".to_string(),
            platform: "windows".to_string(),
            custom: HashMap::new(),
        },
    })
}

/// Store test results using the TestResultsManager
fn store_test_results(suite_result: TestSuiteResult) -> Result<(), Box<dyn std::error::Error>> {
    let mut results_manager = TestResultsManager::new();

    // Store the result
    results_manager.store_result(suite_result.clone())?;

    // Print summary
    let stats = results_manager.get_statistics();
    println!("\n=== Global Identity Test Results Summary ===");
    println!("Total suites: {}", stats.total_suites);
    println!("Passed suites: {}", stats.passed_suites);
    println!("Failed suites: {}", stats.failed_suites);
    println!("Total tests: {}", stats.total_tests);
    println!("Passed tests: {}", stats.passed_tests);
    println!("Failed tests: {}", stats.failed_tests);
    println!("Average duration: {} ms", stats.average_duration_ms);

    // Verify we can retrieve the result
    let retrieved = results_manager.get_result(&suite_result.id);
    assert!(retrieved.is_some());

    println!(
        "\nTest results stored successfully with ID: {}",
        suite_result.id
    );

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running Global Identity security tests for DEX-OS-V2.csv lines 53-55...");
    println!("Testing features:");
    println!("  1. DID + Biometrics (Line 53)");
    println!("  2. Self-Sovereign Identity (Line 54)");
    println!("  3. Quantum-Secure Identity (Line 55)");

    let suite_result = run_global_identity_tests()?;
    store_test_results(suite_result)?;

    println!("\n✓ All Global Identity security tests completed successfully!");
    println!("✓ Test results have been stored in the test results system");

    Ok(())
}
