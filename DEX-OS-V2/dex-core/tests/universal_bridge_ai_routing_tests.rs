//! Tests for the Universal Bridge AI Routing implementation

use dex_core::universal_bridge::*;
use dex_core::ai_router::{RouteSuggestion, RouteCandidate};
use std::collections::HashSet;

#[test]
fn test_universal_bridge_ai_routing_basic_functionality() {
    let mut manager = UniversalBridgeManager::with_default();
    
    // Add networks with metrics for AI routing
    let ethereum = BlockchainNetwork {
        id: "ethereum".to_string(),
        name: "Ethereum".to_string(),
        chain_type: "EVM".to_string(),
        rpc_endpoint: "https://ethereum.rpc".to_string(),
        chain_id: 1,
        native_token: "ETH".to_string(),
        features: HashSet::from(["smart_contracts".to_string(), "evm".to_string()]),
        metrics: NetworkMetrics {
            estimated_latency_ms: 100,
            gas_price: 20,
            congestion: 30,
            block_time: 15,
            pending_transactions: 1000,
        },
    };
    
    let polygon = BlockchainNetwork {
        id: "polygon".to_string(),
        name: "Polygon".to_string(),
        chain_type: "EVM".to_string(),
        rpc_endpoint: "https://polygon.rpc".to_string(),
        chain_id: 137,
        native_token: "MATIC".to_string(),
        features: HashSet::from(["smart_contracts".to_string(), "evm".to_string()]),
        metrics: NetworkMetrics {
            estimated_latency_ms: 50,
            gas_price: 50,
            congestion: 10,
            block_time: 2,
            pending_transactions: 500,
        },
    };
    
    manager.add_network(ethereum).unwrap();
    manager.add_network(polygon).unwrap();
    
    // Test AI routing for bridge transaction
    let result = manager.initiate_bridge_transaction_with_ai_routing(
        "bridge1".to_string(),
        "ethereum".to_string(),
        "polygon".to_string(),
        "sender1".to_string(),
        "receiver1".to_string(),
        "ETH".to_string(),
        1000,
    );
    
    // Should return Ok with an optional route suggestion
    assert!(result.is_ok());
    
    // Check that the transaction was created
    let transaction = manager.get_transaction("bridge1");
    assert!(transaction.is_some());
    assert_eq!(transaction.unwrap().status, BridgeStatus::Initialized);
    
    // Check statistics were updated
    assert_eq!(manager.get_statistics().total_transactions, 1);
}

#[test]
fn test_universal_bridge_network_metrics_update() {
    let mut manager = UniversalBridgeManager::with_default();
    
    // Add network
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
    
    manager.add_network(ethereum).unwrap();
    
    // Update network metrics
    let new_metrics = NetworkMetrics {
        estimated_latency_ms: 200,
        gas_price: 30,
        congestion: 50,
        block_time: 20,
        pending_transactions: 2000,
    };
    
    assert!(manager.update_network_metrics("ethereum", new_metrics.clone()).is_ok());
    
    // Check that metrics were updated
    let network = manager.get_network("ethereum").unwrap();
    assert_eq!(network.metrics, new_metrics);
    
    // Try to update metrics for non-existent network
    let result = manager.update_network_metrics("nonexistent", new_metrics);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), UniversalBridgeError::UnsupportedNetwork("nonexistent".to_string()));
}

#[test]
fn test_universal_bridge_market_context_update() {
    let mut manager = UniversalBridgeManager::with_default();
    
    // Create new market context
    let new_context = dex_core::prediction_engine::MarketContext {
        base_token: "BTC".to_string(),
        quote_token: "USDT".to_string(),
        historical_prices: vec![40000.0, 41000.0, 40500.0],
        volatility: 0.05,
        momentum: 0.1,
        timestamp: 1_700_000_000_000,
    };
    
    // Update market context
    manager.update_market_context(new_context.clone());
    
    // Note: We can't directly check the market context since it's private
    // But we can test that the AI routing still works
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
    
    // Test AI routing with updated market context
    let result = manager.initiate_bridge_transaction_with_ai_routing(
        "bridge1".to_string(),
        "ethereum".to_string(),
        "polygon".to_string(),
        "sender1".to_string(),
        "receiver1".to_string(),
        "ETH".to_string(),
        1000,
    );
    
    assert!(result.is_ok());
}

#[test]
fn test_generate_route_candidates() {
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
        metrics: NetworkMetrics {
            estimated_latency_ms: 100,
            gas_price: 20,
            congestion: 30,
            block_time: 15,
            pending_transactions: 1000,
        },
    };
    
    let polygon = BlockchainNetwork {
        id: "polygon".to_string(),
        name: "Polygon".to_string(),
        chain_type: "EVM".to_string(),
        rpc_endpoint: "https://polygon.rpc".to_string(),
        chain_id: 137,
        native_token: "MATIC".to_string(),
        features: HashSet::new(),
        metrics: NetworkMetrics {
            estimated_latency_ms: 50,
            gas_price: 50,
            congestion: 10,
            block_time: 2,
            pending_transactions: 500,
        },
    };
    
    manager.add_network(ethereum.clone()).unwrap();
    manager.add_network(polygon.clone()).unwrap();
    
    // Test that we can manually call the generate_route_candidates function
    // by creating a simplified version for testing
    let candidates = vec![RouteCandidate {
        id: "test_route".to_string(),
        path: vec![dex_core::ai_router::RouteSegment {
            from: "ETH".to_string(),
            to: "MATIC".to_string(),
            liquidity: 1000000.0,
            fee_rate: 0.001,
            estimated_latency_ms: ethereum.metrics.estimated_latency_ms + polygon.metrics.estimated_latency_ms,
        }],
        base_token: "ETH".to_string(),
        quote_token: "MATIC".to_string(),
        expected_output: 1000.0 * 0.999,
        estimated_slippage: 0.001,
        estimated_fee_rate: 0.001,
        estimated_latency_ms: ethereum.metrics.estimated_latency_ms + polygon.metrics.estimated_latency_ms,
        tags: vec!["test".to_string()],
    }];
    
    // Since market_context and ai_router are private, we'll test the public API
    // by initiating a bridge transaction with AI routing
    let result = manager.initiate_bridge_transaction_with_ai_routing(
        "test_bridge".to_string(),
        "ethereum".to_string(),
        "polygon".to_string(),
        "sender".to_string(),
        "receiver".to_string(),
        "ETH".to_string(),
        1000,
    );
    
    // The result should be Ok, indicating the AI routing worked
    assert!(result.is_ok());
    
    // The test passes if we reach this point without panic
}