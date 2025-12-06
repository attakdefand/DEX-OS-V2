//! Slippage protection policy for the DEX aggregator to guarantee trade safety.
//!
//! Implements Priority 4 feature:
//! - `4,Core Trading,DEX Aggregator,DEX Aggregator,Slippage Protection,Trade Safety,High`
//! - Ensures that aggregator-driven trades stay within configured slippage limits and
//!   banishes trade plans that would consume too many price levels or exceed safety thresholds.
use crate::liquidity_aggregator::{AggregatorError, LiquidityAggregator, SlippageResult};
use crate::types::{OrderSide, TradingPair};
use thiserror::Error;

/// Configuration for slippage protection policy enforcement.
#[derive(Debug, Clone)]
pub struct SlippageProtectionConfig {
    /// Maximum approved slippage in basis points (fractions of 100).
    pub max_slippage_bps: f64,
    /// Optional cap on the number of order book levels consumed.
    pub max_levels_consumed: Option<usize>,
}

impl Default for SlippageProtectionConfig {
    fn default() -> Self {
        Self {
            max_slippage_bps: 50.0,
            max_levels_consumed: None,
        }
    }
}

/// Runtime policy enforcing slippage protection on proposed trades.
#[derive(Debug, Clone)]
pub struct SlippageProtection {
    config: SlippageProtectionConfig,
}

impl SlippageProtection {
    /// Create a new slippage protection guard with the supplied configuration.
    pub fn new(config: SlippageProtectionConfig) -> Self {
        Self { config }
    }

    /// Evaluate a trade plan and ensure it complies with the configured safety thresholds.
    ///
    /// * `override_tolerance_bps` allows per-trade tighter tolerances; default is the config.
    pub fn evaluate_trade(
        &self,
        aggregator: &LiquidityAggregator,
        pair: &TradingPair,
        side: OrderSide,
        quantity: u64,
        override_tolerance_bps: Option<f64>,
    ) -> Result<SlippageProtectionReport, SlippageProtectionError> {
        let tolerance = override_tolerance_bps.unwrap_or(self.config.max_slippage_bps);
        let slippage = aggregator.calculate_slippage(pair, side, quantity)?;

        if slippage.slippage_bps > tolerance {
            return Err(SlippageProtectionError::SlippageExceeded {
                actual: slippage.slippage_bps,
                tolerance,
            });
        }

        if let Some(max_levels) = self.config.max_levels_consumed {
            if slippage.levels_consumed > max_levels {
                return Err(SlippageProtectionError::ExcessiveLevels {
                    levels: slippage.levels_consumed,
                    max_levels,
                });
            }
        }

        Ok(SlippageProtectionReport {
            slippage,
            tolerance_bps: tolerance,
        })
    }
}

/// Outcome of a slippage protection evaluation.
#[derive(Debug, Clone)]
pub struct SlippageProtectionReport {
    /// Slippage metrics computed by the liquidity aggregator.
    pub slippage: SlippageResult,
    /// Tolerance used for the evaluation.
    pub tolerance_bps: f64,
}

/// Errors produced when a trade violates slippage protection boundaries.
#[derive(Debug, Error)]
pub enum SlippageProtectionError {
    #[error("slippage {actual:.2} bps exceeds tolerance {tolerance:.2} bps")]
    SlippageExceeded { actual: f64, tolerance: f64 },
    #[error("trade consumes {levels} levels which exceeds the configured limit {max_levels}")]
    ExcessiveLevels { levels: usize, max_levels: usize },
    #[error("liquidity aggregator error: {0}")]
    Aggregator(#[from] AggregatorError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::liquidity_aggregator::OrderLevel;

    fn sample_pair() -> TradingPair {
        TradingPair {
            base: "BASE".to_string(),
            quote: "QUOTE".to_string(),
        }
    }

    fn sample_aggregator() -> LiquidityAggregator {
        let mut agg = LiquidityAggregator::new();
        let p = sample_pair();

        agg.upsert_venue_book(
            "venue-a".to_string(),
            p.clone(),
            vec![
                OrderLevel {
                    price: 100,
                    quantity: 100,
                },
                OrderLevel {
                    price: 101,
                    quantity: 100,
                },
            ],
            vec![
                OrderLevel {
                    price: 102,
                    quantity: 120,
                },
                OrderLevel {
                    price: 103,
                    quantity: 100,
                },
            ],
        );

        agg
    }

    #[test]
    fn respects_slippage_tolerance() {
        let protector = SlippageProtection::new(SlippageProtectionConfig {
            max_slippage_bps: 500.0,
            max_levels_consumed: None,
        });
        let agg = sample_aggregator();
        let report = protector
            .evaluate_trade(&agg, &sample_pair(), OrderSide::Buy, 150, None)
            .expect("should pass tolerance");

        assert!(report.slippage.slippage_bps <= 500.0);
        assert_eq!(report.tolerance_bps, 500.0);
    }

    #[test]
    fn rejects_trade_with_high_slippage() {
        let protector = SlippageProtection::new(SlippageProtectionConfig {
            max_slippage_bps: 100.0,
            max_levels_consumed: None,
        });
        let agg = sample_aggregator();
        let err = protector
            .evaluate_trade(
                &agg,
                &sample_pair(),
                OrderSide::Buy,
                150,
                Some(10.0),
            )
            .unwrap_err();

        matches!(err, SlippageProtectionError::SlippageExceeded { .. });
    }

    #[test]
    fn enforces_level_limit() {
        let protector = SlippageProtection::new(SlippageProtectionConfig {
            max_slippage_bps: 500.0,
            max_levels_consumed: Some(1),
        });
        let agg = sample_aggregator();
        let err = protector
            .evaluate_trade(&agg, &sample_pair(), OrderSide::Buy, 150, None)
            .unwrap_err();

        matches!(err, SlippageProtectionError::ExcessiveLevels { .. });
    }
}
