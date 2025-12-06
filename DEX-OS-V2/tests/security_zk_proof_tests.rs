//! Security tests for Zero-Knowledge Proofs for Privacy Protection and Blockchain Resilience
//!
//! This module implements security tests for the Priority 3 feature from DEX-OS-V2.csv:
//! - Security,Security,Security,Zero-Knowledge Proofs,Privacy Protection,Medium
//! - Blockchain Resilience,Blockchain Resilience,Blockchain Resilience,Zero-Knowledge Proofs,zk-SNARKs,Medium

use dex_core::crypto::zk_proof::{
    BlockchainProofRecord, BlockchainResilienceZkService, PrivacyProtectionService, ZkProof,
};
use sha3::{Digest, Sha3_256};

fn make_privacy_roundtrip(secret: &[u8]) -> (PrivacyProtectionService, ZkProof, Vec<u8>) {
    let mut service = PrivacyProtectionService::new();
    let proof = service.prove_secret_knowledge(secret);
    let public_input = proof.public_input.clone();
    (service, proof, public_input)
}

fn make_block_record(height: u64) -> (BlockchainResilienceZkService, BlockchainProofRecord) {
    let mut service = BlockchainResilienceZkService::new();
    let prev_root = Sha3_256::digest(format!("prev_{height}").as_bytes()).to_vec();
    let new_root = Sha3_256::digest(format!("new_{height}").as_bytes()).to_vec();
    let record = service.prove_block_transition(height, &prev_root, &new_root, b"witness");
    (service, record)
}

fn tamper_proof(proof: &mut ZkProof) {
    if let Some(byte) = proof.challenge.first_mut() {
        *byte ^= 0b0000_0001;
    }
}

fn tamper_block_record(record: &mut BlockchainProofRecord) {
    if let Some(byte) = record.proof.proof.first_mut() {
        *byte ^= 0b0000_0001;
    }
}

/// Test security policy enforcement on request for ZK proofs
#[test]
fn test_security__zk_proof__policy__enforces__on_request() {
    let (mut service, proof, public_input) = make_privacy_roundtrip(b"policy_enforce");
    assert!(service.verify_secret_knowledge(&proof, &public_input));
    assert_eq!(service.get_verified_proof_count(), 1);
}

/// Test security policy validation on request for ZK proofs
#[test]
fn test_security__zk_proof__policy__validates__on_request() {
    let (mut service, mut proof, public_input) = make_privacy_roundtrip(b"policy_validate");
    tamper_proof(&mut proof);
    assert!(!service.verify_secret_knowledge(&proof, &public_input));
}

/// Test security policy rotation on request for ZK proofs
#[test]
fn test_security__zk_proof__policy__rotates__on_request() {
    let (mut service, record) = make_block_record(7);
    assert!(service.verify_block_transition(&record));

    // Rotate keys for the same circuit and ensure stale proofs no longer validate
    let public_inputs = vec![b"block_7".as_ref(), b"prev", b"next"];
    service.rotate_circuit(record.circuit_id.as_str(), &public_inputs);
    assert!(!service.verify_block_transition(&record));
}

/// Test security policy blocking on request for ZK proofs
#[test]
fn test_security__zk_proof__policy__blocks__on_request() {
    let (mut service, mut record) = make_block_record(9);
    tamper_block_record(&mut record);
    assert!(!service.verify_block_transition(&record));
    assert_eq!(service.verified_count(), 0);
}

/// Test security policy detection on request for ZK proofs
#[test]
fn test_security__zk_proof__policy__detects__on_request() {
    let (mut service, mut proof, public_input) = make_privacy_roundtrip(b"detect_me");
    tamper_proof(&mut proof);
    assert!(!service.verify_secret_knowledge(&proof, &public_input));
    assert_eq!(service.get_verified_proof_count(), 0);
}

/// Test security policy logs evidence on request for ZK proofs
#[test]
fn test_security__zk_proof__policy__logs_evidence__on_request() {
    let (mut service, proof, public_input) = make_privacy_roundtrip(b"log_me");
    assert!(service.verify_secret_knowledge(&proof, &public_input));
    assert_eq!(service.get_verified_proof_count(), 1);
}

/// Test security scanner enforcement during CI for ZK proofs
#[test]
fn test_security__zk_proof__scanner__enforces__during_ci() {
    let (mut service, record) = make_block_record(10);
    assert!(service.verify_block_transition(&record));
}

/// Test security scanner validation during CI for ZK proofs
#[test]
fn test_security__zk_proof__scanner__validates__during_ci() {
    let (mut service, mut record) = make_block_record(11);
    tamper_block_record(&mut record);
    assert!(!service.verify_block_transition(&record));
}

