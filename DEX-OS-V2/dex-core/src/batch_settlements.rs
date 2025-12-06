//! Batch Settlements - Layer 2 Scaling
//!
//! Implements: `4,Scalability & Interoperability,Layer 2 Scaling,Layer 2 Scaling,Batch Settlements,Batch Settlements,High`
//!
//! This module provides batch settlement functionality for aggregating multiple
//! transactions into a single on-chain settlement, reducing gas costs and improving throughput.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors that can occur in batch settlement operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchSettlementError {
    BatchNotFound,
    BatchAlreadySettled,
    BatchFull,
    InvalidTransaction,
    InsufficientBalance,
    InvalidSignature,
    SettlementFailed,
    InvalidBatchSize,
    TransactionNotFound,
}

/// Status of a batch settlement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    Pending,
    Aggregating,
    Ready,
    Settling,
    Settled,
    Failed,
}

/// Transaction type for batch settlement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Trade,
    Swap,
    Deposit,
    Withdrawal,
}

/// Individual transaction in a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub tx_type: TransactionType,
    pub from: String,
    pub to: String,
    pub asset: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        id: String,
        tx_type: TransactionType,
        from: String,
        to: String,
        asset: String,
        amount: u64,
        fee: u64,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            tx_type,
            from,
            to,
            asset,
            amount,
            fee,
            nonce: 0,
            timestamp,
            signature: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Validate transaction
    pub fn validate(&self) -> Result<(), BatchSettlementError> {
        if self.from.is_empty() || self.to.is_empty() {
            return Err(BatchSettlementError::InvalidTransaction);
        }

        if self.asset.is_empty() {
            return Err(BatchSettlementError::InvalidTransaction);
        }

        if self.amount == 0 {
            return Err(BatchSettlementError::InvalidTransaction);
        }

        Ok(())
    }
}

/// Merkle proof for transaction inclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub leaf_index: usize,
    pub proof: Vec<Vec<u8>>,
    pub root: Vec<u8>,
}

/// Batch of transactions for settlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementBatch {
    pub id: String,
    pub transactions: Vec<Transaction>,
    pub status: BatchStatus,
    pub max_size: usize,
    pub created_at: u64,
    pub settled_at: Option<u64>,
    pub merkle_root: Option<Vec<u8>>,
    pub settlement_tx_hash: Option<String>,
    pub total_fees: u64,
    pub net_balances: HashMap<String, HashMap<String, i64>>, // address -> asset -> balance change
}

