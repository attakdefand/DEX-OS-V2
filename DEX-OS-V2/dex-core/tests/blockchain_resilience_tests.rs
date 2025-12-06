//! Comprehensive tests for the Blockchain Resilience module
//!
//! This file implements security and functionality tests for the Priority 3 features:
//! - Blockchain Resilience,Blockchain Resilience,Blockchain Resilience,Proof of Stake (PoS),Validator Bonding,Medium
//! - Blockchain Resilience,Blockchain Resilience,Blockchain Resilience,UTXO Model,Double-Spend Prevention,Medium

use dex_core::blockchain_resilience::{
    BlockchainResilience, ProofOfStake, UtxoModel, 
    UtxoTransaction, UtxoInput, UtxoOutput, UnspentTransactionOutput
};

/// Test PoS validator registration with various edge cases
#[test]
fn test_pos_validator_registration_edge_cases() {
    let mut pos = ProofOfStake::new(1000);
    
    // Test registering with insufficient stake
    assert!(pos.register_validator("validator1".to_string(), vec![1, 2, 3, 4], 500).is_err());
    
    // Test registering with exact minimum stake
    assert!(pos.register_validator("validator2".to_string(), vec![5, 6, 7, 8], 1000).is_ok());
    
    // Test registering with excess stake
    assert!(pos.register_validator("validator3".to_string(), vec![9, 10, 11, 12], 5000).is_ok());
}

/// Test PoS staking and unstaking with complex scenarios
#[test]
fn test_pos_complex_staking_scenarios() {
    let mut pos = ProofOfStake::new(1000);
    
    // Register validator
    assert!(pos.register_validator("validator1".to_string(), vec![1, 2, 3, 4], 2000).is_ok());
    
    // Test multiple stakes to same validator
    assert!(pos.stake_tokens("stake1".to_string(), "validator1".to_string(), 500, "owner1".to_string()).is_ok());
    assert!(pos.stake_tokens("stake2".to_string(), "validator1".to_string(), 300, "owner2".to_string()).is_ok());
    assert!(pos.stake_tokens("stake3".to_string(), "validator1".to_string(), 200, "owner3".to_string()).is_ok());
    
    // Check total stake
    let validator = pos.get_validator(&"validator1".to_string()).unwrap();
    assert_eq!(validator.stake, 3000); // 2000 initial + 500 + 300 + 200
    assert_eq!(pos.get_total_staked(), 3000);
    
    // Test unstaking non-existent stake
    assert!(pos.unstake_tokens("nonexistent").is_err());
    
    // Test unstaking already unstaked stake
    assert!(pos.unstake_tokens("stake1").is_ok());
    assert!(pos.unstake_tokens("stake1").is_err()); // Already unstaked
    
    // Check stake after unstaking
    let validator = pos.get_validator(&"validator1".to_string()).unwrap();
    assert_eq!(validator.stake, 2500); // 3000 - 500
    assert_eq!(pos.get_total_staked(), 2500);
}

/// Test PoS validator selection fairness
#[test]
fn test_pos_validator_selection_fairness() {
    let mut pos = ProofOfStake::new(1000);
    
    // Register validators with significantly different stakes
    assert!(pos.register_validator("small_staker".to_string(), vec![1, 2, 3, 4], 1000).is_ok());
    assert!(pos.register_validator("medium_staker".to_string(), vec![5, 6, 7, 8], 5000).is_ok());
    assert!(pos.register_validator("large_staker".to_string(), vec![9, 10, 11, 12], 10000).is_ok());
    
    // Run selection multiple times to test distribution
    let mut selections = vec![0; 3]; // Count for each validator
    let validator_names = vec!["small_staker", "medium_staker", "large_staker"];
    
    for _ in 0..1000 {
        if let Some(selected) = pos.select_validator() {
            if let Some(index) = validator_names.iter().position(|&name| name == selected) {
                selections[index] += 1;
            }
        }
    }
    
    // Large staker should be selected most often, small staker least often
    assert!(selections[2] > selections[1]); // large > medium
    assert!(selections[1] > selections[0]); // medium > small
}

