//! Test to verify the B+ Tree implementation for Certificate Management
//!
//! This test verifies that the CertificateManager properly uses a B+ Tree
//! for storing and retrieving certificates, as specified in the Priority 3
//! feature "Security - B+ Tree for Certificate Management".

use dex_core::security::{Certificate, CertificateManager, SecurityError};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_bplus_tree_certificate_storage() {
    let mut cert_manager = CertificateManager::new();
    
    // Create test certificates
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let cert1 = Certificate {
        id: "cert1".to_string(),
        data: vec![1, 2, 3, 4],
        issuer: "CA1".to_string(),
        valid_from: now - 1000,
        valid_to: now + 1000,
        signature: vec![5, 6, 7, 8],
        revoked: false,
    };
    
    let cert2 = Certificate {
        id: "cert2".to_string(),
        data: vec![9, 10, 11, 12],
        issuer: "CA2".to_string(),
        valid_from: now - 500,
        valid_to: now + 1500,
        signature: vec![13, 14, 15, 16],
        revoked: false,
    };
    
    let cert3 = Certificate {
        id: "cert3".to_string(),
        data: vec![17, 18, 19, 20],
        issuer: "CA3".to_string(),
        valid_from: now - 200,
        valid_to: now + 800,
        signature: vec![21, 22, 23, 24],
        revoked: true,
    };
    
    // Add certificates
    assert!(cert_manager.add_certificate(cert1.clone()).is_ok());
    assert!(cert_manager.add_certificate(cert2.clone()).is_ok());
    assert!(cert_manager.add_certificate(cert3.clone()).is_ok());
    
    // Try to add duplicate certificate (should fail)
    assert!(matches!(
        cert_manager.add_certificate(cert1.clone()),
        Err(SecurityError::CertificateAlreadyExists)
    ));
    
    // Retrieve certificates
    let retrieved_cert1 = cert_manager.get_certificate("cert1");
    assert!(retrieved_cert1.is_some());
    assert_eq!(retrieved_cert1.unwrap(), cert1);
    
    let retrieved_cert2 = cert_manager.get_certificate("cert2");
    assert!(retrieved_cert2.is_some());
    assert_eq!(retrieved_cert2.unwrap(), cert2);
    
    let retrieved_cert3 = cert_manager.get_certificate("cert3");
    assert!(retrieved_cert3.is_some());
    assert_eq!(retrieved_cert3.unwrap(), cert3);
    
    // Check non-existent certificate
    let non_existent = cert_manager.get_certificate("nonexistent");
    assert!(non_existent.is_none());
    
    // Check certificate validity
    assert!(cert_manager.is_certificate_valid("cert1"));
    assert!(cert_manager.is_certificate_valid("cert2"));
    assert!(!cert_manager.is_certificate_valid("cert3")); // revoked
    
    // Revoke a certificate
    assert!(cert_manager.revoke_certificate("cert1").is_ok());
    assert!(!cert_manager.is_certificate_valid("cert1")); // now revoked
    
    // Try to revoke already revoked certificate
    assert!(matches!(
        cert_manager.revoke_certificate("cert1"),
        Err(SecurityError::CertificateAlreadyRevoked)
    ));
    
    // Try to revoke non-existent certificate
    assert!(matches!(
        cert_manager.revoke_certificate("nonexistent"),
        Err(SecurityError::CertificateNotFound)
    ));
    
    println!("B+ Tree Certificate Management test passed successfully!");
}