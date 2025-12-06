//! Multi-signature wallet implementation for the DEX-OS core engine
//!
//! This module implements the Priority 2 feature from DEX-OS-V1.csv:
//! "Core Trading,Bridge,Bridge,Multi-signature Wallets,Asset Custody,High"
//!
//! It provides functionality for secure asset custody using multi-signature wallets,
//! which require multiple parties to sign transactions before they can be executed.

use crate::types::{Quantity, TokenId, TraderId};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Represents a participant in a multi-signature wallet
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WalletParticipant {
    /// Unique identifier for the participant
    pub id: TraderId,
    /// Public key of the participant
    pub public_key: String,
}

/// Represents a transaction that requires multi-signature approval
#[derive(Debug, Clone, PartialEq)]
pub struct MultiSigTransaction {
    /// Unique identifier for the transaction
    pub id: u64,
    /// Source wallet address
    pub from_wallet: String,
    /// Destination address
    pub to_address: String,
    /// Token being transferred
    pub token_id: TokenId,
    /// Amount being transferred
    pub amount: Quantity,
    /// Required number of signatures
    pub required_signatures: usize,
    /// Participants who have signed
    pub signatures: HashSet<TraderId>,
    /// Timestamp when the transaction was created
    pub created_timestamp: u64,
    /// Timestamp when the transaction was executed (if executed)
    pub executed_timestamp: Option<u64>,
}

impl MultiSigTransaction {
    /// Check if the transaction has enough signatures to be executed
    pub fn is_ready_for_execution(&self) -> bool {
        self.signatures.len() >= self.required_signatures && self.executed_timestamp.is_none()
    }

    /// Check if the transaction has been executed
    pub fn is_executed(&self) -> bool {
        self.executed_timestamp.is_some()
    }

    /// Add a signature to the transaction
    pub fn add_signature(&mut self, participant_id: TraderId) {
        self.signatures.insert(participant_id);
    }

    /// Check if a participant has signed the transaction
    pub fn has_signature_from(&self, participant_id: &TraderId) -> bool {
        self.signatures.contains(participant_id)
    }
}

/// Multi-signature wallet for secure asset custody
#[derive(Debug, Clone)]
pub struct MultiSigWallet {
    /// Unique identifier for the wallet
    pub wallet_id: String,
    /// Participants in the wallet
    pub participants: HashSet<WalletParticipant>,
    /// Required number of signatures for transactions
    pub required_signatures: usize,
    /// Assets held in the wallet
    pub assets: HashMap<TokenId, Quantity>,
    /// Pending transactions
    pub pending_transactions: HashMap<u64, MultiSigTransaction>,
    /// Executed transactions
    pub executed_transactions: HashMap<u64, MultiSigTransaction>,
    /// Transaction counter for generating unique IDs
    transaction_counter: u64,
}

impl MultiSigWallet {
    /// Create a new multi-signature wallet
    pub fn new(
        wallet_id: String,
        participants: Vec<WalletParticipant>,
        required_signatures: usize,
    ) -> Result<Self, MultiSigError> {
        // Validate inputs
        if participants.is_empty() {
            return Err(MultiSigError::NoParticipants);
        }

        if required_signatures == 0 {
            return Err(MultiSigError::InvalidRequiredSignatures);
        }

        if required_signatures > participants.len() {
            return Err(MultiSigError::InvalidRequiredSignatures);
        }

        let participant_set: HashSet<WalletParticipant> = participants.into_iter().collect();

        Ok(Self {
            wallet_id,
            participants: participant_set,
            required_signatures,
            assets: HashMap::new(),
            pending_transactions: HashMap::new(),
            executed_transactions: HashMap::new(),
            transaction_counter: 0,
        })
    }

    /// Get the number of participants in the wallet
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Check if a participant is part of the wallet
    pub fn is_participant(&self, participant_id: &TraderId) -> bool {
        self.participants.iter().any(|p| &p.id == participant_id)
    }

    /// Add assets to the wallet
    pub fn deposit(&mut self, token_id: TokenId, amount: Quantity) {
        let current_amount = self.assets.get(&token_id).copied().unwrap_or(0);
        self.assets.insert(token_id, current_amount + amount);
    }

    /// Get the balance of a specific token in the wallet
    pub fn get_balance(&self, token_id: &TokenId) -> Quantity {
        self.assets.get(token_id).copied().unwrap_or(0)
    }

    /// Get all asset balances
    pub fn get_all_balances(&self) -> &HashMap<TokenId, Quantity> {
        &self.assets
    }

