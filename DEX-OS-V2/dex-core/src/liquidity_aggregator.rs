//! Liquidity Aggregator implementing global order book + slippage calculator
//!
//! Implements Priority 2 features:
//! - `2,Components,Liquidity Aggregator,Aggregator,Global Order Book,Order Book,High`
//! - `2,Components,Liquidity Aggregator,Aggregator,Slippage Calculator,Slippage Calculation,High`
//!
//! The aggregator ingests venue order books, produces a global depth view, and
//! offers deterministic slippage estimates across consolidated liquidity.

use crate::types::{OrderSide, TradingPair};
use std::collections::HashMap;
use thiserror::Error;

/// Represents a single price level from a venue
#[derive(Debug, Clone, PartialEq)]
pub struct OrderLevel {
    pub price: u64,
    pub quantity: u64,
}

/// Aggregated depth across all venues
#[derive(Debug, Clone, PartialEq)]
pub struct AggregatedOrderBook {
    pub pair: TradingPair,
    pub bids: Vec<OrderLevel>,
    pub asks: Vec<OrderLevel>,
}

/// Result of a slippage calculation
#[derive(Debug, Clone, PartialEq)]
pub struct SlippageResult {
    pub average_price: f64,
    pub best_price: u64,
    pub worst_price: u64,
    /// Slippage expressed in basis points relative to best price
    pub slippage_bps: f64,
    /// Number of levels consumed to satisfy the requested size
    pub levels_consumed: usize,
    /// Total quantity that was actually filled
    pub filled_quantity: u64,
}

/// Errors returned by the liquidity aggregator
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AggregatorError {
    #[error("no liquidity found for pair")]
    NoLiquidity,
    #[error("insufficient liquidity to fill request")]
    InsufficientLiquidity,
}

#[derive(Debug, Clone)]
struct VenueBook {
    pair: TradingPair,
    bids: Vec<OrderLevel>,
    asks: Vec<OrderLevel>,
}

/// Aggregator orchestrating venue order books
#[derive(Debug, Default)]
pub struct LiquidityAggregator {
    venues: HashMap<String, VenueBook>,
}

impl LiquidityAggregator {
    /// Create a new aggregator
    pub fn new() -> Self {
        Self {
            venues: HashMap::new(),
        }
    }

    /// Ingest or update a venue order book snapshot for a trading pair
    pub fn upsert_venue_book(
        &mut self,
        venue_id: String,
        pair: TradingPair,
        bids: Vec<OrderLevel>,
        asks: Vec<OrderLevel>,
    ) {
        let normalized_bids = sort_and_merge(bids, OrderSide::Buy);
        let normalized_asks = sort_and_merge(asks, OrderSide::Sell);

        let book = VenueBook {
            pair,
            bids: normalized_bids,
            asks: normalized_asks,
        };

        self.venues.insert(venue_id, book);
    }

    /// Remove a venue entirely (e.g., offline)
    pub fn remove_venue(&mut self, venue_id: &str) -> bool {
        self.venues.remove(venue_id).is_some()
    }

    /// Produce the global aggregated book for the given pair
    pub fn aggregated_book(
        &self,
        pair: &TradingPair,
    ) -> Result<AggregatedOrderBook, AggregatorError> {
        let mut all_bids: Vec<OrderLevel> = Vec::new();
        let mut all_asks: Vec<OrderLevel> = Vec::new();

        for venue in self.venues.values().filter(|v| &v.pair == pair) {
            all_bids.extend_from_slice(&venue.bids);
            all_asks.extend_from_slice(&venue.asks);
        }

        if all_bids.is_empty() && all_asks.is_empty() {
            return Err(AggregatorError::NoLiquidity);
        }

        let bids = sort_and_merge(all_bids, OrderSide::Buy);
        let asks = sort_and_merge(all_asks, OrderSide::Sell);

        Ok(AggregatedOrderBook {
            pair: pair.clone(),
            bids,
            asks,
        })
    }

    /// Estimate slippage for a desired quantity (quoted in base units) on a side
    pub fn calculate_slippage(
        &self,
        pair: &TradingPair,
        side: OrderSide,
        quantity: u64,
    ) -> Result<SlippageResult, AggregatorError> {
        let book = self.aggregated_book(pair)?;
        let (levels, best_price) = match side {
            OrderSide::Buy => (&book.asks, book.asks.first().map(|l| l.price)),
            OrderSide::Sell => (&book.bids, book.bids.first().map(|l| l.price)),
        };

        let best_price = best_price.ok_or(AggregatorError::NoLiquidity)?;

        let mut remaining = quantity;
        let mut cost: u128 = 0;
        let mut filled: u64 = 0;
        let mut levels_used = 0;
        let mut worst_price = best_price;

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

        if remaining > 0 {
            return Err(AggregatorError::InsufficientLiquidity);
        }

        let average_price = cost as f64 / filled as f64;
        let raw_slippage = match side {
            OrderSide::Buy => (average_price - best_price as f64) / best_price as f64,
            OrderSide::Sell => (best_price as f64 - average_price) / best_price as f64,
        };
        let slippage_bps = raw_slippage * 10_000.0;

        Ok(SlippageResult {
            average_price,
            best_price,
            worst_price,
            slippage_bps,
            levels_consumed: levels_used,
            filled_quantity: filled,
        })
    }
}

