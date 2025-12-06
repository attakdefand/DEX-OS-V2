//! MPC Threshold implementation for the DEX-OS core engine
//!
//! This module implements the Priority 3 feature from DEX-OS-V2.csv:
//! "Sub Types,Bridge Subtypes,Bridge,MPC Threshold,MPC Threshold Mechanism,High"
//!
//! It provides functionality for Multi-Party Computation (MPC) threshold mechanisms
//! that enable secure cross-chain asset transfers through threshold cryptography.

use crate::types::{Quantity, TokenId, TraderId};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Represents a participant in the MPC threshold system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MpcParticipant {
    /// Unique identifier for the participant
    pub id: String,
    /// Public key share of the participant
    pub public_key_share: String,
    /// Index of the participant in the threshold scheme
    pub index: usize,
    /// Last activity timestamp
    pub last_activity: u64,
}

/// Represents an MPC threshold transaction
#[derive(Debug, Clone, PartialEq)]
pub struct MpcTransaction {
    /// Unique identifier for the transaction
    pub id: String,
    /// Source blockchain
    pub source_chain: String,
    /// Destination blockchain
    pub destination_chain: String,
    /// Sender address
    pub sender: TraderId,
    /// Receiver address
    pub receiver: TraderId,
    /// Token being transferred
    pub token_id: TokenId,
    /// Amount being transferred
    pub amount: Quantity,
    /// Timestamp when the transaction was initiated
    pub initiated_timestamp: u64,
    /// Timestamp when the transaction was completed
    pub completed_timestamp: Option<u64>,
    /// Current status of the transaction
    pub status: MpcStatus,
    /// Shares collected from participants
    pub shares: HashMap<usize, String>,
    /// Threshold required for reconstruction
    pub threshold: usize,
    /// Total number of participants
    pub total_participants: usize,
    /// Transaction hash on source chain
    pub source_tx_hash: Option<String>,
    /// Transaction hash on destination chain
    pub destination_tx_hash: Option<String>,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Status of an MPC operation
#[derive(Debug, Clone, PartialEq)]
pub enum MpcStatus {
    /// MPC is initialized but not yet active
    Initialized,
    /// MPC is collecting shares
    CollectingShares,
    /// MPC operation completed successfully
    Completed,
    /// MPC operation failed
    Failed,
    /// MPC operation timed out
    Timeout,
}

/// Configuration for the MPC threshold system
#[derive(Debug, Clone)]
pub struct MpcThresholdConfig {
    /// Threshold for reconstruction (t+1 out of n)
    pub threshold: usize,
    /// Total number of participants
    pub total_participants: usize,
    /// Timeout for MPC operations (in seconds)
    pub timeout_secs: u64,
    /// Maximum number of concurrent MPC operations
    pub max_concurrent_operations: usize,
}

impl Default for MpcThresholdConfig {
    fn default() -> Self {
        Self {
            threshold: 3,           // t+1 = 3 means 2 shares can reconstruct
            total_participants: 5,  // Total of 5 participants
            timeout_secs: 3600,     // 1 hour
            max_concurrent_operations: 1000,
        }
    }
}

/// MPC Threshold Manager
#[derive(Debug)]
pub struct MpcThresholdManager {
    /// Configuration for the MPC system
    config: MpcThresholdConfig,
    /// MPC participants
    participants: HashMap<String, MpcParticipant>,
    /// Active MPC transactions
    transactions: HashMap<String, MpcTransaction>,
    /// Completed MPC transactions
    completed_transactions: HashMap<String, MpcTransaction>,
}

/// Errors that can occur in the MPC threshold system
#[derive(Debug, Error, PartialEq)]
pub enum MpcThresholdError {
    #[error("Participant not found")]
    ParticipantNotFound,
    #[error("Participant already exists")]
    ParticipantAlreadyExists,
    #[error("MPC transaction not found")]
    TransactionNotFound,
    #[error("MPC transaction already exists")]
    TransactionAlreadyExists,
    #[error("Insufficient shares")]
    InsufficientShares,
    #[error("Invalid share")]
    InvalidShare,
    #[error("MPC operation timed out")]
    Timeout,
    #[error("Maximum concurrent operations exceeded")]
    MaxConcurrentOperationsExceeded,
    #[error("Invalid configuration")]
    InvalidConfiguration,
    #[error("Index already used")]
    IndexAlreadyUsed,
}

impl MpcThresholdManager {
    /// Create a new MPC Threshold Manager
    pub fn new(config: MpcThresholdConfig) -> Self {
        Self {
            config,
            participants: HashMap::new(),
            transactions: HashMap::new(),
            completed_transactions: HashMap::new(),
        }
    }