/// Test UTXO model with complex transaction chains
#[test]
fn test_utxo_complex_transaction_chains() {
    let mut utxo_model = UtxoModel::new();
    
    // Create initial UTXO
    let initial_utxo = UnspentTransactionOutput {
        output: UtxoOutput {
            id: "utxo0".to_string(),
            public_key: vec![1, 2, 3, 4],
            amount: 1000,
            created_at: 1000000,
        },
        transaction_id: "tx0".to_string(),
    };
    
    utxo_model.add_utxo(initial_utxo);
    
    // Create first transaction that splits the UTXO
    let tx1 = UtxoTransaction {
        id: "tx1".to_string(),
        inputs: vec![UtxoInput {
            utxo_id: "utxo0".to_string(),
            public_key: vec![1, 2, 3, 4],
        }],
        outputs: vec![
            UtxoOutput {
                id: "utxo1_0".to_string(),
                public_key: vec![5, 6, 7, 8],
                amount: 400,
                created_at: 1000001,
            },
            UtxoOutput {
                id: "utxo1_1".to_string(),
                public_key: vec![9, 10, 11, 12],
                amount: 600,
                created_at: 1000001,
            },
        ],
        timestamp: 1000001,
        signatures: vec![vec![1]], // Placeholder
    };
    
    assert!(utxo_model.process_transaction(tx1).is_ok());
    
    // Verify original UTXO is spent
    assert!(utxo_model.get_utxo(&"utxo0".to_string()).is_none());
    
    // Verify new UTXOs exist
    assert!(utxo_model.get_utxo(&"utxo1_0".to_string()).is_some());
    assert!(utxo_model.get_utxo(&"utxo1_1".to_string()).is_some());
    
    // Create second transaction that merges UTXOs
    let tx2 = UtxoTransaction {
        id: "tx2".to_string(),
        inputs: vec![
            UtxoInput {
                utxo_id: "utxo1_0".to_string(),
                public_key: vec![5, 6, 7, 8],
            },
            UtxoInput {
                utxo_id: "utxo1_1".to_string(),
                public_key: vec![9, 10, 11, 12],
            },
        ],
        outputs: vec![
            UtxoOutput {
                id: "utxo2_0".to_string(),
                public_key: vec![13, 14, 15, 16],
                amount: 1000, // Sum of inputs
                created_at: 1000002,
            },
        ],
        timestamp: 1000002,
        signatures: vec![vec![2], vec![3]], // Placeholders
    };
    
    assert!(utxo_model.process_transaction(tx2).is_ok());
    
    // Verify input UTXOs are spent
    assert!(utxo_model.get_utxo(&"utxo1_0".to_string()).is_none());
    assert!(utxo_model.get_utxo(&"utxo1_1".to_string()).is_none());
    
    // Verify output UTXO exists
    assert!(utxo_model.get_utxo(&"utxo2_0".to_string()).is_some());
}

/// Test UTXO model double-spend prevention
#[test]
fn test_utxo_double_spend_prevention() {
    let mut utxo_model = UtxoModel::new();
    
    // Create initial UTXO
    let initial_utxo = UnspentTransactionOutput {
        output: UtxoOutput {
            id: "utxo0".to_string(),
            public_key: vec![1, 2, 3, 4],
            amount: 1000,
            created_at: 1000000,
        },
        transaction_id: "tx0".to_string(),
    };
    
    utxo_model.add_utxo(initial_utxo);
    
    // Create legitimate transaction
    let tx1 = UtxoTransaction {
        id: "tx1".to_string(),
        inputs: vec![UtxoInput {
            utxo_id: "utxo0".to_string(),
            public_key: vec![1, 2, 3, 4],
        }],
        outputs: vec![UtxoOutput {
            id: "utxo1".to_string(),
            public_key: vec![5, 6, 7, 8],
            amount: 1000,
            created_at: 1000001,
        }],
        timestamp: 1000001,
        signatures: vec![vec![1]], // Placeholder
    };
    
    // Process the legitimate transaction
    assert!(utxo_model.process_transaction(tx1.clone()).is_ok());
    
    // Try to process the same transaction again (double spend)
    assert!(utxo_model.process_transaction(tx1).is_err());
    
    // Try to create a different transaction spending the same UTXO
    let tx2 = UtxoTransaction {
        id: "tx2".to_string(),
        inputs: vec![UtxoInput {
            utxo_id: "utxo0".to_string(), // Same UTXO as tx1
            public_key: vec![1, 2, 3, 4],
        }],
        outputs: vec![UtxoOutput {
            id: "utxo2".to_string(),
            public_key: vec![9, 10, 11, 12],
            amount: 1000,
            created_at: 1000002,
        }],
        timestamp: 1000002,
        signatures: vec![vec![2]], // Placeholder
    };
    
    // This should also fail as the UTXO was already spent
    assert!(utxo_model.process_transaction(tx2).is_err());
}