impl SettlementBatch {
    /// Create a new settlement batch
    pub fn new(id: String, max_size: usize) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            transactions: Vec::new(),
            status: BatchStatus::Pending,
            max_size,
            created_at: timestamp,
            settled_at: None,
            merkle_root: None,
            settlement_tx_hash: None,
            total_fees: 0,
            net_balances: HashMap::new(),
        }
    }

    /// Add a transaction to the batch
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), BatchSettlementError> {
        if self.status != BatchStatus::Pending && self.status != BatchStatus::Aggregating {
            return Err(BatchSettlementError::BatchAlreadySettled);
        }

        if self.transactions.len() >= self.max_size {
            return Err(BatchSettlementError::BatchFull);
        }

        // Validate transaction
        tx.validate()?;

        // Update net balances
        self.update_net_balances(&tx);

        // Add transaction
        self.transactions.push(tx);
        self.status = BatchStatus::Aggregating;

        Ok(())
    }

    /// Update net balances based on transaction
    fn update_net_balances(&mut self, tx: &Transaction) {
        // Debit from sender
        let from_balances = self.net_balances.entry(tx.from.clone()).or_insert_with(HashMap::new);
        *from_balances.entry(tx.asset.clone()).or_insert(0) -= (tx.amount + tx.fee) as i64;

        // Credit to receiver
        let to_balances = self.net_balances.entry(tx.to.clone()).or_insert_with(HashMap::new);
        *to_balances.entry(tx.asset.clone()).or_insert(0) += tx.amount as i64;

        // Accumulate fees
        self.total_fees += tx.fee;
    }

    /// Mark batch as ready for settlement
    pub fn mark_ready(&mut self) -> Result<(), BatchSettlementError> {
        if self.transactions.is_empty() {
            return Err(BatchSettlementError::InvalidBatchSize);
        }

        self.status = BatchStatus::Ready;
        self.compute_merkle_root();

        Ok(())
    }

    /// Compute Merkle root for the batch
    fn compute_merkle_root(&mut self) {
        // Simple hash-based Merkle root computation
        let mut hashes: Vec<Vec<u8>> = self
            .transactions
            .iter()
            .map(|tx| {
                let data = format!("{}{}{}{}", tx.id, tx.from, tx.to, tx.amount);
                Self::hash(data.as_bytes())
            })
            .collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    [chunk[0].clone(), chunk[1].clone()].concat()
                } else {
                    [chunk[0].clone(), chunk[0].clone()].concat()
                };
                next_level.push(Self::hash(&combined));
            }
            hashes = next_level;
        }

        self.merkle_root = hashes.into_iter().next();
    }

    /// Simple hash function (in production, use SHA256 or similar)
    fn hash(data: &[u8]) -> Vec<u8> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish().to_be_bytes().to_vec()
    }

    /// Get Merkle proof for a transaction
    pub fn get_merkle_proof(&self, tx_id: &str) -> Result<MerkleProof, BatchSettlementError> {
        let index = self
            .transactions
            .iter()
            .position(|tx| tx.id == tx_id)
            .ok_or(BatchSettlementError::TransactionNotFound)?;

        // Simplified proof generation
        let proof = vec![self.merkle_root.clone().unwrap_or_default()];

        Ok(MerkleProof {
            leaf_index: index,
            proof,
            root: self.merkle_root.clone().unwrap_or_default(),
        })
    }

    /// Settle the batch
    pub fn settle(&mut self, tx_hash: String) -> Result<(), BatchSettlementError> {
        if self.status != BatchStatus::Ready {
            return Err(BatchSettlementError::InvalidTransaction);
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.status = BatchStatus::Settled;
        self.settled_at = Some(timestamp);
        self.settlement_tx_hash = Some(tx_hash);

        Ok(())
    }

    /// Get batch statistics
    pub fn get_statistics(&self) -> BatchStatistics {
        let mut tx_types = HashMap::new();
        for tx in &self.transactions {
            *tx_types.entry(tx.tx_type).or_insert(0) += 1;
        }

        BatchStatistics {
            total_transactions: self.transactions.len(),
            total_fees: self.total_fees,
            unique_participants: self.get_unique_participants(),
            transaction_types: tx_types,
            batch_size_bytes: self.estimate_size(),
        }
    }

    /// Get unique participants in the batch
    fn get_unique_participants(&self) -> usize {
        let mut participants = std::collections::HashSet::new();
        for tx in &self.transactions {
            participants.insert(&tx.from);
            participants.insert(&tx.to);
        }
        participants.len()
    }

    /// Estimate batch size in bytes
    fn estimate_size(&self) -> usize {
        // Rough estimation
        self.transactions.len() * 200 // Assume ~200 bytes per transaction
    }
}

/// Batch settlement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub max_batch_size: usize,
    pub min_batch_size: usize,
    pub batch_timeout: u64,        // Seconds before auto-settlement
    pub max_pending_batches: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            min_batch_size: 10,
            batch_timeout: 300,    // 5 minutes
            max_pending_batches: 100,
        }
    }
}

/// Batch Settlement Manager
pub struct BatchSettlementManager {
    batches: Arc<RwLock<HashMap<String, SettlementBatch>>>,
    pending_queue: Arc<RwLock<VecDeque<String>>>,
    config: BatchConfig,
    current_batch_id: Arc<RwLock<Option<String>>>,
}

