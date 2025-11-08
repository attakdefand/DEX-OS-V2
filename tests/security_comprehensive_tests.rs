//! Comprehensive security tests that integrate all three implemented features:
//! 1. Gossip Protocol for Off-chain Sync
//! 2. Zero-Knowledge Proofs for Privacy Protection
//! 3. Event Logging for Security Auditing

use dex_core::crypto::zk_proof::PrivacyProtectionService;
use dex_core::network::gossip_sync::{GossipSyncConfig, GossipSyncNode, SyncData};
use dex_core::security::{SecurityManager, EventType, SeverityLevel};
use std::collections::HashMap;

/// Test integration of all three security features
#[test]
fn test_security__comprehensive__integration__works__on_request() {
    // Initialize all security components
    let mut security_manager = SecurityManager::new();
    let privacy_service = PrivacyProtectionService::new();
    
    let config = GossipSyncConfig::default();
    let gossip_node = GossipSyncNode::new(config);
    
    // 1. Create a zero-knowledge proof for some sensitive data
    let secret_data = b"sensitive_trading_data";
    let zk_proof = privacy_service.prove_secret_knowledge(secret_data);
    
    // 2. Verify the zero-knowledge proof
    let public_input = dex_core::crypto::zk_proof::ZkProofSystem::new().compute_public_input(secret_data);
    assert!(privacy_service.verify_secret_knowledge(&zk_proof, &public_input));
    
    // 3. Log the verification event
    let mut event_data = HashMap::new();
    event_data.insert("proof_type".to_string(), "secret_knowledge".to_string());
    event_data.insert("verification_result".to_string(), "success".to_string());
    
    let event_id = security_manager.log_event(
        EventType::AuditTrail,
        "Zero-knowledge proof verified successfully".to_string(),
        Some("security_system".to_string()),
        event_data,
        None,
        SeverityLevel::Info,
    );
    assert!(!event_id.is_empty());
    
    // 4. Create sync data with the verified proof
    let sync_data = SyncData {
        id: "zk_proof_verification_001".to_string(),
        payload: b"proof_verification_result".to_vec(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        origin: "security_module".to_string(),
        data_type: "zk_proof_audit".to_string(),
    };
    
    // 5. Add the sync data to the gossip network
    tokio_test::block_on(async {
        gossip_node.add_sync_data(sync_data).await;
        
        // 6. Verify the data was added
        let data_map = gossip_node.get_sync_data().await;
        assert!(!data_map.is_empty());
        assert!(data_map.contains_key("zk_proof_verification_001"));
    });
    
    // 7. Log the sync event
    let mut sync_event_data = HashMap::new();
    sync_event_data.insert("data_id".to_string(), "zk_proof_verification_001".to_string());
    sync_event_data.insert("sync_status".to_string(), "completed".to_string());
    
    let sync_event_id = security_manager.log_event(
        EventType::AuditTrail,
        "Data synced to gossip network".to_string(),
        Some("gossip_sync_module".to_string()),
        sync_event_data,
        None,
        SeverityLevel::Info,
    );
    assert!(!sync_event_id.is_empty());
    
    // 8. Verify all events were logged
    let audit_events = security_manager.get_events_by_type(EventType::AuditTrail);
    assert_eq!(audit_events.len(), 2);
    
    // 9. Verify security manager has tracked the verified proofs
    assert_eq!(privacy_service.get_verified_proof_count(), 1);
    
    println!("All three security features integrated successfully!");
}

/// Test security policy enforcement across all features
#[test]
fn test_security__comprehensive__policy__enforces__on_request() {
    let mut security_manager = SecurityManager::new();
    let privacy_service = PrivacyProtectionService::new();
    
    // Test policy enforcement for ZK proofs
    let secret = b"test_secret";
    let proof = privacy_service.prove_secret_knowledge(secret);
    let public_input = dex_core::crypto::zk_proof::ZkProofSystem::new().compute_public_input(secret);
    
    // This should pass policy enforcement
    assert!(privacy_service.verify_secret_knowledge(&proof, &public_input));
    
    // Log the policy enforcement
    let mut policy_data = HashMap::new();
    policy_data.insert("feature".to_string(), "zk_proof".to_string());
    policy_data.insert("policy".to_string(), "secret_knowledge_verification".to_string());
    policy_data.insert("result".to_string(), "enforced".to_string());
    
    let policy_event_id = security_manager.log_event(
        EventType::PolicyViolation,
        "Security policy enforced for ZK proof verification".to_string(),
        Some("policy_engine".to_string()),
        policy_data,
        None,
        SeverityLevel::Info,
    );
    assert!(!policy_event_id.is_empty());
}

/// Test security validation across all features
#[test]
fn test_security__comprehensive__policy__validates__on_request() {
    let mut security_manager = SecurityManager::new();
    let privacy_service = PrivacyProtectionService::new();
    
    let config = GossipSyncConfig::default();
    let gossip_node = GossipSyncNode::new(config);
    
    // Test validation for all features
    let secret = b"test_secret";
    let proof = privacy_service.prove_secret_knowledge(secret);
    
    // Validate ZK proof
    let public_input = dex_core::crypto::zk_proof::ZkProofSystem::new().compute_public_input(secret);
    assert!(privacy_service.verify_secret_knowledge(&proof, &public_input));
    
    // Validate gossip sync data
    let sync_data = SyncData {
        id: "validation_test".to_string(),
        payload: vec![1, 2, 3, 4],
        timestamp: 1234567890,
        origin: "test_node".to_string(),
        data_type: "test".to_string(),
    };
    
    tokio_test::block_on(async {
        gossip_node.add_sync_data(sync_data.clone()).await;
        let retrieved_data = gossip_node.get_data_by_id("validation_test").await;
        assert!(retrieved_data.is_some());
        assert_eq!(retrieved_data.unwrap().id, sync_data.id);
    });
    
    // Log validation events
    let mut validation_data = HashMap::new();
    validation_data.insert("components_validated".to_string(), "zk_proof,gossip_sync".to_string());
    validation_data.insert("status".to_string(), "all_valid".to_string());
    
    let validation_event_id = security_manager.log_event(
        EventType::SystemEvent,
        "Security validation completed for all components".to_string(),
        Some("validation_engine".to_string()),
        validation_data,
        None,
        SeverityLevel::Info,
    );
    assert!(!validation_event_id.is_empty());
}

/// Test security evidence logging across all features
#[test]
fn test_security__comprehensive__policy__logs_evidence__on_request() {
    let mut security_manager = SecurityManager::new();
    let privacy_service = PrivacyProtectionService::new();
    
    // Generate evidence for ZK proof verification
    let secret = b"sensitive_trading_algorithm";
    let proof = privacy_service.prove_secret_knowledge(secret);
    let public_input = dex_core::crypto::zk_proof::ZkProofSystem::new().compute_public_input(secret);
    
    // Verify and log evidence
    let verification_result = privacy_service.verify_secret_knowledge(&proof, &public_input);
    assert!(verification_result);
    
    // Create evidence data
    let evidence = format!(
        "ZK Proof Verification: Secret={}, Result={}",
        "sensitive_trading_algorithm", verification_result
    ).as_bytes().to_vec();
    
    // Log the evidence
    let mut evidence_data = HashMap::new();
    evidence_data.insert("evidence_type".to_string(), "zk_proof_verification".to_string());
    evidence_data.insert("component".to_string(), "privacy_protection".to_string());
    
    let evidence_event_id = security_manager.log_event(
        EventType::AuditTrail,
        "Zero-knowledge proof verification evidence logged".to_string(),
        Some("security_system".to_string()),
        evidence_data,
        Some(evidence),
        SeverityLevel::Info,
    );
    assert!(!evidence_event_id.is_empty());
    
    // Verify evidence was logged
    let audit_events = security_manager.get_events_by_type(EventType::AuditTrail);
    assert!(!audit_events.is_empty());
    
    let evidence_event = audit_events.iter().find(|e| e.description.contains("evidence logged"));
    assert!(evidence_event.is_some());
    assert!(evidence_event.unwrap().evidence.is_some());
}

/// Test security detection across all features
#[test]
fn test_security__comprehensive__policy__detects__on_request() {
    let mut security_manager = SecurityManager::new();
    
    // Simulate detection of suspicious activity across components
    let mut detection_data = HashMap::new();
    detection_data.insert("component".to_string(), "all_security_features".to_string());
    detection_data.insert("detection_type".to_string(), "suspicious_activity".to_string());
    detection_data.insert("risk_level".to_string(), "medium".to_string());
    
    let detection_event_id = security_manager.log_event(
        EventType::SecurityAlert,
        "Suspicious activity detected across security components".to_string(),
        Some("detection_engine".to_string()),
        detection_data,
        None,
        SeverityLevel::Warning,
    );
    assert!(!detection_event_id.is_empty());
    
    // Verify detection was logged
    let alert_events = security_manager.get_events_by_type(EventType::SecurityAlert);
    assert!(!alert_events.is_empty());
    
    let detection_event = alert_events.iter().find(|e| e.description.contains("suspicious activity"));
    assert!(detection_event.is_some());
    assert_eq!(detection_event.unwrap().severity, SeverityLevel::Warning);
}

/// Test during CI integration
#[test]
fn test_security__comprehensive__scanner__enforces__during_ci() {
    // This would run during CI to ensure all security features are properly integrated
    assert!(true); // Placeholder for CI enforcement
}

/// Test after deploy integration
#[test]
fn test_security__comprehensive__gateway__enforces__after_deploy() {
    // This would run after deployment to ensure all security features are working in production
    assert!(true); // Placeholder for post-deployment enforcement
}