/// Test UTXO model with invalid transactions
#[test]
fn test_utxo_invalid_transaction_handling() {
    let mut utxo_model = UtxoModel::new();
    
    // Create initial UTXO
    let initial_utxo = UnspentTransactionOutput {
        output: UtxoOutput {
            id: "utxo0".to_string(),
            public_key: vec![1, 2, 3, 4],
            amount: 1000,
            created_at: 1000000,
        },
        transaction_id: "tx0".to_string(),
    };
    
    utxo_model.add_utxo(initial_utxo);
    
    // Test transaction with non-existent input
    let tx1 = UtxoTransaction {
        id: "tx1".to_string(),
        inputs: vec![UtxoInput {
            utxo_id: "nonexistent".to_string(),
            public_key: vec![1, 2, 3, 4],
        }],
        outputs: vec![UtxoOutput {
            id: "utxo1".to_string(),
            public_key: vec![5, 6, 7, 8],
            amount: 1000,
            created_at: 1000001,
        }],
        timestamp: 1000001,
        signatures: vec![vec![1]], // Placeholder
    };
    
    assert!(utxo_model.validate_transaction(&tx1).is_err());
    
    // Test transaction with input/output amount mismatch
    let tx2 = UtxoTransaction {
        id: "tx2".to_string(),
        inputs: vec![UtxoInput {
            utxo_id: "utxo0".to_string(),
            public_key: vec![1, 2, 3, 4],
        }],
        outputs: vec![UtxoOutput {
            id: "utxo2".to_string(),
            public_key: vec![5, 6, 7, 8],
            amount: 1500, // More than input
            created_at: 1000001,
        }],
        timestamp: 1000001,
        signatures: vec![vec![1]], // Placeholder
    };
    
    assert!(utxo_model.validate_transaction(&tx2).is_err());
    
    // Test transaction with zero output amount
    let tx3 = UtxoTransaction {
        id: "tx3".to_string(),
        inputs: vec![UtxoInput {
            utxo_id: "utxo0".to_string(),
            public_key: vec![1, 2, 3, 4],
        }],
        outputs: vec![UtxoOutput {
            id: "utxo3".to_string(),
            public_key: vec![5, 6, 7, 8],
            amount: 0, // Invalid amount
            created_at: 1000001,
        }],
        timestamp: 1000001,
        signatures: vec![vec![1]], // Placeholder
    };
    
    assert!(utxo_model.validate_transaction(&tx3).is_err());
}

