//! Unified Liquidity OS slippage controller for the <0.0001% slippage target.
//!
//! Implements Priority 3 feature:
//! - `3,Main Features,Unified Liquidity OS,Liquidity,<0.0001% Slippage,Slippage Control,High`
//!
//! The controller builds on the existing liquidity aggregator and applies an
//! ultra-low slippage target (0.01 bps, i.e., 0.0001%) by enforcing that only
//! top-of-book liquidity is consumed unless additional depth can be added at
//! the same price (via virtual liquidity buffers).

use crate::liquidity_aggregator::{AggregatorError, LiquidityAggregator, OrderLevel, SlippageResult};
use crate::types::{OrderSide, TradingPair};
use std::collections::HashMap;
use thiserror::Error;

/// Ultra-low slippage target expressed in basis points (0.01 bps = 0.0001%).
pub const ULTRA_LOW_SLIPPAGE_TARGET_BPS: f64 = 0.01;

/// Configuration for the Unified Liquidity OS slippage controller.
#[derive(Debug, Clone)]
pub struct UnifiedLiquidityConfig {
    /// Maximum acceptable slippage in basis points.
    pub target_slippage_bps: f64,
    /// Optional guardrail for how many price levels can be consumed.
    pub max_levels_consumed: Option<usize>,
}

impl Default for UnifiedLiquidityConfig {
    fn default() -> Self {
        Self {
            target_slippage_bps: ULTRA_LOW_SLIPPAGE_TARGET_BPS,
            // Keep fills pinned to the best levels unless explicitly relaxed.
            max_levels_consumed: Some(3),
        }
    }
}

/// Virtual liquidity that can be injected to keep slippage within the target.
#[derive(Debug, Default, Clone)]
pub struct VirtualLiquidityBuffer {
    pub bids: Vec<OrderLevel>,
    pub asks: Vec<OrderLevel>,
}

/// A single execution slice that respects the ultra-low slippage target.
#[derive(Debug, Clone)]
pub struct ExecutionSlice {
    pub quantity: u64,
    pub average_price: f64,
    pub worst_price: u64,
    pub slippage_bps: f64,
    pub levels_consumed: usize,
    pub used_virtual_liquidity: bool,
}

/// Execution plan enforcing the <0.0001% slippage constraint.
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub pair: TradingPair,
    pub side: OrderSide,
    pub requested_quantity: u64,
    pub executed_quantity: u64,
    pub achieved_slippage_bps: f64,
    pub slices: Vec<ExecutionSlice>,
    pub fully_covered: bool,
}

/// Errors returned by the Unified Liquidity OS controller.
#[derive(Debug, Error, PartialEq)]
pub enum UnifiedLiquidityError {
    #[error("no liquidity available for pair {0:?}")]
    NoLiquidity(TradingPair),
    #[error(
        "requested quantity {requested} exceeds ultra-low-slippage capacity {max_executable} \
         (achieved {achieved_slippage_bps:.6} bps vs target {target_slippage_bps:.6} bps)"
    )]
    CannotMeetTarget {
        requested: u64,
        max_executable: u64,
        achieved_slippage_bps: f64,
        target_slippage_bps: f64,
    },
    #[error("slippage control exceeded allowed levels ({levels} > {max_levels})")]
    LevelLimitExceeded { levels: usize, max_levels: usize },
    #[error("aggregator error: {0}")]
    Aggregator(#[from] AggregatorError),
}

/// Unified Liquidity OS orchestrates liquidity to guarantee <0.0001% slippage.
pub struct UnifiedLiquidityOS {
    aggregator: LiquidityAggregator,
    config: UnifiedLiquidityConfig,
    virtual_liquidity: HashMap<TradingPair, VirtualLiquidityBuffer>,
}

impl UnifiedLiquidityOS {
    /// Create a new Unified Liquidity OS controller.
    pub fn new(config: UnifiedLiquidityConfig) -> Self {
        Self {
            aggregator: LiquidityAggregator::new(),
            config,
            virtual_liquidity: HashMap::new(),
        }
    }

    /// Upsert a venue book into the underlying aggregator.
    pub fn upsert_venue_book(
        &mut self,
        venue_id: String,
        pair: TradingPair,
        bids: Vec<OrderLevel>,
        asks: Vec<OrderLevel>,
    ) {
        self.aggregator
            .upsert_venue_book(venue_id, pair, bids, asks);
    }

    /// Register synthetic/virtual liquidity at a specific price to maintain the target slippage.
    pub fn upsert_virtual_liquidity(
        &mut self,
        pair: TradingPair,
        side: OrderSide,
        price: u64,
        quantity: u64,
    ) {
        let buffer = self.virtual_liquidity.entry(pair).or_default();
        let level = OrderLevel { price, quantity };
        match side {
            OrderSide::Buy => buffer.asks.push(level),
            OrderSide::Sell => buffer.bids.push(level),
        }
    }

    /// Clear any previously configured virtual liquidity for the pair.
    pub fn clear_virtual_liquidity(&mut self, pair: &TradingPair) {
        self.virtual_liquidity.remove(pair);
    }