impl BatchSettlementManager {
    /// Create a new batch settlement manager
    pub fn new(config: BatchConfig) -> Self {
        Self {
            batches: Arc::new(RwLock::new(HashMap::new())),
            pending_queue: Arc::new(RwLock::new(VecDeque::new())),
            config,
            current_batch_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new batch
    pub fn create_batch(&self, batch_id: String) -> Result<(), BatchSettlementError> {
        let mut batches = self.batches.write().unwrap();

        if batches.len() >= self.config.max_pending_batches {
            return Err(BatchSettlementError::BatchFull);
        }

        let batch = SettlementBatch::new(batch_id.clone(), self.config.max_batch_size);
        batches.insert(batch_id.clone(), batch);

        let mut pending = self.pending_queue.write().unwrap();
        pending.push_back(batch_id.clone());

        let mut current = self.current_batch_id.write().unwrap();
        if current.is_none() {
            *current = Some(batch_id);
        }

        Ok(())
    }

    /// Add transaction to current batch
    pub fn add_transaction(&self, tx: Transaction) -> Result<String, BatchSettlementError> {
        let current_id = {
            let current = self.current_batch_id.read().unwrap();
            current.clone()
        };

        let batch_id = if let Some(id) = current_id {
            id
        } else {
            // Create new batch if none exists
            let new_id = format!("batch_{}", SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs());
            self.create_batch(new_id.clone())?;
            new_id
        };

        let mut batches = self.batches.write().unwrap();
        let batch = batches
            .get_mut(&batch_id)
            .ok_or(BatchSettlementError::BatchNotFound)?;

        match batch.add_transaction(tx) {
            Ok(_) => {
                // Check if batch is full
                if batch.transactions.len() >= self.config.max_batch_size {
                    batch.mark_ready()?;
                    // Move to next batch
                    let mut current = self.current_batch_id.write().unwrap();
                    *current = None;
                }
                Ok(batch_id)
            }
            Err(BatchSettlementError::BatchFull) => {
                // Batch is full, mark as ready and create new batch
                batch.mark_ready()?;
                let mut current = self.current_batch_id.write().unwrap();
                *current = None;
                Err(BatchSettlementError::BatchFull)
            }
            Err(e) => Err(e),
        }
    }

    /// Finalize and settle a batch
    pub fn settle_batch(&self, batch_id: &str, tx_hash: String) -> Result<(), BatchSettlementError> {
        let mut batches = self.batches.write().unwrap();

        let batch = batches
            .get_mut(batch_id)
            .ok_or(BatchSettlementError::BatchNotFound)?;

        if batch.status != BatchStatus::Ready {
            batch.mark_ready()?;
        }

        batch.settle(tx_hash)?;

        // Remove from pending queue
        let mut pending = self.pending_queue.write().unwrap();
        pending.retain(|id| id != batch_id);

        Ok(())
    }

    /// Get batch information
    pub fn get_batch(&self, batch_id: &str) -> Result<SettlementBatch, BatchSettlementError> {
        let batches = self.batches.read().unwrap();

        batches
            .get(batch_id)
            .cloned()
            .ok_or(BatchSettlementError::BatchNotFound)
    }

    /// Get all pending batches
    pub fn get_pending_batches(&self) -> Vec<SettlementBatch> {
        let batches = self.batches.read().unwrap();
        let pending = self.pending_queue.read().unwrap();

        pending
            .iter()
            .filter_map(|id| batches.get(id).cloned())
            .collect()
    }

    /// Get ready batches for settlement
    pub fn get_ready_batches(&self) -> Vec<SettlementBatch> {
        let batches = self.batches.read().unwrap();

        batches
            .values()
            .filter(|b| b.status == BatchStatus::Ready)
            .cloned()
            .collect()
    }

    /// Get overall statistics
    pub fn get_statistics(&self) -> ManagerStatistics {
        let batches = self.batches.read().unwrap();

        let total_batches = batches.len();
        let settled_batches = batches
            .values()
            .filter(|b| b.status == BatchStatus::Settled)
            .count();
        let total_transactions: usize = batches
            .values()
            .map(|b| b.transactions.len())
            .sum();
        let total_fees: u64 = batches
            .values()
            .map(|b| b.total_fees)
            .sum();

        ManagerStatistics {
            total_batches,
            settled_batches,
            pending_batches: total_batches - settled_batches,
            total_transactions,
            total_fees,
        }
    }

    /// Force settle current batch if minimum size is met
    pub fn force_settle_current(&self) -> Result<String, BatchSettlementError> {
        let current_id = {
            let current = self.current_batch_id.read().unwrap();
            current.clone().ok_or(BatchSettlementError::BatchNotFound)?
        };

        let mut batches = self.batches.write().unwrap();
        let batch = batches
            .get_mut(&current_id)
            .ok_or(BatchSettlementError::BatchNotFound)?;

        if batch.transactions.len() < self.config.min_batch_size {
            return Err(BatchSettlementError::InvalidBatchSize);
        }

        batch.mark_ready()?;

        // Clear current batch
        let mut current = self.current_batch_id.write().unwrap();
        *current = None;

        Ok(current_id)
    }
}

/// Statistics for a single batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatistics {
    pub total_transactions: usize,
    pub total_fees: u64,
    pub unique_participants: usize,
    pub transaction_types: HashMap<TransactionType, usize>,
    pub batch_size_bytes: usize,
}

/// Statistics for the batch settlement manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerStatistics {
    pub total_batches: usize,
    pub settled_batches: usize,
    pub pending_batches: usize,
    pub total_transactions: usize,
    pub total_fees: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            "tx1".to_string(),
            TransactionType::Transfer,
            "alice".to_string(),
            "bob".to_string(),
            "ETH".to_string(),
            1000,
            10,
        );

        assert_eq!(tx.id, "tx1");
        assert_eq!(tx.amount, 1000);
        assert_eq!(tx.fee, 10);
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn test_batch_creation() {
        let batch = SettlementBatch::new("batch1".to_string(), 100);

        assert_eq!(batch.id, "batch1");
        assert_eq!(batch.status, BatchStatus::Pending);
        assert_eq!(batch.transactions.len(), 0);
    }

    #[test]
    fn test_add_transaction_to_batch() {
        let mut batch = SettlementBatch::new("batch1".to_string(), 100);

        let tx = Transaction::new(
            "tx1".to_string(),
            TransactionType::Transfer,
            "alice".to_string(),
            "bob".to_string(),
            "ETH".to_string(),
            1000,
            10,
        );

        assert!(batch.add_transaction(tx).is_ok());
        assert_eq!(batch.transactions.len(), 1);
        assert_eq!(batch.status, BatchStatus::Aggregating);
    }

    #[test]
    fn test_batch_full() {
        let mut batch = SettlementBatch::new("batch1".to_string(), 2);

        let tx1 = Transaction::new(
            "tx1".to_string(),
            TransactionType::Transfer,
            "alice".to_string(),
            "bob".to_string(),
            "ETH".to_string(),
            1000,
            10,
        );

        let tx2 = Transaction::new(
            "tx2".to_string(),
            TransactionType::Transfer,
            "bob".to_string(),
            "charlie".to_string(),
            "ETH".to_string(),
            500,
            5,
        );

        let tx3 = Transaction::new(
            "tx3".to_string(),
            TransactionType::Transfer,
            "charlie".to_string(),
            "alice".to_string(),
            "ETH".to_string(),
            250,
            3,
        );

        assert!(batch.add_transaction(tx1).is_ok());
        assert!(batch.add_transaction(tx2).is_ok());
        assert_eq!(
            batch.add_transaction(tx3),
            Err(BatchSettlementError::BatchFull)
        );
    }

    #[test]
    fn test_net_balances() {
        let mut batch = SettlementBatch::new("batch1".to_string(), 100);

        let tx = Transaction::new(
            "tx1".to_string(),
            TransactionType::Transfer,
            "alice".to_string(),
            "bob".to_string(),
            "ETH".to_string(),
            1000,
            10,
        );

        batch.add_transaction(tx).unwrap();

        let alice_balance = batch.net_balances.get("alice").unwrap().get("ETH").unwrap();
        let bob_balance = batch.net_balances.get("bob").unwrap().get("ETH").unwrap();

        assert_eq!(*alice_balance, -1010); // -1000 - 10 (fee)
        assert_eq!(*bob_balance, 1000);
    }

    #[test]
    fn test_batch_settlement() {
        let mut batch = SettlementBatch::new("batch1".to_string(), 100);

        let tx = Transaction::new(
            "tx1".to_string(),
            TransactionType::Transfer,
            "alice".to_string(),
            "bob".to_string(),
            "ETH".to_string(),
            1000,
            10,
        );

        batch.add_transaction(tx).unwrap();
        assert!(batch.mark_ready().is_ok());
        assert_eq!(batch.status, BatchStatus::Ready);

        assert!(batch.settle("0xabc123".to_string()).is_ok());
        assert_eq!(batch.status, BatchStatus::Settled);
        assert!(batch.settled_at.is_some());
    }

    #[test]
    fn test_manager_operations() {
        let config = BatchConfig::default();
        let manager = BatchSettlementManager::new(config);

        // Create batch
        assert!(manager.create_batch("batch1".to_string()).is_ok());

        // Add transaction
        let tx = Transaction::new(
            "tx1".to_string(),
            TransactionType::Transfer,
            "alice".to_string(),
            "bob".to_string(),
            "ETH".to_string(),
            1000,
            10,
        );

        assert!(manager.add_transaction(tx).is_ok());

        // Get batch
        let batch = manager.get_batch("batch1").unwrap();
        assert_eq!(batch.transactions.len(), 1);

        // Get statistics
        let stats = manager.get_statistics();
        assert_eq!(stats.total_batches, 1);
        assert_eq!(stats.total_transactions, 1);
    }

    #[test]
    fn test_merkle_proof() {
        let mut batch = SettlementBatch::new("batch1".to_string(), 100);

        let tx = Transaction::new(
            "tx1".to_string(),
            TransactionType::Transfer,
            "alice".to_string(),
            "bob".to_string(),
            "ETH".to_string(),
            1000,
            10,
        );

        batch.add_transaction(tx).unwrap();
        batch.mark_ready().unwrap();

        let proof = batch.get_merkle_proof("tx1").unwrap();
        assert_eq!(proof.leaf_index, 0);
        assert!(!proof.root.is_empty());
    }

    #[test]
    fn test_batch_statistics() {
        let mut batch = SettlementBatch::new("batch1".to_string(), 100);

        let tx1 = Transaction::new(
            "tx1".to_string(),
            TransactionType::Transfer,
            "alice".to_string(),
            "bob".to_string(),
            "ETH".to_string(),
            1000,
            10,
        );

        let tx2 = Transaction::new(
            "tx2".to_string(),
            TransactionType::Trade,
            "bob".to_string(),
            "charlie".to_string(),
            "BTC".to_string(),
            500,
            5,
        );

        batch.add_transaction(tx1).unwrap();
        batch.add_transaction(tx2).unwrap();

        let stats = batch.get_statistics();
        assert_eq!(stats.total_transactions, 2);
        assert_eq!(stats.total_fees, 15);
        assert_eq!(stats.unique_participants, 3);
    }
}
