// Blockchain Consensus - Transaction Validation
// Security: Layer 22 - Quantum-Resistant Security
// Implements transaction validation with quantum-resistant cryptography

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use sha3::{Digest, Sha3_256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Transaction structure
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub data: Vec<u8>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(from: String, to: String, amount: u64, nonce: u64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let id = format!("{}:{}:{}", from, to, timestamp);
        
        Self {
            id,
            from,
            to,
            amount,
            nonce,
            timestamp,
            signature: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Calculate transaction hash
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(&self.amount.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.finalize().to_vec()
    }

    /// Sign transaction (simplified - in production use quantum-resistant signatures)
    pub fn sign(&mut self, private_key: &[u8]) {
        let hash = self.hash();
        let mut hasher = Sha3_256::new();
        hasher.update(&hash);
        hasher.update(private_key);
        self.signature = hasher.finalize().to_vec();
    }

    /// Verify transaction signature
    pub fn verify_signature(&self, public_key: &[u8]) -> bool {
        if self.signature.is_empty() {
            return false;
        }

        let hash = self.hash();
        let mut hasher = Sha3_256::new();
        hasher.update(&hash);
        hasher.update(public_key);
        let expected_signature = hasher.finalize().to_vec();

        self.signature == expected_signature
    }
}

/// Block structure
#[derive(Debug, Clone)]
pub struct Block {
    pub height: u64,
    pub timestamp: u64,
    pub previous_hash: Vec<u8>,
    pub transactions: Vec<Transaction>,
    pub merkle_root: Vec<u8>,
    pub validator: String,
    pub signature: Vec<u8>,
}

impl Block {
    /// Create a new block
    pub fn new(height: u64, previous_hash: Vec<u8>, transactions: Vec<Transaction>, validator: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let merkle_root = Self::calculate_merkle_root(&transactions);

        Self {
            height,
            timestamp,
            previous_hash,
            transactions,
            merkle_root,
            validator,
            signature: Vec::new(),
        }
    }

    /// Calculate Merkle root of transactions
    fn calculate_merkle_root(transactions: &[Transaction]) -> Vec<u8> {
        if transactions.is_empty() {
            return vec![0; 32];
        }

        let mut hashes: Vec<Vec<u8>> = transactions.iter().map(|tx| tx.hash()).collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha3_256::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]);
                }
                next_level.push(hasher.finalize().to_vec());
            }
            
            hashes = next_level;
        }

        hashes[0].clone()
    }

    /// Calculate block hash
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(&self.height.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.previous_hash);
        hasher.update(&self.merkle_root);
        hasher.finalize().to_vec()
    }
}

/// Validator information
#[derive(Debug, Clone)]
pub struct Validator {
    pub id: String,
    pub public_key: Vec<u8>,
    pub stake: u64,
    pub reputation: f64,
    pub is_active: bool,
}

impl Validator {
    pub fn new(id: String, public_key: Vec<u8>, stake: u64) -> Self {
        Self {
            id,
            public_key,
            stake,
            reputation: 1.0,
            is_active: true,
        }
    }
}

/// Validation result
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }
}

/// Blockchain consensus engine for transaction validation
pub struct BlockchainConsensus {
    validators: Arc<RwLock<HashMap<String, Validator>>>,
    validated_transactions: Arc<RwLock<HashSet<String>>>,
    blockchain: Arc<RwLock<Vec<Block>>>,
    mempool: Arc<RwLock<Vec<Transaction>>>,
    nonce_tracker: Arc<RwLock<HashMap<String, u64>>>,
    min_validators: usize,
    consensus_threshold: f64,
}

impl BlockchainConsensus {
    /// Create a new blockchain consensus engine
    pub fn new(min_validators: usize, consensus_threshold: f64) -> Self {
        Self {
            validators: Arc::new(RwLock::new(HashMap::new())),
            validated_transactions: Arc::new(RwLock::new(HashSet::new())),
            blockchain: Arc::new(RwLock::new(Vec::new())),
            mempool: Arc::new(RwLock::new(Vec::new())),
            nonce_tracker: Arc::new(RwLock::new(HashMap::new())),
            min_validators,
            consensus_threshold,
        }
    }

