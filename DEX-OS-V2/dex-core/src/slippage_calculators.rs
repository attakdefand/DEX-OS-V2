//! Slippage calculators for the DEX aggregator.
//!
//! Implements the Priority 4 feature from `DEX-OS-V2.csv`:
//! - `4,Core Trading,DEX Aggregator,DEX Aggregator,Slippage Calculators,Slippage Calculation,High`
//!
//! These calculators provide pre-trade slippage budgeting so callers can
//! generate limit prices and compute the maximum executable size that remains
//! inside a specified slippage tolerance.

use crate::liquidity_aggregator::{
    AggregatedOrderBook, AggregatorError, LiquidityAggregator, OrderLevel, SlippageResult,
};
use crate::types::{OrderSide, TradingPair};
use thiserror::Error;

/// Slippage quote aligned to a caller-provided tolerance.
#[derive(Debug, Clone)]
pub struct SlippageQuote {
    /// Computed slippage metrics for the requested size.
    pub slippage: SlippageResult,
    /// Limit price that enforces the tolerance (worst acceptable execution).
    pub limit_price: f64,
    /// Tolerance used for the computation, in basis points.
    pub tolerance_bps: f64,
}

/// Maximum executable size that respects a slippage tolerance.
#[derive(Debug, Clone)]
pub struct SlippageCapacity {
    /// Maximum quantity that can be filled without breaching tolerance.
    pub max_quantity: u64,
    /// Slippage metrics at `max_quantity`.
    pub slippage: SlippageResult,
    /// Tolerance used for the computation, in basis points.
    pub tolerance_bps: f64,
    /// Indicates the search stopped because the tolerance was hit (rather than running out of liquidity).
    pub hit_tolerance: bool,
}

/// Stateful slippage calculator that derives results from the liquidity aggregator.
#[derive(Debug, Clone)]
pub struct SlippageCalculator<'a> {
    aggregator: &'a LiquidityAggregator,
}

impl<'a> SlippageCalculator<'a> {
    /// Create a new calculator backed by the given aggregator.
    pub fn new(aggregator: &'a LiquidityAggregator) -> Self {
        Self { aggregator }
    }

    /// Produce a slippage quote that must remain within `max_slippage_bps`.
    ///
    /// Returns the computed slippage plus a limit price callers can attach to
    /// enforcement logic (e.g., limit orders or execution guards).
    pub fn quote_with_tolerance(
        &self,
        pair: &TradingPair,
        side: OrderSide,
        quantity: u64,
        max_slippage_bps: f64,
    ) -> Result<SlippageQuote, SlippageCalculatorError> {
        let slippage = self
            .aggregator
            .calculate_slippage(pair, side, quantity)
            .map_err(SlippageCalculatorError::from)?;

        if slippage.slippage_bps > max_slippage_bps {
            return Err(SlippageCalculatorError::ToleranceExceeded {
                actual: slippage.slippage_bps,
                tolerance: max_slippage_bps,
            });
        }

        let limit_price = match side {
            OrderSide::Buy => slippage.best_price as f64 * (1.0 + max_slippage_bps / 10_000.0),
            OrderSide::Sell => slippage.best_price as f64 * (1.0 - max_slippage_bps / 10_000.0),
        };

        Ok(SlippageQuote {
            slippage,
            limit_price,
            tolerance_bps: max_slippage_bps,
        })
    }