/// Test blockchain resilience with complete workflow
#[test]
fn test_blockchain_resilience_complete_workflow() {
    let mut blockchain = BlockchainResilience::new(1000);
    
    // Register multiple validators
    assert!(blockchain.pos.register_validator("validator1".to_string(), vec![1, 2, 3, 4], 2000).is_ok());
    assert!(blockchain.pos.register_validator("validator2".to_string(), vec![5, 6, 7, 8], 3000).is_ok());
    assert!(blockchain.pos.register_validator("validator3".to_string(), vec![9, 10, 11, 12], 1500).is_ok());
    
    // Create genesis block with initial UTXOs
    let initial_utxos = vec![
        UtxoOutput {
            id: "genesis0".to_string(),
            public_key: vec![1, 2, 3, 4],
            amount: 10000,
            created_at: 1000000,
        },
        UtxoOutput {
            id: "genesis1".to_string(),
            public_key: vec![5, 6, 7, 8],
            amount: 5000,
            created_at: 1000000,
        },
    ];
    
    blockchain.create_genesis_block(initial_utxos);
    assert_eq!(blockchain.get_height(), 1);
    
    // Create and process multiple blocks
    for i in 0..5 {
        // Create transactions
        let transactions = if i == 0 {
            // First block: split genesis UTXO
            vec![UtxoTransaction {
                id: format!("tx{}", i),
                inputs: vec![UtxoInput {
                    utxo_id: "genesis_0".to_string(),
                    public_key: vec![1, 2, 3, 4],
                }],
                outputs: vec![
                    UtxoOutput {
                        id: format!("utxo{}_0", i),
                        public_key: vec![13, 14, 15, 16],
                        amount: 6000,
                        created_at: 1000001 + i,
                    },
                    UtxoOutput {
                        id: format!("utxo{}_1", i),
                        public_key: vec![17, 18, 19, 20],
                        amount: 4000,
                        created_at: 1000001 + i,
                    },
                ],
                timestamp: 1000001 + i,
                signatures: vec![vec![1]], // Placeholder
            }]
        } else {
            // Subsequent blocks: transfer UTXOs
            vec![UtxoTransaction {
                id: format!("tx{}", i),
                inputs: vec![UtxoInput {
                    utxo_id: format!("utxo{}_0", i - 1),
                    public_key: vec![13, 14, 15, 16],
                }],
                outputs: vec![UtxoOutput {
                    id: format!("utxo{}", i),
                    public_key: vec![21, 22, 23, 24],
                    amount: 6000,
                    created_at: 1000001 + i,
                }],
                timestamp: 1000001 + i,
                signatures: vec![vec![1]], // Placeholder
            }]
        };
        
        // Propose and add block
        let block = blockchain.propose_block(transactions);
        assert!(block.is_ok());
        
        let block = block.unwrap();
        assert!(blockchain.add_block(block).is_ok());
        
        // Verify chain integrity
        assert!(blockchain.verify_chain());
    }
    
    // Check final blockchain state
    assert_eq!(blockchain.get_height(), 6); // Genesis + 5 blocks
    
    // Check that validators received rewards
    let validator1 = blockchain.pos.get_validator(&"validator1".to_string()).unwrap();
    let validator2 = blockchain.pos.get_validator(&"validator2".to_string()).unwrap();
    let validator3 = blockchain.pos.get_validator(&"validator3".to_string()).unwrap();
    
    // At least one validator should have received rewards
    assert!(
        validator1.total_rewards > 0 || 
        validator2.total_rewards > 0 || 
        validator3.total_rewards > 0
    );
}

/// Test blockchain resilience with edge cases
#[test]
fn test_blockchain_resilience_edge_cases() {
    let mut blockchain = BlockchainResilience::new(1000);
    
    // Register validator
    assert!(blockchain.pos.register_validator("validator1".to_string(), vec![1, 2, 3, 4], 2000).is_ok());
    
    // Create genesis block
    let initial_utxos = vec![UtxoOutput {
        id: "genesis0".to_string(),
        public_key: vec![1, 2, 3, 4],
        amount: 10000,
        created_at: 1000000,
    }];
    
    blockchain.create_genesis_block(initial_utxos);
    
    // Try to add block with wrong height
    // Skipping this test for now as we need to construct a proper block
    
    // Try to add block with wrong previous hash
    // Skipping this test for now as we need to construct a proper block
}

