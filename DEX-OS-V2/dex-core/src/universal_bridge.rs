//! Universal Bridge implementation for DEX-OS
//!
//! This module implements the Priority 3 features from DEX-OS-V2.csv:
//! - "Core Components,Universal Bridge,Bridge,10,000+ Chain Integrations,Multi-Chain Integration,High"
//! - "Core Components,Universal Bridge,Bridge,AI Routing,Routing,High"
//!
//! It provides functionality for seamless cross-chain asset transfers and interactions
//! with support for 10,000+ blockchain networks through a unified interface.

use crate::atomic_swaps::AtomicSwapManager;
use crate::cross_chain_asset_mapping::CrossChainAssetMapper;
use crate::ai_router::{AiRouter, RouteCandidate, RouteEvaluationContext, RouteSegment, RouteSuggestion};
use crate::prediction_engine::{MarketContext, PredictionEngine};
use crate::types::{TokenId, TraderId, Quantity, TradingPair};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Represents a blockchain network
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockchainNetwork {
    /// Unique identifier for the chain
    pub id: String,
    /// Human-readable name of the chain
    pub name: String,
    /// Chain type (EVM, Substrate, Cosmos, etc.)
    pub chain_type: String,
    /// RPC endpoint for the chain
    pub rpc_endpoint: String,
    /// Chain ID for transaction signing
    pub chain_id: u64,
    /// Native token of the chain
    pub native_token: TokenId,
    /// Supported features
    pub features: HashSet<String>,
    /// Network metrics for AI routing
    pub metrics: NetworkMetrics,
}

/// Network metrics for AI routing evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkMetrics {
    /// Estimated latency in milliseconds
    pub estimated_latency_ms: u64,
    /// Gas price in wei or equivalent
    pub gas_price: u64,
    /// Network congestion level (0-100)
    pub congestion: u32,
    /// Block time in seconds
    pub block_time: u32,
    /// Pending transactions count
    pub pending_transactions: u64,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            estimated_latency_ms: 100,
            gas_price: 20,
            congestion: 30,
            block_time: 15,
            pending_transactions: 1000,
        }
    }
}

/// Bridge configuration
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Maximum number of concurrent bridges
    pub max_concurrent_bridges: usize,
    /// Default timeout for bridge operations (in seconds)
    pub default_timeout_secs: u64,
    /// Retry attempts for failed operations
    pub retry_attempts: u32,
    /// Minimum confirmation blocks required
    pub min_confirmations: u32,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_bridges: 10000,
            default_timeout_secs: 3600, // 1 hour
            retry_attempts: 3,
            min_confirmations: 12,
        }
    }
}

/// Status of a bridge operation
#[derive(Debug, Clone, PartialEq)]
pub enum BridgeStatus {
    /// Bridge is initialized but not yet active
    Initialized,
    /// Bridge is active and ready for operations
    Active,
    /// Bridge operation is in progress
    Processing,
    /// Bridge operation completed successfully
    Completed,
    /// Bridge operation failed
    Failed,
    /// Bridge operation timed out
    Timeout,
    /// Bridge is paused
    Paused,
}

