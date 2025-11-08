//! DEX-OS core engine library

pub mod amm;
pub mod atomic_swaps;
pub mod avl_tree;
pub mod cross_chain_asset_mapping;
pub mod fee_distribution;
pub mod fee_management;
pub mod gas_abstraction;
pub mod governance;
pub mod identity;
pub mod lending;
pub mod merkle_tree;
pub mod multisig_wallet;
pub mod observability;
pub mod orderbook;
pub mod partial_fill;
pub mod path_routing;
pub mod payments;
pub mod price_prediction;
pub mod quadratic_voting;
pub mod quantum_consensus;
pub mod reference_common;
pub mod reward_distribution;
pub mod security;
pub mod stableswap;
pub mod test_results;
pub mod trade_prevention;
pub mod treasury;
pub mod types;

// New modules for Priority 3 features
pub mod consensus;
pub mod crypto;
pub mod indexer;
pub mod keeper;
pub mod network;
pub mod snapshot;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