    /// Produce an execution plan that satisfies the ultra-low slippage target.
    pub fn plan_execution(
        &self,
        pair: &TradingPair,
        side: OrderSide,
        quantity: u64,
    ) -> Result<ExecutionPlan, UnifiedLiquidityError> {
        let (levels, used_virtual) = self.build_levels(pair, side)?;
        let max_safe = self
            .max_fill_within_target(&levels, side)
            .ok_or_else(|| UnifiedLiquidityError::NoLiquidity(pair.clone()))?;

        if quantity > max_safe.filled_quantity {
            return Err(UnifiedLiquidityError::CannotMeetTarget {
                requested: quantity,
                max_executable: max_safe.filled_quantity,
                achieved_slippage_bps: max_safe.slippage_bps,
                target_slippage_bps: self.config.target_slippage_bps,
            });
        }

        let final_slippage =
            calculate_slippage_for_levels(&levels, side, quantity).ok_or_else(|| {
                UnifiedLiquidityError::NoLiquidity(pair.clone())
            })?;

        if let Some(max_levels) = self.config.max_levels_consumed {
            if final_slippage.levels_consumed > max_levels {
                return Err(UnifiedLiquidityError::LevelLimitExceeded {
                    levels: final_slippage.levels_consumed,
                    max_levels,
                });
            }
        }

        Ok(ExecutionPlan {
            pair: pair.clone(),
            side,
            requested_quantity: quantity,
            executed_quantity: quantity,
            achieved_slippage_bps: final_slippage.slippage_bps,
            slices: vec![ExecutionSlice {
                quantity,
                average_price: final_slippage.average_price,
                worst_price: final_slippage.worst_price,
                slippage_bps: final_slippage.slippage_bps,
                levels_consumed: final_slippage.levels_consumed,
                used_virtual_liquidity: used_virtual,
            }],
            fully_covered: true,
        })
    }

    fn build_levels(
        &self,
        pair: &TradingPair,
        side: OrderSide,
    ) -> Result<(Vec<OrderLevel>, bool), UnifiedLiquidityError> {
        let book = self.aggregator.aggregated_book(pair)?;
        let mut levels = match side {
            OrderSide::Buy => book.asks.clone(),
            OrderSide::Sell => book.bids.clone(),
        };

        let mut used_virtual = false;
        if let Some(buffer) = self.virtual_liquidity.get(pair) {
            match side {
                OrderSide::Buy => {
                    if !buffer.asks.is_empty() {
                        levels.extend_from_slice(&buffer.asks);
                        used_virtual = true;
                    }
                }
                OrderSide::Sell => {
                    if !buffer.bids.is_empty() {
                        levels.extend_from_slice(&buffer.bids);
                        used_virtual = true;
                    }
                }
            }
        }

        Ok((normalize_levels(levels, side), used_virtual))
    }

    fn max_fill_within_target(
        &self,
        levels: &[OrderLevel],
        side: OrderSide,
    ) -> Option<SlippageResult> {
        let best_price = levels.first()?.price;
        if best_price == 0 {
            return None;
        }

        let mut cost: u128 = 0;
        let mut filled: u64 = 0;
        let mut best_result: Option<SlippageResult> = None;

        for (idx, level) in levels.iter().enumerate() {
            filled += level.quantity;
            cost += (level.price as u128) * (level.quantity as u128);
            let average_price = cost as f64 / filled as f64;
            let raw_slippage = match side {
                OrderSide::Buy => (average_price - best_price as f64) / best_price as f64,
                OrderSide::Sell => (best_price as f64 - average_price) / best_price as f64,
            };
            let slippage_bps = raw_slippage * 10_000.0;

            if slippage_bps <= self.config.target_slippage_bps {
                best_result = Some(SlippageResult {
                    average_price,
                    best_price,
                    worst_price: level.price,
                    slippage_bps,
                    levels_consumed: idx + 1,
                    filled_quantity: filled,
                });
            } else {
                break;
            }
        }

        best_result
    }
}

fn normalize_levels(mut levels: Vec<OrderLevel>, side: OrderSide) -> Vec<OrderLevel> {
    let mut merged: HashMap<u64, u64> = HashMap::new();
    for level in levels.drain(..) {
        *merged.entry(level.price).or_insert(0) += level.quantity;
    }

    let mut normalized: Vec<OrderLevel> = merged
        .into_iter()
        .map(|(price, quantity)| OrderLevel { price, quantity })
        .collect();

    normalized.sort_by(|a, b| match side {
        OrderSide::Buy => a.price.cmp(&b.price),
        OrderSide::Sell => b.price.cmp(&a.price),
    });

    normalized
}

fn calculate_slippage_for_levels(
    levels: &[OrderLevel],
    side: OrderSide,
    quantity: u64,
) -> Option<SlippageResult> {
    let best_price = levels.first()?.price;
    if best_price == 0 {
        return None;
    }

    let mut remaining = quantity;
    let mut cost: u128 = 0;
    let mut filled: u64 = 0;
    let mut worst_price = best_price;
    let mut levels_used = 0usize;

    for level in levels {
        if remaining == 0 {
            break;
        }

        let take = remaining.min(level.quantity);
        remaining -= take;
        filled += take;
        cost += (level.price as u128) * (take as u128);
        worst_price = level.price;
        levels_used += 1;
    }

    if remaining > 0 || filled == 0 {
        return None;
    }

    let average_price = cost as f64 / filled as f64;
    let raw_slippage = match side {
        OrderSide::Buy => (average_price - best_price as f64) / best_price as f64,
        OrderSide::Sell => (best_price as f64 - average_price) / best_price as f64,
    };
    let slippage_bps = raw_slippage * 10_000.0;

    Some(SlippageResult {
        average_price,
        best_price,
        worst_price,
        slippage_bps,
        levels_consumed: levels_used,
        filled_quantity: filled,
    })
}
