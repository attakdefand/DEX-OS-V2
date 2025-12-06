//! Tests for consensus algorithms (Paxos and Two-Phase Commit)
//!
//! This module provides full validation of the Priority 3 consensus features from DEX-OS-V2.csv:
//! - Distributed Systems,Distributed Systems,Distributed Systems,Consensus,Paxos Algorithm,Medium
//! - Distributed Systems,Distributed Systems,Distributed Systems,Consensus,Two-Phase Commit,Medium

use dex_core::consensus::paxos::{PaxosConfig, PaxosError, PaxosNode, PaxosValue, ProposalNumber, NodeRole};
use dex_core::consensus::two_phase_commit::{TwoPhaseCommitConfig, TwoPhaseCommitError, TwoPhaseCommitNode, TransactionId, NodeRole as TpcNodeRole};

/// Test Paxos node creation and basic functionality
#[test]
fn test_paxos_node_creation() {
    let config = PaxosConfig::default();
    let proposer = PaxosNode::new(config.clone(), NodeRole::Proposer);
    let acceptor = PaxosNode::new(config.clone(), NodeRole::Acceptor);
    let learner = PaxosNode::new(config, NodeRole::Learner);
    
    assert_eq!(*proposer.get_role(), NodeRole::Proposer);
    assert_eq!(*acceptor.get_role(), NodeRole::Acceptor);
    assert_eq!(*learner.get_role(), NodeRole::Learner);
}

/// Test Paxos prepare request handling
#[test]
fn test_paxos_prepare_handling() {
    let config = PaxosConfig::default();
    let mut acceptor = PaxosNode::new(config, NodeRole::Acceptor);
    
    let request = dex_core::consensus::paxos::PrepareRequest {
        proposal_number: ProposalNumber::new(1, 1),
        proposer_id: 1,
    };
    
    let response = acceptor.handle_prepare(request).unwrap();
    
    assert_eq!(response.proposal_number.round, 1);
    assert_eq!(response.acceptor_id, 0);
}

/// Test Paxos accept request handling
#[test]
fn test_paxos_accept_handling() {
    let config = PaxosConfig::default();
    let mut acceptor = PaxosNode::new(config, NodeRole::Acceptor);
    
    let value = PaxosValue::new(b"test_value".to_vec());
    let request = dex_core::consensus::paxos::AcceptRequest {
        proposal_number: ProposalNumber::new(1, 1),
        value: value.clone(),
        proposer_id: 1,
    };
    
    let response = acceptor.handle_accept(request).unwrap();
    
    assert_eq!(response.proposal_number.round, 1);
}

/// Test Paxos learn notification handling
#[test]
fn test_paxos_learn_handling() {
    let config = PaxosConfig::default();
    let mut learner = PaxosNode::new(config, NodeRole::Learner);
    
    let value = PaxosValue::new(b"test_value".to_vec());
    let notification = dex_core::consensus::paxos::LearnNotification {
        proposal_number: ProposalNumber::new(1, 1),
        value: value.clone(),
        proposer_id: 1,
    };
    
    assert!(learner.handle_learn(notification).is_ok());
    assert_eq!(learner.get_decided_value().unwrap().data, b"test_value");
}

/// Test Paxos proposal number ordering
#[test]
fn test_paxos_proposal_number_ordering() {
    let p1 = ProposalNumber::new(1, 1);
    let p2 = ProposalNumber::new(1, 2);
    let p3 = ProposalNumber::new(2, 1);
    
    assert!(p1 < p2);
    assert!(p1 < p3);
    assert!(p2 < p3);
}

/// Test Two-Phase Commit node creation and basic functionality
#[test]
fn test_two_phase_commit_node_creation() {
    let config = TwoPhaseCommitConfig::default();
    let coordinator = TwoPhaseCommitNode::new(config.clone(), TpcNodeRole::Coordinator);
    let participant = TwoPhaseCommitNode::new(config, TpcNodeRole::Participant);
    
    assert_eq!(*coordinator.get_role(), TpcNodeRole::Coordinator);
    assert_eq!(*participant.get_role(), TpcNodeRole::Participant);
}