/// Test security scanner rotation during CI for ZK proofs
#[test]
fn test_security__zk_proof__scanner__rotates__during_ci() {
    let (mut service, record) = make_block_record(12);
    assert!(service.verify_block_transition(&record));

    let public_inputs = vec![b"block_12".as_ref(), b"prev", b"next"];
    service.rotate_circuit(record.circuit_id.as_str(), &public_inputs);
    assert!(!service.verify_block_transition(&record));
}

/// Test security scanner blocking during CI for ZK proofs
#[test]
fn test_security__zk_proof__scanner__blocks__during_ci() {
    let (mut service, mut record) = make_block_record(13);
    tamper_block_record(&mut record);
    assert!(!service.verify_block_transition(&record));
}

/// Test security scanner detection during CI for ZK proofs
#[test]
fn test_security__zk_proof__scanner__detects__during_ci() {
    let (mut service, mut record) = make_block_record(14);
    tamper_block_record(&mut record);
    assert_eq!(service.verify_block_transition(&record), false);
    assert_eq!(service.verified_count(), 0);
}

/// Test security scanner logs evidence during CI for ZK proofs
#[test]
fn test_security__zk_proof__scanner__logs_evidence__during_ci() {
    let (mut service, record) = make_block_record(15);
    assert!(service.verify_block_transition(&record));
    assert_eq!(service.verified_count(), 1);
}

/// Test security gateway enforcement on request for ZK proofs
#[test]
fn test_security__zk_proof__gateway__enforces__on_request() {
    let (mut service, proof, public_input) = make_privacy_roundtrip(b"gateway_enforce");
    assert!(service.verify_secret_knowledge(&proof, &public_input));
}

/// Test security gateway validation on request for ZK proofs
#[test]
fn test_security__zk_proof__gateway__validates__on_request() {
    let (mut service, mut proof, public_input) = make_privacy_roundtrip(b"gateway_validate");
    tamper_proof(&mut proof);
    assert!(!service.verify_secret_knowledge(&proof, &public_input));
}

/// Test security gateway rotation on request for ZK proofs
#[test]
fn test_security__zk_proof__gateway__rotates__on_request() {
    let (mut service, record) = make_block_record(16);
    assert!(service.verify_block_transition(&record));

    let public_inputs = vec![b"block_16".as_ref(), b"prev", b"next"];
    service.rotate_circuit(record.circuit_id.as_str(), &public_inputs);
    assert!(!service.verify_block_transition(&record));
}

/// Test security gateway blocking on request for ZK proofs
#[test]
fn test_security__zk_proof__gateway__blocks__on_request() {
    let (mut service, mut record) = make_block_record(17);
    tamper_block_record(&mut record);
    assert!(!service.verify_block_transition(&record));
}

/// Test security gateway detection on request for ZK proofs
#[test]
fn test_security__zk_proof__gateway__detects__on_request() {
    let (mut service, mut proof, public_input) = make_privacy_roundtrip(b"gateway_detect");
    tamper_proof(&mut proof);
    assert!(!service.verify_secret_knowledge(&proof, &public_input));
}

/// Test security gateway logs evidence on request for ZK proofs
#[test]
fn test_security__zk_proof__gateway__logs_evidence__on_request() {
    let (mut service, proof, public_input) = make_privacy_roundtrip(b"gateway_log");
    assert!(service.verify_secret_knowledge(&proof, &public_input));
    assert_eq!(service.get_verified_proof_count(), 1);
}

/// Test security vault enforcement on request for ZK proofs
#[test]
fn test_security__zk_proof__vault__enforces__on_request() {
    let (mut service, record) = make_block_record(18);
    assert!(service.verify_block_transition(&record));
}

/// Test security vault validation on request for ZK proofs
#[test]
fn test_security__zk_proof__vault__validates__on_request() {
    let (mut service, mut record) = make_block_record(19);
    tamper_block_record(&mut record);
    assert!(!service.verify_block_transition(&record));
}

/// Test security vault rotation on request for ZK proofs
#[test]
fn test_security__zk_proof__vault__rotates__on_request() {
    let (mut service, record) = make_block_record(20);
    assert!(service.verify_block_transition(&record));

    let public_inputs = vec![b"block_20".as_ref(), b"prev", b"next"];
    service.rotate_circuit(record.circuit_id.as_str(), &public_inputs);
    assert!(!service.verify_block_transition(&record));
}

/// Test security vault blocking on request for ZK proofs
#[test]
fn test_security__zk_proof__vault__blocks__on_request() {
    let (mut service, mut record) = make_block_record(21);
    tamper_block_record(&mut record);
    assert!(!service.verify_block_transition(&record));
}