    /// Create a new transaction
    pub fn create_transaction(
        &mut self,
        to_address: String,
        token_id: TokenId,
        amount: Quantity,
    ) -> Result<u64, MultiSigError> {
        // Check if the wallet has sufficient balance
        let balance = self.get_balance(&token_id);
        if amount > balance {
            return Err(MultiSigError::InsufficientFunds);
        }

        // Deduct the amount from the wallet's balance (it's now locked for this transaction)
        self.assets.insert(token_id.clone(), balance - amount);

        // Generate a new transaction ID
        self.transaction_counter += 1;
        let transaction_id = self.transaction_counter;

        // Create the transaction
        let transaction = MultiSigTransaction {
            id: transaction_id,
            from_wallet: self.wallet_id.clone(),
            to_address,
            token_id,
            amount,
            required_signatures: self.required_signatures,
            signatures: HashSet::new(),
            created_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            executed_timestamp: None,
        };

        // Add to pending transactions
        self.pending_transactions
            .insert(transaction_id, transaction);

        Ok(transaction_id)
    }

    /// Sign a transaction
    pub fn sign_transaction(
        &mut self,
        transaction_id: u64,
        participant_id: TraderId,
    ) -> Result<(), MultiSigError> {
        // Check if the participant is part of the wallet
        if !self.is_participant(&participant_id) {
            return Err(MultiSigError::NotParticipant);
        }

        // Check if the transaction exists and is pending
        let transaction = self
            .pending_transactions
            .get_mut(&transaction_id)
            .ok_or(MultiSigError::TransactionNotFound)?;

        // Check if the transaction has already been executed
        if transaction.is_executed() {
            return Err(MultiSigError::TransactionAlreadyExecuted);
        }

        // Add the signature
        transaction.add_signature(participant_id);

        Ok(())
    }

    /// Execute a transaction if it has enough signatures
    pub fn execute_transaction(&mut self, transaction_id: u64) -> Result<(), MultiSigError> {
        // Check if the transaction exists and is pending
        let mut transaction = self
            .pending_transactions
            .remove(&transaction_id)
            .ok_or(MultiSigError::TransactionNotFound)?
            .clone();

        // Check if the transaction has already been executed
        if transaction.is_executed() {
            return Err(MultiSigError::TransactionAlreadyExecuted);
        }

        // Check if the transaction has enough signatures
        if !transaction.is_ready_for_execution() {
            // Put it back as pending
            self.pending_transactions
                .insert(transaction_id, transaction);
            return Err(MultiSigError::InsufficientSignatures);
        }

        // Mark the transaction as executed
        transaction.executed_timestamp = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        // Move to executed transactions
        self.executed_transactions
            .insert(transaction_id, transaction);

        Ok(())
    }

    /// Get a pending transaction
    pub fn get_pending_transaction(&self, transaction_id: u64) -> Option<&MultiSigTransaction> {
        self.pending_transactions.get(&transaction_id)
    }

    /// Get an executed transaction
    pub fn get_executed_transaction(&self, transaction_id: u64) -> Option<&MultiSigTransaction> {
        self.executed_transactions.get(&transaction_id)
    }

    /// Get all pending transactions
    pub fn get_pending_transactions(&self) -> Vec<&MultiSigTransaction> {
        self.pending_transactions.values().collect()
    }

    /// Get all executed transactions
    pub fn get_executed_transactions(&self) -> Vec<&MultiSigTransaction> {
        self.executed_transactions.values().collect()
    }

    /// Get the number of pending transactions
    pub fn pending_transaction_count(&self) -> usize {
        self.pending_transactions.len()
    }

    /// Get the number of executed transactions
    pub fn executed_transaction_count(&self) -> usize {
        self.executed_transactions.len()
    }

    /// Cancel a pending transaction (return funds to wallet)
    pub fn cancel_transaction(&mut self, transaction_id: u64) -> Result<(), MultiSigError> {
        // Check if the transaction exists and is pending
        let transaction = self
            .pending_transactions
            .remove(&transaction_id)
            .ok_or(MultiSigError::TransactionNotFound)?;

        // Check if the transaction has already been executed
        if transaction.is_executed() {
            return Err(MultiSigError::TransactionAlreadyExecuted);
        }

        // Return the funds to the wallet
        let current_balance = self.get_balance(&transaction.token_id);
        self.assets.insert(
            transaction.token_id.clone(),
            current_balance + transaction.amount,
        );

        // Note: In a real implementation, you might want to move this to a separate "cancelled" list
        // For now, we'll just drop it

        Ok(())
    }
}

