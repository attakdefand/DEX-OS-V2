//! Security tests for B+ Tree Certificate Management
//!
//! This file implements security tests for the B+ Tree Certificate Management feature
//! based on the security_tests_full.csv requirements.

use dex_core::security::{Certificate, CertificateManager, SecurityError, SecurityManager};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn test_security__security__security__bplus_tree__certificate_management__enforces__on_request() {
    let mut cert_manager = CertificateManager::new();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create a certificate
    let certificate = Certificate {
        id: "test_cert_1".to_string(),
        data: vec![1, 2, 3, 4],
        issuer: "Test CA".to_string(),
        valid_from: now - 1000,
        valid_to: now + 1000,
        signature: vec![5, 6, 7, 8],
        revoked: false,
    };

    // Enforce that certificates can be added to the B+ Tree
    let result = cert_manager.add_certificate(certificate);
    assert!(result.is_ok());

    // Verify certificate was stored
    let retrieved = cert_manager.get_certificate("test_cert_1");
    assert!(retrieved.is_some());
}

pub fn test_security__security__security__bplus_tree__certificate_management__validates__on_request() {
    let mut cert_manager = CertificateManager::new();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create a valid certificate
    let valid_cert = Certificate {
        id: "valid_cert".to_string(),
        data: vec![1, 2, 3, 4],
        issuer: "Test CA".to_string(),
        valid_from: now - 1000,
        valid_to: now + 1000,
        signature: vec![5, 6, 7, 8],
        revoked: false,
    };

    // Create an expired certificate
    let expired_cert = Certificate {
        id: "expired_cert".to_string(),
        data: vec![9, 10, 11, 12],
        issuer: "Test CA".to_string(),
        valid_from: now - 2000,
        valid_to: now - 1000, // Expired
        signature: vec![13, 14, 15, 16],
        revoked: false,
    };

    // Create a revoked certificate
    let revoked_cert = Certificate {
        id: "revoked_cert".to_string(),
        data: vec![17, 18, 19, 20],
        issuer: "Test CA".to_string(),
        valid_from: now - 1000,
        valid_to: now + 1000,
        signature: vec![21, 22, 23, 24],
        revoked: true, // Revoked
    };

    // Add certificates
    assert!(cert_manager.add_certificate(valid_cert).is_ok());
    assert!(cert_manager.add_certificate(expired_cert).is_ok());
    assert!(cert_manager.add_certificate(revoked_cert).is_ok());

    // Validate certificate status
    assert!(cert_manager.is_certificate_valid("valid_cert"));
    assert!(!cert_manager.is_certificate_valid("expired_cert"));
    assert!(!cert_manager.is_certificate_valid("revoked_cert"));
}

pub fn test_security__security__security__bplus_tree__certificate_management__blocks__on_request() {
    let mut cert_manager = CertificateManager::new();

    // Block attempts to add duplicate certificates
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let certificate = Certificate {
        id: "duplicate_cert".to_string(),
        data: vec![1, 2, 3, 4],
        issuer: "Test CA".to_string(),
        valid_from: now - 1000,
        valid_to: now + 1000,
        signature: vec![5, 6, 7, 8],
        revoked: false,
    };

    // First addition should succeed
    assert!(cert_manager.add_certificate(certificate.clone()).is_ok());

    // Second addition should be blocked
    let result = cert_manager.add_certificate(certificate);
    assert!(matches!(result, Err(SecurityError::CertificateAlreadyExists)));

    // Block attempts to revoke non-existent certificates
    let revoke_result = cert_manager.revoke_certificate("nonexistent_cert");
    assert!(matches!(revoke_result, Err(SecurityError::CertificateNotFound)));

    // Block attempts to revoke already revoked certificates
    // First revoke should succeed
    let cert = Certificate {
        id: "to_revoke".to_string(),
        data: vec![1, 2, 3, 4],
        issuer: "Test CA".to_string(),
        valid_from: now - 1000,
        valid_to: now + 1000,
        signature: vec![5, 6, 7, 8],
        revoked: false,
    };
    assert!(cert_manager.add_certificate(cert).is_ok());
    assert!(cert_manager.revoke_certificate("to_revoke").is_ok());
    
    // Second revoke should be blocked
    let second_revoke_result = cert_manager.revoke_certificate("to_revoke");
    assert!(matches!(second_revoke_result, Err(SecurityError::CertificateAlreadyRevoked)));
}

