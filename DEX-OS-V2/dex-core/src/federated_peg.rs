//! Federated Peg implementation for the DEX-OS core engine
//!
//! This module implements the Priority 3 feature from DEX-OS-V2.csv:
//! "Sub Types,Bridge Subtypes,Bridge,Federated Peg,Federated Peg Mechanism,High"
//!
//! It provides functionality for federated peg mechanisms that enable cross-chain
//! asset transfers through a federation of validators/signers.

use crate::types::{Quantity, TokenId, TraderId};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Represents a signer in the federated peg system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signer {
    /// Unique identifier for the signer
    pub id: String,
    /// Public key of the signer
    pub public_key: String,
    /// Weight/power of the signer in the federation
    pub weight: u32,
    /// Last activity timestamp
    pub last_activity: u64,
}

/// Represents a peg transaction in the federated system
#[derive(Debug, Clone, PartialEq)]
pub struct PegTransaction {
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
    pub status: PegStatus,
    /// Signatures collected from federation members
    pub signatures: HashMap<String, String>,
    /// Required signatures for completion
    pub required_signatures: usize,
    /// Transaction hash on source chain
    pub source_tx_hash: Option<String>,
    /// Transaction hash on destination chain
    pub destination_tx_hash: Option<String>,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Status of a peg operation
#[derive(Debug, Clone, PartialEq)]
pub enum PegStatus {
    /// Peg is initialized but not yet active
    Initialized,
    /// Peg is waiting for sufficient signatures
    WaitingForSignatures,
    /// Peg operation completed successfully
    Completed,
    /// Peg operation failed
    Failed,
    /// Peg operation timed out
    Timeout,
}

/// Configuration for the federated peg system
#[derive(Debug, Clone)]
pub struct FederatedPegConfig {
    /// Minimum number of signatures required
    pub min_signatures: usize,
    /// Timeout for peg operations (in seconds)
    pub timeout_secs: u64,
    /// Maximum number of concurrent peg operations
    pub max_concurrent_operations: usize,
}

impl Default for FederatedPegConfig {
    fn default() -> Self {
        Self {
            min_signatures: 3,
            timeout_secs: 3600, // 1 hour
            max_concurrent_operations: 1000,
        }
    }
}

/// Federated Peg Manager
#[derive(Debug)]
pub struct FederatedPegManager {
    /// Configuration for the peg system
    config: FederatedPegConfig,
    /// Federation signers
    signers: HashMap<String, Signer>,
    /// Active peg transactions
    transactions: HashMap<String, PegTransaction>,
    /// Completed peg transactions
    completed_transactions: HashMap<String, PegTransaction>,
    /// Total weight of all signers
    total_weight: u32,
    /// Threshold weight required for operations
    threshold_weight: u32,
}

/// Errors that can occur in the federated peg system
#[derive(Debug, Error, PartialEq)]
pub enum FederatedPegError {
    #[error("Signer not found")]
    SignerNotFound,
    #[error("Signer already exists")]
    SignerAlreadyExists,
    #[error("Peg transaction not found")]
    TransactionNotFound,
    #[error("Peg transaction already exists")]
    TransactionAlreadyExists,
    #[error("Insufficient signatures")]
    InsufficientSignatures,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Peg operation timed out")]
    Timeout,
    #[error("Maximum concurrent operations exceeded")]
    MaxConcurrentOperationsExceeded,
    #[error("Invalid configuration")]
    InvalidConfiguration,
}

impl FederatedPegManager {
    /// Create a new Federated Peg Manager
    pub fn new(config: FederatedPegConfig) -> Self {
        Self {
            config,
            signers: HashMap::new(),
            transactions: HashMap::new(),
            completed_transactions: HashMap::new(),
            total_weight: 0,
            threshold_weight: 0,
        }
    }

    /// Create a new Federated Peg Manager with default configuration
    pub fn with_default() -> Self {
        Self::new(FederatedPegConfig::default())
    }

    /// Add a signer to the federation
    pub fn add_signer(&mut self, signer: Signer) -> Result<(), FederatedPegError> {
        if signer.id.is_empty() || signer.public_key.is_empty() {
            return Err(FederatedPegError::InvalidConfiguration);
        }

        if self.signers.contains_key(&signer.id) {
            return Err(FederatedPegError::SignerAlreadyExists);
        }

        self.total_weight += signer.weight;
        self.signers.insert(signer.id.clone(), signer);
        
        // Recalculate threshold (2/3 of total weight)
        self.threshold_weight = (self.total_weight * 2 + 2) / 3;
        
        Ok(())
    }

