//! Tests for the Universal Bridge implementation

use dex_core::universal_bridge::*;
use dex_core::universal_bridge::NetworkMetrics;

#[test]
fn test_universal_bridge_basic_functionality() {
    let mut manager = UniversalBridgeManager::with_default();
    
    // Test creating a blockchain network
    let ethereum = BlockchainNetwork {
        id: "ethereum".to_string(),
        name: "Ethereum".to_string(),
        chain_type: "EVM".to_string(),
        rpc_endpoint: "https://ethereum.rpc".to_string(),
        chain_id: 1,
        native_token: "ETH".to_string(),
        features: std::collections::HashSet::new(),
        metrics: NetworkMetrics::default(),
    };
    
    // Test adding network
    assert!(manager.add_network(ethereum).is_ok());
    assert_eq!(manager.network_count(), 1);
    assert!(manager.is_network_supported("ethereum"));
    
    // Test getting network
    let network = manager.get_network("ethereum");
    assert!(network.is_some());
    assert_eq!(network.unwrap().name, "Ethereum");
    
    // Test getting supported networks
    let networks = manager.get_supported_networks();
    assert_eq!(networks.len(), 1);
}

#[test]
fn test_bridge_transaction_lifecycle() {
    let mut manager = UniversalBridgeManager::with_default();
    
    // Add networks
    let ethereum = BlockchainNetwork {
        id: "ethereum".to_string(),
        name: "Ethereum".to_string(),
        chain_type: "EVM".to_string(),
        rpc_endpoint: "https://ethereum.rpc".to_string(),
        chain_id: 1,
        native_token: "ETH".to_string(),
        features: std::collections::HashSet::new(),
        metrics: NetworkMetrics::default(),
    };
    
    let polygon = BlockchainNetwork {
        id: "polygon".to_string(),
        name: "Polygon".to_string(),
        chain_type: "EVM".to_string(),
        rpc_endpoint: "https://polygon.rpc".to_string(),
        chain_id: 137,
        native_token: "MATIC".to_string(),
        features: std::collections::HashSet::new(),
        metrics: NetworkMetrics::default(),
    };
    
    manager.add_network(ethereum).unwrap();
    manager.add_network(polygon).unwrap();
    
    // Test initiating bridge transaction
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
    
    // Test getting transaction
    let transaction = manager.get_transaction("bridge1");
    assert!(transaction.is_some());
    let transaction = transaction.unwrap();
    assert_eq!(transaction.status, BridgeStatus::Initialized);
    assert_eq!(transaction.amount, 1000);
    
    // Test activating transaction
    assert!(manager.activate_bridge_transaction("bridge1").is_ok());
    
    let transaction = manager.get_transaction("bridge1").unwrap();
    assert_eq!(transaction.status, BridgeStatus::Active);
    
    // Test processing transaction
    assert!(manager.process_bridge_transaction("bridge1").is_ok());
    
    let transaction = manager.get_transaction("bridge1").unwrap();
    assert_eq!(transaction.status, BridgeStatus::Processing);
    
    // Test completing transaction
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
}

#[test]
fn test_unsupported_network_error() {
    let mut manager = UniversalBridgeManager::with_default();
    
    // Add only one network
    let ethereum = BlockchainNetwork {
        id: "ethereum".to_string(),
        name: "Ethereum".to_string(),
        chain_type: "EVM".to_string(),
        rpc_endpoint: "https://ethereum.rpc".to_string(),
        chain_id: 1,
        native_token: "ETH".to_string(),
        features: std::collections::HashSet::new(),
        metrics: NetworkMetrics::default(),
    };
    
    manager.add_network(ethereum).unwrap();
    
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
    assert_eq!(result.unwrap_err(), UniversalBridgeError::UnsupportedNetwork("unsupported_chain".to_string()));
}

#[test]
fn test_statistics_tracking() {
    let mut manager = UniversalBridgeManager::with_default();
    
    // Add networks
    let ethereum = BlockchainNetwork {
        id: "ethereum".to_string(),
        name: "Ethereum".to_string(),
        chain_type: "EVM".to_string(),
        rpc_endpoint: "https://ethereum.rpc".to_string(),
        chain_id: 1,
        native_token: "ETH".to_string(),
        features: std::collections::HashSet::new(),
        metrics: NetworkMetrics::default(),
    };
    
    let polygon = BlockchainNetwork {
        id: "polygon".to_string(),
        name: "Polygon".to_string(),
        chain_type: "EVM".to_string(),
        rpc_endpoint: "https://polygon.rpc".to_string(),
        chain_id: 137,
        native_token: "MATIC".to_string(),
        features: std::collections::HashSet::new(),
        metrics: NetworkMetrics::default(),
    };
    
    manager.add_network(ethereum).unwrap();
    manager.add_network(polygon).unwrap();
    
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
        "ETH".to_string(),
        500,
    ).unwrap();
    
    let stats = manager.get_statistics();
    assert_eq!(stats.total_transactions, 2);
    assert_eq!(stats.total_volume.get("ETH"), Some(&1500));
}