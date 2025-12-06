//! Integration test for consensus subtypes functionality

#[cfg(test)]
mod tests {
    use dex_core::quantum_consensus::{
        QuantumConsensusEngine, QVRF, LatticeBFTCore, GlobalFinalityTracker
    };
    use dex_core::types::{Validator, Block, Transaction};

    #[test]
    fn test_qvrf_integration() {
        // Test QVRF functionality
        let qvrf = QVRF::new(vec![1, 2, 3, 4], vec![5, 6, 7, 8]);
        let input = b"test input";

        let (output, proof) = qvrf.generate(input).unwrap();
        assert!(!output.is_empty());
        assert!(!proof.is_empty());

        assert!(qvrf.verify(input, &output, &proof).unwrap());
        
        // Test QVRF with seed
        let seed = b"test seed";
        let round = 42u64;

        let (output, proof) = qvrf.generate_with_seed(seed, round).unwrap();
        assert!(!output.is_empty());
        assert!(!proof.is_empty());

        assert!(qvrf.verify_with_seed(seed, round, &output, &proof).unwrap());
    }

    #[test]
    fn test_lattice_bft_integration() {
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

        let mut lattice_core = LatticeBFTCore::new(2, validators);

        // Test basic functionality
        assert_eq!(lattice_core.threshold(), 2);
        assert_eq!(lattice_core.validators().len(), 3);
        assert_eq!(lattice_core.round(), 0);
        assert!(lattice_core.proposer().is_none());

        // Test round increment
        lattice_core.increment_round();
        assert_eq!(lattice_core.round(), 1);

        // Test proposer setting
        lattice_core.set_proposer("validator1".to_string());
        assert_eq!(lattice_core.proposer(), Some(&"validator1".to_string()));

        // Test sufficient signatures
        assert!(lattice_core.has_sufficient_signatures(2));
        assert!(!lattice_core.has_sufficient_signatures(1));

        // Test proposal validation with sufficient signatures
        let signatures = vec![
            ("validator1".to_string(), vec![1, 2, 3]),
            ("validator2".to_string(), vec![4, 5, 6]),
        ];
        
        assert!(lattice_core.validate_proposal(b"test proposal", &signatures).unwrap());
        
        // Test proposal validation with insufficient signatures
        let insufficient_signatures = vec![
            ("validator1".to_string(), vec![1, 2, 3]),
        ];
        
        assert!(!lattice_core.validate_proposal(b"test proposal", &insufficient_signatures).unwrap());
    }

    #[test]
    fn test_quantum_consensus_engine_integration() {
        let mut engine = QuantumConsensusEngine::new();

        // Test basic engine functionality
        // Note: We can't directly access private fields, so we'll test indirectly
        // Test that we can get the current leader which should fail since there are no validators
        let leader_result = engine.get_current_leader();
        assert!(leader_result.is_err());
        // Add validators
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

        engine.add_validator(validator1).unwrap();
        engine.add_validator(validator2).unwrap();
        // Note: We can't directly access private validators field
        // assert_eq!(engine.validators.len(), 2);

        // Test leader selection
        let leader = engine.get_current_leader().unwrap();
        assert!(leader == "validator1" || leader == "validator2");

        // Initialize shards
        assert!(engine.initialize_shards(3).is_ok());
        assert_eq!(engine.get_shards().len(), 3);

        // Test shard routing
        engine.add_shard_routing(0, vec![1, 2]);
        engine.add_shard_routing(1, vec![0, 2]);
        engine.add_shard_routing(2, vec![0, 1]);

        let routing_0 = engine.get_shard_routing(0).unwrap();
        assert_eq!(routing_0, &vec![1, 2]);

        // Test message routing
        assert!(engine.route_message_between_shards(0, 1, b"test message").unwrap());
        assert!(!engine.route_message_between_shards(0, 3, b"test message").unwrap());

        // Test default shard routing
        engine.initialize_default_shard_routing();
        for from_shard in 0..3 {
            let routing = engine.get_shard_routing(from_shard).unwrap();
            // Each shard should be able to communicate with 2 other shards (all except itself)
            assert_eq!(routing.len(), 2);
        }

        // Test shard block processing
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

        // Process block for shard 1
        assert!(engine.process_block_with_sharding(1, block).is_ok());

        // Check that block was added to shard
        let shard = engine.get_shard(1).unwrap();
        assert_eq!(shard.blocks.len(), 1);
        assert_eq!(shard.blocks[0].id, 1);

        // Test shard finality updates
        engine.update_shard_finality(0, 100);
        engine.update_shard_finality(1, 90);
        engine.update_shard_finality(2, 95);

        // Global finality should be the minimum (90)
        assert_eq!(engine.get_global_finalized_height(), 90);

        // Check individual shard finalities
        assert_eq!(engine.get_shard_finalized_height(0), Some(100));
        assert_eq!(engine.get_shard_finalized_height(1), Some(90));
        assert_eq!(engine.get_shard_finalized_height(2), Some(95));
    }

    #[test]
    fn test_global_finality_tracker() {
        let mut tracker = GlobalFinalityTracker::new();

        // Initially should be 0
        assert_eq!(tracker.get_global_finalized_height(), 0);

        // Update shard finalities
        tracker.update_shard_finality(1, 100);
        tracker.update_shard_finality(2, 90);
        tracker.update_shard_finality(3, 95);

        // Global finality should be the minimum (90)
        assert_eq!(tracker.get_global_finalized_height(), 90);

        // Update one shard to have a lower finality
        tracker.update_shard_finality(2, 85);
        assert_eq!(tracker.get_global_finalized_height(), 85);

        // Check individual shard finalities
        assert_eq!(tracker.get_shard_finalized_height(1), Some(100));
        assert_eq!(tracker.get_shard_finalized_height(2), Some(85));
        assert_eq!(tracker.get_shard_finalized_height(4), None); // Non-existent shard
    }
}