/// Represents a bridge transaction
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeTransaction {
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
    pub status: BridgeStatus,
    /// Associated atomic swap ID (if applicable)
    pub swap_id: Option<String>,
    /// Transaction hash on source chain
    pub source_tx_hash: Option<String>,
    /// Transaction hash on destination chain
    pub destination_tx_hash: Option<String>,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Universal Bridge Manager
#[derive(Debug)]
pub struct UniversalBridgeManager {
    /// Configuration for the bridge
    config: BridgeConfig,
    /// Supported blockchain networks
    networks: HashMap<String, BlockchainNetwork>,
    /// Cross-chain asset mappings
    asset_mapper: CrossChainAssetMapper,
    /// Atomic swap manager for cross-chain swaps
    swap_manager: AtomicSwapManager,
    /// Active bridge transactions
    transactions: HashMap<String, BridgeTransaction>,
    /// Completed bridge transactions
    completed_transactions: HashMap<String, BridgeTransaction>,
    /// Bridge statistics
    stats: BridgeStatistics,
    /// AI Router for route optimization
    ai_router: AiRouter,
    /// Market context for AI predictions
    market_context: MarketContext,
}

/// Statistics for bridge operations
#[derive(Debug, Clone)]
pub struct BridgeStatistics {
    /// Total number of bridge transactions
    pub total_transactions: u64,
    /// Number of successful transactions
    pub successful_transactions: u64,
    /// Number of failed transactions
    pub failed_transactions: u64,
    /// Total volume bridged
    pub total_volume: HashMap<TokenId, Quantity>,
    /// Average transaction time (in seconds)
    pub avg_transaction_time: f64,
}

/// Errors that can occur in the universal bridge
#[derive(Debug, Error, PartialEq)]
pub enum UniversalBridgeError {
    #[error("Blockchain network not supported: {0}")]
    UnsupportedNetwork(String),
    #[error("Insufficient funds for bridge transaction")]
    InsufficientFunds,
    #[error("Bridge transaction not found")]
    TransactionNotFound,
    #[error("Bridge transaction already exists")]
    TransactionAlreadyExists,
    #[error("Invalid bridge configuration")]
    InvalidConfiguration,
    #[error("Bridge operation timed out")]
    Timeout,
    #[error("Maximum concurrent bridges exceeded")]
    MaxConcurrentBridgesExceeded,
    #[error("Cross-chain asset mapping error: {0}")]
    AssetMappingError(String),
    #[error("Atomic swap error: {0}")]
    AtomicSwapError(String),
}

impl UniversalBridgeManager {
    /// Create a new Universal Bridge Manager
    pub fn new(config: BridgeConfig) -> Self {
        // Create a default prediction engine for the AI router
        let models: Vec<Box<dyn crate::prediction_engine::Predictor>> = vec![
            Box::new(crate::prediction_engine::TransformerPredictor::new("transformer", 1.0, 42)),
            Box::new(crate::prediction_engine::ReinforcementLearningPredictor::new("rl", 24)),
        ];
        let prediction_engine = PredictionEngine::new(models, crate::prediction_engine::AggregationStrategy::ConfidenceWeighted);
        let ai_router = AiRouter::with_default(prediction_engine);
        
        Self {
            config,
            networks: HashMap::new(),
            asset_mapper: CrossChainAssetMapper::new(),
            swap_manager: AtomicSwapManager::new(),
            transactions: HashMap::new(),
            completed_transactions: HashMap::new(),
            stats: BridgeStatistics {
                total_transactions: 0,
                successful_transactions: 0,
                failed_transactions: 0,
                total_volume: HashMap::new(),
                avg_transaction_time: 0.0,
            },
            ai_router,
            market_context: MarketContext {
                base_token: "ETH".to_string(), // Default base token
                quote_token: "USDC".to_string(), // Default quote token
                historical_prices: vec![],
                volatility: 0.0,
                momentum: 0.0,
                timestamp: 0,
            },
        }
    }

    /// Create a new Universal Bridge Manager with default configuration
    pub fn with_default() -> Self {
        Self::new(BridgeConfig::default())
    }

    /// Add a blockchain network to the bridge
    pub fn add_network(&mut self, network: BlockchainNetwork) -> Result<(), UniversalBridgeError> {
        // Validate network
        if network.id.is_empty() || network.name.is_empty() || network.rpc_endpoint.is_empty() {
            return Err(UniversalBridgeError::InvalidConfiguration);
        }

        self.networks.insert(network.id.clone(), network);
        Ok(())
    }

    /// Get a blockchain network by ID
    pub fn get_network(&self, network_id: &str) -> Option<&BlockchainNetwork> {
        self.networks.get(network_id)
    }

    /// Get all supported networks
    pub fn get_supported_networks(&self) -> Vec<&BlockchainNetwork> {
        self.networks.values().collect()
    }

    /// Check if a network is supported
    pub fn is_network_supported(&self, network_id: &str) -> bool {
        self.networks.contains_key(network_id)
    }

    /// Get the number of supported networks
    pub fn network_count(&self) -> usize {
        self.networks.len()
    }

    /// Initiate a bridge transaction
    pub fn initiate_bridge_transaction(
        &mut self,
        id: String,
        source_chain: String,
        destination_chain: String,
        sender: TraderId,
        receiver: TraderId,
        token_id: TokenId,
        amount: Quantity,
    ) -> Result<(), UniversalBridgeError> {
        // Check if source and destination chains are supported
        if !self.is_network_supported(&source_chain) {
            return Err(UniversalBridgeError::UnsupportedNetwork(source_chain));
        }

        if !self.is_network_supported(&destination_chain) {
            return Err(UniversalBridgeError::UnsupportedNetwork(destination_chain));
        }

        // Check if we've exceeded maximum concurrent bridges
        if self.transactions.len() >= self.config.max_concurrent_bridges {
            return Err(UniversalBridgeError::MaxConcurrentBridgesExceeded);
        }

        // Check if transaction already exists
        if self.transactions.contains_key(&id) || self.completed_transactions.contains_key(&id) {
            return Err(UniversalBridgeError::TransactionAlreadyExists);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let transaction = BridgeTransaction {
            id: id.clone(),
            source_chain,
            destination_chain,
            sender,
            receiver,
            token_id: token_id.clone(),
            amount,
            initiated_timestamp: now,
            completed_timestamp: None,
            status: BridgeStatus::Initialized,
            swap_id: None,
            source_tx_hash: None,
            destination_tx_hash: None,
            error_message: None,
        };

        self.transactions.insert(id, transaction);
        
        // Update statistics
        self.stats.total_transactions += 1;
        let current_volume = self.stats.total_volume.get(&token_id).copied().unwrap_or(0);
        self.stats.total_volume.insert(token_id, current_volume + amount);

        Ok(())
    }

    /// Activate a bridge transaction
    pub fn activate_bridge_transaction(&mut self, id: &str) -> Result<(), UniversalBridgeError> {
        let transaction = self
            .transactions
            .get_mut(id)
            .ok_or(UniversalBridgeError::TransactionNotFound)?;

        // Check if transaction is in the correct state
        if transaction.status != BridgeStatus::Initialized {
            return Err(UniversalBridgeError::InvalidConfiguration);
        }

        // Update status
        transaction.status = BridgeStatus::Active;

        Ok(())
    }

    /// Process a bridge transaction (this would interact with actual blockchain networks)
    pub fn process_bridge_transaction(&mut self, id: &str) -> Result<(), UniversalBridgeError> {
        let transaction = self
            .transactions
            .get_mut(id)
            .ok_or(UniversalBridgeError::TransactionNotFound)?;

        // Check if transaction is in the correct state
        if transaction.status != BridgeStatus::Active {
            return Err(UniversalBridgeError::InvalidConfiguration);
        }

        // Update status
        transaction.status = BridgeStatus::Processing;

        // In a real implementation, this would:
        // 1. Lock assets on source chain
        // 2. Initiate atomic swap or other cross-chain mechanism
        // 3. Monitor for completion
        // 4. Release assets on destination chain

        Ok(())
    }

    /// Complete a bridge transaction
    pub fn complete_bridge_transaction(
        &mut self,
        id: &str,
        source_tx_hash: Option<String>,
        destination_tx_hash: Option<String>,
    ) -> Result<(), UniversalBridgeError> {
        let mut transaction = self
            .transactions
            .remove(id)
            .ok_or(UniversalBridgeError::TransactionNotFound)?;

        // Update transaction details
        transaction.status = BridgeStatus::Completed;
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

        // Update statistics
        self.stats.successful_transactions += 1;

        Ok(())
    }

    /// Fail a bridge transaction
    pub fn fail_bridge_transaction(
        &mut self,
        id: &str,
        error_message: String,
    ) -> Result<(), UniversalBridgeError> {
        let mut transaction = self
            .transactions
            .remove(id)
            .ok_or(UniversalBridgeError::TransactionNotFound)?;

        // Update transaction details
        transaction.status = BridgeStatus::Failed;
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

        // Update statistics
        self.stats.failed_transactions += 1;

        Ok(())
    }

    /// Get a bridge transaction by ID
    pub fn get_transaction(&self, id: &str) -> Option<&BridgeTransaction> {
        self.transactions.get(id)
    }

    /// Get a completed bridge transaction by ID
    pub fn get_completed_transaction(&self, id: &str) -> Option<&BridgeTransaction> {
        self.completed_transactions.get(id)
    }

    /// Get all active transactions for a trader
    pub fn get_transactions_for_trader(&self, trader_id: &TraderId) -> Vec<&BridgeTransaction> {
        self.transactions
            .values()
            .filter(|tx| &tx.sender == trader_id || &tx.receiver == trader_id)
            .collect()
    }

    /// Get bridge statistics
    pub fn get_statistics(&self) -> &BridgeStatistics {
        &self.stats
    }

    /// Pause the bridge (for maintenance or emergencies)
    pub fn pause(&mut self) {
        // In a real implementation, this would pause all operations
    }

    /// Resume the bridge
    pub fn resume(&mut self) {
        // In a real implementation, this would resume operations
    }

    /// Check if the bridge supports 10,000+ chains
    pub fn supports_massive_integration(&self) -> bool {
        self.networks.len() >= 10000
    }

    /// Initiate a bridge transaction with AI routing optimization
    pub fn initiate_bridge_transaction_with_ai_routing(
        &mut self,
        id: String,
        source_chain: String,
        destination_chain: String,
        sender: TraderId,
        receiver: TraderId,
        token_id: TokenId,
        amount: Quantity,
    ) -> Result<Option<RouteSuggestion>, UniversalBridgeError> {
        // Check if source and destination chains are supported
        if !self.is_network_supported(&source_chain) {
            return Err(UniversalBridgeError::UnsupportedNetwork(source_chain));
        }

        if !self.is_network_supported(&destination_chain) {
            return Err(UniversalBridgeError::UnsupportedNetwork(destination_chain));
        }

        // Check if we've exceeded maximum concurrent bridges
        if self.transactions.len() >= self.config.max_concurrent_bridges {
            return Err(UniversalBridgeError::MaxConcurrentBridgesExceeded);
        }

        // Check if transaction already exists
        if self.transactions.contains_key(&id) || self.completed_transactions.contains_key(&id) {
            return Err(UniversalBridgeError::TransactionAlreadyExists);
        }

        // Get network information for route evaluation
        let source_network = self.get_network(&source_chain).unwrap().clone();
        let dest_network = self.get_network(&destination_chain).unwrap().clone();

        // Create route candidates for AI evaluation
        let candidates = self.generate_route_candidates(&source_network, &dest_network, &token_id, amount);

        // Create route evaluation context
        let ctx = RouteEvaluationContext {
            pair: TradingPair {
                base: token_id.clone(),
                quote: dest_network.native_token.clone(),
            },
            base_amount: amount as f64,
            market_context: self.market_context.clone(),
        };

        // Use AI router to select the optimal route
        let optimal_route = self.ai_router.select_route(&ctx, &candidates);

        // If we have an optimal route, initiate the transaction
        if optimal_route.is_some() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let transaction = BridgeTransaction {
                id: id.clone(),
                source_chain,
                destination_chain,
                sender,
                receiver,
                token_id: token_id.clone(),
                amount,
                initiated_timestamp: now,
                completed_timestamp: None,
                status: BridgeStatus::Initialized,
                swap_id: None,
                source_tx_hash: None,
                destination_tx_hash: None,
                error_message: None,
            };

            self.transactions.insert(id, transaction);
            
            // Update statistics
            self.stats.total_transactions += 1;
            let current_volume = self.stats.total_volume.get(&token_id).copied().unwrap_or(0);
            self.stats.total_volume.insert(token_id, current_volume + amount);
        }

        Ok(optimal_route)
    }

    /// Generate route candidates for AI evaluation
    fn generate_route_candidates(
        &self,
        source_network: &BlockchainNetwork,
        dest_network: &BlockchainNetwork,
        token_id: &TokenId,
        amount: Quantity,
    ) -> Vec<RouteCandidate> {
        let mut candidates = Vec::new();

        // Direct bridge route
        let direct_segment = RouteSegment {
            from: token_id.clone(),
            to: dest_network.native_token.clone(),
            liquidity: 1000000.0, // Placeholder value
            fee_rate: 0.001, // 0.1% fee
            estimated_latency_ms: source_network.metrics.estimated_latency_ms + dest_network.metrics.estimated_latency_ms,
        };

        let direct_candidate = RouteCandidate {
            id: "direct_bridge".to_string(),
            path: vec![direct_segment],
            base_token: token_id.clone(),
            quote_token: dest_network.native_token.clone(),
            expected_output: amount as f64 * 0.999, // Account for fees
            estimated_slippage: 0.001, // 0.1% slippage
            estimated_fee_rate: 0.001,
            estimated_latency_ms: source_network.metrics.estimated_latency_ms + dest_network.metrics.estimated_latency_ms,
            tags: vec!["direct".to_string(), "bridge".to_string()],
        };

        candidates.push(direct_candidate);

        // Add more route candidates here as needed for different bridge paths
        // For example, through a hub chain like Ethereum or Polygon

        candidates
    }

    /// Update market context for AI routing
    pub fn update_market_context(&mut self, context: MarketContext) {
        self.market_context = context;
    }

    /// Update network metrics for AI routing
    pub fn update_network_metrics(&mut self, network_id: &str, metrics: NetworkMetrics) -> Result<(), UniversalBridgeError> {
        if let Some(network) = self.networks.get_mut(network_id) {
            network.metrics = metrics;
            Ok(())
        } else {
            Err(UniversalBridgeError::UnsupportedNetwork(network_id.to_string()))
        }
    }
}

impl Default for UniversalBridgeManager {
    fn default() -> Self {
        Self::with_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_manager_creation() {
        let manager = UniversalBridgeManager::new(BridgeConfig::default());
        assert_eq!(manager.network_count(), 0);
        assert_eq!(manager.get_statistics().total_transactions, 0);
    }

    #[test]
    fn test_add_network() {
        let mut manager = UniversalBridgeManager::new(BridgeConfig::default());
        
        let network = BlockchainNetwork {
            id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://ethereum.rpc".to_string(),
            chain_id: 1,
            native_token: "ETH".to_string(),
            features: HashSet::from(["smart_contracts".to_string(), "evm".to_string()]),
            metrics: NetworkMetrics::default(),
        };

        assert!(manager.add_network(network).is_ok());
        assert_eq!(manager.network_count(), 1);
        assert!(manager.is_network_supported("ethereum"));
    }

    #[test]
    fn test_invalid_network() {
        let mut manager = UniversalBridgeManager::new(BridgeConfig::default());
        
        let invalid_network = BlockchainNetwork {
            id: "".to_string(), // Empty ID
            name: "Ethereum".to_string(),
            chain_type: "EVM".to_string(),
            rpc_endpoint: "https://ethereum.rpc".to_string(),
            chain_id: 1,
            native_token: "ETH".to_string(),
            features: HashSet::new(),
            metrics: NetworkMetrics::default(),
        };

        assert!(manager.add_network(invalid_network).is_err());
        assert_eq!(manager.network_count(), 0);
    }

    #[test]
    fn test_initiate_bridge_transaction() {
        let mut manager = UniversalBridgeManager::new(BridgeConfig::default());
        
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
        assert_eq!(manager.get_statistics().total_transactions, 1);
        
        let transaction = manager.get_transaction("bridge1");
        assert!(transaction.is_some());
        let transaction = transaction.unwrap();
        assert_eq!(transaction.status, BridgeStatus::Initialized);
        assert_eq!(transaction.amount, 1000);
    }

    #[test]
    fn test_unsupported_network() {
        let mut manager = UniversalBridgeManager::new(BridgeConfig::default());
        
        // Add only one network
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
    fn test_activate_and_process_transaction() {
        let mut manager = UniversalBridgeManager::new(BridgeConfig::default());
        
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

        // Process transaction
        assert!(manager.process_bridge_transaction("bridge1").is_ok());
        
        let transaction = manager.get_transaction("bridge1").unwrap();
        assert_eq!(transaction.status, BridgeStatus::Processing);
    }

    #[test]
    fn test_complete_transaction() {
        let mut manager = UniversalBridgeManager::new(BridgeConfig::default());
        
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

        // Complete transaction
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
    }

    #[test]
    fn test_fail_transaction() {
        let mut manager = UniversalBridgeManager::new(BridgeConfig::default());
        
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

        // Fail transaction
        assert!(manager.fail_bridge_transaction(
            "bridge1",
            "Network error occurred".to_string()
        ).is_ok());

        // Transaction should now be in completed transactions with failed status
        assert!(manager.get_transaction("bridge1").is_none());
        assert!(manager.get_completed_transaction("bridge1").is_some());
        
        let completed_tx = manager.get_completed_transaction("bridge1").unwrap();
        assert_eq!(completed_tx.status, BridgeStatus::Failed);
        assert_eq!(completed_tx.error_message, Some("Network error occurred".to_string()));
        
        // Statistics should be updated
        assert_eq!(manager.get_statistics().failed_transactions, 1);
    }

    #[test]
    fn test_get_transactions_for_trader() {
        let mut manager = UniversalBridgeManager::new(BridgeConfig::default());
        
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

        // Initiate bridge transactions
        manager.initiate_bridge_transaction(
            "bridge1".to_string(),
            "ethereum".to_string(),
            "polygon".to_string(),
            "sender1".to_string(), // This trader is the sender
            "receiver1".to_string(),
            "ETH".to_string(),
            1000,
        ).unwrap();
        
        manager.initiate_bridge_transaction(
            "bridge2".to_string(),
            "polygon".to_string(),
            "ethereum".to_string(),
            "sender2".to_string(),
            "sender1".to_string(), // This trader is the receiver
            "MATIC".to_string(),
            2000,
        ).unwrap();

        // Get transactions for sender1 (should get both as sender and receiver)
        let transactions = manager.get_transactions_for_trader(&"sender1".to_string());
        assert_eq!(transactions.len(), 2);
    }

    #[test]
    fn test_bridge_statistics() {
        let mut manager = UniversalBridgeManager::new(BridgeConfig::default());
        
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
}