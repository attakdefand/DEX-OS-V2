//! Comprehensive tests for Consensus Finality implementation
//!
//! This file implements security and functionality tests for the Priority 3 feature:
//! - Blockchain Resilience,Blockchain Resilience,Blockchain Resilience,Consensus Finality,Casper FFG,Medium

use dex_core::quantum_consensus::{
    QuantumConsensusEngine, GlobalFinalityTracker
};
use dex_core::types::{Block, Transaction, Validator};

/// Test comprehensive quantum consensus engine functionality
#[test]
fn test_quantum_consensus_engine_comprehensive() {
    let mut engine = QuantumConsensusEngine::new();
    
    // Test initial state
    // Note: current_round and current_leader are private fields, so we'll test indirectly
    // We can't directly access engine.current_round or engine.current_leader
    // But we can test that the engine was initialized properly through other methods    
    // Add validators
    let validator1 = Validator {
        id: "validator1".to_string(),
        public_key: vec![1, 2, 3, 4],
        stake: 1000,
    };
    
    let validator2 = Validator {
        id: "validator2".to_string(),
        public_key: vec![5, 6, 7, 8],
        stake: 1500,
    };
    
    let validator3 = Validator {
        id: "validator3".to_string(),
        public_key: vec![9, 10, 11, 12],
        stake: 2000,
    };
    
    engine.add_validator(validator1).unwrap();
    engine.add_validator(validator2).unwrap();
    engine.add_validator(validator3).unwrap();
    
    // Test validator management
    // Test that we can now get a leader since we have validators
    // We'll skip directly checking validators map since it's private
    
    // Test leader selection
    let leader = engine.get_current_leader().unwrap();
    assert!(["validator1", "validator2", "validator3"].contains(&leader.as_str()));
    
    // Test transaction validation
    let valid_tx = Transaction {
        from: "user1".to_string(),
        to: "user2".to_string(),
        amount: 100,
        nonce: 1,
        signature: vec![],
    };
    
    // Note: validate_transaction is private, so we'll test through validate_block_proposal
    let block_with_valid_tx = Block {
        id: 1,
        height: 1,
        timestamp: 1234567890,
        transactions: vec![valid_tx],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };
    assert!(engine.validate_block_proposal(&block_with_valid_tx, "validator1").unwrap());
    
    let invalid_tx = Transaction {
        from: "".to_string(),
        to: "user2".to_string(),
        amount: 100,
        nonce: 1,
        signature: vec![],
    };
    
    // Note: validate_transaction is private, so we'll test through validate_block_proposal
    let block_with_invalid_tx = Block {
        id: 2,
        height: 1,
        timestamp: 1234567890,
        transactions: vec![invalid_tx],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };
    assert!(!engine.validate_block_proposal(&block_with_invalid_tx, "validator1").unwrap());
    
    // Test block validation
    let valid_tx2 = Transaction {
        from: "user1".to_string(),
        to: "user2".to_string(),
        amount: 100,
        nonce: 1,
        signature: vec![],
    };
    
    let block = Block {
        id: 1,
        height: 1,
        timestamp: 1234567890,
        transactions: vec![valid_tx2],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };
    
    assert!(engine.validate_block_proposal(&block, "validator1").unwrap());
    
    // Test shard initialization
    assert!(engine.initialize_shards(5).is_ok());
    // Skip checking shards length directly since get_shards() returns a reference to a private field
    // We'll test by checking individual shards instead
    for shard_id in 0..5 {
        assert!(engine.get_shard(shard_id).is_some());
    }
    
    // Check that each shard was created properly
    for shard_id in 0..5 {
        let shard = engine.get_shard(shard_id).unwrap();
        assert_eq!(shard.id, shard_id);
        assert!(!shard.validators.is_empty());
        assert_eq!(shard.blocks.len(), 0);
    }
    
    // Test block processing with sharding
    let tx2 = Transaction {
        from: "user3".to_string(),
        to: "user4".to_string(),
        amount: 200,
        nonce: 1,
        signature: vec![],
    };
    
    let block2 = Block {
        id: 2,
        height: 1,
        timestamp: 1234567891,
        transactions: vec![tx2],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };
    
    assert!(engine.process_block_with_sharding(2, block2).is_ok());
    
    // Check that block was added to shard
    let shard = engine.get_shard(2).unwrap();
    assert_eq!(shard.blocks.len(), 1);
    assert_eq!(shard.blocks[0].id, 2);
}

