//! Blockchain Resilience Module
//!
//! This module implements the Priority 3 features from DEX-OS-V2.csv:
//! - Blockchain Resilience,Blockchain Resilience,Blockchain Resilience,Proof of Stake (PoS),Validator Bonding,Medium
//! - Blockchain Resilience,Blockchain Resilience,Blockchain Resilience,UTXO Model,Double-Spend Prevention,Medium

use fastrand;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

/// Unique identifier for a validator
pub type ValidatorId = String;

/// Unique identifier for a transaction
pub type TransactionId = String;

/// Unique identifier for a block
pub type BlockId = String;

/// Unique identifier for a UTXO
pub type UtxoId = String;

/// Stake amount representation
pub type StakeAmount = u64;

/// Token amount representation
pub type TokenAmount = u64;

/// Block height representation
pub type BlockHeight = u64;

/// Timestamp representation
pub type Timestamp = u64;

/// Represents a validator in the PoS system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Validator {
    /// Unique identifier for the validator
    pub id: ValidatorId,
    /// Amount of tokens staked by the validator
    pub stake: StakeAmount,
    /// Public key for signature verification
    pub public_key: Vec<u8>,
    /// Whether the validator is currently active
    pub is_active: bool,
    /// Timestamp when the validator was registered
    pub registered_at: Timestamp,
    /// Total rewards earned by the validator
    pub total_rewards: TokenAmount,
}

/// Represents a staking operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stake {
    /// Unique identifier for the stake
    pub id: String,
    /// Validator this stake is bonded to
    pub validator_id: ValidatorId,
    /// Amount of tokens staked
    pub amount: StakeAmount,
    /// Owner of the stake
    pub owner: String,
    /// Timestamp when the stake was created
    pub created_at: Timestamp,
    /// Whether the stake is currently active
    pub is_active: bool,
}

/// Represents a transaction in the UTXO model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UtxoTransaction {
    /// Unique identifier for the transaction
    pub id: TransactionId,
    /// Inputs consumed by this transaction
    pub inputs: Vec<UtxoInput>,
    /// Outputs created by this transaction
    pub outputs: Vec<UtxoOutput>,
    /// Timestamp when the transaction was created
    pub timestamp: Timestamp,
    /// Signatures for the inputs
    pub signatures: Vec<Vec<u8>>,
}

/// Represents an input in a UTXO transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UtxoInput {
    /// Reference to the UTXO being spent
    pub utxo_id: UtxoId,
    /// Public key of the owner
    pub public_key: Vec<u8>,
}

/// Represents an output in a UTXO transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UtxoOutput {
    /// Unique identifier for this UTXO
    pub id: UtxoId,
    /// Public key of the recipient
    pub public_key: Vec<u8>,
    /// Amount of tokens in this UTXO
    pub amount: TokenAmount,
    /// Timestamp when this UTXO was created
    pub created_at: Timestamp,
}

/// Represents a UTXO that is available to be spent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnspentTransactionOutput {
    /// The UTXO output
    pub output: UtxoOutput,
    /// Transaction ID that created this UTXO
    pub transaction_id: TransactionId,
}

/// Represents a block in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    /// Unique identifier for the block
    pub id: BlockId,
    /// Height of the block in the chain
    pub height: BlockHeight,
    /// Hash of the previous block
    pub previous_hash: Vec<u8>,
    /// Timestamp when the block was created
    pub timestamp: Timestamp,
    /// Transactions included in this block
    pub transactions: Vec<UtxoTransaction>,
    /// Validator who proposed this block
    pub validator_id: ValidatorId,
    /// Signature of the validator
    pub signature: Vec<u8>,
}

/// Proof of Stake implementation with validator bonding
pub struct ProofOfStake {
    /// Registered validators
    validators: HashMap<ValidatorId, Validator>,
    /// Active stakes
    stakes: HashMap<String, Stake>,
    /// Total staked amount across all validators
    total_staked: StakeAmount,
    /// Minimum stake required to become a validator
    min_stake_required: StakeAmount,
}