    /// Determine the maximum fillable quantity that stays within `max_slippage_bps`.
    ///
    /// The calculator walks aggregated order book levels, consuming them until
    /// either the tolerance is hit (potentially with a partial level fill) or
    /// liquidity is exhausted.
    pub fn max_fill_for_tolerance(
        &self,
        pair: &TradingPair,
        side: OrderSide,
        max_slippage_bps: f64,
    ) -> Result<SlippageCapacity, SlippageCalculatorError> {
        let book = self
            .aggregator
            .aggregated_book(pair)
            .map_err(SlippageCalculatorError::from)?;
        let levels = levels_for_side(&book, side);
        let best_price = levels
            .first()
            .map(|l| l.price)
            .ok_or(SlippageCalculatorError::Aggregator(
                AggregatorError::NoLiquidity,
            ))?;

        let mut filled: u64 = 0;
        let mut cost: u128 = 0;
        let mut levels_used = 0usize;
        let mut worst_price = best_price;
        let mut hit_tolerance = false;

        for level in levels {
            let prospective_cost = cost + (level.price as u128) * (level.quantity as u128);
            let prospective_qty = filled + level.quantity;
            let prospective_slippage =
                compute_slippage_bps(prospective_cost, prospective_qty, best_price, side);

            if prospective_slippage <= max_slippage_bps + f64::EPSILON {
                cost = prospective_cost;
                filled = prospective_qty;
                worst_price = level.price;
                levels_used += 1;
                continue;
            }

            // Try to consume a partial amount from this level that still respects tolerance.
            let allowed = allowed_partial_fill(
                cost,
                filled,
                level,
                best_price,
                side,
                max_slippage_bps,
            );

            if allowed > 0 {
                cost += (level.price as u128) * (allowed as u128);
                filled += allowed;
                worst_price = level.price;
                levels_used += 1;
            }

            hit_tolerance = true;
            break;
        }

        if filled == 0 {
            return Err(SlippageCalculatorError::NoFillWithinTolerance {
                tolerance_bps: max_slippage_bps,
            });
        }

        let average_price = cost as f64 / filled as f64;
        let slippage_bps = compute_slippage_bps(cost, filled, best_price, side);

        Ok(SlippageCapacity {
            max_quantity: filled,
            slippage: SlippageResult {
                average_price,
                best_price,
                worst_price,
                slippage_bps,
                levels_consumed: levels_used,
                filled_quantity: filled,
            },
            tolerance_bps: max_slippage_bps,
            hit_tolerance,
        })
    }
}

#[derive(Debug, Error)]
pub enum SlippageCalculatorError {
    #[error("slippage {actual:.2} bps exceeds tolerance {tolerance:.2} bps")]
    ToleranceExceeded { actual: f64, tolerance: f64 },
    #[error("no fill available within {tolerance_bps:.2} bps tolerance")]
    NoFillWithinTolerance { tolerance_bps: f64 },
    #[error("aggregator error: {0}")]
    Aggregator(#[from] AggregatorError),
}

fn levels_for_side<'a>(book: &'a AggregatedOrderBook, side: OrderSide) -> &'a [OrderLevel] {
    match side {
        OrderSide::Buy => &book.asks,
        OrderSide::Sell => &book.bids,
    }
}

fn compute_slippage_bps(cost: u128, quantity: u64, best_price: u64, side: OrderSide) -> f64 {
    let average_price = cost as f64 / quantity as f64;
    let best_price = best_price as f64;

    let raw_slippage = match side {
        OrderSide::Buy => (average_price - best_price) / best_price,
        OrderSide::Sell => (best_price - average_price) / best_price,
    };

    raw_slippage * 10_000.0
}

