//! Comprehensive tests for Multi-signature Wallets implementation
//!
//! This file implements security and functionality tests for the Priority 3 feature:
//! - Blockchain Resilience,Blockchain Resilience,Blockchain Resilience,Multisig Wallets,Key Distribution,Medium

use dex_core::multisig_wallet::{
    MultiSigWallet, MultiSigWalletManager, WalletParticipant, MultiSigError
};
use dex_core::types::{TokenId, Quantity};

/// Test comprehensive multisig wallet functionality
#[test]
fn test_multisig_wallet_comprehensive_functionality() {
    // Create participants
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

    // Create wallet with 2-of-3 multisig
    let mut wallet = MultiSigWallet::new("wallet1".to_string(), participants, 2).unwrap();
    
    // Test initial state
    assert_eq!(wallet.wallet_id, "wallet1");
    assert_eq!(wallet.participant_count(), 3);
    assert_eq!(wallet.required_signatures, 2);
    assert_eq!(wallet.pending_transaction_count(), 0);
    assert_eq!(wallet.executed_transaction_count(), 0);
    
    // Test asset deposit
    wallet.deposit("BTC".to_string(), 1000);
    assert_eq!(wallet.get_balance(&"BTC".to_string()), 1000);
    
    wallet.deposit("ETH".to_string(), 50);
    assert_eq!(wallet.get_balance(&"BTC".to_string()), 1000);
    assert_eq!(wallet.get_balance(&"ETH".to_string()), 50);
    
    // Test participant verification
    assert!(wallet.is_participant(&"participant1".to_string()));
    assert!(wallet.is_participant(&"participant2".to_string()));
    assert!(wallet.is_participant(&"participant3".to_string()));
    assert!(!wallet.is_participant(&"participant4".to_string()));
    
    // Test transaction creation
    let transaction_id = wallet
        .create_transaction("recipient1".to_string(), "BTC".to_string(), 500)
        .unwrap();
    
    assert_eq!(transaction_id, 1);
    assert_eq!(wallet.pending_transaction_count(), 1);
    assert_eq!(wallet.executed_transaction_count(), 0);
    // Check that funds are locked
    assert_eq!(wallet.get_balance(&"BTC".to_string()), 500);
    
    // Test transaction signing
    let transaction = wallet.get_pending_transaction(transaction_id).unwrap();
    assert_eq!(transaction.signatures.len(), 0);
    assert!(!transaction.is_ready_for_execution());
    
    // Sign with first participant
    wallet.sign_transaction(transaction_id, "participant1".to_string()).unwrap();
    let transaction = wallet.get_pending_transaction(transaction_id).unwrap();
    assert_eq!(transaction.signatures.len(), 1);
    assert!(transaction.has_signature_from(&"participant1".to_string()));
    assert!(!transaction.is_ready_for_execution());
    
    // Sign with second participant
    wallet.sign_transaction(transaction_id, "participant2".to_string()).unwrap();
    let transaction = wallet.get_pending_transaction(transaction_id).unwrap();
    assert_eq!(transaction.signatures.len(), 2);
    assert!(transaction.has_signature_from(&"participant2".to_string()));
    assert!(transaction.is_ready_for_execution());
    
    // Test transaction execution
    wallet.execute_transaction(transaction_id).unwrap();
    assert_eq!(wallet.pending_transaction_count(), 0);
    assert_eq!(wallet.executed_transaction_count(), 1);
    
    let transaction = wallet.get_executed_transaction(transaction_id).unwrap();
    assert!(transaction.is_executed());
    assert!(transaction.executed_timestamp.is_some());
    
    // Test transaction cancellation (create new transaction first)
    wallet.deposit("BTC".to_string(), 300);
    let transaction_id2 = wallet
        .create_transaction("recipient2".to_string(), "BTC".to_string(), 200)
        .unwrap();
    assert_eq!(wallet.get_balance(&"BTC".to_string()), 600); // 500 remaining + 300 new - 200 locked
    
    wallet.cancel_transaction(transaction_id2).unwrap();
    assert_eq!(wallet.pending_transaction_count(), 0);
    assert_eq!(wallet.executed_transaction_count(), 1);
    assert_eq!(wallet.get_balance(&"BTC".to_string()), 800); // 500 remaining + 300 returned
}