/// UTXO Model implementation with double-spend prevention
pub struct UtxoModel {
    /// Unspent transaction outputs
    utxo_set: HashMap<UtxoId, UnspentTransactionOutput>,
    /// Processed transactions to prevent double-spending
    processed_transactions: HashSet<TransactionId>,
    /// Current block height
    current_height: BlockHeight,
}

/// Blockchain resilience system combining PoS and UTXO
pub struct BlockchainResilience {
    /// Proof of Stake subsystem
    pub pos: ProofOfStake,
    /// UTXO Model subsystem
    pub utxo: UtxoModel,
    /// Blockchain storage
    blocks: Vec<Block>,
    /// Genesis block hash
    genesis_hash: Vec<u8>,
}

impl ProofOfStake {
    /// Create a new Proof of Stake system
    pub fn new(min_stake_required: StakeAmount) -> Self {
        Self {
            validators: HashMap::new(),
            stakes: HashMap::new(),
            total_staked: 0,
            min_stake_required,
        }
    }

    /// Register a new validator
    pub fn register_validator(
        &mut self,
        id: ValidatorId,
        public_key: Vec<u8>,
        initial_stake: StakeAmount,
    ) -> Result<(), String> {
        if self.validators.contains_key(&id) {
            return Err("Validator already registered".to_string());
        }

        if initial_stake < self.min_stake_required {
            return Err(format!(
                "Initial stake {} is less than minimum required {}",
                initial_stake, self.min_stake_required
            ));
        }

        let validator = Validator {
            id: id.clone(),
            stake: initial_stake,
            public_key,
            is_active: true,
            registered_at: now(),
            total_rewards: 0,
        };

        self.validators.insert(id.clone(), validator);
        self.total_staked += initial_stake;

        // Create initial stake
        let stake_id = format!("stake_{}_initial", id.clone());
        let stake = Stake {
            id: stake_id.clone(),
            validator_id: id,
            amount: initial_stake,
            owner: "initial".to_string(),
            created_at: now(),
            is_active: true,
        };

        self.stakes.insert(stake_id, stake);

        Ok(())
    }

    /// Stake tokens to a validator
    pub fn stake_tokens(
        &mut self,
        stake_id: String,
        validator_id: ValidatorId,
        amount: StakeAmount,
        owner: String,
    ) -> Result<(), String> {
        if !self.validators.contains_key(&validator_id) {
            return Err("Validator not found".to_string());
        }

        if self.stakes.contains_key(&stake_id) {
            return Err("Stake ID already exists".to_string());
        }

        let stake = Stake {
            id: stake_id.clone(),
            validator_id: validator_id.clone(),
            amount,
            owner,
            created_at: now(),
            is_active: true,
        };

        self.stakes.insert(stake_id, stake);

        // Update validator's stake
        if let Some(validator) = self.validators.get_mut(&validator_id) {
            validator.stake += amount;
        }

        self.total_staked += amount;

        Ok(())
    }

    /// Unstake tokens from a validator
    pub fn unstake_tokens(&mut self, stake_id: &str) -> Result<StakeAmount, String> {
        let stake = self
            .stakes
            .get(stake_id)
            .ok_or("Stake not found")?
            .clone();

        if !stake.is_active {
            return Err("Stake is already inactive".to_string());
        }

        // Mark stake as inactive
        if let Some(mut_stake) = self.stakes.get_mut(stake_id) {
            mut_stake.is_active = false;
        }

        // Update validator's stake
        if let Some(validator) = self.validators.get_mut(&stake.validator_id) {
            validator.stake = validator.stake.saturating_sub(stake.amount);
        }

        self.total_staked = self.total_staked.saturating_sub(stake.amount);

        Ok(stake.amount)
    }

    /// Get validator by ID
    pub fn get_validator(&self, id: &ValidatorId) -> Option<&Validator> {
        self.validators.get(id)
    }

    /// Get all active validators
    pub fn get_active_validators(&self) -> Vec<&Validator> {
        self.validators
            .values()
            .filter(|v| v.is_active)
            .collect()
    }