    /// Remove a signer from the federation
    pub fn remove_signer(&mut self, signer_id: &str) -> Result<(), FederatedPegError> {
        let signer = self
            .signers
            .remove(signer_id)
            .ok_or(FederatedPegError::SignerNotFound)?;

        self.total_weight -= signer.weight;
        
        // Recalculate threshold (2/3 of total weight)
        self.threshold_weight = (self.total_weight * 2 + 2) / 3;
        
        Ok(())
    }

    /// Get a signer by ID
    pub fn get_signer(&self, signer_id: &str) -> Option<&Signer> {
        self.signers.get(signer_id)
    }

    /// Get all signers
    pub fn get_signers(&self) -> Vec<&Signer> {
        self.signers.values().collect()
    }

    /// Get the number of signers
    pub fn signer_count(&self) -> usize {
        self.signers.len()
    }

    /// Initiate a federated peg transaction
    pub fn initiate_peg_transaction(
        &mut self,
        id: String,
        source_chain: String,
        destination_chain: String,
        sender: TraderId,
        receiver: TraderId,
        token_id: TokenId,
        amount: Quantity,
    ) -> Result<(), FederatedPegError> {
        // Check if we've exceeded maximum concurrent operations
        if self.transactions.len() >= self.config.max_concurrent_operations {
            return Err(FederatedPegError::MaxConcurrentOperationsExceeded);
        }

        // Check if transaction already exists
        if self.transactions.contains_key(&id) || self.completed_transactions.contains_key(&id) {
            return Err(FederatedPegError::TransactionAlreadyExists);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let transaction = PegTransaction {
            id: id.clone(),
            source_chain,
            destination_chain,
            sender,
            receiver,
            token_id: token_id.clone(),
            amount,
            initiated_timestamp: now,
            completed_timestamp: None,
            status: PegStatus::Initialized,
            signatures: HashMap::new(),
            required_signatures: self.config.min_signatures,
            source_tx_hash: None,
            destination_tx_hash: None,
            error_message: None,
        };

        self.transactions.insert(id, transaction);
        Ok(())
    }

    /// Add a signature to a peg transaction
    pub fn add_signature(
        &mut self,
        transaction_id: &str,
        signer_id: &str,
        signature: String,
    ) -> Result<(), FederatedPegError> {
        let transaction = self
            .transactions
            .get_mut(transaction_id)
            .ok_or(FederatedPegError::TransactionNotFound)?;

        // Check if signer exists
        if !self.signers.contains_key(signer_id) {
            return Err(FederatedPegError::SignerNotFound);
        }

        // Add signature
        transaction.signatures.insert(signer_id.to_string(), signature);

        // Check if we have enough signatures
        if transaction.signatures.len() >= transaction.required_signatures {
            transaction.status = PegStatus::WaitingForSignatures;
        }

        Ok(())
    }

    /// Complete a peg transaction
    pub fn complete_peg_transaction(
        &mut self,
        id: &str,
        source_tx_hash: Option<String>,
        destination_tx_hash: Option<String>,
    ) -> Result<(), FederatedPegError> {
        let mut transaction = self
            .transactions
            .remove(id)
            .ok_or(FederatedPegError::TransactionNotFound)?;

        // Update transaction details
        transaction.status = PegStatus::Completed;
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

    /// Fail a peg transaction
    pub fn fail_peg_transaction(
        &mut self,
        id: &str,
        error_message: String,
    ) -> Result<(), FederatedPegError> {
        let mut transaction = self
            .transactions
            .remove(id)
            .ok_or(FederatedPegError::TransactionNotFound)?;

        // Update transaction details
        transaction.status = PegStatus::Failed;
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

    /// Get a peg transaction by ID
    pub fn get_transaction(&self, id: &str) -> Option<&PegTransaction> {
        self.transactions.get(id)
    }

    /// Get a completed peg transaction by ID
    pub fn get_completed_transaction(&self, id: &str) -> Option<&PegTransaction> {
        self.completed_transactions.get(id)
    }

    /// Get all active transactions for a trader
    pub fn get_transactions_for_trader(&self, trader_id: &TraderId) -> Vec<&PegTransaction> {
        self.transactions
            .values()
            .filter(|tx| &tx.sender == trader_id || &tx.receiver == trader_id)
            .collect()
    }

    /// Check if a transaction has sufficient signatures
    pub fn has_sufficient_signatures(&self, transaction_id: &str) -> Result<bool, FederatedPegError> {
        let transaction = self
            .transactions
            .get(transaction_id)
            .ok_or(FederatedPegError::TransactionNotFound)?;

        Ok(transaction.signatures.len() >= transaction.required_signatures)
    }

    /// Get the current threshold weight required for operations
    pub fn get_threshold_weight(&self) -> u32 {
        self.threshold_weight
    }

    /// Get the total weight of all signers
    pub fn get_total_weight(&self) -> u32 {
        self.total_weight
    }
}

impl Default for FederatedPegManager {
    fn default() -> Self {
        Self::with_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peg_manager_creation() {
        let manager = FederatedPegManager::new(FederatedPegConfig::default());
        assert_eq!(manager.signer_count(), 0);
        assert_eq!(manager.get_total_weight(), 0);
        assert_eq!(manager.get_threshold_weight(), 0);
    }

    #[test]
    fn test_add_signer() {
        let mut manager = FederatedPegManager::new(FederatedPegConfig::default());
        
        let signer = Signer {
            id: "signer1".to_string(),
            public_key: "public_key_1".to_string(),
            weight: 10,
            last_activity: 0,
        };

        assert!(manager.add_signer(signer).is_ok());
        assert_eq!(manager.signer_count(), 1);
        assert_eq!(manager.get_total_weight(), 10);
        // Threshold should be 2/3 of total weight: (10 * 2 + 2) / 3 = 7
        assert_eq!(manager.get_threshold_weight(), 7);
    }

    #[test]
    fn test_remove_signer() {
        let mut manager = FederatedPegManager::new(FederatedPegConfig::default());
        
        let signer = Signer {
            id: "signer1".to_string(),
            public_key: "public_key_1".to_string(),
            weight: 10,
            last_activity: 0,
        };

        manager.add_signer(signer).unwrap();
        assert_eq!(manager.signer_count(), 1);
        
        assert!(manager.remove_signer("signer1").is_ok());
        assert_eq!(manager.signer_count(), 0);
        assert_eq!(manager.get_total_weight(), 0);
        assert_eq!(manager.get_threshold_weight(), 0);
    }

    #[test]
    fn test_initiate_peg_transaction() {
        let mut manager = FederatedPegManager::new(FederatedPegConfig::default());
        
        let result = manager.initiate_peg_transaction(
            "peg1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        );

        assert!(result.is_ok());
        
        let transaction = manager.get_transaction("peg1");
        assert!(transaction.is_some());
        let transaction = transaction.unwrap();
        assert_eq!(transaction.status, PegStatus::Initialized);
        assert_eq!(transaction.amount, 1000);
    }

    #[test]
    fn test_add_signature() {
        let mut manager = FederatedPegManager::new(FederatedPegConfig::default());
        
        // Add signer
        let signer = Signer {
            id: "signer1".to_string(),
            public_key: "public_key_1".to_string(),
            weight: 10,
            last_activity: 0,
        };
        manager.add_signer(signer).unwrap();
        
        // Initiate transaction
        manager.initiate_peg_transaction(
            "peg1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        // Add signature
        let result = manager.add_signature("peg1", "signer1", "signature1".to_string());
        assert!(result.is_ok());
        
        let transaction = manager.get_transaction("peg1").unwrap();
        assert_eq!(transaction.signatures.len(), 1);
        assert!(transaction.signatures.contains_key("signer1"));
    }

    #[test]
    fn test_add_signature_nonexistent_signer() {
        let mut manager = FederatedPegManager::new(FederatedPegConfig::default());
        
        // Initiate transaction
        manager.initiate_peg_transaction(
            "peg1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        // Try to add signature from non-existent signer
        let result = manager.add_signature("peg1", "nonexistent_signer", "signature1".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), FederatedPegError::SignerNotFound);
    }

    #[test]
    fn test_add_signature_nonexistent_transaction() {
        let mut manager = FederatedPegManager::new(FederatedPegConfig::default());
        
        // Add signer
        let signer = Signer {
            id: "signer1".to_string(),
            public_key: "public_key_1".to_string(),
            weight: 10,
            last_activity: 0,
        };
        manager.add_signer(signer).unwrap();
        
        // Try to add signature to non-existent transaction
        let result = manager.add_signature("nonexistent_peg", "signer1", "signature1".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), FederatedPegError::TransactionNotFound);
    }

    #[test]
    fn test_complete_transaction() {
        let mut manager = FederatedPegManager::new(FederatedPegConfig::default());
        
        // Initiate transaction
        manager.initiate_peg_transaction(
            "peg1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        // Complete transaction
        assert!(manager.complete_peg_transaction(
            "peg1",
            Some("0xsource_hash".to_string()),
            Some("0xdest_hash".to_string())
        ).is_ok());
        
        // Transaction should now be in completed transactions
        assert!(manager.get_transaction("peg1").is_none());
        assert!(manager.get_completed_transaction("peg1").is_some());
        
        let completed_tx = manager.get_completed_transaction("peg1").unwrap();
        assert_eq!(completed_tx.status, PegStatus::Completed);
        assert_eq!(completed_tx.source_tx_hash, Some("0xsource_hash".to_string()));
        assert_eq!(completed_tx.destination_tx_hash, Some("0xdest_hash".to_string()));
    }

    #[test]
    fn test_fail_transaction() {
        let mut manager = FederatedPegManager::new(FederatedPegConfig::default());
        
        // Initiate transaction
        manager.initiate_peg_transaction(
            "peg1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        // Fail transaction
        assert!(manager.fail_peg_transaction(
            "peg1",
            "Network error occurred".to_string()
        ).is_ok());
        
        // Transaction should now be in completed transactions with failed status
        assert!(manager.get_transaction("peg1").is_none());
        assert!(manager.get_completed_transaction("peg1").is_some());
        
        let completed_tx = manager.get_completed_transaction("peg1").unwrap();
        assert_eq!(completed_tx.status, PegStatus::Failed);
        assert_eq!(completed_tx.error_message, Some("Network error occurred".to_string()));
    }

    #[test]
    fn test_multiple_signers_threshold() {
        let mut manager = FederatedPegManager::new(FederatedPegConfig {
            min_signatures: 2,
            timeout_secs: 3600,
            max_concurrent_operations: 1000,
        });
        
        // Add multiple signers
        let signer1 = Signer {
            id: "signer1".to_string(),
            public_key: "public_key_1".to_string(),
            weight: 5,
            last_activity: 0,
        };
        
        let signer2 = Signer {
            id: "signer2".to_string(),
            public_key: "public_key_2".to_string(),
            weight: 5,
            last_activity: 0,
        };
        
        let signer3 = Signer {
            id: "signer3".to_string(),
            public_key: "public_key_3".to_string(),
            weight: 5,
            last_activity: 0,
        };
        
        manager.add_signer(signer1).unwrap();
        manager.add_signer(signer2).unwrap();
        manager.add_signer(signer3).unwrap();
        
        assert_eq!(manager.signer_count(), 3);
        assert_eq!(manager.get_total_weight(), 15);
        // Threshold should be 2/3 of total weight: (15 * 2 + 2) / 3 = 10
        assert_eq!(manager.get_threshold_weight(), 10);
        
        // Initiate transaction
        manager.initiate_peg_transaction(
            "peg1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        // Add signatures from two signers
        manager.add_signature("peg1", "signer1", "signature1".to_string()).unwrap();
        manager.add_signature("peg1", "signer2", "signature2".to_string()).unwrap();
        
        // Check if we have sufficient signatures
        assert!(manager.has_sufficient_signatures("peg1").unwrap());
        
        let transaction = manager.get_transaction("peg1").unwrap();
        assert_eq!(transaction.signatures.len(), 2);
        assert_eq!(transaction.status, PegStatus::WaitingForSignatures);
    }

    #[test]
    fn test_get_transactions_for_trader() {
        let mut manager = FederatedPegManager::new(FederatedPegConfig::default());
        
        // Initiate transactions
        manager.initiate_peg_transaction(
            "peg1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        manager.initiate_peg_transaction(
            "peg2".to_string(),
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
        let mut manager = FederatedPegManager::new(FederatedPegConfig {
            min_signatures: 1,
            timeout_secs: 3600,
            max_concurrent_operations: 2,
        });
        
        // Initiate maximum allowed transactions
        manager.initiate_peg_transaction(
            "peg1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        manager.initiate_peg_transaction(
            "peg2".to_string(),
            "polygon".to_string(),
            "ethereum".to_string(),
            "sender2".to_string(),
            "receiver2".to_string(),
            "MATIC".to_string(),
            2000,
        ).unwrap();
        
        // Try to initiate one more - should fail
        let result = manager.initiate_peg_transaction(
            "peg3".to_string(),
            "bsc".to_string(),
            "ethereum".to_string(),
            "sender3".to_string(),
            "receiver3".to_string(),
            "BNB".to_string(),
            3000,
        );
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), FederatedPegError::MaxConcurrentOperationsExceeded);
    }
}