/// Manages multiple multi-signature wallets
#[derive(Debug, Clone)]
pub struct MultiSigWalletManager {
    /// Wallets managed by this manager
    wallets: HashMap<String, MultiSigWallet>,
}

impl MultiSigWalletManager {
    /// Create a new multi-signature wallet manager
    pub fn new() -> Self {
        Self {
            wallets: HashMap::new(),
        }
    }

    /// Create a new multi-signature wallet
    pub fn create_wallet(
        &mut self,
        wallet_id: String,
        participants: Vec<WalletParticipant>,
        required_signatures: usize,
    ) -> Result<(), MultiSigError> {
        let wallet = MultiSigWallet::new(wallet_id.clone(), participants, required_signatures)?;
        self.wallets.insert(wallet_id, wallet);
        Ok(())
    }

    /// Get a wallet by ID
    pub fn get_wallet(&self, wallet_id: &str) -> Option<&MultiSigWallet> {
        self.wallets.get(wallet_id)
    }

    /// Get a mutable reference to a wallet by ID
    pub fn get_wallet_mut(&mut self, wallet_id: &str) -> Option<&mut MultiSigWallet> {
        self.wallets.get_mut(wallet_id)
    }

    /// Get all wallets
    pub fn get_all_wallets(&self) -> Vec<&MultiSigWallet> {
        self.wallets.values().collect()
    }

    /// Get the number of wallets
    pub fn wallet_count(&self) -> usize {
        self.wallets.len()
    }

    /// Check if a wallet exists
    pub fn has_wallet(&self, wallet_id: &str) -> bool {
        self.wallets.contains_key(wallet_id)
    }

    /// Remove a wallet
    pub fn remove_wallet(&mut self, wallet_id: &str) -> bool {
        self.wallets.remove(wallet_id).is_some()
    }
}

