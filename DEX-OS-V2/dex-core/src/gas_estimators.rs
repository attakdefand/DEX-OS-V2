//! Gas estimators for the DEX Aggregator to surface gas costs alongside pricing.
//!
//! Implements Priority 4 feature:
//! - `4,Core Trading,DEX Aggregator,DEX Aggregator,Gas Estimators,Gas Cost Estimation,High`
//!
//! The estimator models gas usage for multi-hop aggregator routes and slippage-driven fills so
//! routing code can present end-to-end execution costs to callers.

use crate::liquidity_aggregator::SlippageResult;
use crate::path_routing::{RoutingPath, TradingEdge};
use std::collections::HashMap;
use thiserror::Error;

/// Configuration for gas estimation of aggregator trades.
#[derive(Debug, Clone)]
pub struct GasEstimatorConfig {
    /// Base gas for the aggregator dispatch/settlement contract call.
    pub base_call_gas: u64,
    /// Gas consumed per swap hop.
    pub per_hop_swap_gas: u64,
    /// Gas to set token approval when required.
    pub approval_gas: u64,
    /// Transfer overhead per token movement between hops.
    pub transfer_gas: u64,
    /// Settlement overhead per consumed order-book level.
    pub per_level_settlement_gas: u64,
    /// Default gas price when the caller does not provide one (in gwei).
    pub default_gas_price_gwei: u64,
}

impl Default for GasEstimatorConfig {
    fn default() -> Self {
        Self {
            base_call_gas: 80_000,
            per_hop_swap_gas: 120_000,
            approval_gas: 45_000,
            transfer_gas: 21_000,
            per_level_settlement_gas: 8_000,
            default_gas_price_gwei: 30,
        }
    }
}

/// Detailed gas component breakdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GasBreakdown {
    pub base_call: u64,
    pub swaps: u64,
    pub approvals: u64,
    pub transfers: u64,
    pub venue_overhead: u64,
    pub level_settlement: u64,
}

impl GasBreakdown {
    /// Total gas from all components.
    pub fn total(&self) -> u64 {
        self.base_call
            + self.swaps
            + self.approvals
            + self.transfers
            + self.venue_overhead
            + self.level_settlement
    }
}

/// Estimated gas and fee information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GasEstimate {
    pub gas_units: u64,
    pub gas_price_gwei: u64,
    pub total_cost_wei: u128,
    pub breakdown: GasBreakdown,
}

/// Errors surfaced by the gas estimator.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GasEstimatorError {
    #[error("gas price must be greater than zero")]
    GasPriceUnavailable,
}

/// Gas estimator entry point with optional venue-specific overheads.
#[derive(Debug, Default)]
pub struct GasEstimator {
    config: GasEstimatorConfig,
    venue_overheads: HashMap<String, u64>,
}

impl GasEstimator {
    /// Create a new estimator with the provided configuration.
    pub fn new(config: GasEstimatorConfig) -> Self {
        Self {
            config,
            venue_overheads: HashMap::new(),
        }
    }

    /// Set per-venue overhead to account for different router/bridge behaviors.
    pub fn set_venue_overhead(&mut self, venue: String, overhead_gas: u64) {
        self.venue_overheads.insert(venue, overhead_gas);
    }

    /// Estimate gas for a multi-hop routing path.
    pub fn estimate_for_route(
        &self,
        path: &RoutingPath,
        approvals_needed: usize,
        gas_price_override_gwei: Option<u64>,
    ) -> Result<GasEstimate, GasEstimatorError> {
        let swaps = (path.edges.len() as u64) * self.config.per_hop_swap_gas;
        let transfers = ((path.edges.len() as u64) + 1) * self.config.transfer_gas;
        let approvals = (approvals_needed as u64) * self.config.approval_gas;
        let venue_overhead = self.sum_venue_overhead(&path.edges);

        let breakdown = GasBreakdown {
            base_call: self.config.base_call_gas,
            swaps,
            approvals,
            transfers,
            venue_overhead,
            level_settlement: 0,
        };

        self.build_estimate(breakdown, gas_price_override_gwei)
    }

    /// Estimate gas for an aggregated order-book fill using slippage output.
    pub fn estimate_for_slippage(
        &self,
        slippage: &SlippageResult,
        approvals_needed: usize,
        gas_price_override_gwei: Option<u64>,
    ) -> Result<GasEstimate, GasEstimatorError> {
        let approvals = (approvals_needed as u64) * self.config.approval_gas;
        let level_settlement =
            (slippage.levels_consumed as u64) * self.config.per_level_settlement_gas;
        let transfers =
            (slippage.levels_consumed.max(1) as u64) * self.config.transfer_gas;

        let breakdown = GasBreakdown {
            base_call: self.config.base_call_gas,
            swaps: 0,
            approvals,
            transfers,
            venue_overhead: 0,
            level_settlement,
        };

        self.build_estimate(breakdown, gas_price_override_gwei)
    }