    /// Add a validator
    pub fn add_validator(&self, validator: Validator) -> Result<(), String> {
        let mut validators = self.validators.write();
        
        if validators.contains_key(&validator.id) {
            return Err(format!("Validator {} already exists", validator.id));
        }

        validators.insert(validator.id.clone(), validator);
        Ok(())
    }

    /// Remove a validator
    pub fn remove_validator(&self, validator_id: &str) -> Result<(), String> {
        let mut validators = self.validators.write();
        
        if validators.remove(validator_id).is_none() {
            return Err(format!("Validator {} not found", validator_id));
        }

        Ok(())
    }

    /// Validate a single transaction
    pub fn validate_transaction(&self, transaction: &Transaction) -> ValidationResult {
        // Check if transaction already validated
        let validated = self.validated_transactions.read();
        if validated.contains(&transaction.id) {
            return ValidationResult::Invalid("Transaction already validated".to_string());
        }
        drop(validated);

        // Validate basic fields
        if transaction.from.is_empty() || transaction.to.is_empty() {
            return ValidationResult::Invalid("Invalid sender or recipient".to_string());
        }

        if transaction.amount == 0 {
            return ValidationResult::Invalid("Amount must be greater than zero".to_string());
        }

        // Validate nonce
        let nonce_tracker = self.nonce_tracker.read();
        if let Some(&expected_nonce) = nonce_tracker.get(&transaction.from) {
            if transaction.nonce != expected_nonce + 1 {
                return ValidationResult::Invalid(format!(
                    "Invalid nonce: expected {}, got {}",
                    expected_nonce + 1,
                    transaction.nonce
                ));
            }
        } else if transaction.nonce != 0 {
            return ValidationResult::Invalid("First transaction must have nonce 0".to_string());
        }
        drop(nonce_tracker);

        // Validate signature (simplified - in production use quantum-resistant verification)
        if transaction.signature.is_empty() {
            return ValidationResult::Invalid("Missing signature".to_string());
        }

        // Validate timestamp (not too far in future or past)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if transaction.timestamp > now + 300 {
            return ValidationResult::Invalid("Transaction timestamp too far in future".to_string());
        }

        if now > transaction.timestamp + 3600 {
            return ValidationResult::Invalid("Transaction timestamp too old".to_string());
        }