    /// Select a validator for block proposal using weighted random selection
    pub fn select_validator(&self) -> Option<ValidatorId> {
        if self.validators.is_empty() || self.total_staked == 0 {
            return None;
        }

        // Simple weighted selection based on stake
        let mut rng = fastrand::Rng::with_seed(now() as u64);
        let target = rng.u64(0..self.total_staked);
        
        let mut cumulative_stake = 0;
        for (id, validator) in &self.validators {
            if validator.is_active {
                cumulative_stake += validator.stake;
                if cumulative_stake > target {
                    return Some(id.clone());
                }
            }
        }

        // Fallback to first active validator
        self.validators
            .iter()
            .find(|(_, v)| v.is_active)
            .map(|(id, _)| id.clone())
    }

    /// Reward a validator for proposing a block
    pub fn reward_validator(&mut self, validator_id: &ValidatorId, amount: TokenAmount) -> Result<(), String> {
        let validator = self
            .validators
            .get_mut(validator_id)
            .ok_or("Validator not found")?;

        validator.total_rewards += amount;

        Ok(())
    }

    /// Get total staked amount
    pub fn get_total_staked(&self) -> StakeAmount {
        self.total_staked
    }
}

impl UtxoModel {
    /// Create a new UTXO Model
    pub fn new() -> Self {
        Self {
            utxo_set: HashMap::new(),
            processed_transactions: HashSet::new(),
            current_height: 0,
        }
    }

    /// Add a new UTXO to the set
    pub fn add_utxo(&mut self, utxo: UnspentTransactionOutput) {
        self.utxo_set.insert(utxo.output.id.clone(), utxo);
    }

    /// Spend a UTXO (remove it from the set)
    pub fn spend_utxo(&mut self, utxo_id: &UtxoId) -> Result<UnspentTransactionOutput, String> {
        self.utxo_set
            .remove(utxo_id)
            .ok_or_else(|| format!("UTXO {} not found", utxo_id))
    }

    /// Validate a transaction to prevent double-spending
    pub fn validate_transaction(&self, transaction: &UtxoTransaction) -> Result<(), String> {
        // Check if transaction has already been processed
        if self.processed_transactions.contains(&transaction.id) {
            return Err("Transaction already processed (double spend attempt)".to_string());
        }

        // Check that all inputs exist and are unspent
        for input in &transaction.inputs {
            if !self.utxo_set.contains_key(&input.utxo_id) {
                return Err(format!("Input UTXO {} not found or already spent", input.utxo_id));
            }
        }

        // Verify that output amounts are positive
        for output in &transaction.outputs {
            if output.amount == 0 {
                return Err("Output amount must be positive".to_string());
            }
        }

        // Verify that total input equals total output (simplified - in a real system, fees would be considered)
        let total_input: TokenAmount = transaction
            .inputs
            .iter()
            .map(|input| {
                self.utxo_set
                    .get(&input.utxo_id)
                    .map(|utxo| utxo.output.amount)
                    .unwrap_or(0)
            })
            .sum();

        let total_output: TokenAmount = transaction
            .outputs
            .iter()
            .map(|output| output.amount)
            .sum();

        if total_input != total_output {
            return Err("Total input amount does not equal total output amount".to_string());
        }

        Ok(())
    }

    /// Process a valid transaction and update the UTXO set
    pub fn process_transaction(&mut self, transaction: UtxoTransaction) -> Result<(), String> {
        // First validate the transaction
        self.validate_transaction(&transaction)?;

        // Mark transaction as processed to prevent double-spending
        self.processed_transactions.insert(transaction.id.clone());

        // Remove spent UTXOs
        for input in &transaction.inputs {
            self.utxo_set.remove(&input.utxo_id);
        }

        // Add new UTXOs
        for (index, output) in transaction.outputs.iter().enumerate() {
            let utxo_id = format!("{}_{}", transaction.id, index);
            let utxo = UnspentTransactionOutput {
                output: output.clone(),
                transaction_id: transaction.id.clone(),
            };
            self.utxo_set.insert(utxo_id, utxo);
        }

        Ok(())
    }

    /// Get UTXO by ID
    pub fn get_utxo(&self, utxo_id: &UtxoId) -> Option<&UnspentTransactionOutput> {
        self.utxo_set.get(utxo_id)
    }

    /// Get all UTXOs for a specific public key
    pub fn get_utxos_for_address(&self, public_key: &[u8]) -> Vec<&UnspentTransactionOutput> {
        self.utxo_set
            .values()
            .filter(|utxo| utxo.output.public_key == public_key)
            .collect()
    }