/// Test Two-Phase Commit transaction beginning
#[test]
fn test_two_phase_commit_begin_transaction() {
    let config = TwoPhaseCommitConfig::default();
    let mut coordinator = TwoPhaseCommitNode::new(config, TpcNodeRole::Coordinator);
    
    let tx_id = TransactionId::new("test-transaction".to_string());
    let data = b"test_data".to_vec();
    
    assert!(coordinator.begin_transaction(tx_id.clone(), data.clone()).is_ok());
    
    // Check that transaction state is Active
    assert_eq!(
        *coordinator.get_transaction_state(&tx_id).unwrap(),
        dex_core::consensus::two_phase_commit::TransactionState::Active
    );
}

/// Test Two-Phase Commit prepare request handling
#[test]
fn test_two_phase_commit_prepare_handling() {
    let config = TwoPhaseCommitConfig::default();
    let mut participant = TwoPhaseCommitNode::new(config, TpcNodeRole::Participant);
    
    let tx_id = TransactionId::new("test-transaction".to_string());
    let request = dex_core::consensus::two_phase_commit::PrepareRequest {
        transaction_id: tx_id.clone(),
        data: b"test_data".to_vec(),
    };
    
    let response = participant.handle_prepare(request).unwrap();
    
    assert_eq!(response.transaction_id.id, "test-transaction");
    assert_eq!(response.vote, true);
}

/// Test Two-Phase Commit commit request handling
#[test]
fn test_two_phase_commit_commit_handling() {
    let config = TwoPhaseCommitConfig::default();
    let mut participant = TwoPhaseCommitNode::new(config, TpcNodeRole::Participant);
    
    let tx_id = TransactionId::new("test-transaction".to_string());
    let request = dex_core::consensus::two_phase_commit::CommitRequest {
        transaction_id: tx_id.clone(),
    };
    
    let response = participant.handle_commit(request).unwrap();
    
    assert_eq!(response.transaction_id.id, "test-transaction");
    assert_eq!(response.success, true);
}

/// Test Two-Phase Commit abort request handling
#[test]
fn test_two_phase_commit_abort_handling() {
    let config = TwoPhaseCommitConfig::default();
    let mut participant = TwoPhaseCommitNode::new(config, TpcNodeRole::Participant);
    
    let tx_id = TransactionId::new("test-transaction".to_string());
    let request = dex_core::consensus::two_phase_commit::AbortRequest {
        transaction_id: tx_id.clone(),
    };
    
    let response = participant.handle_abort(request).unwrap();
    
    assert_eq!(response.transaction_id.id, "test-transaction");
    assert_eq!(response.success, true);
}

/// Test error handling for non-participant nodes
#[test]
fn test_paxos_non_participant_error() {
    let config = PaxosConfig::default();
    let mut node = PaxosNode::new(config, NodeRole::Proposer);
    
    // Try to handle prepare as proposer (should fail)
    let request = dex_core::consensus::paxos::PrepareRequest {
        proposal_number: ProposalNumber::new(1, 1),
        proposer_id: 1,
    };
    
    assert_eq!(node.handle_prepare(request), Err(PaxosError::NotParticipant));
}

/// Test error handling for non-participant nodes in Two-Phase Commit
#[test]
fn test_two_phase_commit_non_participant_error() {
    let config = TwoPhaseCommitConfig::default();
    let mut node = TwoPhaseCommitNode::new(config, TpcNodeRole::Participant);
    
    let tx_id = TransactionId::new("test-transaction".to_string());
    
    // Try to begin transaction as participant (should fail)
    assert_eq!(
        node.begin_transaction(tx_id, b"test_data".to_vec()),
        Err(TwoPhaseCommitError::NotParticipant)
    );
}

/// Test Paxos value creation
#[test]
fn test_paxos_value_creation() {
    let value = PaxosValue::new(b"test_value".to_vec());
    
    assert_eq!(value.data, b"test_value");
    assert!(value.timestamp > 0);
}

/// Test TransactionId creation
#[test]
fn test_transaction_id_creation() {
    let tx_id = TransactionId::new("test-transaction".to_string());
    
    assert_eq!(tx_id.id, "test-transaction");
    assert!(tx_id.timestamp > 0);
}