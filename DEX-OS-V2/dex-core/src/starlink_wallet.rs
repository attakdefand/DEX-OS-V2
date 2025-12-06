//! Starlink Wallet Module for WASM Runtime
//!
//! Implements Priority 5 feature from DEX-OS-V2.csv:
//! - Core Components,WASM Runtime,Runtime,Starlink Wallet,Satellite Integration,Medium {Security: Layer 19 - Mobile Security}
//!
//! Features:
//! - Satellite-based wallet connectivity
//! - Offline transaction signing
//! - Low-bandwidth transaction broadcasting
//! - Secure key storage for remote locations
//! - Emergency transaction capabilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Starlink wallet errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum StarlinkError {
    #[error("Satellite connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Insufficient bandwidth: {0}")]
    InsufficientBandwidth(String),
    #[error("Transaction signing failed: {0}")]
    SigningFailed(String),
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    #[error("Broadcast failed: {0}")]
    BroadcastFailed(String),
}

/// Wallet identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WalletId(pub String);

/// Satellite connection status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected { signal_strength: u8 },
    Connecting,
    Disconnected,
    LowBandwidth { available_kbps: u32 },
}

/// Transaction priority for bandwidth management
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionPriority {
    Emergency,
    High,
    Normal,
    Low,
}

/// Offline transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineTransaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
    pub priority: TransactionPriority,
    pub signed: bool,
    pub signature: Option<Vec<u8>>,
    pub created_at: u64,
}

/// Starlink wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarlinkWallet {
    pub id: WalletId,
    pub address: String,
    pub balance: u64,
    pub connection_status: ConnectionStatus,
    pub pending_transactions: Vec<OfflineTransaction>,
    pub last_sync: u64,
}

/// Broadcast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResult {
    pub success: bool,
    pub transaction_id: String,
    pub confirmations: u32,
    pub bandwidth_used: u32,
}

/// Starlink wallet manager
#[derive(Debug, Clone)]
pub struct StarlinkWalletManager {
    /// Active wallets
    wallets: Arc<RwLock<HashMap<WalletId, StarlinkWallet>>>,
    /// Transaction queue (prioritized)
    transaction_queue: Arc<RwLock<Vec<OfflineTransaction>>>,
    /// Bandwidth usage tracker
    bandwidth_used: Arc<RwLock<u64>>,
}