    /// Check if a transaction has been processed
    pub fn is_transaction_processed(&self, transaction_id: &TransactionId) -> bool {
        self.processed_transactions.contains(transaction_id)
    }
}

impl BlockchainResilience {
    /// Create a new blockchain resilience system
    pub fn new(min_stake_required: StakeAmount) -> Self {
        let genesis_hash = vec![0u8; 32]; // Placeholder genesis hash
        
        Self {
            pos: ProofOfStake::new(min_stake_required),
            utxo: UtxoModel::new(),
            blocks: Vec::new(),
            genesis_hash,
        }
    }

    /// Create genesis block with initial UTXOs
    pub fn create_genesis_block(&mut self, initial_utxos: Vec<UtxoOutput>) {
        let genesis_block = Block {
            id: "genesis".to_string(),
            height: 0,
            previous_hash: self.genesis_hash.clone(),
            timestamp: now(),
            transactions: vec![], // Genesis block typically has no transactions
            validator_id: "genesis".to_string(),
            signature: vec![],
        };

        self.blocks.push(genesis_block);

        // Add initial UTXOs to the UTXO set
        for (index, output) in initial_utxos.into_iter().enumerate() {
            let utxo_id = format!("genesis_{}", index);
            let utxo = UnspentTransactionOutput {
                output,
                transaction_id: "genesis".to_string(),
            };
            self.utxo.add_utxo(utxo);
        }
    }

    /// Propose a new block using PoS validator selection
    pub fn propose_block(&mut self, transactions: Vec<UtxoTransaction>) -> Result<Block, String> {
        // Validate all transactions first
        for transaction in &transactions {
            self.utxo.validate_transaction(transaction)?;
        }

        // Select validator using PoS
        let validator_id = self
            .pos
            .select_validator()
            .ok_or("No active validators available")?;

        // Create block
        let previous_hash = if let Some(last_block) = self.blocks.last() {
            // In a real implementation, we would hash the block
            last_block.id.as_bytes().to_vec()
        } else {
            self.genesis_hash.clone()
        };

        let block_height = self.blocks.len() as BlockHeight;

        let block = Block {
            id: format!("block_{}", block_height),
            height: block_height,
            previous_hash,
            timestamp: now(),
            transactions,
            validator_id: validator_id.clone(),
            signature: vec![], // In a real implementation, this would be a cryptographic signature
        };

        Ok(block)
    }

    /// Add a block to the blockchain after validation
    pub fn add_block(&mut self, mut block: Block) -> Result<(), String> {
        // Validate block
        if block.height != self.blocks.len() as BlockHeight {
            return Err("Block height mismatch".to_string());
        }

        // Validate that previous hash matches
        let expected_previous_hash = if let Some(last_block) = self.blocks.last() {
            last_block.id.as_bytes().to_vec()
        } else {
            self.genesis_hash.clone()
        };

        if block.previous_hash != expected_previous_hash {
            return Err("Previous hash mismatch".to_string());
        }

        // Validate all transactions in the block
        for transaction in &block.transactions {
            self.utxo.validate_transaction(transaction)?;
        }

        // Process all transactions in the block
        for transaction in block.transactions.clone() {
            self.utxo.process_transaction(transaction)?;
        }

        // Add block to chain
        self.blocks.push(block.clone());

        // Reward validator
        self.pos.reward_validator(&block.validator_id, 1000)?; // Fixed reward for simplicity

        Ok(())
    }

    /// Get the latest block
    pub fn get_latest_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    /// Get block by height
    pub fn get_block_by_height(&self, height: BlockHeight) -> Option<&Block> {
        self.blocks.get(height as usize)
    }

    /// Get blockchain height
    pub fn get_height(&self) -> BlockHeight {
        self.blocks.len() as BlockHeight
    }

    /// Verify the integrity of the blockchain
    pub fn verify_chain(&self) -> bool {
        // Verify genesis block
        if self.blocks.is_empty() {
            return true; // Empty chain is valid
        }

        // Verify each block links to the previous one
        for i in 1..self.blocks.len() {
            let current_block = &self.blocks[i];
            let previous_block = &self.blocks[i - 1];

            if current_block.previous_hash != previous_block.id.as_bytes() {
                return false;
            }

            if current_block.height != i as BlockHeight {
                return false;
            }
        }

        true
    }
}

