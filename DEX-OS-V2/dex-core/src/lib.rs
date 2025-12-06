//! DEX-OS core engine library

pub mod amm;
pub mod analytics;
pub mod analytics_security_tests;
pub mod atomic_swaps;
pub mod avl_tree;
pub mod cross_chain_asset_mapping;
pub mod dashboard_queries;
pub mod fee_distribution;
pub mod fee_management;
pub mod gas_abstraction;
pub mod gas_estimators;
pub mod governance;
pub mod identity;
pub mod impermanent_loss_protection;
pub mod iot_wallet;
pub mod lending;
pub mod liquidity_aggregator;
pub mod dex_integration;
pub mod liquidity_provision;
// pub mod swap_contract; // Missing module
pub mod merkle_tree;
pub mod multisig_wallet;
pub mod observability;
pub mod orderbook;
pub mod input_validation;
pub mod partial_fill;
pub mod path_routing;
pub mod payments;
pub mod price_prediction;
pub mod quadratic_voting;
pub mod quantum_consensus;
pub mod ramp_client;
pub mod reference_common;
pub mod reward_distribution;
pub mod security;
pub mod rate_limiting;
pub mod stableswap;
pub mod test_coverage;
pub mod test_results;
pub mod trade_prevention;
pub mod treasury;
pub mod types;
pub mod wallet;
pub mod yield_farming;
pub mod ai_router;
pub mod execution_engine;
pub mod unified_liquidity_os;

// New modules for Priority 3 features
pub mod bulkhead;
pub mod blockchain_resilience;
pub mod consensus;
pub mod crypto;
pub mod distributed_systems;
pub mod event_sourcing;
pub mod consistent_hashing;
pub mod indexer;
pub mod keeper;
pub mod network;
pub mod saga;
pub mod snapshot;
pub mod supply_chain;
pub mod slippage_protection;
pub mod cqrs;
pub mod sre_patterns;
pub mod prediction_engine;
pub mod genesis_pool;
pub mod concentrated_liquidity;
pub mod advanced_defi;
pub mod universal_bridge;

// Bridge Subtypes modules for Priority 3 features
pub mod federated_peg;
pub mod mpc_threshold;
pub mod bridge_subtypes_demo;

// SRE Patterns modules for Canary Releases, Chaos Engineering, and Handling Overload
pub mod canary_release;
pub mod chaos_engineering;
pub mod slippage_calculators;

// Zero-Downtime Deployment modules
pub mod rolling_update;
pub mod feature_toggle;

// WASM Runtime modules for Priority 5 features
pub mod tesla_integration;
pub mod starlink_wallet;
pub mod neuralink_interface;

// Layer 2 Scaling and Cross-Chain Protocols modules for Priority 4 features
pub mod state_channels;
pub mod batch_settlements;
pub mod ibc_protocol;

// Database Infrastructure modules for Priority 5 features
pub mod database;

// Infrastructure Core modules for Priority 5 features
pub mod blockchain_consensus;  // Blockchain Consensus - Transaction Validation
pub mod virtual_dom;           // Virtual DOM - UI Rendering
pub mod state_reducer;         // State Reducer Pattern - State Management

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

// Re-exports
pub use amm::*;
pub use orderbook::*;
pub use path_routing::*;
pub use types::*;
pub use advanced_defi::*;

pub mod staking_contracts;

pub mod cross_asset_verification;

pub mod walletconnect_protocol;

pub mod signature_verifiers;

pub mod gas_abstraction__meta_transactions_;

pub mod react_vue_js_ui;

pub mod web3_js_ethers_js_libraries;

pub mod real_time_charting__e_g__tradingview_api_;

pub mod fee_distribution__0_3__per_swap_;

pub mod reward_emission_curves;