pub fn test_security__security__security__bplus_tree__certificate_management__detects__on_request() {
    let mut cert_manager = CertificateManager::new();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create certificates
    let cert1 = Certificate {
        id: "cert_1".to_string(),
        data: vec![1, 2, 3, 4],
        issuer: "CA1".to_string(),
        valid_from: now - 1000,
        valid_to: now + 1000,
        signature: vec![5, 6, 7, 8],
        revoked: false,
    };

    let cert2 = Certificate {
        id: "cert_2".to_string(),
        data: vec![9, 10, 11, 12],
        issuer: "CA2".to_string(),
        valid_from: now - 500,
        valid_to: now + 1500,
        signature: vec![13, 14, 15, 16],
        revoked: true,
    };

    // Add certificates
    assert!(cert_manager.add_certificate(cert1).is_ok());
    assert!(cert_manager.add_certificate(cert2).is_ok());

    // Detect certificate existence
    assert!(cert_manager.get_certificate("cert_1").is_some());
    assert!(cert_manager.get_certificate("cert_2").is_some());
    assert!(cert_manager.get_certificate("nonexistent").is_none());

    // Detect certificate validity
    assert!(cert_manager.is_certificate_valid("cert_1"));
    assert!(!cert_manager.is_certificate_valid("cert_2")); // Revoked

    // Detect certificate revocation status
    if let Some(cert) = cert_manager.get_certificate("cert_1") {
        assert!(!cert.revoked);
    }
    if let Some(cert) = cert_manager.get_certificate("cert_2") {
        assert!(cert.revoked);
    }
}

pub fn test_security__security__security__bplus_tree__certificate_management__logs_evidence__on_request() {
    let mut security_manager = SecurityManager::new();
    let mut cert_manager = CertificateManager::new();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create a certificate
    let certificate = Certificate {
        id: "logged_cert".to_string(),
        data: vec![1, 2, 3, 4],
        issuer: "Test CA".to_string(),
        valid_from: now - 1000,
        valid_to: now + 1000,
        signature: vec![5, 6, 7, 8],
        revoked: false,
    };

    // Add certificate
    assert!(cert_manager.add_certificate(certificate).is_ok());

    // Log evidence of certificate management operation
    use std::collections::HashMap;
    let mut event_data = HashMap::new();
    event_data.insert("certificate_id".to_string(), "logged_cert".to_string());
    event_data.insert("operation".to_string(), "add_certificate".to_string());

    let event_id = security_manager.log_event(
        dex_core::security::EventType::AuditTrail,
        "Certificate added to B+ Tree storage".to_string(),
        Some("certificate_manager".to_string()),
        event_data,
        None,
        dex_core::security::SeverityLevel::Info,
    );

    assert!(!event_id.is_empty());

    // Verify event was logged
    let events = security_manager.get_events_by_type(dex_core::security::EventType::AuditTrail);
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| e.data.contains_key("certificate_id")));
}

pub fn test_security__security__security__bplus_tree__certificate_management__rotates__on_request() {
    let mut cert_manager = CertificateManager::new();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Add initial certificate
    let initial_cert = Certificate {
        id: "rotating_cert".to_string(),
        data: vec![1, 2, 3, 4],
        issuer: "CA1".to_string(),
        valid_from: now - 1000,
        valid_to: now + 1000,
        signature: vec![5, 6, 7, 8],
        revoked: false,
    };

    assert!(cert_manager.add_certificate(initial_cert).is_ok());

    // "Rotate" by revoking the old certificate and adding a new one
    assert!(cert_manager.revoke_certificate("rotating_cert").is_ok());

    let new_cert = Certificate {
        id: "rotating_cert".to_string(), // Same ID but new data
        data: vec![9, 10, 11, 12], // Different data
        issuer: "CA2".to_string(), // Different issuer
        valid_from: now - 500,
        valid_to: now + 1500,
        signature: vec![13, 14, 15, 16],
        revoked: false,
    };

    // Add the "rotated" certificate
    assert!(cert_manager.add_certificate(new_cert).is_ok());

    // Verify the certificate was updated
    if let Some(cert) = cert_manager.get_certificate("rotating_cert") {
        assert_eq!(cert.issuer, "CA2");
        assert_eq!(cert.data, vec![9, 10, 11, 12]);
    } else {
        panic!("Certificate not found after rotation");
    }
}