/// Get current timestamp in seconds
fn now() -> Timestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Represents a commitment in a commit-reveal scheme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Commitment {
    /// The hash of the data + salt
    pub commitment_hash: Vec<u8>,
    /// Timestamp when the commitment was made
    pub timestamp: Timestamp,
    /// ID of the committer (e.g., ValidatorId or User address)
    pub committer_id: String,
}

/// Represents a reveal in a commit-reveal scheme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reveal {
    /// The original data
    pub data: Vec<u8>,
    /// The salt used
    pub salt: Vec<u8>,
}

/// Commit-Reveal Scheme implementation
pub struct CommitRevealScheme {
    /// Active commitments awaiting reveal
    commitments: HashMap<String, Commitment>, // Key could be committer_id or a unique ID
    /// Time window for reveal phase (in seconds)
    reveal_window: u64,
}

impl CommitRevealScheme {
    pub fn new(reveal_window: u64) -> Self {
        Self {
            commitments: HashMap::new(),
            reveal_window,
        }
    }

    /// Submit a commitment
    pub fn commit(&mut self, id: String, commitment_hash: Vec<u8>, committer_id: String) -> Result<(), String> {
        if self.commitments.contains_key(&id) {
            return Err("Commitment ID already exists".to_string());
        }

        let commitment = Commitment {
            commitment_hash,
            timestamp: now(),
            committer_id,
        };

        self.commitments.insert(id, commitment);
        Ok(())
    }

    /// Reveal and verify
    pub fn reveal(&mut self, id: &str, reveal: Reveal) -> Result<Vec<u8>, String> {
        let commitment = self.commitments.get(id).ok_or("Commitment not found")?;

        // Check if reveal window has passed (optional, depending on requirements)
        // if now() > commitment.timestamp + self.reveal_window {
        //     return Err("Reveal window expired".to_string());
        // }

        // Verify hash
        let mut hasher = Sha256::new();
        hasher.update(&reveal.data);
        hasher.update(&reveal.salt);
        let calculated_hash = hasher.finalize().to_vec();

        if calculated_hash != commitment.commitment_hash {
            return Err("Hash mismatch! Invalid reveal.".to_string());
        }

        // Remove commitment after successful reveal (prevention of replay)
        self.commitments.remove(id);

        Ok(reveal.data)
    }
    
    /// Create a commitment hash (utility)
    pub fn create_hash(data: &[u8], salt: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(salt);
        hasher.finalize().to_vec()
    }
}

/// MEV Protection using Commit-Reveal for transaction ordering
pub struct MevProtection {
    /// The commit-reveal scheme
    scheme: CommitRevealScheme,
    /// Ordered transactions (revealed)
    transaction_pool: Vec<UtxoTransaction>,
}

impl MevProtection {
    pub fn new(reveal_window: u64) -> Self {
        Self {
            scheme: CommitRevealScheme::new(reveal_window),
            transaction_pool: Vec::new(),
        }
    }

    /// Submit a transaction commitment
    pub fn submit_commitment(&mut self, tx_id: String, hash: Vec<u8>, committer: String) -> Result<(), String> {
        self.scheme.commit(tx_id, hash, committer)
    }

    /// Reveal a transaction
    pub fn reveal_transaction(&mut self, tx_id: &str, tx_data: Vec<u8>, salt: Vec<u8>) -> Result<(), String> {
        let revealed_data = self.scheme.reveal(tx_id, Reveal { data: tx_data.clone(), salt })?;
        
        // Deserialize transaction
        let transaction: UtxoTransaction = serde_json::from_slice(&revealed_data)
            .map_err(|_| "Failed to deserialize transaction".to_string())?;
            
        self.transaction_pool.push(transaction);
        Ok(())
    }
    
    /// Get ordered transactions
    pub fn get_transactions(&self) -> &Vec<UtxoTransaction> {
        &self.transaction_pool
    }
    