/// Test multisig wallet manager functionality
#[test]
fn test_multisig_wallet_manager_comprehensive() {
    let mut manager = MultiSigWalletManager::new();
    
    // Test initial state
    assert_eq!(manager.wallet_count(), 0);
    
    // Create multiple wallets
    let participants1 = vec![
        WalletParticipant {
            id: "participant1".to_string(),
            public_key: "pubkey1".to_string(),
        },
        WalletParticipant {
            id: "participant2".to_string(),
            public_key: "pubkey2".to_string(),
        },
    ];
    
    let participants2 = vec![
        WalletParticipant {
            id: "participant3".to_string(),
            public_key: "pubkey3".to_string(),
        },
        WalletParticipant {
            id: "participant4".to_string(),
            public_key: "pubkey4".to_string(),
        },
    ];
    
    manager.create_wallet("wallet1".to_string(), participants1, 2).unwrap();
    manager.create_wallet("wallet2".to_string(), participants2, 1).unwrap();
    
    // Test wallet management
    assert_eq!(manager.wallet_count(), 2);
    assert!(manager.has_wallet("wallet1"));
    assert!(manager.has_wallet("wallet2"));
    assert!(!manager.has_wallet("wallet3"));
    
    // Test wallet retrieval
    let wallet1 = manager.get_wallet("wallet1").unwrap();
    assert_eq!(wallet1.wallet_id, "wallet1");
    assert_eq!(wallet1.required_signatures, 2);
    
    let wallet2 = manager.get_wallet("wallet2").unwrap();
    assert_eq!(wallet2.wallet_id, "wallet2");
    assert_eq!(wallet2.required_signatures, 1);
    
    // Test mutable wallet access
    let wallet1_mut = manager.get_wallet_mut("wallet1").unwrap();
    wallet1_mut.deposit("BTC".to_string(), 1000);
    assert_eq!(wallet1_mut.get_balance(&"BTC".to_string()), 1000);
    
    // Test wallet removal
    assert!(manager.remove_wallet("wallet1"));
    assert_eq!(manager.wallet_count(), 1);
    assert!(!manager.has_wallet("wallet1"));
    assert!(manager.has_wallet("wallet2"));
}

/// Test edge cases and error conditions for multisig wallets
#[test]
fn test_multisig_wallet_edge_cases() {
    // Test wallet creation with invalid parameters
    let participants = vec![
        WalletParticipant {
            id: "participant1".to_string(),
            public_key: "pubkey1".to_string(),
        },
    ];
    
    // Test with zero required signatures
    let result = MultiSigWallet::new("wallet1".to_string(), participants.clone(), 0);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MultiSigError::InvalidRequiredSignatures));
    
    // Test with required signatures greater than participants
    let result = MultiSigWallet::new("wallet1".to_string(), participants.clone(), 2);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MultiSigError::InvalidRequiredSignatures));
    
    // Test with no participants
    let result = MultiSigWallet::new("wallet1".to_string(), vec![], 1);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MultiSigError::NoParticipants));
    
    // Create a valid wallet for further testing
    let mut wallet = MultiSigWallet::new("wallet1".to_string(), participants, 1).unwrap();
    
    // Test transaction with insufficient funds
    let result = wallet.create_transaction("recipient".to_string(), "BTC".to_string(), 1000);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MultiSigError::InsufficientFunds));
    
    // Deposit some funds
    wallet.deposit("BTC".to_string(), 1000);
    
    // Test transaction creation
    let transaction_id = wallet.create_transaction("recipient".to_string(), "BTC".to_string(), 500).unwrap();
    
    // Test signing with non-participant
    let result = wallet.sign_transaction(transaction_id, "nonparticipant".to_string());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MultiSigError::NotParticipant));
    
    // Test signing non-existent transaction
    let result = wallet.sign_transaction(999, "participant1".to_string());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MultiSigError::TransactionNotFound));
    
    // Sign the transaction
    wallet.sign_transaction(transaction_id, "participant1".to_string()).unwrap();
    
    // Execute the transaction
    wallet.execute_transaction(transaction_id).unwrap();
    
    // Test executing already executed transaction
    let result = wallet.execute_transaction(transaction_id);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MultiSigError::TransactionNotFound));
    
    // Test canceling non-existent transaction
    let result = wallet.cancel_transaction(999);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MultiSigError::TransactionNotFound));
    
    // Create another transaction and try to cancel executed transaction
    let transaction_id2 = wallet.create_transaction("recipient2".to_string(), "BTC".to_string(), 200).unwrap();
    wallet.sign_transaction(transaction_id2, "participant1".to_string()).unwrap();
    wallet.execute_transaction(transaction_id2).unwrap();
    
    let result = wallet.cancel_transaction(transaction_id2);
    assert!(result.is_err());
    // Note: This should fail because the transaction is already executed, but the error type might vary
}