fn sort_and_merge(levels: Vec<OrderLevel>, side: OrderSide) -> Vec<OrderLevel> {
    let mut map: HashMap<u64, u64> = HashMap::new();
    for level in levels {
        *map.entry(level.price).or_insert(0) += level.quantity;
    }

    let mut merged: Vec<OrderLevel> = map
        .into_iter()
        .map(|(price, quantity)| OrderLevel { price, quantity })
        .collect();

    merged.sort_by(|a, b| match side {
        OrderSide::Buy => b.price.cmp(&a.price), // bids: high to low
        OrderSide::Sell => a.price.cmp(&b.price), // asks: low to high
    });

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> TradingPair {
        TradingPair {
            base: "BASE".to_string(),
            quote: "QUOTE".to_string(),
        }
    }

    #[test]
    fn aggregates_global_order_book_across_venues() {
        let mut agg = LiquidityAggregator::new();
        let p = pair();

        agg.upsert_venue_book(
            "venue-a".to_string(),
            p.clone(),
            vec![
                OrderLevel {
                    price: 99,
                    quantity: 50,
                },
                OrderLevel {
                    price: 98,
                    quantity: 25,
                },
            ],
            vec![
                OrderLevel {
                    price: 101,
                    quantity: 40,
                },
                OrderLevel {
                    price: 102,
                    quantity: 30,
                },
            ],
        );

        agg.upsert_venue_book(
            "venue-b".to_string(),
            p.clone(),
            vec![OrderLevel {
                price: 100,
                quantity: 75,
            }],
            vec![OrderLevel {
                price: 103,
                quantity: 20,
            }],
        );

        let book = agg.aggregated_book(&p).unwrap();
        assert_eq!(book.bids.len(), 3);
        assert_eq!(book.asks.len(), 3);

        // Bids sorted desc
        assert_eq!(book.bids[0].price, 100);
        assert_eq!(book.bids[1].price, 99);
        assert_eq!(book.bids[2].price, 98);

        // Asks sorted asc
        assert_eq!(book.asks[0].price, 101);
        assert_eq!(book.asks[1].price, 102);
        assert_eq!(book.asks[2].price, 103);
    }

    #[test]
    fn merges_levels_by_price() {
        let mut agg = LiquidityAggregator::new();
        let p = pair();

        agg.upsert_venue_book(
            "venue-a".to_string(),
            p.clone(),
            vec![OrderLevel {
                price: 100,
                quantity: 10,
            }],
            vec![OrderLevel {
                price: 101,
                quantity: 5,
            }],
        );

        // Same prices on another venue should merge
        agg.upsert_venue_book(
            "venue-b".to_string(),
            p.clone(),
            vec![OrderLevel {
                price: 100,
                quantity: 15,
            }],
            vec![OrderLevel {
                price: 101,
                quantity: 7,
            }],
        );

        let book = agg.aggregated_book(&p).unwrap();
        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.bids[0].quantity, 25);
        assert_eq!(book.asks[0].quantity, 12);
    }

    #[test]
    fn calculates_slippage_for_buy_side() {
        let mut agg = LiquidityAggregator::new();
        let p = pair();

        agg.upsert_venue_book(
            "venue-a".to_string(),
            p.clone(),
            vec![OrderLevel {
                price: 100,
                quantity: 50,
            }],
            vec![
                OrderLevel {
                    price: 101,
                    quantity: 100,
                },
                OrderLevel {
                    price: 102,
                    quantity: 100,
                },
            ],
        );

        let result = agg
            .calculate_slippage(&p, OrderSide::Buy, 150)
            .expect("slippage should succeed");

        assert_eq!(result.best_price, 101);
        assert_eq!(result.worst_price, 102);
        assert_eq!(result.levels_consumed, 2);
        assert_eq!(result.filled_quantity, 150);

        // Average price = (101*100 + 102*50) / 150 = 101.333...
        assert!((result.average_price - 101.333).abs() < 0.01);
        assert!(result.slippage_bps > 0.0);
    }

    #[test]
    fn calculates_slippage_for_sell_side() {
        let mut agg = LiquidityAggregator::new();
        let p = pair();

        agg.upsert_venue_book(
            "venue-a".to_string(),
            p.clone(),
            vec![
                OrderLevel {
                    price: 105,
                    quantity: 60,
                },
                OrderLevel {
                    price: 104,
                    quantity: 60,
                },
            ],
            vec![OrderLevel {
                price: 106,
                quantity: 20,
            }],
        );

        let result = agg
            .calculate_slippage(&p, OrderSide::Sell, 110)
            .expect("slippage should succeed");

        assert_eq!(result.best_price, 105);
        assert_eq!(result.worst_price, 104);
        assert_eq!(result.levels_consumed, 2);
        assert_eq!(result.filled_quantity, 110);
        assert!(result.slippage_bps >= 0.0);
    }

    #[test]
    fn errors_on_insufficient_liquidity() {
        let mut agg = LiquidityAggregator::new();
        let p = pair();

        agg.upsert_venue_book(
            "venue-a".to_string(),
            p.clone(),
            vec![OrderLevel {
                price: 100,
                quantity: 10,
            }],
            vec![OrderLevel {
                price: 101,
                quantity: 5,
            }],
        );

        let err = agg.calculate_slippage(&p, OrderSide::Buy, 50).unwrap_err();
        assert_eq!(err, AggregatorError::InsufficientLiquidity);
    }
}
