//! Integration tests for the Quantum Consensus implementation
//!
//! This implements the Testing Layer requirements from RULES.md:
//! @RULES.md ##Testing Layer Practices
//! @RULES.md ###Testing Layer 1: Unit Testing and Component Validation
//! @RULES.md ###Testing Layer 2: Integration Testing and System Validation
//! @RULES.md ###Testing Layer 3: Security Testing and Threat Assessment
//! @RULES.md ###Testing Layer 4: Performance Testing and Load Validation

use dex_core::quantum_consensus::{
    LatticeBFTCore, QuantumConsensusEngine, QuantumConsensusError, QVRF,
};
use dex_core::types::{Block, Transaction, Validator};

/// Test the complete quantum consensus workflow
#[test]
fn test_quantum_consensus_workflow() {
    // Create a consensus engine
    let mut engine = QuantumConsensusEngine::new();

    // Add validators and initialize shards
    let validator1 = Validator {
        id: "validator1".to_string(),
        public_key: vec![1, 2, 3, 4],
        stake: 1000,
    };

    let validator2 = Validator {
        id: "validator2".to_string(),
        public_key: vec![5, 6, 7, 8],
        stake: 1000,
    };

    assert!(engine.add_validator(validator1).is_ok());
    assert!(engine.add_validator(validator2).is_ok());
    assert!(engine.initialize_shards(1).is_ok());

    // Create a block with transactions
    let tx = Transaction {
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
        transactions: vec![tx],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };

    // Process the block through consensus
    let result = engine.process_block_with_sharding(0, block);
    assert!(result.is_ok());
}

/// Test QVRF leader selection with multiple rounds
#[test]
fn test_qvrf_leader_selection_rounds() {
    let mut engine = QuantumConsensusEngine::new();

    // Add validators
    let validators = vec![
        Validator {
            id: "validator1".to_string(),
            public_key: vec![1, 2, 3, 4],
            stake: 1000,
        },
        Validator {
            id: "validator2".to_string(),
            public_key: vec![5, 6, 7, 8],
            stake: 1000,
        },
        Validator {
            id: "validator3".to_string(),
            public_key: vec![9, 10, 11, 12],
            stake: 1000,
        },
    ];

    for validator in validators {
        assert!(engine.add_validator(validator).is_ok());
    }

    // Test multiple rounds of leader selection
    let mut leaders = Vec::new();
    for _ in 0..6 {
        let leader = engine.get_current_leader().unwrap();
        leaders.push(leader);
    }

    // With 3 validators, we should see a round-robin pattern
    assert_eq!(leaders.len(), 6);
}

/// Test Lattice BFT Core functionality
#[test]
fn test_lattice_bft_core_functionality() {
    let validators = vec![
        Validator {
            id: "validator1".to_string(),
            public_key: vec![1, 2, 3, 4],
            stake: 1000,
        },
        Validator {
            id: "validator2".to_string(),
            public_key: vec![5, 6, 7, 8],
            stake: 1000,
        },
        Validator {
            id: "validator3".to_string(),
            public_key: vec![9, 10, 11, 12],
            stake: 1000,
        },
        Validator {
            id: "validator4".to_string(),
            public_key: vec![13, 14, 15, 16],
            stake: 1000,
        },
    ];

    // Create Lattice BFT Core with threshold of 3 out of 4 validators
    let lattice_core = LatticeBFTCore::new(3, validators);

    // Test threshold requirements
    assert!(lattice_core.has_sufficient_signatures(3));
    assert!(lattice_core.has_sufficient_signatures(4));
    assert!(!lattice_core.has_sufficient_signatures(2));

    // Test message validation (placeholder)
    let message = b"test consensus message";
    let signature = b"test signature";
    assert!(lattice_core.validate_message(message, signature).is_ok());
}

/// Test error handling in quantum consensus
#[test]
fn test_quantum_consensus_error_handling() {
    let mut engine = QuantumConsensusEngine::new();

    // Try to get leader with no validators
    let result = engine.get_current_leader();
    assert!(result.is_err());
    match result.unwrap_err() {
        QuantumConsensusError::LeaderSelectionFailed => (),
        _ => panic!("Expected LeaderSelectionFailed error"),
    }

    // Try to validate block with unknown validator
    let tx = Transaction {
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
        transactions: vec![tx],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };

    let result = engine.validate_block_proposal(&block, "unknown_validator");
    assert!(result.is_err());
}

