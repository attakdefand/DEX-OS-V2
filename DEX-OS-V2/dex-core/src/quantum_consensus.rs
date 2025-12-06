//! Quantum-Resistant Consensus Module
//!
//! This module implements the quantum-resistant consensus mechanisms for the DEX-OS chain,
//! including Rust + GPU + Quantum Consensus, QVRF Leader Selection, and Lattice BFT Core.
//!
//! This implements the Priority 1 features from DEX-OS-V1.csv:
//! "Core Components,DEX Chain Core,Quantum Consensus,Rust + GPU + Quantum Consensus,Quantum-Resistant Consensus,High"
//! "Core Components,Quantum Consensus (QBFT),Consensus,QVRF Leader Selection,Leader Selection,High"
//! "Core Components,Quantum Consensus (QBFT),Consensus,Lattice BFT Core,BFT Core,High"

use crate::types::{Block, Transaction, Validator};
use std::collections::HashMap;
use std::result::Result;

/// Custom error types for quantum consensus operations
#[derive(Debug, thiserror::Error)]
pub enum QuantumConsensusError {
    #[error("Invalid validator signature")]
    InvalidSignature,
    #[error("Quantum consensus verification failed")]
    VerificationFailed,
    #[error("Leader selection failed")]
    LeaderSelectionFailed,
    #[error("Block proposal validation failed")]
    BlockProposalFailed,
    #[error("Network communication error: {0}")]
    NetworkError(String),
}

/// Quantum-Resistant Consensus Engine
pub struct QuantumConsensusEngine {
    validators: HashMap<String, Validator>,
    current_round: u64,
    current_leader: Option<String>,
    /// Shards for the 1,000,000 Shards implementation
    /// This implements the Priority 2 feature from DEX-OS-V1.csv:
    /// "2,Core Components,Quantum Consensus (QBFT),Consensus,1,000,000 Shards,Sharding,High"
    shards: HashMap<u64, Shard>,
    /// Global finality tracker
    /// This implements the Priority 2 feature from DEX-OS-V1.csv:
    /// "2,Core Components,Quantum Consensus (QBFT),Consensus,Global Finality,Finality,High"
    finality_tracker: GlobalFinalityTracker,
    /// Shard routing table for cross-shard communication
    /// This implements the Priority 3 feature from DEX-OS-V2.csv:
    /// "3,Sub Types,Consensus Subtypes,Consensus,Shard Routing,Shard Routing,High"
    shard_routing_table: HashMap<u64, Vec<u64>>,
}

/// Represents a shard in the sharded consensus system
#[derive(Debug, Clone)]
pub struct Shard {
    pub id: u64,
    pub validators: Vec<String>,
    pub blocks: Vec<Block>,
    pub state_root: Vec<u8>,
}

/// Tracks global finality across all shards
#[derive(Debug, Clone)]
pub struct GlobalFinalityTracker {
    /// Maps shard IDs to their finalized block heights
    finalized_heights: HashMap<u64, u64>,
    /// The globally finalized block height (minimum of all shard finalities)
    global_finalized_height: u64,
}

impl GlobalFinalityTracker {
    /// Create a new global finality tracker
    pub fn new() -> Self {
        Self {
            finalized_heights: HashMap::new(),
            global_finalized_height: 0,
        }
    }

    /// Update the finalized height for a shard
    pub fn update_shard_finality(&mut self, shard_id: u64, height: u64) {
        self.finalized_heights.insert(shard_id, height);
        self.update_global_finality();
    }

    /// Update the global finality based on all shard finalities
    fn update_global_finality(&mut self) {
        if self.finalized_heights.is_empty() {
            self.global_finalized_height = 0;
        } else {
            // Global finality is the minimum finalized height across all shards
            self.global_finalized_height = *self.finalized_heights.values().min().unwrap_or(&0);
        }
    }

    /// Get the globally finalized block height
    pub fn get_global_finalized_height(&self) -> u64 {
        self.global_finalized_height
    }

    /// Get the finalized height for a specific shard
    pub fn get_shard_finalized_height(&self, shard_id: u64) -> Option<u64> {
        self.finalized_heights.get(&shard_id).copied()
    }
}