    fn build_estimate(
        &self,
        breakdown: GasBreakdown,
        gas_price_override_gwei: Option<u64>,
    ) -> Result<GasEstimate, GasEstimatorError> {
        let gas_price_gwei = self.resolve_gas_price(gas_price_override_gwei)?;
        let gas_units = breakdown.total();
        let total_cost_wei =
            gas_units as u128 * gas_price_gwei as u128 * 1_000_000_000u128;

        Ok(GasEstimate {
            gas_units,
            gas_price_gwei,
            total_cost_wei,
            breakdown,
        })
    }

    fn resolve_gas_price(
        &self,
        gas_price_override_gwei: Option<u64>,
    ) -> Result<u64, GasEstimatorError> {
        let gas_price = gas_price_override_gwei.unwrap_or(self.config.default_gas_price_gwei);
        if gas_price == 0 {
            Err(GasEstimatorError::GasPriceUnavailable)
        } else {
            Ok(gas_price)
        }
    }

    fn sum_venue_overhead(&self, edges: &[TradingEdge]) -> u64 {
        edges
            .iter()
            .map(|edge| self.venue_overheads.get(&edge.dex_name).copied().unwrap_or(0))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path_routing::TradingEdge;

    fn test_config() -> GasEstimatorConfig {
        GasEstimatorConfig {
            base_call_gas: 50_000,
            per_hop_swap_gas: 120_000,
            approval_gas: 45_000,
            transfer_gas: 21_000,
            per_level_settlement_gas: 8_000,
            default_gas_price_gwei: 30,
        }
    }

    fn sample_edge(dex_name: &str) -> TradingEdge {
        TradingEdge {
            from_token: "BASE".to_string(),
            to_token: "QUOTE".to_string(),
            dex_name: dex_name.to_string(),
            exchange_rate: 1.0,
            fee: 0.001,
            liquidity: 100_000,
        }
    }

    fn sample_path() -> RoutingPath {
        RoutingPath {
            edges: vec![sample_edge("Layer2Swap"), sample_edge("MainnetDEX")],
            total_exchange_rate: 1.0,
            total_fee: 0.0,
            min_liquidity: 10_000,
        }
    }

    #[test]
    fn estimates_multi_hop_route_costs() {
        let mut estimator = GasEstimator::new(test_config());
        estimator.set_venue_overhead("Layer2Swap".to_string(), 5_000);

        let estimate = estimator
            .estimate_for_route(&sample_path(), 1, None)
            .expect("gas estimate should succeed");

        assert_eq!(estimate.gas_price_gwei, 30);
        assert_eq!(estimate.breakdown.swaps, 240_000);
        assert_eq!(estimate.breakdown.approvals, 45_000);
        assert_eq!(estimate.breakdown.transfers, 63_000);
        assert_eq!(estimate.breakdown.venue_overhead, 5_000);

        let expected_units = 403_000; // 50k base + 240k swaps + 45k approval + 63k transfers + 5k venue
        assert_eq!(estimate.gas_units, expected_units);
        assert_eq!(
            estimate.total_cost_wei,
            expected_units as u128 * 30u128 * 1_000_000_000u128
        );
    }

    #[test]
    fn estimates_level_based_slippage_costs() {
        let estimator = GasEstimator::new(test_config());

        let slippage = SlippageResult {
            average_price: 101.2,
            best_price: 100,
            worst_price: 102,
            slippage_bps: 75.0,
            levels_consumed: 3,
            filled_quantity: 150,
        };

        let estimate = estimator
            .estimate_for_slippage(&slippage, 0, Some(25))
            .expect("gas estimate should succeed");

        assert_eq!(estimate.gas_price_gwei, 25);
        assert_eq!(estimate.breakdown.approvals, 0);
        assert_eq!(estimate.breakdown.level_settlement, 24_000);
        assert_eq!(estimate.breakdown.transfers, 63_000);

        let expected_units = 137_000; // 50k base + 24k settlement + 63k transfers
        assert_eq!(estimate.gas_units, expected_units);
        assert_eq!(
            estimate.total_cost_wei,
            expected_units as u128 * 25u128 * 1_000_000_000u128
        );
    }

    #[test]
    fn rejects_zero_gas_price() {
        let mut cfg = test_config();
        cfg.default_gas_price_gwei = 0;
        let estimator = GasEstimator::new(cfg);

        let result = estimator.estimate_for_route(&sample_path(), 0, None);
        assert!(matches!(result, Err(GasEstimatorError::GasPriceUnavailable)));
    }
}