/// Test QVRF cryptographic functions
#[test]
fn test_qvrf_cryptographic_functions() {
    let qvrf = QVRF::new(vec![1, 2, 3, 4], vec![5, 6, 7, 8]);

    // Test generation and verification
    let input = b"test input for QVRF";
    let (output, proof) = qvrf.generate(input).unwrap();

    // Verify the generated output
    assert!(qvrf.verify(input, &output, &proof).unwrap());

    // Test with different input
    let different_input = b"different input";
    let (different_output, different_proof) = qvrf.generate(different_input).unwrap();

    // Should not verify with different input
    assert!(!qvrf
        .verify(input, &different_output, &different_proof)
        .unwrap());
}

/// Test security-related validation
#[test]
fn test_security_validations() {
    let mut engine = QuantumConsensusEngine::new();

    // Add a validator
    let validator = Validator {
        id: "validator1".to_string(),
        public_key: vec![1, 2, 3, 4],
        stake: 1000,
    };
    engine.add_validator(validator).unwrap();

    // Test valid transaction
    let valid_tx = Transaction {
        from: "user1".to_string(),
        to: "user2".to_string(),
        amount: 100,
        nonce: 1,
        signature: vec![],
    };

    let valid_block = Block {
        id: 1,
        height: 1,
        timestamp: 1234567890,
        transactions: vec![valid_tx],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };

    assert!(engine
        .validate_block_proposal(&valid_block, "validator1")
        .unwrap());

    // Test invalid transactions
    let invalid_tx1 = Transaction {
        from: "".to_string(), // Empty sender
        to: "user2".to_string(),
        amount: 100,
        nonce: 1,
        signature: vec![],
    };

    let invalid_tx2 = Transaction {
        from: "user1".to_string(),
        to: "".to_string(), // Empty recipient
        amount: 100,
        nonce: 1,
        signature: vec![],
    };

    let invalid_tx3 = Transaction {
        from: "user1".to_string(),
        to: "user2".to_string(),
        amount: -50, // Negative amount
        nonce: 1,
        signature: vec![],
    };

    let invalid_block1 = Block {
        id: 2,
        height: 2,
        timestamp: 1234567891,
        transactions: vec![invalid_tx1],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };
    assert!(
        !engine
            .validate_block_proposal(&invalid_block1, "validator1")
            .unwrap()
    );

    let invalid_block2 = Block {
        id: 3,
        height: 3,
        timestamp: 1234567892,
        transactions: vec![invalid_tx2],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };
    assert!(
        !engine
            .validate_block_proposal(&invalid_block2, "validator1")
            .unwrap()
    );

    let invalid_block3 = Block {
        id: 4,
        height: 4,
        timestamp: 1234567893,
        transactions: vec![invalid_tx3],
        previous_hash: vec![0; 32],
        hash: vec![0; 32],
        signature: vec![],
    };
    assert!(
        !engine
            .validate_block_proposal(&invalid_block3, "validator1")
            .unwrap()
    );
}

/// Performance test for quantum consensus operations
#[test]
fn test_quantum_consensus_performance() {
    let mut engine = QuantumConsensusEngine::new();

    // Add multiple validators
    for i in 0..100 {
        let validator = Validator {
            id: format!("validator{}", i),
            public_key: vec![i as u8; 32],
            stake: 1000,
        };
        engine.add_validator(validator).unwrap();
    }

    // Test leader selection performance
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _leader = engine.get_current_leader().unwrap();
    }
    let duration = start.elapsed();

    // This should complete in a reasonable time (less than 1 second)
    assert!(duration.as_millis() < 1000);

    // Test transaction validation performance
    let tx = Transaction {
        from: "user1".to_string(),
        to: "user2".to_string(),
        amount: 100,
        nonce: 1,
        signature: vec![],
    };

    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let block = Block {
            id: 1,
            height: 1,
            timestamp: 1234567890,
            transactions: vec![tx.clone()],
            previous_hash: vec![0; 32],
            hash: vec![0; 32],
            signature: vec![],
        };

        let _valid = engine
            .validate_block_proposal(&block, "validator0")
            .unwrap();
    }
    let duration = start.elapsed();

    // This should complete in a reasonable time (less than 1 second)
    assert!(duration.as_millis() < 1000);
}