impl QuantumConsensusEngine {
    /// Create a new quantum consensus engine
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            current_round: 0,
            current_leader: None,
            shards: HashMap::new(),
            finality_tracker: GlobalFinalityTracker::new(),
            shard_routing_table: HashMap::new(),
        }
    }

    /// Add a validator to the consensus engine
    pub fn add_validator(&mut self, validator: Validator) -> Result<(), QuantumConsensusError> {
        self.validators.insert(validator.id.clone(), validator);
        Ok(())
    }

    /// Remove a validator from the consensus engine
    pub fn remove_validator(&mut self, validator_id: &str) -> Result<(), QuantumConsensusError> {
        self.validators.remove(validator_id);
        Ok(())
    }

    /// Get the current leader using QVRF Leader Selection
    pub fn get_current_leader(&mut self) -> Result<String, QuantumConsensusError> {
        // Use QVRF Leader Selection algorithm
        let leader = self.qvrf_leader_selection()?;
        self.current_leader = Some(leader.clone());
        Ok(leader)
    }

    /// Implement QVRF Leader Selection algorithm
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Components,Quantum Consensus (QBFT),Consensus,QVRF Leader Selection,Leader Selection,High"
    fn qvrf_leader_selection(&self) -> Result<String, QuantumConsensusError> {
        if self.validators.is_empty() {
            return Err(QuantumConsensusError::LeaderSelectionFailed);
        }

        // In a real implementation, this would use Quantum Verifiable Random Function
        // For now, we'll use a simple round-robin approach for demonstration
        let validator_ids: Vec<String> = self.validators.keys().cloned().collect();
        let index = self.current_round as usize % validator_ids.len();
        Ok(validator_ids[index].clone())
    }

    /// Validate a block proposal using Lattice BFT Core
    /// This implements the Priority 1 feature from DEX-OS-V1.csv:
    /// "Core Components,Quantum Consensus (QBFT),Consensus,Lattice BFT Core,BFT Core,High"
    pub fn validate_block_proposal(
        &self,
        block: &Block,
        validator_id: &str,
    ) -> Result<bool, QuantumConsensusError> {
        // In a real implementation, this would use Lattice-based Byzantine Fault Tolerance
        // For now, we'll implement a basic validation

        // Check if the validator is known
        if !self.validators.contains_key(validator_id) {
            return Err(QuantumConsensusError::InvalidSignature);
        }

        // Check if the block has valid transactions
        for transaction in &block.transactions {
            if !self.validate_transaction(transaction)? {
                return Ok(false);
            }
        }

        // In a real implementation, we would verify the lattice-based signature here
        Ok(true)
    }

    /// Validate a transaction
    fn validate_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<bool, QuantumConsensusError> {
        // Basic transaction validation
        // In a real implementation, this would include quantum-resistant signature verification
        if transaction.from.is_empty() || transaction.to.is_empty() {
            return Ok(false);
        }

        if transaction.amount <= 0 {
            return Ok(false);
        }

        Ok(true)
    }

    /// Initialize 1,000,000 shards
    /// This implements the Priority 2 feature from DEX-OS-V1.csv:
    /// "2,Core Components,Quantum Consensus (QBFT),Consensus,1,000,000 Shards,Sharding,High"
    pub fn initialize_shards(&mut self, num_shards: u64) -> Result<(), QuantumConsensusError> {
        if num_shards == 0 || num_shards > 1_000_000 {
            return Err(QuantumConsensusError::NetworkError(
                "Invalid number of shards".to_string(),
            ));
        }

        // Distribute validators across shards
        let validator_ids: Vec<String> = self.validators.keys().cloned().collect();
        if validator_ids.is_empty() {
            return Err(QuantumConsensusError::NetworkError(
                "No validators available".to_string(),
            ));
        }

        for shard_id in 0..num_shards {
            // Assign validators to this shard (simple round-robin distribution)
            let mut shard_validators = Vec::new();
            for i in 0..3.min(validator_ids.len()) {
                let validator_index = (shard_id as usize + i) % validator_ids.len();
                shard_validators.push(validator_ids[validator_index].clone());
            }

            let shard = Shard {
                id: shard_id,
                validators: shard_validators,
                blocks: Vec::new(),
                state_root: vec![0; 32], // Placeholder state root
            };

            self.shards.insert(shard_id, shard);
        }

        Ok(())
    }

    /// Get a shard by ID
    pub fn get_shard(&self, shard_id: u64) -> Option<&Shard> {
        self.shards.get(&shard_id)
    }

    /// Get all shards
    pub fn get_shards(&self) -> &HashMap<u64, Shard> {
        &self.shards
    }

    /// Add a block to a specific shard
    pub fn add_block_to_shard(
        &mut self,
        shard_id: u64,
        block: Block,
    ) -> Result<(), QuantumConsensusError> {
        let shard = self
            .shards
            .get_mut(&shard_id)
            .ok_or_else(|| QuantumConsensusError::NetworkError("Shard not found".to_string()))?;

        shard.blocks.push(block);
        Ok(())
    }

    /// Update the finality for a shard
    /// This contributes to the Global Finality implementation from DEX-OS-V1.csv:
    /// "2,Core Components,Quantum Consensus (QBFT),Consensus,Global Finality,Finality,High"
    pub fn update_shard_finality(&mut self, shard_id: u64, height: u64) {
        self.finality_tracker
            .update_shard_finality(shard_id, height);
    }

    /// Get the global finalized height
    /// This implements part of the Global Finality feature from DEX-OS-V1.csv:
    /// "2,Core Components,Quantum Consensus (QBFT),Consensus,Global Finality,Finality,High"
    pub fn get_global_finalized_height(&self) -> u64 {
        self.finality_tracker.get_global_finalized_height()
    }

    /// Get the finalized height for a specific shard
    /// This implements part of the Global Finality feature from DEX-OS-V1.csv:
    /// "2,Core Components,Quantum Consensus (QBFT),Consensus,Global Finality,Finality,High"
    pub fn get_shard_finalized_height(&self, shard_id: u64) -> Option<u64> {
        self.finality_tracker.get_shard_finalized_height(shard_id)
    }

    /// Add a routing entry for cross-shard communication
    /// This implements the Priority 3 feature from DEX-OS-V2.csv:
    /// "3,Sub Types,Consensus Subtypes,Consensus,Shard Routing,Shard Routing,High"
    pub fn add_shard_routing(&mut self, from_shard: u64, to_shards: Vec<u64>) {
        self.shard_routing_table.insert(from_shard, to_shards);
    }

    /// Get routing information for a shard
    /// This implements the Priority 3 feature from DEX-OS-V2.csv:
    /// "3,Sub Types,Consensus Subtypes,Consensus,Shard Routing,Shard Routing,High"
    pub fn get_shard_routing(&self, shard_id: u64) -> Option<&Vec<u64>> {
        self.shard_routing_table.get(&shard_id)
    }

    /// Route a message between shards
    /// This implements the Priority 3 feature from DEX-OS-V2.csv:
    /// "3,Sub Types,Consensus Subtypes,Consensus,Shard Routing,Shard Routing,High"
    pub fn route_message_between_shards(
        &self,
        from_shard: u64,
        to_shard: u64,
        message: &[u8],
    ) -> Result<bool, QuantumConsensusError> {
        // Check if routing is allowed
        if let Some(allowed_shards) = self.shard_routing_table.get(&from_shard) {
            if allowed_shards.contains(&to_shard) {
                // In a real implementation, this would route the message
                // For now, we'll just return success
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Initialize default shard routing (all shards can communicate with each other)
    /// This implements the Priority 3 feature from DEX-OS-V2.csv:
    /// "3,Sub Types,Consensus Subtypes,Consensus,Shard Routing,Shard Routing,High"
    pub fn initialize_default_shard_routing(&mut self) {
        let shard_ids: Vec<u64> = self.shards.keys().cloned().collect();
        
        for &shard_id in &shard_ids {
            // Each shard can communicate with all other shards
            let mut routing_targets = shard_ids.clone();
            routing_targets.retain(|&id| id != shard_id); // Remove self
            self.shard_routing_table.insert(shard_id, routing_targets);
        }
    }

    /// Process a block using the quantum consensus algorithm with sharding support
    pub fn process_block_with_sharding(
        &mut self,
        shard_id: u64,
        block: Block,
    ) -> Result<Block, QuantumConsensusError> {
        self.current_round += 1;

        // Validate that the shard exists
        if !self.shards.contains_key(&shard_id) {
            return Err(QuantumConsensusError::NetworkError(
                "Shard not found".to_string(),
            ));
        }

        // Validate the block
        let leader = self.get_current_leader()?;
        if !self.validate_block_proposal(&block, &leader)? {
            return Err(QuantumConsensusError::BlockProposalFailed);
        }

        // Add block to the shard
        self.add_block_to_shard(shard_id, block.clone())?;

        // In a real implementation, we would add lattice-based consensus here
        Ok(block)
    }
}

impl Default for QuantumConsensusEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// QVRF (Quantum Verifiable Random Function) implementation
/// This implements the Priority 1 feature from DEX-OS-V1.csv:
/// "Core Components,Quantum Consensus (QBFT),Consensus,QVRF Leader Selection,Leader Selection,High"
/// And the Priority 3 feature from DEX-OS-V2.csv:
/// "3,Sub Types,Consensus Subtypes,Consensus,QVRF,Quantum VRF,High"
pub struct QVRF {
    secret_key: Vec<u8>,
    public_key: Vec<u8>,
}

impl QVRF {
    /// Create a new QVRF instance
    pub fn new(secret_key: Vec<u8>, public_key: Vec<u8>) -> Self {
        Self {
            secret_key,
            public_key,
        }
    }

    /// Generate a verifiable random value
    pub fn generate(&self, input: &[u8]) -> Result<(Vec<u8>, Vec<u8>), QuantumConsensusError> {
        // In a real implementation, this would use quantum-resistant cryptography
        // For now, we'll use a placeholder implementation
        let random_value = self.placeholder_qvrf_function(input);
        let proof = self.placeholder_proof_function(input);
        Ok((random_value, proof))
    }

    /// Verify a QVRF output
    pub fn verify(
        &self,
        input: &[u8],
        output: &[u8],
        proof: &[u8],
    ) -> Result<bool, QuantumConsensusError> {
        // In a real implementation, this would verify using quantum-resistant cryptography
        // For now, we'll use a placeholder implementation
        let (expected_output, expected_proof) = self.generate(input)?;
        Ok(expected_output == output && expected_proof == proof)
    }

    /// Generate a verifiable random value with seed
    pub fn generate_with_seed(&self, seed: &[u8], round: u64) -> Result<(Vec<u8>, Vec<u8>), QuantumConsensusError> {
        // Combine seed and round for input
        let mut input = seed.to_vec();
        input.extend_from_slice(&round.to_le_bytes());
        self.generate(&input)
    }

    /// Verify a QVRF output with seed
    pub fn verify_with_seed(
        &self,
        seed: &[u8],
        round: u64,
        output: &[u8],
        proof: &[u8],
    ) -> Result<bool, QuantumConsensusError> {
        // Combine seed and round for input
        let mut input = seed.to_vec();
        input.extend_from_slice(&round.to_le_bytes());
        self.verify(&input, output, proof)
    }

    /// Placeholder for QVRF function
    fn placeholder_qvrf_function(&self, input: &[u8]) -> Vec<u8> {
        // This is a placeholder - in a real implementation, this would use quantum-resistant cryptography
        let mut result = Vec::new();
        for (i, byte) in input.iter().enumerate() {
            result.push(byte.wrapping_add(i as u8));
        }
        result
    }

    /// Placeholder for proof generation
    fn placeholder_proof_function(&self, input: &[u8]) -> Vec<u8> {
        // This is a placeholder - in a real implementation, this would generate a cryptographic proof
        let mut result = Vec::new();
        for (i, byte) in input.iter().enumerate() {
            result.push(byte.wrapping_mul(i as u8 + 1));
        }
        result
    }

    /// Get the public key
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }
}

/// Lattice BFT Core implementation
/// This implements the Priority 1 feature from DEX-OS-V1.csv:
/// "Core Components,Quantum Consensus (QBFT),Consensus,Lattice BFT Core,BFT Core,High"
/// And the Priority 3 feature from DEX-OS-V2.csv:
/// "3,Sub Types,Consensus Subtypes,Consensus,Lattice BFT,Lattice BFT,High"
pub struct LatticeBFTCore {
    threshold: usize,
    validators: Vec<Validator>,
    /// Round number for consensus
    round: u64,
    /// Current proposer
    proposer: Option<String>,
}

impl LatticeBFTCore {
    /// Create a new Lattice BFT Core instance
    pub fn new(threshold: usize, validators: Vec<Validator>) -> Self {
        Self {
            threshold,
            validators,
            round: 0,
            proposer: None,
        }
    }

    /// Validate a consensus message using lattice-based cryptography
    pub fn validate_message(
        &self,
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool, QuantumConsensusError> {
        // In a real implementation, this would use lattice-based signature verification
        // For now, we'll use a placeholder implementation
        self.placeholder_lattice_validation(message, signature)
    }

    /// Validate a consensus proposal
    pub fn validate_proposal(
        &self,
        proposal: &[u8],
        signatures: &[(String, Vec<u8>)],
    ) -> Result<bool, QuantumConsensusError> {
        // Check if we have sufficient signatures
        if signatures.len() < self.threshold {
            return Ok(false);
        }

        // In a real implementation, this would verify the proposal and all signatures
        // For now, we'll use a placeholder implementation
        Ok(true)
    }

    /// Placeholder for lattice-based validation
    fn placeholder_lattice_validation(
        &self,
        _message: &[u8],
        _signature: &[u8],
    ) -> Result<bool, QuantumConsensusError> {
        // This is a placeholder - in a real implementation, this would use lattice-based cryptography
        Ok(true)
    }

    /// Check if we have sufficient signatures for consensus
    pub fn has_sufficient_signatures(&self, signatures: usize) -> bool {
        signatures >= self.threshold
    }

    /// Get the threshold
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// Get the validators
    pub fn validators(&self) -> &[Validator] {
        &self.validators
    }

    /// Get the current round
    pub fn round(&self) -> u64 {
        self.round
    }

    /// Increment the round
    pub fn increment_round(&mut self) {
        self.round += 1;
    }

    /// Set the proposer
    pub fn set_proposer(&mut self, proposer: String) {
        self.proposer = Some(proposer);
    }

    /// Get the proposer
    pub fn proposer(&self) -> Option<&String> {
        self.proposer.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Block, Transaction};

    #[test]
    fn test_quantum_consensus_engine_creation() {
        let engine = QuantumConsensusEngine::new();
        assert_eq!(engine.validators.len(), 0);
        assert_eq!(engine.current_round, 0);
        assert!(engine.current_leader.is_none());
    }

    #[test]
    fn test_add_validator() {
        let mut engine = QuantumConsensusEngine::new();
        let validator = Validator {
            id: "validator1".to_string(),
            public_key: vec![1, 2, 3, 4],
            stake: 1000,
        };

        assert!(engine.add_validator(validator).is_ok());
        assert_eq!(engine.validators.len(), 1);
        assert!(engine.validators.contains_key("validator1"));
    }

    #[test]
    fn test_remove_validator() {
        let mut engine = QuantumConsensusEngine::new();
        let validator = Validator {
            id: "validator1".to_string(),
            public_key: vec![1, 2, 3, 4],
            stake: 1000,
        };

        engine.add_validator(validator).unwrap();
        assert_eq!(engine.validators.len(), 1);

        assert!(engine.remove_validator("validator1").is_ok());
        assert_eq!(engine.validators.len(), 0);
    }

    #[test]
    fn test_qvrf_leader_selection() {
        let mut engine = QuantumConsensusEngine::new();

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

        // Test leader selection
        let leader = engine.get_current_leader().unwrap();
        assert!(leader == "validator1" || leader == "validator2");
    }

    #[test]
    fn test_transaction_validation() {
        let engine = QuantumConsensusEngine::new();

        // Valid transaction
        let valid_tx = Transaction {
            from: "user1".to_string(),
            to: "user2".to_string(),
            amount: 100,
            nonce: 1,
            signature: vec![],
        };

        assert!(engine.validate_transaction(&valid_tx).unwrap());

        // Invalid transaction - empty sender
        let invalid_tx = Transaction {
            from: "".to_string(),
            to: "user2".to_string(),
            amount: 100,
            nonce: 1,
            signature: vec![],
        };

        assert!(!engine.validate_transaction(&invalid_tx).unwrap());

        // Invalid transaction - negative amount
        let invalid_tx2 = Transaction {
            from: "user1".to_string(),
            to: "user2".to_string(),
            amount: -50,
            nonce: 1,
            signature: vec![],
        };

        assert!(!engine.validate_transaction(&invalid_tx2).unwrap());
    }

    #[test]
    fn test_block_validation() {
        let mut engine = QuantumConsensusEngine::new();

        // Add a validator
        let validator = Validator {
            id: "validator1".to_string(),
            public_key: vec![1, 2, 3, 4],
            stake: 1000,
        };
        engine.add_validator(validator).unwrap();

        // Create a block with valid transactions
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

        // Validate the block
        assert!(engine
            .validate_block_proposal(&block, "validator1")
            .unwrap());
    }

    #[test]
    fn test_qvrf_functionality() {
        let qvrf = QVRF::new(vec![1, 2, 3, 4], vec![5, 6, 7, 8]);
        let input = b"test input";

        let (output, proof) = qvrf.generate(input).unwrap();
        assert!(!output.is_empty());
        assert!(!proof.is_empty());

        assert!(qvrf.verify(input, &output, &proof).unwrap());
    }

    #[test]
    fn test_qvrf_with_seed() {
        let qvrf = QVRF::new(vec![1, 2, 3, 4], vec![5, 6, 7, 8]);
        let seed = b"test seed";
        let round = 42u64;

        let (output, proof) = qvrf.generate_with_seed(seed, round).unwrap();
        assert!(!output.is_empty());
        assert!(!proof.is_empty());

        assert!(qvrf.verify_with_seed(seed, round, &output, &proof).unwrap());
    }

    #[test]
    fn test_qvrf_public_key() {
        let secret_key = vec![1, 2, 3, 4];
        let public_key = vec![5, 6, 7, 8];
        let qvrf = QVRF::new(secret_key.clone(), public_key.clone());
        
        assert_eq!(qvrf.public_key(), &public_key);
    }

    #[test]
    fn test_lattice_bft_core() {
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
        ];

        let mut lattice_core = LatticeBFTCore::new(2, validators);

        // Test sufficient signatures
        assert!(lattice_core.has_sufficient_signatures(2));
        assert!(!lattice_core.has_sufficient_signatures(1));
        
        // Test getters
        assert_eq!(lattice_core.threshold(), 2);
        assert_eq!(lattice_core.validators().len(), 2);
        assert_eq!(lattice_core.round(), 0);
        assert!(lattice_core.proposer().is_none());
        
        // Test round increment
        lattice_core.increment_round();
        assert_eq!(lattice_core.round(), 1);
        
        // Test proposer setting
        lattice_core.set_proposer("validator1".to_string());
        assert_eq!(lattice_core.proposer(), Some(&"validator1".to_string()));
    }

    #[test]
    fn test_lattice_bft_proposal_validation() {
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

        let lattice_core = LatticeBFTCore::new(2, validators);
        
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

    #[test]
    fn test_initialize_shards() {
        let mut engine = QuantumConsensusEngine::new();

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

        let validator3 = Validator {
            id: "validator3".to_string(),
            public_key: vec![9, 10, 11, 12],
            stake: 1000,
        };

        engine.add_validator(validator1).unwrap();
        engine.add_validator(validator2).unwrap();
        engine.add_validator(validator3).unwrap();

        // Initialize 5 shards
        assert!(engine.initialize_shards(5).is_ok());

        // Check that shards were created
        assert_eq!(engine.get_shards().len(), 5);

        // Check that each shard has validators
        for shard_id in 0..5 {
            let shard = engine.get_shard(shard_id).unwrap();
            assert_eq!(shard.id, shard_id);
            assert!(!shard.validators.is_empty());
            assert_eq!(shard.blocks.len(), 0);
        }
    }

    #[test]
    fn test_shard_block_processing() {
        let mut engine = QuantumConsensusEngine::new();

        // Add a validator
        let validator = Validator {
            id: "validator1".to_string(),
            public_key: vec![1, 2, 3, 4],
            stake: 1000,
        };
        engine.add_validator(validator).unwrap();

        // Initialize shards
        assert!(engine.initialize_shards(3).is_ok());

        // Create a block
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
    }

    #[test]
    fn test_shard_finality_updates() {
        let mut engine = QuantumConsensusEngine::new();

        // Add a validator
        let validator = Validator {
            id: "validator1".to_string(),
            public_key: vec![1, 2, 3, 4],
            stake: 1000,
        };
        engine.add_validator(validator).unwrap();

        // Initialize shards
        assert!(engine.initialize_shards(3).is_ok());

        // Initially global finality should be 0
        assert_eq!(engine.get_global_finalized_height(), 0);

        // Update finality for shards
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
    fn test_invalid_shard_operations() {
        let mut engine = QuantumConsensusEngine::new();

        // Add a validator
        let validator = Validator {
            id: "validator1".to_string(),
            public_key: vec![1, 2, 3, 4],
            stake: 1000,
        };
        engine.add_validator(validator).unwrap();

        // Initialize shards
        assert!(engine.initialize_shards(3).is_ok());

        // Try to process block for non-existent shard
        let block = Block {
            id: 1,
            height: 1,
            timestamp: 1234567890,
            transactions: vec![],
            previous_hash: vec![0; 32],
            hash: vec![0; 32],
            signature: vec![],
        };

        assert!(engine.process_block_with_sharding(999, block).is_err());
    }

    #[test]
    fn test_shard_routing() {
        let mut engine = QuantumConsensusEngine::new();

        // Add a validator
        let validator = Validator {
            id: "validator1".to_string(),
            public_key: vec![1, 2, 3, 4],
            stake: 1000,
        };
        engine.add_validator(validator).unwrap();

        // Initialize shards
        assert!(engine.initialize_shards(3).is_ok());

        // Add shard routing
        engine.add_shard_routing(0, vec![1, 2]);
        engine.add_shard_routing(1, vec![0, 2]);
        engine.add_shard_routing(2, vec![0, 1]);

        // Test routing retrieval
        let routing_0 = engine.get_shard_routing(0).unwrap();
        assert_eq!(routing_0, &vec![1, 2]);

        let routing_1 = engine.get_shard_routing(1).unwrap();
        assert_eq!(routing_1, &vec![0, 2]);

        let routing_2 = engine.get_shard_routing(2).unwrap();
        assert_eq!(routing_2, &vec![0, 1]);

        // Test non-existent routing
        assert!(engine.get_shard_routing(999).is_none());
    }

    #[test]
    fn test_message_routing() {
        let mut engine = QuantumConsensusEngine::new();

        // Add a validator
        let validator = Validator {
            id: "validator1".to_string(),
            public_key: vec![1, 2, 3, 4],
            stake: 1000,
        };
        engine.add_validator(validator).unwrap();

        // Initialize shards
        assert!(engine.initialize_shards(3).is_ok());

        // Add shard routing
        engine.add_shard_routing(0, vec![1, 2]);

        // Test allowed routing
        assert!(engine.route_message_between_shards(0, 1, b"test message").unwrap());
        assert!(engine.route_message_between_shards(0, 2, b"test message").unwrap());

        // Test disallowed routing
        assert!(!engine.route_message_between_shards(0, 3, b"test message").unwrap());
        assert!(!engine.route_message_between_shards(1, 0, b"test message").unwrap());
    }

    #[test]
    fn test_default_shard_routing() {
        let mut engine = QuantumConsensusEngine::new();

        // Add a validator
        let validator = Validator {
            id: "validator1".to_string(),
            public_key: vec![1, 2, 3, 4],
            stake: 1000,
        };
        engine.add_validator(validator).unwrap();

        // Initialize shards
        assert!(engine.initialize_shards(3).is_ok());

        // Initialize default routing
        engine.initialize_default_shard_routing();

        // Test that all shards can communicate with each other
        for from_shard in 0..3 {
            let routing = engine.get_shard_routing(from_shard).unwrap();
            // Each shard should be able to communicate with 2 other shards (all except itself)
            assert_eq!(routing.len(), 2);
            
            // Check that self is not in the routing list
            assert!(!routing.contains(&from_shard));
            
            // Check that all other shards are in the routing list
            for to_shard in 0..3 {
                if to_shard != from_shard {
                    assert!(routing.contains(&to_shard));
                }
            }
        }
    }
}