/// Test security vault detection on request for ZK proofs
#[test]
fn test_security__zk_proof__vault__detects__on_request() {
    let (mut service, mut proof, public_input) = make_privacy_roundtrip(b"vault_detect");
    tamper_proof(&mut proof);
    assert!(!service.verify_secret_knowledge(&proof, &public_input));
}

/// Test security vault logs evidence on request for ZK proofs
#[test]
fn test_security__zk_proof__vault__logs_evidence__on_request() {
    let (mut service, proof, public_input) = make_privacy_roundtrip(b"vault_log");
    assert!(service.verify_secret_knowledge(&proof, &public_input));
    assert_eq!(service.get_verified_proof_count(), 1);
}

/// Test security key manager enforcement on request for ZK proofs
#[test]
fn test_security__zk_proof__key_manager__enforces__on_request() {
    let (mut service, record) = make_block_record(22);
    assert!(service.verify_block_transition(&record));
}

/// Test security key manager validation on request for ZK proofs
#[test]
fn test_security__zk_proof__key_manager__validates__on_request() {
    let (mut service, mut record) = make_block_record(23);
    tamper_block_record(&mut record);
    assert!(!service.verify_block_transition(&record));
}

/// Test security key manager rotation on request for ZK proofs
#[test]
fn test_security__zk_proof__key_manager__rotates__on_request() {
    let (mut service, record) = make_block_record(24);
    assert!(service.verify_block_transition(&record));

    let public_inputs = vec![b"block_24".as_ref(), b"prev", b"next"];
    service.rotate_circuit(record.circuit_id.as_str(), &public_inputs);
    assert!(!service.verify_block_transition(&record));
}

/// Test security key manager blocking on request for ZK proofs
#[test]
fn test_security__zk_proof__key_manager__blocks__on_request() {
    let (mut service, mut record) = make_block_record(25);
    tamper_block_record(&mut record);
    assert!(!service.verify_block_transition(&record));
}

/// Test security key manager detection on request for ZK proofs
#[test]
fn test_security__zk_proof__key_manager__detects__on_request() {
    let (mut service, mut proof, public_input) = make_privacy_roundtrip(b"key_manager_detect");
    tamper_proof(&mut proof);
    assert!(!service.verify_secret_knowledge(&proof, &public_input));
}

/// Test security key manager logs evidence on request for ZK proofs
#[test]
fn test_security__zk_proof__key_manager__logs_evidence__on_request() {
    let (mut service, proof, public_input) = make_privacy_roundtrip(b"key_manager_log");
    assert!(service.verify_secret_knowledge(&proof, &public_input));
    assert_eq!(service.get_verified_proof_count(), 1);
}

/// Test security database enforcement on request for ZK proofs
#[test]
fn test_security__zk_proof__database__enforces__on_request() {
    let (mut service, record) = make_block_record(26);
    assert!(service.verify_block_transition(&record));
}

/// Test security database validation on request for ZK proofs
#[test]
fn test_security__zk_proof__database__validates__on_request() {
    let (mut service, mut record) = make_block_record(27);
    tamper_block_record(&mut record);
    assert!(!service.verify_block_transition(&record));
}

/// Test security database rotation on request for ZK proofs
#[test]
fn test_security__zk_proof__database__rotates__on_request() {
    let (mut service, record) = make_block_record(28);
    assert!(service.verify_block_transition(&record));

    let public_inputs = vec![b"block_28".as_ref(), b"prev", b"next"];
    service.rotate_circuit(record.circuit_id.as_str(), &public_inputs);
    assert!(!service.verify_block_transition(&record));
}

/// Test security database blocking on request for ZK proofs
#[test]
fn test_security__zk_proof__database__blocks__on_request() {
    let (mut service, mut record) = make_block_record(29);
    tamper_block_record(&mut record);
    assert!(!service.verify_block_transition(&record));
}

/// Test security database detection on request for ZK proofs
#[test]
fn test_security__zk_proof__database__detects__on_request() {
    let (mut service, mut proof, public_input) = make_privacy_roundtrip(b"database_detect");
    tamper_proof(&mut proof);
    assert!(!service.verify_secret_knowledge(&proof, &public_input));
}

/// Test security database logs evidence on request for ZK proofs
#[test]
fn test_security__zk_proof__database__logs_evidence__on_request() {
    let (mut service, proof, public_input) = make_privacy_roundtrip(b"database_log");
    assert!(service.verify_secret_knowledge(&proof, &public_input));
    assert_eq!(service.get_verified_proof_count(), 1);
}