    /// Create a new MPC Threshold Manager with default configuration
    pub fn with_default() -> Self {
        Self::new(MpcThresholdConfig::default())
    }

    /// Add a participant to the MPC system
    pub fn add_participant(&mut self, participant: MpcParticipant) -> Result<(), MpcThresholdError> {
        if participant.id.is_empty() || participant.public_key_share.is_empty() {
            return Err(MpcThresholdError::InvalidConfiguration);
        }

        if self.participants.contains_key(&participant.id) {
            return Err(MpcThresholdError::ParticipantAlreadyExists);
        }

        // Check if index is already used
        if self
            .participants
            .values()
            .any(|p| p.index == participant.index)
        {
            return Err(MpcThresholdError::IndexAlreadyUsed);
        }

        self.participants
            .insert(participant.id.clone(), participant);
        Ok(())
    }

    /// Remove a participant from the MPC system
    pub fn remove_participant(&mut self, participant_id: &str) -> Result<(), MpcThresholdError> {
        if self.participants.remove(participant_id).is_none() {
            return Err(MpcThresholdError::ParticipantNotFound);
        }

        Ok(())
    }

    /// Get a participant by ID
    pub fn get_participant(&self, participant_id: &str) -> Option<&MpcParticipant> {
        self.participants.get(participant_id)
    }

    /// Get all participants
    pub fn get_participants(&self) -> Vec<&MpcParticipant> {
        self.participants.values().collect()
    }

    /// Get the number of participants
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Initiate an MPC threshold transaction
    pub fn initiate_mpc_transaction(
        &mut self,
        id: String,
        source_chain: String,
        destination_chain: String,
        sender: TraderId,
        receiver: TraderId,
        token_id: TokenId,
        amount: Quantity,
    ) -> Result<(), MpcThresholdError> {
        // Check if we've exceeded maximum concurrent operations
        if self.transactions.len() >= self.config.max_concurrent_operations {
            return Err(MpcThresholdError::MaxConcurrentOperationsExceeded);
        }

        // Check if transaction already exists
        if self.transactions.contains_key(&id) || self.completed_transactions.contains_key(&id) {
            return Err(MpcThresholdError::TransactionAlreadyExists);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let transaction = MpcTransaction {
            id: id.clone(),
            source_chain,
            destination_chain,
            sender,
            receiver,
            token_id: token_id.clone(),
            amount,
            initiated_timestamp: now,
            completed_timestamp: None,
            status: MpcStatus::Initialized,
            shares: HashMap::new(),
            threshold: self.config.threshold,
            total_participants: self.config.total_participants,
            source_tx_hash: None,
            destination_tx_hash: None,
            error_message: None,
        };

        self.transactions.insert(id, transaction);
        Ok(())
    }

    /// Add a share to an MPC transaction
    pub fn add_share(
        &mut self,
        transaction_id: &str,
        participant_index: usize,
        share: String,
    ) -> Result<(), MpcThresholdError> {
        let transaction = self
            .transactions
            .get_mut(transaction_id)
            .ok_or(MpcThresholdError::TransactionNotFound)?;

        // Add share
        transaction.shares.insert(participant_index, share);

        // Update status if we have enough shares
        if transaction.shares.len() >= transaction.threshold {
            transaction.status = MpcStatus::CollectingShares;
        }

        Ok(())
    }

    /// Complete an MPC transaction
    pub fn complete_mpc_transaction(
        &mut self,
        id: &str,
        source_tx_hash: Option<String>,
        destination_tx_hash: Option<String>,
    ) -> Result<(), MpcThresholdError> {
        let mut transaction = self
            .transactions
            .remove(id)
            .ok_or(MpcThresholdError::TransactionNotFound)?;

        // Update transaction details
        transaction.status = MpcStatus::Completed;
        transaction.completed_timestamp = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        transaction.source_tx_hash = source_tx_hash;
        transaction.destination_tx_hash = destination_tx_hash;

        // Move to completed transactions
        self.completed_transactions
            .insert(transaction.id.clone(), transaction);

        Ok(())
    }

    /// Fail an MPC transaction
    pub fn fail_mpc_transaction(
        &mut self,
        id: &str,
        error_message: String,
    ) -> Result<(), MpcThresholdError> {
        let mut transaction = self
            .transactions
            .remove(id)
            .ok_or(MpcThresholdError::TransactionNotFound)?;

        // Update transaction details
        transaction.status = MpcStatus::Failed;
        transaction.completed_timestamp = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        transaction.error_message = Some(error_message);

        // Move to completed transactions
        self.completed_transactions
            .insert(transaction.id.clone(), transaction);

        Ok(())
    }

    /// Get an MPC transaction by ID
    pub fn get_transaction(&self, id: &str) -> Option<&MpcTransaction> {
        self.transactions.get(id)
    }

    /// Get a completed MPC transaction by ID
    pub fn get_completed_transaction(&self, id: &str) -> Option<&MpcTransaction> {
        self.completed_transactions.get(id)
    }

    /// Get all active transactions for a trader
    pub fn get_transactions_for_trader(&self, trader_id: &TraderId) -> Vec<&MpcTransaction> {
        self.transactions
            .values()
            .filter(|tx| &tx.sender == trader_id || &tx.receiver == trader_id)
            .collect()
    }

    /// Check if a transaction has sufficient shares
    pub fn has_sufficient_shares(&self, transaction_id: &str) -> Result<bool, MpcThresholdError> {
        let transaction = self
            .transactions
            .get(transaction_id)
            .ok_or(MpcThresholdError::TransactionNotFound)?;

        Ok(transaction.shares.len() >= transaction.threshold)
    }

    /// Get the current threshold
    pub fn get_threshold(&self) -> usize {
        self.config.threshold
    }

    /// Get the total number of participants
    pub fn get_total_participants(&self) -> usize {
        self.config.total_participants
    }
}

impl Default for MpcThresholdManager {
    fn default() -> Self {
        Self::with_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpc_manager_creation() {
        let manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        assert_eq!(manager.participant_count(), 0);
    }

    #[test]
    fn test_add_participant() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        
        let participant = MpcParticipant {
            id: "participant1".to_string(),
            public_key_share: "public_key_share_1".to_string(),
            index: 1,
            last_activity: 0,
        };

        assert!(manager.add_participant(participant).is_ok());
        assert_eq!(manager.participant_count(), 1);
    }