/// Test global finality tracker comprehensive functionality
#[test]
fn test_global_finality_tracker_comprehensive() {
    let mut tracker = GlobalFinalityTracker::new();
    
    // Test initial state
    assert_eq!(tracker.get_global_finalized_height(), 0);
    // Rely on the get_global_finalized_height method since finalized_heights is private
    
    // Test updating shard finalities
    tracker.update_shard_finality(1, 100);
    assert_eq!(tracker.get_global_finalized_height(), 100);
    assert_eq!(tracker.get_shard_finalized_height(1), Some(100));
    assert_eq!(tracker.get_shard_finalized_height(2), None);
    
    tracker.update_shard_finality(2, 90);
    assert_eq!(tracker.get_global_finalized_height(), 90); // Minimum of all shards
    assert_eq!(tracker.get_shard_finalized_height(2), Some(90));
    
    tracker.update_shard_finality(3, 95);
    assert_eq!(tracker.get_global_finalized_height(), 90); // Still minimum
    assert_eq!(tracker.get_shard_finalized_height(3), Some(95));
    
    // Test updating existing shard to higher value
    tracker.update_shard_finality(2, 105);
    assert_eq!(tracker.get_global_finalized_height(), 95); // New minimum
    assert_eq!(tracker.get_shard_finalized_height(2), Some(105));
    
    // Test updating existing shard to lower value
    tracker.update_shard_finality(1, 85);
    assert_eq!(tracker.get_global_finalized_height(), 85); // New minimum
    assert_eq!(tracker.get_shard_finalized_height(1), Some(85));
    
    // Test with many shards
    for i in 4..10 {
        tracker.update_shard_finality(i, 100 + i as u64);
    }
    
    assert_eq!(tracker.get_global_finalized_height(), 85); // Still the minimum
    // Test by checking individual shard finalities since finalized_heights is private
    assert_eq!(tracker.get_shard_finalized_height(1), Some(85));
    assert_eq!(tracker.get_shard_finalized_height(2), Some(105));
    // ... and so on for other shards
    
    // Update shard 1 to be higher than others
    tracker.update_shard_finality(1, 200);
    assert_eq!(tracker.get_global_finalized_height(), 90); // Next minimum (shard 2)
}

/// Test edge cases and error conditions for quantum consensus
#[test]
fn test_quantum_consensus_edge_cases() {
    let mut engine = QuantumConsensusEngine::new();
    
    // Test leader selection with no validators
    let result = engine.get_current_leader();
    assert!(result.is_err());
    
    // Test block validation with unknown validator
    let block = Block {
        id: 1,
        height: 1,
        timestamp: 1234567890,
        transactions: vec![],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };
    
    let result = engine.validate_block_proposal(&block, "unknown_validator");
    assert!(result.is_err());
    
    // Add a validator for further testing
    let validator = Validator {
        id: "validator1".to_string(),
        public_key: vec![1, 2, 3, 4],
        stake: 1000,
    };
    engine.add_validator(validator).unwrap();
    
    // Test shard initialization with invalid parameters
    let result = engine.initialize_shards(0);
    assert!(result.is_err());
    
    let result = engine.initialize_shards(1_000_001);
    assert!(result.is_err());
    
    // Test block processing for non-existent shard
    let block = Block {
        id: 1,
        height: 1,
        timestamp: 1234567890,
        transactions: vec![],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };
    
    let result = engine.process_block_with_sharding(999, block);
    assert!(result.is_err());
    
    // Initialize shards and test valid block processing
    engine.initialize_shards(3).unwrap();
    let block = Block {
        id: 1,
        height: 1,
        timestamp: 1234567890,
        transactions: vec![],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };
    
    assert!(engine.process_block_with_sharding(1, block).is_ok());
    
    // Test removing validator
    assert!(engine.remove_validator("validator1").is_ok());
    // Test by trying to get the current leader which should now fail since we removed the validator
    let leader_result = engine.get_current_leader();
    assert!(leader_result.is_err());
    
    // Test removing non-existent validator
    assert!(engine.remove_validator("nonexistent").is_ok()); // Should not error
}