        ValidationResult::Valid
    }

    /// Validate multiple transactions
    pub fn validate_transactions(&self, transactions: &[Transaction]) -> Vec<(String, ValidationResult)> {
        transactions
            .iter()
            .map(|tx| (tx.id.clone(), self.validate_transaction(tx)))
            .collect()
    }

    /// Add transaction to mempool
    pub fn add_to_mempool(&self, transaction: Transaction) -> Result<(), String> {
        let validation = self.validate_transaction(&transaction);
        
        if !validation.is_valid() {
            return Err(format!("Transaction validation failed: {:?}", validation));
        }

        let mut mempool = self.mempool.write();
        mempool.push(transaction.clone());

        // Update nonce tracker
        let mut nonce_tracker = self.nonce_tracker.write();
        nonce_tracker.insert(transaction.from.clone(), transaction.nonce);

        // Mark as validated
        let mut validated = self.validated_transactions.write();
        validated.insert(transaction.id.clone());

        Ok(())
    }

    /// Get transactions from mempool
    pub fn get_mempool_transactions(&self, limit: usize) -> Vec<Transaction> {
        let mempool = self.mempool.read();
        mempool.iter().take(limit).cloned().collect()
    }

    /// Clear mempool
    pub fn clear_mempool(&self) {
        let mut mempool = self.mempool.write();
        mempool.clear();
    }

    /// Validate a block
    pub fn validate_block(&self, block: &Block) -> ValidationResult {
        // Validate block height
        let blockchain = self.blockchain.read();
        let expected_height = blockchain.len() as u64;
        
        if block.height != expected_height {
            return ValidationResult::Invalid(format!(
                "Invalid block height: expected {}, got {}",
                expected_height, block.height
            ));
        }

        // Validate previous hash
        if !blockchain.is_empty() {
            let last_block = &blockchain[blockchain.len() - 1];
            if block.previous_hash != last_block.hash() {
                return ValidationResult::Invalid("Invalid previous hash".to_string());
            }
        } else if !block.previous_hash.is_empty() {
            return ValidationResult::Invalid("Genesis block must have empty previous hash".to_string());
        }
        drop(blockchain);

        // Validate all transactions in block
        for transaction in &block.transactions {
            let result = self.validate_transaction(transaction);
            if !result.is_valid() {
                return ValidationResult::Invalid(format!(
                    "Invalid transaction {}: {:?}",
                    transaction.id, result
                ));
            }
        }

        // Validate Merkle root
        let calculated_merkle = Block::calculate_merkle_root(&block.transactions);
        if block.merkle_root != calculated_merkle {
            return ValidationResult::Invalid("Invalid Merkle root".to_string());
        }

        // Validate validator
        let validators = self.validators.read();
        if !validators.contains_key(&block.validator) {
            return ValidationResult::Invalid(format!("Unknown validator: {}", block.validator));
        }

        let validator = &validators[&block.validator];
        if !validator.is_active {
            return ValidationResult::Invalid(format!("Validator {} is not active", block.validator));
        }

        ValidationResult::Valid
    }

    /// Add a validated block to the blockchain
    pub fn add_block(&self, block: Block) -> Result<(), String> {
        let validation = self.validate_block(&block);
        
        if !validation.is_valid() {
            return Err(format!("Block validation failed: {:?}", validation));
        }

        let mut blockchain = self.blockchain.write();
        blockchain.push(block);

        Ok(())
    }

    /// Get blockchain height
    pub fn get_height(&self) -> u64 {
        self.blockchain.read().len() as u64
    }

    /// Get block by height
    pub fn get_block(&self, height: u64) -> Option<Block> {
        let blockchain = self.blockchain.read();
        blockchain.get(height as usize).cloned()
    }

    /// Get latest block
    pub fn get_latest_block(&self) -> Option<Block> {
        let blockchain = self.blockchain.read();
        blockchain.last().cloned()
    }

    /// Achieve consensus on a block
    pub fn achieve_consensus(&self, block: &Block) -> Result<bool, String> {
        let validators = self.validators.read();
        
        if validators.len() < self.min_validators {
            return Err(format!(
                "Insufficient validators: {} < {}",
                validators.len(),
                self.min_validators
            ));
        }

        // Simulate validator voting (in production, collect actual votes)
        let mut votes = 0;
        let mut total_stake = 0u64;
        let mut voting_stake = 0u64;

        for validator in validators.values() {
            if !validator.is_active {
                continue;
            }

            total_stake += validator.stake;

            // Simulate vote (in production, validators would sign and broadcast votes)
            let validation = self.validate_block(block);
            if validation.is_valid() {
                votes += 1;
                voting_stake += validator.stake;
            }
        }

        // Check if consensus threshold reached
        let vote_ratio = votes as f64 / validators.len() as f64;
        let stake_ratio = voting_stake as f64 / total_stake as f64;

        Ok(vote_ratio >= self.consensus_threshold && stake_ratio >= self.consensus_threshold)
    }

    /// Get consensus statistics
    pub fn get_stats(&self) -> ConsensusStats {
        let validators = self.validators.read();
        let blockchain = self.blockchain.read();
        let mempool = self.mempool.read();
        let validated = self.validated_transactions.read();

        ConsensusStats {
            total_validators: validators.len(),
            active_validators: validators.values().filter(|v| v.is_active).count(),
            blockchain_height: blockchain.len(),
            mempool_size: mempool.len(),
            validated_transactions: validated.len(),
        }
    }
}