    /// Clear pool
    pub fn clear_pool(&mut self) {
        self.transaction_pool.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_validator_registration() {
        let mut pos = ProofOfStake::new(1000);
        
        // Register a validator
        assert!(pos.register_validator("validator1".to_string(), vec![1, 2, 3, 4], 1500).is_ok());
        
        // Try to register the same validator again
        assert!(pos.register_validator("validator1".to_string(), vec![1, 2, 3, 4], 1500).is_err());
        
        // Check validator exists
        let validator = pos.get_validator(&"validator1".to_string());
        assert!(validator.is_some());
        assert_eq!(validator.unwrap().stake, 1500);
        
        // Check total staked
        assert_eq!(pos.get_total_staked(), 1500);
    }

    #[test]
    fn test_pos_staking() {
        let mut pos = ProofOfStake::new(1000);
        
        // Register a validator
        assert!(pos.register_validator("validator1".to_string(), vec![1, 2, 3, 4], 1500).is_ok());
        
        // Stake additional tokens
        assert!(pos.stake_tokens("stake1".to_string(), "validator1".to_string(), 500, "owner1".to_string()).is_ok());
        
        // Check validator stake increased
        let validator = pos.get_validator(&"validator1".to_string()).unwrap();
        assert_eq!(validator.stake, 2000);
        
        // Check total staked increased
        assert_eq!(pos.get_total_staked(), 2000);
    }

    #[test]
    fn test_pos_unstaking() {
        let mut pos = ProofOfStake::new(1000);
        
        // Register a validator
        assert!(pos.register_validator("validator1".to_string(), vec![1, 2, 3, 4], 1500).is_ok());
        
        // Stake additional tokens
        assert!(pos.stake_tokens("stake1".to_string(), "validator1".to_string(), 500, "owner1".to_string()).is_ok());
        
        // Unstake tokens
        let unstaked_amount = pos.unstake_tokens("stake1");
        assert!(unstaked_amount.is_ok());
        assert_eq!(unstaked_amount.unwrap(), 500);
        
        // Check validator stake decreased
        let validator = pos.get_validator(&"validator1".to_string()).unwrap();
        assert_eq!(validator.stake, 1500);
        
        // Check total staked decreased
        assert_eq!(pos.get_total_staked(), 1500);
    }

    #[test]
    fn test_pos_validator_selection() {
        let mut pos = ProofOfStake::new(1000);
        
        // Register multiple validators with different stakes
        assert!(pos.register_validator("validator1".to_string(), vec![1, 2, 3, 4], 1500).is_ok());
        assert!(pos.register_validator("validator2".to_string(), vec![5, 6, 7, 8], 3000).is_ok());
        assert!(pos.register_validator("validator3".to_string(), vec![9, 10, 11, 12], 500).is_ok()); // Below minimum, should not be selected
        
        // Select validator multiple times - higher stake validators should be selected more often
        let mut selections = HashMap::new();
        for _ in 0..100 {
            if let Some(selected) = pos.select_validator() {
                *selections.entry(selected).or_insert(0) += 1;
            }
        }
        
        // Validator2 should be selected more often than validator1 due to higher stake
        let count1 = *selections.get("validator1").unwrap_or(&0);
        let count2 = *selections.get("validator2").unwrap_or(&0);
        
        // This is probabilistic, but validator2 should generally be selected more often
        assert!(count2 >= count1);
    }

    #[test]
    fn test_utxo_add_and_spend() {
        let mut utxo_model = UtxoModel::new();
        
        // Create a UTXO
        let output = UtxoOutput {
            id: "utxo1".to_string(),
            public_key: vec![1, 2, 3, 4],
            amount: 1000,
            created_at: now(),
        };
        
        let utxo = UnspentTransactionOutput {
            output,
            transaction_id: "tx1".to_string(),
        };
        
        // Add UTXO
        utxo_model.add_utxo(utxo);
        
        // Check UTXO exists
        assert!(utxo_model.get_utxo(&"utxo1".to_string()).is_some());
        
        // Spend UTXO
        let spent_utxo = utxo_model.spend_utxo(&"utxo1".to_string());
        assert!(spent_utxo.is_ok());
        
        // Check UTXO no longer exists
        assert!(utxo_model.get_utxo(&"utxo1".to_string()).is_none());
    }

    #[test]
    fn test_utxo_transaction_validation() {
        let mut utxo_model = UtxoModel::new();
        
        // Create initial UTXO
        let output = UtxoOutput {
            id: "utxo1".to_string(),
            public_key: vec![1, 2, 3, 4],
            amount: 1000,
            created_at: now(),
        };
        
        let utxo = UnspentTransactionOutput {
            output,
            transaction_id: "tx1".to_string(),
        };
        
        utxo_model.add_utxo(utxo);
        
        // Create a valid transaction
        let transaction = UtxoTransaction {
            id: "tx2".to_string(),
            inputs: vec![UtxoInput {
                utxo_id: "utxo1".to_string(),
                public_key: vec![1, 2, 3, 4],
            }],
            outputs: vec![UtxoOutput {
                id: "utxo2".to_string(),
                public_key: vec![5, 6, 7, 8],
                amount: 1000,
                created_at: now(),
            }],
            timestamp: now(),
            signatures: vec![vec![1, 2, 3]], // Placeholder signature
        };
        
        // Validate transaction
        assert!(utxo_model.validate_transaction(&transaction).is_ok());
    }

    #[test]
    fn test_utxo_double_spend_prevention() {
        let mut utxo_model = UtxoModel::new();
        
        // Create initial UTXO
        let output = UtxoOutput {
            id: "utxo1".to_string(),
            public_key: vec![1, 2, 3, 4],
            amount: 1000,
            created_at: now(),
        };
        
        let utxo = UnspentTransactionOutput {
            output,
            transaction_id: "tx1".to_string(),
        };
        
        utxo_model.add_utxo(utxo);
        
        // Create a transaction
        let transaction = UtxoTransaction {
            id: "tx2".to_string(),
            inputs: vec![UtxoInput {
                utxo_id: "utxo1".to_string(),
                public_key: vec![1, 2, 3, 4],
            }],
            outputs: vec![UtxoOutput {
                id: "utxo2".to_string(),
                public_key: vec![5, 6, 7, 8],
                amount: 1000,
                created_at: now(),
            }],
            timestamp: now(),
            signatures: vec![vec![1, 2, 3]], // Placeholder signature
        };
        
        // Process transaction first time
        assert!(utxo_model.process_transaction(transaction.clone()).is_ok());
        
        // Try to process the same transaction again (double spend)
        assert!(utxo_model.process_transaction(transaction).is_err());
    }

    #[test]
    fn test_blockchain_resilience_genesis_block() {
        let mut blockchain = BlockchainResilience::new(1000);
        
        // Create initial UTXOs
        let initial_utxos = vec![UtxoOutput {
            id: "genesis_utxo1".to_string(),
            public_key: vec![1, 2, 3, 4],
            amount: 10000,
            created_at: now(),
        }];
        
        blockchain.create_genesis_block(initial_utxos);
        
        // Check genesis block exists
        assert_eq!(blockchain.get_height(), 1);
        assert!(blockchain.get_latest_block().is_some());
        
        // Check initial UTXO exists
        assert!(blockchain.utxo.get_utxo(&"genesis_0".to_string()).is_some());
    }

    #[test]
    fn test_blockchain_resilience_block_proposal_and_addition() {
        let mut blockchain = BlockchainResilience::new(1000);
        
        // Register a validator
        assert!(blockchain.pos.register_validator("validator1".to_string(), vec![1, 2, 3, 4], 1500).is_ok());
        
        // Create genesis block
        let initial_utxos = vec![UtxoOutput {
            id: "genesis_utxo1".to_string(),
            public_key: vec![1, 2, 3, 4],
            amount: 10000,
            created_at: now(),
        }];
        
        blockchain.create_genesis_block(initial_utxos);
        
        // Create a transaction
        let transaction = UtxoTransaction {
            id: "tx1".to_string(),
            inputs: vec![UtxoInput {
                utxo_id: "genesis_0".to_string(),
                public_key: vec![1, 2, 3, 4],
            }],
            outputs: vec![UtxoOutput {
                id: "utxo1".to_string(),
                public_key: vec![5, 6, 7, 8],
                amount: 10000,
                created_at: now(),
            }],
            timestamp: now(),
            signatures: vec![vec![1, 2, 3]], // Placeholder signature
        };
        
        // Propose a block
        let block = blockchain.propose_block(vec![transaction]);
        assert!(block.is_ok());
        
        // Add the block
        assert!(blockchain.add_block(block.unwrap()).is_ok());
        
        // Check blockchain height increased
        assert_eq!(blockchain.get_height(), 2);
        
        // Check validator was rewarded
        let validator = blockchain.pos.get_validator(&"validator1".to_string()).unwrap();
        assert_eq!(validator.total_rewards, 1000);
    }

    #[test]
    fn test_blockchain_resilience_chain_verification() {
        let mut blockchain = BlockchainResilience::new(1000);
        
        // Register a validator
        assert!(blockchain.pos.register_validator("validator1".to_string(), vec![1, 2, 3, 4], 1500).is_ok());
        
        // Create genesis block
        let initial_utxos = vec![UtxoOutput {
            id: "genesis_utxo1".to_string(),
            public_key: vec![1, 2, 3, 4],
            amount: 10000,
            created_at: now(),
        }];
        
        blockchain.create_genesis_block(initial_utxos);
        
        // Create and add a block
        let transaction = UtxoTransaction {
            id: "tx1".to_string(),
            inputs: vec![UtxoInput {
                utxo_id: "genesis_0".to_string(),
                public_key: vec![1, 2, 3, 4],
            }],
            outputs: vec![UtxoOutput {
                id: "utxo1".to_string(),
                public_key: vec![5, 6, 7, 8],
                amount: 10000,
                created_at: now(),
            }],
            timestamp: now(),
            signatures: vec![vec![1, 2, 3]], // Placeholder signature
        };
        
        let block = blockchain.propose_block(vec![transaction]).unwrap();
        assert!(blockchain.add_block(block).is_ok());
        
        // Verify chain integrity
        assert!(blockchain.verify_chain());
    }
    #[test]
    fn test_commit_reveal_scheme() {
        let mut scheme = CommitRevealScheme::new(60);
        let data = b"secret_transaction_data";
        let salt = b"random_salt";
        let committer = "user1".to_string();
        let tx_id = "tx1".to_string();

        let hash = CommitRevealScheme::create_hash(data, salt);

        // Commit
        assert!(scheme.commit(tx_id.clone(), hash.clone(), committer.clone()).is_ok());

        // Try to commit same ID again
        assert!(scheme.commit(tx_id.clone(), hash.clone(), committer.clone()).is_err());

        // Reveal with wrong salt
        let wrong_reveal = Reveal {
            data: data.to_vec(),
            salt: b"wrong_salt".to_vec(),
        };
        assert!(scheme.reveal(&tx_id, wrong_reveal).is_err());

        // Reveal with correct data
        let correct_reveal = Reveal {
            data: data.to_vec(),
            salt: salt.to_vec(),
        };
        let revealed_data = scheme.reveal(&tx_id, correct_reveal);
        assert!(revealed_data.is_ok());
        assert_eq!(revealed_data.unwrap(), data.to_vec());

        // Try to reveal again (should be removed)
        let correct_reveal_again = Reveal {
            data: data.to_vec(),
            salt: salt.to_vec(),
        };
        assert!(scheme.reveal(&tx_id, correct_reveal_again).is_err());
    }

    #[test]
    fn test_mev_protection() {
        let mut mev_protection = MevProtection::new(60);
        
        let tx = UtxoTransaction {
            id: "tx1".to_string(),
            inputs: vec![],
            outputs: vec![],
            timestamp: now(),
            signatures: vec![],
        };
        
        let tx_data = serde_json::to_vec(&tx).unwrap();
        let salt = b"salt123";
        let hash = CommitRevealScheme::create_hash(&tx_data, salt);
        
        // Submit commitment
        assert!(mev_protection.submit_commitment("tx1".to_string(), hash, "user1".to_string()).is_ok());
        
        // Reveal transaction
        assert!(mev_protection.reveal_transaction("tx1", tx_data, salt.to_vec()).is_ok());
        
        // Check transaction is in pool
        let pool = mev_protection.get_transactions();
        assert_eq!(pool.len(), 1);
        assert_eq!(pool[0].id, "tx1");
    }
}