    #[test]
    fn test_remove_participant() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        
        let participant = MpcParticipant {
            id: "participant1".to_string(),
            public_key_share: "public_key_share_1".to_string(),
            index: 1,
            last_activity: 0,
        };

        manager.add_participant(participant).unwrap();
        assert_eq!(manager.participant_count(), 1);
        
        assert!(manager.remove_participant("participant1").is_ok());
        assert_eq!(manager.participant_count(), 0);
    }

    #[test]
    fn test_initiate_mpc_transaction() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        
        let result = manager.initiate_mpc_transaction(
            "mpc1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        );

        assert!(result.is_ok());
        
        let transaction = manager.get_transaction("mpc1");
        assert!(transaction.is_some());
        let transaction = transaction.unwrap();
        assert_eq!(transaction.status, MpcStatus::Initialized);
        assert_eq!(transaction.amount, 1000);
        assert_eq!(transaction.threshold, 3);
        assert_eq!(transaction.total_participants, 5);
    }

    #[test]
    fn test_add_share() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        
        // Initiate transaction
        manager.initiate_mpc_transaction(
            "mpc1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        // Add share
        let result = manager.add_share("mpc1", 1, "share1".to_string());
        assert!(result.is_ok());
        
        let transaction = manager.get_transaction("mpc1").unwrap();
        assert_eq!(transaction.shares.len(), 1);
        assert!(transaction.shares.contains_key(&1));
    }

    #[test]
    fn test_add_share_nonexistent_transaction() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        
        // Try to add share to non-existent transaction
        let result = manager.add_share("nonexistent_mpc", 1, "share1".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MpcThresholdError::TransactionNotFound);
    }

    #[test]
    fn test_duplicate_index() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        
        // Add first participant
        let participant1 = MpcParticipant {
            id: "participant1".to_string(),
            public_key_share: "public_key_share_1".to_string(),
            index: 1,
            last_activity: 0,
        };
        manager.add_participant(participant1).unwrap();
        
        // Try to add second participant with same index
        let participant2 = MpcParticipant {
            id: "participant2".to_string(),
            public_key_share: "public_key_share_2".to_string(),
            index: 1, // Same index as participant1
            last_activity: 0,
        };
        