/// Errors that can occur during multi-signature wallet operations
#[derive(Debug, Error)]
pub enum MultiSigError {
    #[error("No participants in the wallet")]
    NoParticipants,
    #[error("Invalid required signatures count")]
    InvalidRequiredSignatures,
    #[error("Insufficient funds in the wallet")]
    InsufficientFunds,
    #[error("Participant is not part of the wallet")]
    NotParticipant,
    #[error("Transaction not found")]
    TransactionNotFound,
    #[error("Transaction has already been executed")]
    TransactionAlreadyExecuted,
    #[error("Insufficient signatures to execute transaction")]
    InsufficientSignatures,
    #[error("Invalid transaction data")]
    InvalidTransactionData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multisig_wallet_creation() {
        let participants = vec![
            WalletParticipant {
                id: "participant1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            WalletParticipant {
                id: "participant2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ];

        let wallet = MultiSigWallet::new("wallet1".to_string(), participants, 2).unwrap();
        assert_eq!(wallet.wallet_id, "wallet1");
        assert_eq!(wallet.participant_count(), 2);
        assert_eq!(wallet.required_signatures, 2);
        assert_eq!(wallet.pending_transaction_count(), 0);
        assert_eq!(wallet.executed_transaction_count(), 0);
    }

    #[test]
    fn test_invalid_wallet_creation() {
        // Test with no participants
        let result = MultiSigWallet::new("wallet1".to_string(), vec![], 1);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MultiSigError::NoParticipants));

        // Test with required signatures greater than participants
        let participants = vec![WalletParticipant {
            id: "participant1".to_string(),
            public_key: "pubkey1".to_string(),
        }];

        let result = MultiSigWallet::new("wallet1".to_string(), participants, 2);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MultiSigError::InvalidRequiredSignatures
        ));

        // Test with zero required signatures
        let participants = vec![WalletParticipant {
            id: "participant1".to_string(),
            public_key: "pubkey1".to_string(),
        }];

        let result = MultiSigWallet::new("wallet1".to_string(), participants, 0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MultiSigError::InvalidRequiredSignatures
        ));
    }

    #[test]
    fn test_asset_management() {
        let participants = vec![
            WalletParticipant {
                id: "participant1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            WalletParticipant {
                id: "participant2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ];

        let mut wallet = MultiSigWallet::new("wallet1".to_string(), participants, 2).unwrap();

        // Test deposit
        wallet.deposit("BTC".to_string(), 1000);
        assert_eq!(wallet.get_balance(&"BTC".to_string()), 1000);

        // Test get all balances
        let balances = wallet.get_all_balances();
        assert_eq!(balances.len(), 1);
        assert_eq!(balances.get(&"BTC".to_string()), Some(&1000));

        // Test deposit more of the same token
        wallet.deposit("BTC".to_string(), 500);
        assert_eq!(wallet.get_balance(&"BTC".to_string()), 1500);

        // Test deposit different token
        wallet.deposit("ETH".to_string(), 100);
        assert_eq!(wallet.get_balance(&"BTC".to_string()), 1500);
        assert_eq!(wallet.get_balance(&"ETH".to_string()), 100);
    }

    #[test]
    fn test_transaction_creation() {
        let participants = vec![
            WalletParticipant {
                id: "participant1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            WalletParticipant {
                id: "participant2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ];

        let mut wallet = MultiSigWallet::new("wallet1".to_string(), participants, 2).unwrap();

        // Deposit some assets
        wallet.deposit("BTC".to_string(), 1000);

        // Create a transaction
        let transaction_id = wallet
            .create_transaction("recipient1".to_string(), "BTC".to_string(), 500)
            .unwrap();
        assert_eq!(transaction_id, 1);
        assert_eq!(wallet.pending_transaction_count(), 1);
        assert_eq!(wallet.executed_transaction_count(), 0);

        // Check that the funds were deducted from the wallet
        assert_eq!(wallet.get_balance(&"BTC".to_string()), 500);

        // Try to create a transaction with insufficient funds
        let result = wallet.create_transaction("recipient2".to_string(), "BTC".to_string(), 1000);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MultiSigError::InsufficientFunds
        ));
    }

    #[test]
    fn test_transaction_signing() {
        let participants = vec![
            WalletParticipant {
                id: "participant1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            WalletParticipant {
                id: "participant2".to_string(),
                public_key: "pubkey2".to_string(),
            },
            WalletParticipant {
                id: "participant3".to_string(),
                public_key: "pubkey3".to_string(),
            },
        ];

        let mut wallet = MultiSigWallet::new("wallet1".to_string(), participants, 2).unwrap();

        // Deposit some assets
        wallet.deposit("BTC".to_string(), 1000);

        // Create a transaction
        let transaction_id = wallet
            .create_transaction("recipient1".to_string(), "BTC".to_string(), 500)
            .unwrap();

        // Get the transaction
        let transaction = wallet.get_pending_transaction(transaction_id).unwrap();
        assert_eq!(transaction.signatures.len(), 0);
        assert!(!transaction.is_ready_for_execution());

        // Sign the transaction with participant1
        wallet
            .sign_transaction(transaction_id, "participant1".to_string())
            .unwrap();

        // Check that the signature was added
        let transaction = wallet.get_pending_transaction(transaction_id).unwrap();
        assert_eq!(transaction.signatures.len(), 1);
        assert!(transaction.has_signature_from(&"participant1".to_string()));
        assert!(!transaction.is_ready_for_execution()); // Still need one more signature

        // Sign the transaction with participant2
        wallet
            .sign_transaction(transaction_id, "participant2".to_string())
            .unwrap();

        // Check that the signature was added
        let transaction = wallet.get_pending_transaction(transaction_id).unwrap();
        assert_eq!(transaction.signatures.len(), 2);
        assert!(transaction.has_signature_from(&"participant2".to_string()));
        assert!(transaction.is_ready_for_execution()); // Now has enough signatures
    }

    #[test]
    fn test_transaction_execution() {
        let participants = vec![
            WalletParticipant {
                id: "participant1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            WalletParticipant {
                id: "participant2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ];

        let mut wallet = MultiSigWallet::new("wallet1".to_string(), participants, 2).unwrap();

        // Deposit some assets
        wallet.deposit("BTC".to_string(), 1000);

        // Create a transaction
        let transaction_id = wallet
            .create_transaction("recipient1".to_string(), "BTC".to_string(), 500)
            .unwrap();

        // Sign the transaction with both participants
        wallet
            .sign_transaction(transaction_id, "participant1".to_string())
            .unwrap();
        wallet
            .sign_transaction(transaction_id, "participant2".to_string())
            .unwrap();

        // Execute the transaction
        wallet.execute_transaction(transaction_id).unwrap();

        // Check that the transaction was moved to executed
        assert_eq!(wallet.pending_transaction_count(), 0);
        assert_eq!(wallet.executed_transaction_count(), 1);

        // Check that the transaction is marked as executed
        let transaction = wallet.get_executed_transaction(transaction_id).unwrap();
        assert!(transaction.is_executed());
        assert!(transaction.executed_timestamp.is_some());
    }

    #[test]
    fn test_transaction_cancellation() {
        let participants = vec![
            WalletParticipant {
                id: "participant1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            WalletParticipant {
                id: "participant2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ];

        let mut wallet = MultiSigWallet::new("wallet1".to_string(), participants, 2).unwrap();

        // Deposit some assets
        wallet.deposit("BTC".to_string(), 1000);

        // Create a transaction
        let transaction_id = wallet
            .create_transaction("recipient1".to_string(), "BTC".to_string(), 500)
            .unwrap();

        // Check that the funds were deducted from the wallet
        assert_eq!(wallet.get_balance(&"BTC".to_string()), 500);

        // Cancel the transaction
        wallet.cancel_transaction(transaction_id).unwrap();

        // Check that the transaction was removed
        assert_eq!(wallet.pending_transaction_count(), 0);
        assert_eq!(wallet.executed_transaction_count(), 0);

        // Check that the funds were returned to the wallet
        assert_eq!(wallet.get_balance(&"BTC".to_string()), 1000);
    }

    #[test]
    fn test_wallet_manager() {
        let mut manager = MultiSigWalletManager::new();

        // Create a wallet
        let participants = vec![
            WalletParticipant {
                id: "participant1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            WalletParticipant {
                id: "participant2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ];

        manager
            .create_wallet("wallet1".to_string(), participants, 2)
            .unwrap();

        // Check that the wallet was created
        assert_eq!(manager.wallet_count(), 1);
        assert!(manager.has_wallet("wallet1"));

        // Get the wallet
        let wallet = manager.get_wallet("wallet1").unwrap();
        assert_eq!(wallet.wallet_id, "wallet1");

        // Get all wallets
        let wallets = manager.get_all_wallets();
        assert_eq!(wallets.len(), 1);

        // Remove the wallet
        assert!(manager.remove_wallet("wallet1"));
        assert_eq!(manager.wallet_count(), 0);
        assert!(!manager.has_wallet("wallet1"));
    }

    #[test]
    fn test_participant_verification() {
        let participants = vec![
            WalletParticipant {
                id: "participant1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            WalletParticipant {
                id: "participant2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ];

        let wallet = MultiSigWallet::new("wallet1".to_string(), participants, 2).unwrap();

        // Test participant verification
        assert!(wallet.is_participant(&"participant1".to_string()));
        assert!(wallet.is_participant(&"participant2".to_string()));
        assert!(!wallet.is_participant(&"participant3".to_string()));
    }

    #[test]
    fn test_invalid_transaction_signing() {
        let participants = vec![
            WalletParticipant {
                id: "participant1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            WalletParticipant {
                id: "participant2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ];

        let mut wallet = MultiSigWallet::new("wallet1".to_string(), participants, 2).unwrap();

        // Try to sign a non-existent transaction
        let result = wallet.sign_transaction(999, "participant1".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MultiSigError::TransactionNotFound
        ));

        // Create a transaction
        wallet.deposit("BTC".to_string(), 1000);
        let transaction_id = wallet
            .create_transaction("recipient1".to_string(), "BTC".to_string(), 500)
            .unwrap();

        // Try to sign with a non-participant
        let result = wallet.sign_transaction(transaction_id, "participant3".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MultiSigError::NotParticipant));
    }

    #[test]
    fn test_invalid_transaction_execution() {
        let participants = vec![
            WalletParticipant {
                id: "participant1".to_string(),
                public_key: "pubkey1".to_string(),
            },
            WalletParticipant {
                id: "participant2".to_string(),
                public_key: "pubkey2".to_string(),
            },
        ];

        let mut wallet = MultiSigWallet::new("wallet1".to_string(), participants, 2).unwrap();

        // Try to execute a non-existent transaction
        let result = wallet.execute_transaction(999);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MultiSigError::TransactionNotFound
        ));

        // Create a transaction
        wallet.deposit("BTC".to_string(), 1000);
        let transaction_id = wallet
            .create_transaction("recipient1".to_string(), "BTC".to_string(), 500)
            .unwrap();

        // Try to execute without enough signatures
        let result = wallet.execute_transaction(transaction_id);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MultiSigError::InsufficientSignatures
        ));

        // Sign the transaction
        wallet
            .sign_transaction(transaction_id, "participant1".to_string())
            .unwrap();
        wallet
            .sign_transaction(transaction_id, "participant2".to_string())
            .unwrap();

        // Execute the transaction
        wallet.execute_transaction(transaction_id).unwrap();

        // Try to execute again (should fail)
        let result = wallet.execute_transaction(transaction_id);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MultiSigError::TransactionNotFound
        ));
    }
}