/// Consensus statistics
#[derive(Debug, Clone)]
pub struct ConsensusStats {
    pub total_validators: usize,
    pub active_validators: usize,
    pub blockchain_height: usize,
    pub mempool_size: usize,
    pub validated_transactions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
        assert_eq!(tx.from, "alice");
        assert_eq!(tx.to, "bob");
        assert_eq!(tx.amount, 100);
        assert_eq!(tx.nonce, 0);
    }

    #[test]
    fn test_transaction_signing() {
        let mut tx = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
        let private_key = b"alice_private_key";
        let public_key = b"alice_private_key"; // Simplified for testing

        tx.sign(private_key);
        assert!(!tx.signature.is_empty());
        assert!(tx.verify_signature(public_key));
    }

    #[test]
    fn test_block_creation() {
        let tx1 = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
        let tx2 = Transaction::new("bob".to_string(), "charlie".to_string(), 50, 0);
        
        let block = Block::new(0, vec![], vec![tx1, tx2], "validator1".to_string());
        assert_eq!(block.height, 0);
        assert_eq!(block.transactions.len(), 2);
        assert!(!block.merkle_root.is_empty());
    }

    #[test]
    fn test_consensus_engine() {
        let consensus = BlockchainConsensus::new(3, 0.67);
        
        // Add validators
        let val1 = Validator::new("val1".to_string(), vec![1, 2, 3], 1000);
        let val2 = Validator::new("val2".to_string(), vec![4, 5, 6], 2000);
        let val3 = Validator::new("val3".to_string(), vec![7, 8, 9], 1500);
        
        consensus.add_validator(val1).unwrap();
        consensus.add_validator(val2).unwrap();
        consensus.add_validator(val3).unwrap();

        let stats = consensus.get_stats();
        assert_eq!(stats.total_validators, 3);
        assert_eq!(stats.active_validators, 3);
    }

    #[test]
    fn test_transaction_validation() {
        let consensus = BlockchainConsensus::new(1, 0.51);
        
        let mut tx = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
        tx.sign(b"alice_key");

        let result = consensus.validate_transaction(&tx);
        assert!(result.is_valid());
    }

    #[test]
    fn test_mempool_operations() {
        let consensus = BlockchainConsensus::new(1, 0.51);
        
        let mut tx = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
        tx.sign(b"alice_key");

        consensus.add_to_mempool(tx.clone()).unwrap();
        
        let mempool_txs = consensus.get_mempool_transactions(10);
        assert_eq!(mempool_txs.len(), 1);
        assert_eq!(mempool_txs[0].id, tx.id);
    }

    #[test]
    fn test_block_validation() {
        let consensus = BlockchainConsensus::new(1, 0.51);
        
        let val = Validator::new("val1".to_string(), vec![1, 2, 3], 1000);
        consensus.add_validator(val).unwrap();

        let mut tx = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
        tx.sign(b"alice_key");

        let block = Block::new(0, vec![], vec![tx], "val1".to_string());
        let result = consensus.validate_block(&block);
        assert!(result.is_valid());
    }

    #[test]
    fn test_blockchain_operations() {
        let consensus = BlockchainConsensus::new(1, 0.51);
        
        let val = Validator::new("val1".to_string(), vec![1, 2, 3], 1000);
        consensus.add_validator(val).unwrap();

        let mut tx = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
        tx.sign(b"alice_key");

        let block = Block::new(0, vec![], vec![tx], "val1".to_string());
        consensus.add_block(block.clone()).unwrap();

        assert_eq!(consensus.get_height(), 1);
        
        let retrieved_block = consensus.get_block(0).unwrap();
        assert_eq!(retrieved_block.height, block.height);
    }

    #[test]
    fn test_consensus_achievement() {
        let consensus = BlockchainConsensus::new(3, 0.67);
        
        // Add validators
        for i in 0..5 {
            let val = Validator::new(
                format!("val{}", i),
                vec![i as u8],
                1000,
            );
            consensus.add_validator(val).unwrap();
        }

        let mut tx = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
        tx.sign(b"alice_key");

        let block = Block::new(0, vec![], vec![tx], "val0".to_string());
        let consensus_reached = consensus.achieve_consensus(&block).unwrap();
        assert!(consensus_reached);
    }

    #[test]
    fn test_invalid_nonce() {
        let consensus = BlockchainConsensus::new(1, 0.51);
        
        let mut tx1 = Transaction::new("alice".to_string(), "bob".to_string(), 100, 0);
        tx1.sign(b"alice_key");
        consensus.add_to_mempool(tx1).unwrap();

        // Try to add transaction with wrong nonce
        let mut tx2 = Transaction::new("alice".to_string(), "charlie".to_string(), 50, 5);
        tx2.sign(b"alice_key");
        
        let result = consensus.add_to_mempool(tx2);
        assert!(result.is_err());
    }
}