        let result = manager.add_participant(participant2);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MpcThresholdError::IndexAlreadyUsed);
    }

    #[test]
    fn test_complete_transaction() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        
        // Initiate transaction
        manager.initiate_mpc_transaction(
            "mpc1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        // Complete transaction
        assert!(manager.complete_mpc_transaction(
            "mpc1",
            Some("0xsource_hash".to_string()),
            Some("0xdest_hash".to_string())
        ).is_ok());
        
        // Transaction should now be in completed transactions
        assert!(manager.get_transaction("mpc1").is_none());
        assert!(manager.get_completed_transaction("mpc1").is_some());
        
        let completed_tx = manager.get_completed_transaction("mpc1").unwrap();
        assert_eq!(completed_tx.status, MpcStatus::Completed);
        assert_eq!(completed_tx.source_tx_hash, Some("0xsource_hash".to_string()));
        assert_eq!(completed_tx.destination_tx_hash, Some("0xdest_hash".to_string()));
    }

    #[test]
    fn test_fail_transaction() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        
        // Initiate transaction
        manager.initiate_mpc_transaction(
            "mpc1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        // Fail transaction
        assert!(manager.fail_mpc_transaction(
            "mpc1",
            "Network error occurred".to_string()
        ).is_ok());
        
        // Transaction should now be in completed transactions with failed status
        assert!(manager.get_transaction("mpc1").is_none());
        assert!(manager.get_completed_transaction("mpc1").is_some());
        
        let completed_tx = manager.get_completed_transaction("mpc1").unwrap();
        assert_eq!(completed_tx.status, MpcStatus::Failed);
        assert_eq!(completed_tx.error_message, Some("Network error occurred".to_string()));
    }

    #[test]
    fn test_multiple_participants_threshold() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig {
            threshold: 3,
            total_participants: 5,
            timeout_secs: 3600,
            max_concurrent_operations: 1000,
        });
        
        // Add multiple participants
        let participant1 = MpcParticipant {
            id: "participant1".to_string(),
            public_key_share: "public_key_share_1".to_string(),
            index: 1,
            last_activity: 0,
        };
        
        let participant2 = MpcParticipant {
            id: "participant2".to_string(),
            public_key_share: "public_key_share_2".to_string(),
            index: 2,
            last_activity: 0,
        };
        
        let participant3 = MpcParticipant {
            id: "participant3".to_string(),
            public_key_share: "public_key_share_3".to_string(),
            index: 3,
            last_activity: 0,
        };
        
        manager.add_participant(participant1).unwrap();
        manager.add_participant(participant2).unwrap();
        manager.add_participant(participant3).unwrap();
        
        assert_eq!(manager.participant_count(), 3);
        assert_eq!(manager.get_threshold(), 3);
        assert_eq!(manager.get_total_participants(), 5);
        
        // Initiate transaction
        manager.initiate_mpc_transaction(
            "mpc1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        // Add shares from participants
        manager.add_share("mpc1", 1, "share1".to_string()).unwrap();
        manager.add_share("mpc1", 2, "share2".to_string()).unwrap();
        manager.add_share("mpc1", 3, "share3".to_string()).unwrap();
        
        // Check if we have sufficient shares
        assert!(manager.has_sufficient_shares("mpc1").unwrap());
        
        let transaction = manager.get_transaction("mpc1").unwrap();
        assert_eq!(transaction.shares.len(), 3);
        assert_eq!(transaction.status, MpcStatus::CollectingShares);
    }

    #[test]
    fn test_get_transactions_for_trader() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig::default());
        
        // Initiate transactions
        manager.initiate_mpc_transaction(
            "mpc1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        manager.initiate_mpc_transaction(
            "mpc2".to_string(),
            "polygon".to_string(),
            "ethereum".to_string(),
            "sender2".to_string(),
            "sender1".to_string(), // This transaction involves sender1 as receiver
            "MATIC".to_string(),
            2000,
        ).unwrap();
        
        // Get transactions for sender1 (should get both as sender and receiver)
        let transactions = manager.get_transactions_for_trader(&"sender1".to_string());
        assert_eq!(transactions.len(), 2);
        
        // Get transactions for sender2 (should get only one)
        let transactions = manager.get_transactions_for_trader(&"sender2".to_string());
        assert_eq!(transactions.len(), 1);
    }

    #[test]
    fn test_concurrent_operations_limit() {
        let mut manager = MpcThresholdManager::new(MpcThresholdConfig {
            threshold: 2,
            total_participants: 5,
            timeout_secs: 3600,
            max_concurrent_operations: 2,
        });
        
        // Initiate maximum allowed transactions
        manager.initiate_mpc_transaction(
            "mpc1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        manager.initiate_mpc_transaction(
            "mpc2".to_string(),
            "polygon".to_string(),
            "ethereum".to_string(),
            "sender2".to_string(),
            "receiver2".to_string(),
            "MATIC".to_string(),
            2000,
        ).unwrap();
        
        // Try to initiate one more - should fail
        let result = manager.initiate_mpc_transaction(
            "mpc3".to_string(),
            "bsc".to_string(),
            "ethereum".to_string(),
            "sender3".to_string(),
            "receiver3".to_string(),
            "BNB".to_string(),
            3000,
        );
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MpcThresholdError::MaxConcurrentOperationsExceeded);
    }
}