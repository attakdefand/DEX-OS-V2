//! Tests for Lock & Mint mechanism from DEX-OS-V2.csv line 159
//!
//! This file tests the Lock & Mint bridge mechanism implementation.
//! The tests verify that the bridge can lock assets on one chain and mint equivalent assets on another.

use dex_core::universal_bridge::{
    UniversalBridgeManager, BlockchainNetwork, BridgeConfig, BridgeStatus, NetworkMetrics
};
use dex_core::types::{TokenId, TraderId, Quantity};
use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Lock & Mint mechanism basic functionality
    /// This test verifies that the bridge can lock assets on source chain and mint on destination
    #[test]
    fn test_lock_mint_basic_functionality() {
        let mut manager = UniversalBridgeManager::with_default();
        
        // Add networks
        let ethereum = BlockchainNetwork {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://ethereum.rpc".to_string(),
            chain_id: 1,
            native_token: "ETH".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        let polygon = BlockchainNetwork {
            id: "polygon".to_string(),
            name: "Polygon".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://polygon.rpc".to_string(),
            chain_id: 137,
            native_token: "MATIC".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        manager.add_network(ethereum).unwrap();
        manager.add_network(polygon).unwrap();
        
        // Initiate bridge transaction (this represents the lock phase)
        let result = manager.initiate_bridge_transaction(
            "bridge1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        );
        
        assert!(result.is_ok());
        
        // Verify transaction is initialized (locked state)
        let transaction = manager.get_transaction("bridge1");
        assert!(transaction.is_some());
        let transaction = transaction.unwrap();
        assert_eq!(transaction.status, BridgeStatus::Initialized);
        assert_eq!(transaction.amount, 1000);
        assert_eq!(transaction.token_id, "ETH".to_string());
        
        println!("✓ Lock & Mint basic functionality test passed");
    }

    /// Test Lock & Mint mechanism processing phase
    /// This test verifies that the bridge processes the lock and prepares for mint
    #[test]
    fn test_lock_mint_processing_phase() {
        let mut manager = UniversalBridgeManager::with_default();
        
        // Add networks
        let ethereum = BlockchainNetwork {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://ethereum.rpc".to_string(),
            chain_id: 1,
            native_token: "ETH".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        let polygon = BlockchainNetwork {
            id: "polygon".to_string(),
            name: "Polygon".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://polygon.rpc".to_string(),
            chain_id: 137,
            native_token: "MATIC".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        manager.add_network(ethereum).unwrap();
        manager.add_network(polygon).unwrap();
        
        // Initiate bridge transaction
        manager.initiate_bridge_transaction(
            "bridge1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        // Activate transaction
        assert!(manager.activate_bridge_transaction("bridge1").is_ok());
        
        let transaction = manager.get_transaction("bridge1").unwrap();
        assert_eq!(transaction.status, BridgeStatus::Active);
        
        // Process transaction (this represents the lock confirmation and mint preparation)
        assert!(manager.process_bridge_transaction("bridge1").is_ok());
        
        let transaction = manager.get_transaction("bridge1").unwrap();
        assert_eq!(transaction.status, BridgeStatus::Processing);
        
        println!("✓ Lock & Mint processing phase test passed");
    }

    /// Test Lock & Mint mechanism completion
    /// This test verifies that the bridge completes the mint process
    #[test]
    fn test_lock_mint_completion() {
        let mut manager = UniversalBridgeManager::with_default();
        
        // Add networks
        let ethereum = BlockchainNetwork {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://ethereum.rpc".to_string(),
            chain_id: 1,
            native_token: "ETH".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        let polygon = BlockchainNetwork {
            id: "polygon".to_string(),
            name: "Polygon".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://polygon.rpc".to_string(),
            chain_id: 137,
            native_token: "MATIC".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        manager.add_network(ethereum).unwrap();
        manager.add_network(polygon).unwrap();
        
        // Initiate and activate bridge transaction
        manager.initiate_bridge_transaction(
            "bridge1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        manager.activate_bridge_transaction("bridge1").unwrap();
        
        // Complete transaction (this represents the mint completion)
        assert!(manager.complete_bridge_transaction(
            "bridge1",
            Some("0xsource_hash".to_string()),
            Some("0xdest_hash".to_string())
        ).is_ok());
        
        // Transaction should now be in completed transactions
        assert!(manager.get_transaction("bridge1").is_none());
        assert!(manager.get_completed_transaction("bridge1").is_some());
        
        let completed_tx = manager.get_completed_transaction("bridge1").unwrap();
        assert_eq!(completed_tx.status, BridgeStatus::Completed);
        assert_eq!(completed_tx.source_tx_hash, Some("0xsource_hash".to_string()));
        assert_eq!(completed_tx.destination_tx_hash, Some("0xdest_hash".to_string()));
        
        // Statistics should be updated
        assert_eq!(manager.get_statistics().successful_transactions, 1);
        
        println!("✓ Lock & Mint completion test passed");
    }

    /// Test Lock & Mint mechanism with different token types
    /// This test verifies that the mechanism works with various token types
    #[test]
    fn test_lock_mint_different_tokens() {
        let mut manager = UniversalBridgeManager::with_default();
        
        // Add networks
        let ethereum = BlockchainNetwork {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://ethereum.rpc".to_string(),
            chain_id: 1,
            native_token: "ETH".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        let binance = BlockchainNetwork {
            id: "binance".to_string(),
            name: "Binance Smart Chain".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://binance.rpc".to_string(),
            chain_id: 56,
            native_token: "BNB".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        manager.add_network(ethereum).unwrap();
        manager.add_network(binance).unwrap();
        
        // Test with ERC-20 token
        manager.initiate_bridge_transaction(
            "bridge_erc20".to_string(),
            "ethereum".to_string(),
            "binance".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "USDC".to_string(), // ERC-20 token
            5000,
        ).unwrap();
        
        // Test with native token
        manager.initiate_bridge_transaction(
            "bridge_native".to_string(),
            "ethereum".to_string(),
            "binance".to_string(),
            "sender2".to_string(),
            "receiver2".to_string(),
            "ETH".to_string(), // Native token
            2000,
        ).unwrap();
        
        // Verify both transactions are initialized
        assert!(manager.get_transaction("bridge_erc20").is_some());
        assert!(manager.get_transaction("bridge_native").is_some());
        
        let erc20_tx = manager.get_transaction("bridge_erc20").unwrap();
        assert_eq!(erc20_tx.token_id, "USDC".to_string());
        assert_eq!(erc20_tx.amount, 5000);
        
        let native_tx = manager.get_transaction("bridge_native").unwrap();
        assert_eq!(native_tx.token_id, "ETH".to_string());
        assert_eq!(native_tx.amount, 2000);
        
        println!("✓ Lock & Mint different tokens test passed");
    }

    /// Test Lock & Mint mechanism error handling
    /// This test verifies proper error handling in the mechanism
    #[test]
    fn test_lock_mint_error_handling() {
        let mut manager = UniversalBridgeManager::with_default();
        
        // Add networks
        let ethereum = BlockchainNetwork {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://ethereum.rpc".to_string(),
            chain_id: 1,
            native_token: "ETH".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        let polygon = BlockchainNetwork {
            id: "polygon".to_string(),
            name: "Polygon".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://polygon.rpc".to_string(),
            chain_id: 137,
            native_token: "MATIC".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        manager.add_network(ethereum).unwrap();
        manager.add_network(polygon).unwrap();
        
        // Try to initiate bridge to unsupported network
        let result = manager.initiate_bridge_transaction(
            "bridge1".to_string(),
            "ethereum".to_string(),
            "unsupported_chain".to_string(), // This network is not supported
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        );
        
        assert!(result.is_err());
        
        // Try to activate non-existent transaction
        let result = manager.activate_bridge_transaction("nonexistent");
        assert!(result.is_err());
        
        // Try to process non-existent transaction
        let result = manager.process_bridge_transaction("nonexistent");
        assert!(result.is_err());
        
        println!("✓ Lock & Mint error handling test passed");
    }

    /// Test Lock & Mint mechanism statistics tracking
    /// This test verifies that the mechanism properly tracks statistics
    #[test]
    fn test_lock_mint_statistics_tracking() {
        let mut manager = UniversalBridgeManager::with_default();
        
        // Add networks
        let ethereum = BlockchainNetwork {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://ethereum.rpc".to_string(),
            chain_id: 1,
            native_token: "ETH".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        let polygon = BlockchainNetwork {
            id: "polygon".to_string(),
            name: "Polygon".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://polygon.rpc".to_string(),
            chain_id: 137,
            native_token: "MATIC".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };
        
        manager.add_network(ethereum).unwrap();
        manager.add_network(polygon).unwrap();
        
        // Initial statistics check
        let initial_stats = manager.get_statistics();
        assert_eq!(initial_stats.total_transactions, 0);
        assert_eq!(initial_stats.successful_transactions, 0);
        assert_eq!(initial_stats.failed_transactions, 0);
        
        // Initiate some transactions
        manager.initiate_bridge_transaction(
            "bridge1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(),
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        manager.initiate_bridge_transaction(
            "bridge2".to_string(),
            "polygon".to_string(),
            "ethereum".to_string(),
            "sender2".to_string(),
            "receiver2".to_string(),
            "MATIC".to_string(),
            2000,
        ).unwrap();
        
        // Check updated statistics
        let stats = manager.get_statistics();
        assert_eq!(stats.total_transactions, 2);
        assert_eq!(stats.total_volume.get("ETH"), Some(&1000));
        assert_eq!(stats.total_volume.get("MATIC"), Some(&2000));
        
        println!("✓ Lock & Mint statistics tracking test passed");
    }
}