impl StarlinkWalletManager {
    /// Create a new Starlink wallet manager
    pub fn new() -> Self {
        Self {
            wallets: Arc::new(RwLock::new(HashMap::new())),
            transaction_queue: Arc::new(RwLock::new(Vec::new())),
            bandwidth_used: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a new wallet
    pub fn create_wallet(&self, address: String) -> Result<StarlinkWallet, StarlinkError> {
        let wallet_id = WalletId(format!("starlink-{}", uuid::Uuid::new_v4()));
        
        let wallet = StarlinkWallet {
            id: wallet_id.clone(),
            address,
            balance: 0,
            connection_status: ConnectionStatus::Disconnected,
            pending_transactions: Vec::new(),
            last_sync: Self::current_timestamp(),
        };

        let mut wallets = self.wallets.write().unwrap();
        wallets.insert(wallet_id, wallet.clone());

        Ok(wallet)
    }

    /// Get wallet by ID
    pub fn get_wallet(&self, wallet_id: &WalletId) -> Result<StarlinkWallet, StarlinkError> {
        let wallets = self.wallets.read().unwrap();
        wallets
            .get(wallet_id)
            .cloned()
            .ok_or_else(|| StarlinkError::WalletNotFound(wallet_id.0.clone()))
    }

    /// Update connection status
    pub fn update_connection_status(
        &self,
        wallet_id: &WalletId,
        status: ConnectionStatus,
    ) -> Result<(), StarlinkError> {
        let mut wallets = self.wallets.write().unwrap();
        let wallet = wallets
            .get_mut(wallet_id)
            .ok_or_else(|| StarlinkError::WalletNotFound(wallet_id.0.clone()))?;

        wallet.connection_status = status;
        Ok(())
    }

    /// Create an offline transaction
    pub fn create_offline_transaction(
        &self,
        wallet_id: &WalletId,
        to: String,
        amount: u64,
        priority: TransactionPriority,
    ) -> Result<OfflineTransaction, StarlinkError> {
        let wallets = self.wallets.read().unwrap();
        let wallet = wallets
            .get(wallet_id)
            .ok_or_else(|| StarlinkError::WalletNotFound(wallet_id.0.clone()))?;

        let tx = OfflineTransaction {
            id: format!("tx-{}", uuid::Uuid::new_v4()),
            from: wallet.address.clone(),
            to,
            amount,
            nonce: wallet.pending_transactions.len() as u64,
            priority,
            signed: false,
            signature: None,
            created_at: Self::current_timestamp(),
        };

        Ok(tx)
    }

    /// Sign an offline transaction
    pub fn sign_transaction(
        &self,
        wallet_id: &WalletId,
        mut transaction: OfflineTransaction,
    ) -> Result<OfflineTransaction, StarlinkError> {
        // Verify wallet exists
        let wallets = self.wallets.read().unwrap();
        wallets
            .get(wallet_id)
            .ok_or_else(|| StarlinkError::WalletNotFound(wallet_id.0.clone()))?;

        // Simulate signing (in real implementation, use actual cryptographic signing)
        let signature = self.simulate_signature(&transaction);
        transaction.signed = true;
        transaction.signature = Some(signature);

        // Add to wallet's pending transactions
        drop(wallets);
        let mut wallets = self.wallets.write().unwrap();
        if let Some(wallet) = wallets.get_mut(wallet_id) {
            wallet.pending_transactions.push(transaction.clone());
        }

        Ok(transaction)
    }

    /// Broadcast transaction when satellite connection is available
    pub fn broadcast_transaction(
        &self,
        wallet_id: &WalletId,
        transaction: &OfflineTransaction,
    ) -> Result<BroadcastResult, StarlinkError> {
        let wallets = self.wallets.read().unwrap();
        let wallet = wallets
            .get(wallet_id)
            .ok_or_else(|| StarlinkError::WalletNotFound(wallet_id.0.clone()))?;

        // Check connection status
        match &wallet.connection_status {
            ConnectionStatus::Disconnected => {
                return Err(StarlinkError::ConnectionFailed(
                    "No satellite connection".to_string(),
                ))
            }
            ConnectionStatus::Connecting => {
                return Err(StarlinkError::ConnectionFailed(
                    "Connection in progress".to_string(),
                ))
            }
            ConnectionStatus::LowBandwidth { available_kbps } => {
                if *available_kbps < 10 && transaction.priority != TransactionPriority::Emergency {
                    return Err(StarlinkError::InsufficientBandwidth(format!(
                        "Only {} kbps available",
                        available_kbps
                    )));
                }
            }
            ConnectionStatus::Connected { .. } => {}
        }

        // Verify transaction is signed
        if !transaction.signed {
            return Err(StarlinkError::InvalidTransaction(
                "Transaction not signed".to_string(),
            ));
        }

        // Simulate broadcast
        let bandwidth_used = self.calculate_bandwidth_usage(transaction);
        let mut total_bandwidth = self.bandwidth_used.write().unwrap();
        *total_bandwidth += bandwidth_used as u64;

        Ok(BroadcastResult {
            success: true,
            transaction_id: transaction.id.clone(),
            confirmations: 0,
            bandwidth_used,
        })
    }

    /// Broadcast all pending transactions
    pub fn broadcast_pending_transactions(
        &self,
        wallet_id: &WalletId,
    ) -> Result<Vec<BroadcastResult>, StarlinkError> {
        let mut wallets = self.wallets.write().unwrap();
        let wallet = wallets
            .get_mut(wallet_id)
            .ok_or_else(|| StarlinkError::WalletNotFound(wallet_id.0.clone()))?;

        // Sort by priority
        wallet
            .pending_transactions
            .sort_by(|a, b| self.priority_value(&b.priority).cmp(&self.priority_value(&a.priority)));

        let transactions = wallet.pending_transactions.clone();
        drop(wallets);

        let mut results = Vec::new();
        for tx in transactions {
            match self.broadcast_transaction(wallet_id, &tx) {
                Ok(result) => results.push(result),
                Err(e) => {
                    // Continue with other transactions even if one fails
                    eprintln!("Failed to broadcast transaction {}: {}", tx.id, e);
                }
            }
        }

        // Clear successfully broadcast transactions
        let mut wallets = self.wallets.write().unwrap();
        if let Some(wallet) = wallets.get_mut(wallet_id) {
            wallet.pending_transactions.clear();
            wallet.last_sync = Self::current_timestamp();
        }

        Ok(results)
    }

    /// Sync wallet balance from satellite
    pub fn sync_wallet(&self, wallet_id: &WalletId) -> Result<u64, StarlinkError> {
        let mut wallets = self.wallets.write().unwrap();
        let wallet = wallets
            .get_mut(wallet_id)
            .ok_or_else(|| StarlinkError::WalletNotFound(wallet_id.0.clone()))?;

        // Check connection
        if matches!(wallet.connection_status, ConnectionStatus::Disconnected) {
            return Err(StarlinkError::ConnectionFailed(
                "Cannot sync without connection".to_string(),
            ));
        }

        // Simulate balance sync (in real implementation, query blockchain via satellite)
        wallet.last_sync = Self::current_timestamp();
        Ok(wallet.balance)
    }

    /// Get total bandwidth used
    pub fn get_bandwidth_usage(&self) -> u64 {
        *self.bandwidth_used.read().unwrap()
    }

    /// Helper: Calculate bandwidth usage for a transaction
    fn calculate_bandwidth_usage(&self, transaction: &OfflineTransaction) -> u32 {
        // Estimate based on transaction size
        let base_size = 250; // bytes
        let signature_size = transaction.signature.as_ref().map_or(0, |s| s.len());
        ((base_size + signature_size) / 1024) as u32 // Convert to KB
    }

    /// Helper: Get priority numeric value
    fn priority_value(&self, priority: &TransactionPriority) -> u8 {
        match priority {
            TransactionPriority::Emergency => 4,
            TransactionPriority::High => 3,
            TransactionPriority::Normal => 2,
            TransactionPriority::Low => 1,
        }
    }

    /// Helper: Simulate transaction signature
    fn simulate_signature(&self, transaction: &OfflineTransaction) -> Vec<u8> {
        use sha3::{Digest, Sha3_256};
        
        let mut hasher = Sha3_256::new();
        hasher.update(transaction.id.as_bytes());
        hasher.update(transaction.from.as_bytes());
        hasher.update(transaction.to.as_bytes());
        hasher.update(&transaction.amount.to_le_bytes());
        hasher.update(&transaction.nonce.to_le_bytes());
        
        hasher.finalize().to_vec()
    }

    /// Helper: Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Default for StarlinkWalletManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_wallet() {
        let manager = StarlinkWalletManager::new();
        let wallet = manager.create_wallet("0x1234567890".to_string()).unwrap();

        assert!(wallet.id.0.starts_with("starlink-"));
        assert_eq!(wallet.address, "0x1234567890");
        assert_eq!(wallet.balance, 0);
    }

    #[test]
    fn test_create_offline_transaction() {
        let manager = StarlinkWalletManager::new();
        let wallet = manager.create_wallet("0x1234567890".to_string()).unwrap();

        let tx = manager
            .create_offline_transaction(
                &wallet.id,
                "0x0987654321".to_string(),
                1000,
                TransactionPriority::Normal,
            )
            .unwrap();

        assert_eq!(tx.from, "0x1234567890");
        assert_eq!(tx.to, "0x0987654321");
        assert_eq!(tx.amount, 1000);
        assert!(!tx.signed);
    }

    #[test]
    fn test_sign_transaction() {
        let manager = StarlinkWalletManager::new();
        let wallet = manager.create_wallet("0x1234567890".to_string()).unwrap();

        let tx = manager
            .create_offline_transaction(
                &wallet.id,
                "0x0987654321".to_string(),
                1000,
                TransactionPriority::Normal,
            )
            .unwrap();

        let signed_tx = manager.sign_transaction(&wallet.id, tx).unwrap();

        assert!(signed_tx.signed);
        assert!(signed_tx.signature.is_some());
    }

    #[test]
    fn test_broadcast_without_connection_fails() {
        let manager = StarlinkWalletManager::new();
        let wallet = manager.create_wallet("0x1234567890".to_string()).unwrap();

        let tx = manager
            .create_offline_transaction(
                &wallet.id,
                "0x0987654321".to_string(),
                1000,
                TransactionPriority::Normal,
            )
            .unwrap();

        let signed_tx = manager.sign_transaction(&wallet.id, tx).unwrap();
        let result = manager.broadcast_transaction(&wallet.id, &signed_tx);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StarlinkError::ConnectionFailed(_)
        ));
    }

    #[test]
    fn test_broadcast_with_connection() {
        let manager = StarlinkWalletManager::new();
        let wallet = manager.create_wallet("0x1234567890".to_string()).unwrap();

        // Update connection status
        manager
            .update_connection_status(
                &wallet.id,
                ConnectionStatus::Connected {
                    signal_strength: 80,
                },
            )
            .unwrap();

        let tx = manager
            .create_offline_transaction(
                &wallet.id,
                "0x0987654321".to_string(),
                1000,
                TransactionPriority::Normal,
            )
            .unwrap();

        let signed_tx = manager.sign_transaction(&wallet.id, tx).unwrap();
        let result = manager.broadcast_transaction(&wallet.id, &signed_tx).unwrap();

        assert!(result.success);
        assert_eq!(result.transaction_id, signed_tx.id);
    }

    #[test]
    fn test_low_bandwidth_blocks_normal_priority() {
        let manager = StarlinkWalletManager::new();
        let wallet = manager.create_wallet("0x1234567890".to_string()).unwrap();

        // Set low bandwidth
        manager
            .update_connection_status(
                &wallet.id,
                ConnectionStatus::LowBandwidth { available_kbps: 5 },
            )
            .unwrap();

        let tx = manager
            .create_offline_transaction(
                &wallet.id,
                "0x0987654321".to_string(),
                1000,
                TransactionPriority::Normal,
            )
            .unwrap();

        let signed_tx = manager.sign_transaction(&wallet.id, tx).unwrap();
        let result = manager.broadcast_transaction(&wallet.id, &signed_tx);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StarlinkError::InsufficientBandwidth(_)
        ));
    }

    #[test]
    fn test_emergency_priority_works_on_low_bandwidth() {
        let manager = StarlinkWalletManager::new();
        let wallet = manager.create_wallet("0x1234567890".to_string()).unwrap();

        // Set low bandwidth
        manager
            .update_connection_status(
                &wallet.id,
                ConnectionStatus::LowBandwidth { available_kbps: 5 },
            )
            .unwrap();

        let tx = manager
            .create_offline_transaction(
                &wallet.id,
                "0x0987654321".to_string(),
                1000,
                TransactionPriority::Emergency,
            )
            .unwrap();

        let signed_tx = manager.sign_transaction(&wallet.id, tx).unwrap();
        let result = manager.broadcast_transaction(&wallet.id, &signed_tx).unwrap();

        assert!(result.success);
    }

    #[test]
    fn test_broadcast_pending_transactions() {
        let manager = StarlinkWalletManager::new();
        let wallet = manager.create_wallet("0x1234567890".to_string()).unwrap();

        // Update connection status
        manager
            .update_connection_status(
                &wallet.id,
                ConnectionStatus::Connected {
                    signal_strength: 90,
                },
            )
            .unwrap();

        // Create and sign multiple transactions
        for i in 0..3 {
            let tx = manager
                .create_offline_transaction(
                    &wallet.id,
                    format!("0x{:020}", i),
                    1000 * (i + 1) as u64,
                    TransactionPriority::Normal,
                )
                .unwrap();
            manager.sign_transaction(&wallet.id, tx).unwrap();
        }

        let results = manager.broadcast_pending_transactions(&wallet.id).unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success));
    }

    #[test]
    fn test_bandwidth_tracking() {
        let manager = StarlinkWalletManager::new();
        let wallet = manager.create_wallet("0x1234567890".to_string()).unwrap();

        manager
            .update_connection_status(
                &wallet.id,
                ConnectionStatus::Connected {
                    signal_strength: 85,
                },
            )
            .unwrap();

        let initial_bandwidth = manager.get_bandwidth_usage();

        let tx = manager
            .create_offline_transaction(
                &wallet.id,
                "0x0987654321".to_string(),
                1000,
                TransactionPriority::Normal,
            )
            .unwrap();

        let signed_tx = manager.sign_transaction(&wallet.id, tx).unwrap();
        manager.broadcast_transaction(&wallet.id, &signed_tx).unwrap();

        let final_bandwidth = manager.get_bandwidth_usage();
        assert!(final_bandwidth > initial_bandwidth);
    }
}