fn allowed_partial_fill(
    current_cost: u128,
    current_qty: u64,
    level: &OrderLevel,
    best_price: u64,
    side: OrderSide,
    max_slippage_bps: f64,
) -> u64 {
    if current_qty == 0 {
        // First level always fits into tolerance because slippage is zero.
        return level.quantity;
    }

    let tolerance_ratio = max_slippage_bps / 10_000.0;
    let cost_f = current_cost as f64;
    let qty_f = current_qty as f64;
    let level_price_f = level.price as f64;
    let best_price_f = best_price as f64;

    match side {
        OrderSide::Buy => {
            let allowed_average = best_price_f * (1.0 + tolerance_ratio);
            let numerator = allowed_average * qty_f - cost_f;
            let denominator = level_price_f - allowed_average;

            if numerator <= 0.0 {
                return 0;
            }

            if denominator <= 0.0 {
                return level.quantity;
            }

            let max_delta = (numerator / denominator).floor();
            max_delta.max(0.0).min(level.quantity as f64).floor() as u64
        }
        OrderSide::Sell => {
            let minimum_average = best_price_f * (1.0 - tolerance_ratio);
            let numerator = cost_f - minimum_average * qty_f;
            let denominator = minimum_average - level_price_f;

            if numerator <= 0.0 {
                return 0;
            }

            if denominator <= 0.0 {
                return level.quantity;
            }

            let max_delta = (numerator / denominator).floor();
            max_delta.max(0.0).min(level.quantity as f64).floor() as u64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::liquidity_aggregator::OrderLevel;

    fn pair() -> TradingPair {
        TradingPair {
            base: "BASE".to_string(),
            quote: "QUOTE".to_string(),
        }
    }

    fn seeded_aggregator() -> LiquidityAggregator {
        let mut agg = LiquidityAggregator::new();
        let p = pair();

        agg.upsert_venue_book(
            "venue-a".to_string(),
            p.clone(),
            vec![
                OrderLevel {
                    price: 100,
                    quantity: 50,
                },
                OrderLevel {
                    price: 99,
                    quantity: 50,
                },
            ],
            vec![
                OrderLevel {
                    price: 101,
                    quantity: 50,
                },
                OrderLevel {
                    price: 105,
                    quantity: 100,
                },
            ],
        );

        agg
    }

    #[test]
    fn quotes_within_tolerance_and_limit_price() {
        let agg = seeded_aggregator();
        let calc = SlippageCalculator::new(&agg);

        let quote = calc
            .quote_with_tolerance(&pair(), OrderSide::Buy, 40, 50.0)
            .expect("slippage should be inside tolerance");

        assert_eq!(quote.slippage.best_price, 101);
        assert!(quote.slippage.slippage_bps <= 50.0);
        assert!((quote.limit_price - 101.0 * 1.005).abs() < 1e-6);
        assert_eq!(quote.tolerance_bps, 50.0);
    }

    #[test]
    fn rejects_quote_exceeding_tolerance() {
        let agg = seeded_aggregator();
        let calc = SlippageCalculator::new(&agg);

        let err = calc
            .quote_with_tolerance(&pair(), OrderSide::Buy, 120, 10.0)
            .unwrap_err();

        matches!(err, SlippageCalculatorError::ToleranceExceeded { .. });
    }

    #[test]
    fn caps_fill_for_buy_side_when_tolerance_hit() {
        let agg = seeded_aggregator();
        let calc = SlippageCalculator::new(&agg);

        // Tolerance of 100 bps should partially consume the second ask level.
        let capacity = calc
            .max_fill_for_tolerance(&pair(), OrderSide::Buy, 100.0)
            .expect("should find capacity");

        assert_eq!(capacity.max_quantity, 66);
        assert!(capacity.hit_tolerance);
        assert_eq!(capacity.slippage.levels_consumed, 2);
        assert!(capacity.slippage.slippage_bps <= 100.0 + 1e-6);
    }

    #[test]
    fn caps_fill_for_sell_side_when_tolerance_hit() {
        let agg = seeded_aggregator();
        let calc = SlippageCalculator::new(&agg);

        let capacity = calc
            .max_fill_for_tolerance(&pair(), OrderSide::Sell, 10.0)
            .expect("should find capacity");

        assert_eq!(capacity.max_quantity, 55);
        assert!(capacity.hit_tolerance);
        assert_eq!(capacity.slippage.levels_consumed, 2);
        assert!(capacity.slippage.slippage_bps <= 10.0 + 1e-6);
    }

    #[test]
    fn errors_when_no_liquidity_present() {
        let agg = LiquidityAggregator::new();
        let calc = SlippageCalculator::new(&agg);

        let err = calc
            .max_fill_for_tolerance(&pair(), OrderSide::Buy, 25.0)
            .unwrap_err();

        matches!(err, SlippageCalculatorError::Aggregator(_));
    }
}