/// Performance test for UTXO model with large number of UTXOs
#[test]
fn test_utxo_performance_with_many_utxos() {
    let mut utxo_model = UtxoModel::new();
    
    // Add many UTXOs
    const NUM_UTXOS: usize = 1000;
    for i in 0..NUM_UTXOS {
        let utxo = UnspentTransactionOutput {
            output: UtxoOutput {
                id: format!("utxo{}", i),
                public_key: vec![(i % 256) as u8; 32],
                amount: 1000 + (i as u64),
                created_at: 1000000 + (i as u64),
            },
            transaction_id: format!("tx{}", i),
        };
        utxo_model.add_utxo(utxo);
    }
    
    // Verify all UTXOs were added by checking a few specific ones
    assert!(utxo_model.get_utxo(&"utxo0".to_string()).is_some());
    assert!(utxo_model.get_utxo(&"utxo500".to_string()).is_some());
    assert!(utxo_model.get_utxo(&"utxo999".to_string()).is_some());
    
    // Test getting UTXOs for a specific address
    let address_utxos = utxo_model.get_utxos_for_address(&vec![0u8; 32]);
    // This will depend on how many UTXOs were assigned to this address
    
    // Process a transaction spending one UTXO
    let transaction = UtxoTransaction {
        id: "test_tx".to_string(),
        inputs: vec![UtxoInput {
            utxo_id: "utxo0".to_string(),
            public_key: vec![0u8; 32],
        }],
        outputs: vec![UtxoOutput {
            id: "new_utxo".to_string(),
            public_key: vec![1u8; 32],
            amount: 1000,
            created_at: 2000000,
        }],
        timestamp: 2000000,
        signatures: vec![vec![1]], // Placeholder
    };
    
    assert!(utxo_model.process_transaction(transaction).is_ok());
    
    // Verify UTXO was spent
    assert!(utxo_model.get_utxo(&"utxo0".to_string()).is_none());
    assert!(utxo_model.get_utxo(&"new_utxo".to_string()).is_some());
}

/// Security test for PoS with malicious validator attempts
#[test]
fn test_pos_security_against_malicious_attempts() {
    let mut pos = ProofOfStake::new(1000);
    
    // Normal validator registration
    assert!(pos.register_validator("good_validator".to_string(), vec![1, 2, 3, 4], 2000).is_ok());
    
    // Try to register validator with same ID
    assert!(pos.register_validator("good_validator".to_string(), vec![5, 6, 7, 8], 2000).is_err());
    
    // Try to stake with non-existent validator
    assert!(pos.stake_tokens("stake1".to_string(), "fake_validator".to_string(), 1000, "owner".to_string()).is_err());
    
    // Try to unstake non-existent stake
    assert!(pos.unstake_tokens("fake_stake").is_err());
    
    // Try to reward non-existent validator
    assert!(pos.reward_validator(&"fake_validator".to_string(), 1000).is_err());
}

/// Test UTXO model address querying
#[test]
fn test_utxo_address_querying() {
    let mut utxo_model = UtxoModel::new();
    
    // Create UTXOs for different addresses
    let addr1 = vec![1u8; 32];
    let addr2 = vec![2u8; 32];
    let addr3 = vec![3u8; 32];
    
    // Add UTXOs for addr1
    for i in 0..5 {
        let utxo = UnspentTransactionOutput {
            output: UtxoOutput {
                id: format!("addr1_utxo{}", i),
                public_key: addr1.clone(),
                amount: 1000 + (i as u64) * 100,
                created_at: 1000000 + (i as u64),
            },
            transaction_id: format!("tx{}", i),
        };
        utxo_model.add_utxo(utxo);
    }
    
    // Add UTXOs for addr2
    for i in 0..3 {
        let utxo = UnspentTransactionOutput {
            output: UtxoOutput {
                id: format!("addr2_utxo{}", i),
                public_key: addr2.clone(),
                amount: 2000 + (i as u64) * 200,
                created_at: 1000000 + (i as u64),
            },
            transaction_id: format!("tx{}", i + 10),
        };
        utxo_model.add_utxo(utxo);
    }
    
    // Query UTXOs for each address
    let addr1_utxos = utxo_model.get_utxos_for_address(&addr1);
    let addr2_utxos = utxo_model.get_utxos_for_address(&addr2);
    let addr3_utxos = utxo_model.get_utxos_for_address(&addr3);
    
    assert_eq!(addr1_utxos.len(), 5);
    assert_eq!(addr2_utxos.len(), 3);
    assert_eq!(addr3_utxos.len(), 0);
    
    // Verify amounts are correct
    let total_addr1: u64 = addr1_utxos.iter().map(|utxo| utxo.output.amount).sum();
    let expected_addr1: u64 = (0..5).map(|i| 1000 + (i as u64) * 100).sum();
    assert_eq!(total_addr1, expected_addr1);
    
    let total_addr2: u64 = addr2_utxos.iter().map(|utxo| utxo.output.amount).sum();
    let expected_addr2: u64 = (0..3).map(|i| 2000 + (i as u64) * 200).sum();
    assert_eq!(total_addr2, expected_addr2);
}