//! Concentrated Liquidity implementation (clAMM)
//!
//! This module implements concentrated liquidity pools similar to Uniswap V3,
//! allowing liquidity providers to allocate capital within specific price ranges.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::types::{TokenId, Price};

/// Amount representation
pub type Amount = u64;

/// Represents a tick in the concentrated liquidity system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub index: i32,
    pub price: Price,
    pub liquidity_gross: u128,
    pub liquidity_net: i128,
}

/// Position of a liquidity provider within a price range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub owner: String,
    pub pool_id: String,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: u128,
    pub fee_growth_inside_0_last: u128,
    pub fee_growth_inside_1_last: u128,
    pub tokens_owed_0: u128,
    pub tokens_owed_1: u128,
}

/// Concentrated Liquidity Pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentratedPool {
    pub id: String,
    pub token0: TokenId,
    pub token1: TokenId,
    pub fee: u32, // Fee in basis points (e.g., 30 for 0.3%)
    pub tick_spacing: i32,
    pub tick_current: i32,
    pub sqrt_price_x96: u128,
    pub liquidity: u128,
    pub fee_growth_global_0: u128,
    pub fee_growth_global_1: u128,
    pub ticks: HashMap<i32, Tick>,
    pub positions: HashMap<String, Position>,
}

impl ConcentratedPool {
    /// Create a new concentrated liquidity pool
    pub fn new(
        id: String,
        token0: TokenId,
        token1: TokenId,
        fee: u32,
        tick_spacing: i32,
        initial_price: Price,
    ) -> Self {
        let sqrt_price_x96 = Self::price_to_sqrt_price_x96(initial_price);
        let tick_current = Self::sqrt_price_x96_to_tick(sqrt_price_x96);
        
        Self {
            id,
            token0,
            token1,
            fee,
            tick_spacing,
            tick_current,
            sqrt_price_x96,
            liquidity: 0,
            fee_growth_global_0: 0,
            fee_growth_global_1: 0,
            ticks: HashMap::new(),
            positions: HashMap::new(),
        }
    }

    /// Convert price to sqrt(price) * 2^96
    fn price_to_sqrt_price_x96(price: Price) -> u128 {
        // Simplified conversion - in practice this would use fixed-point math
        // Simplified conversion for u64 price
        ((price as f64).sqrt() * (2f64.powi(96))) as u128
    }

    /// Convert sqrt(price) * 2^96 to tick
    fn sqrt_price_x96_to_tick(sqrt_price_x96: u128) -> i32 {
        // Simplified conversion - in practice this would use logarithms
        // Simplified conversion for u64 price
        ((sqrt_price_x96 as f64 / 2f64.powi(96)).ln() / 1.0001f64.ln()) as i32
    }

    /// Create a new position (add liquidity)
    pub fn create_position(
        &mut self,
        owner: String,
        tick_lower: i32,
        tick_upper: i32,
        amount0: Amount,
        amount1: Amount,
    ) -> Result<String, String> {
        // Validate ticks
        if tick_lower >= tick_upper {
            return Err("Invalid tick range".to_string());
        }
        
        if tick_lower % self.tick_spacing != 0 || tick_upper % self.tick_spacing != 0 {
            return Err("Invalid tick spacing".to_string());
        }

        // Generate position ID
        let position_id = format!("{}_{}_{}", owner, tick_lower, tick_upper);

        // Create position
        let position = Position {
            id: position_id.clone(),
            owner,
            pool_id: self.id.clone(),
            tick_lower,
            tick_upper,
            liquidity: Self::calculate_liquidity(amount0, amount1, tick_lower, tick_upper),
            fee_growth_inside_0_last: self.fee_growth_global_0,
            fee_growth_inside_1_last: self.fee_growth_global_1,
            tokens_owed_0: 0,
            tokens_owed_1: 0,
        };

        // Update ticks
        self.update_tick(tick_lower, position.liquidity as i128, true);
        self.update_tick(tick_upper, position.liquidity as i128, false);

        // Update pool liquidity
        if tick_lower <= self.tick_current && self.tick_current < tick_upper {
            self.liquidity += position.liquidity;
        }

        // Store position
        self.positions.insert(position_id.clone(), position);

        Ok(position_id)
    }

    /// Calculate liquidity from amounts and tick range
    fn calculate_liquidity(amount0: Amount, amount1: Amount, tick_lower: i32, tick_upper: i32) -> u128 {
        // Simplified calculation - in practice this would use more complex math
        ((amount0 as f64 * amount1 as f64).sqrt() * 1000.0) as u128
    }

    /// Update tick data
    fn update_tick(&mut self, tick_index: i32, liquidity_delta: i128, upper: bool) {
        let liquidity_net = if upper { -liquidity_delta } else { liquidity_delta };
        
        let tick = self.ticks.entry(tick_index).or_insert(Tick {
            index: tick_index,
            price: 0, // Would be calculated properly in a real implementation
            liquidity_gross: 0,
            liquidity_net: 0,
        });
        
        tick.liquidity_gross = (tick.liquidity_gross as i128 + liquidity_delta.abs()) as u128;
        tick.liquidity_net += liquidity_net;
    }

    /// Swap tokens in the pool
    pub fn swap(&mut self, zero_for_one: bool, amount_specified: Amount) -> Result<(Amount, Amount), String> {
        let mut amount0 = 0i128;
        let mut amount1 = 0i128;

        // Simplified swap logic - in practice this would iterate through ticks
        // and calculate the exact amount based on the liquidity curve
        
        if zero_for_one {
            // Swapping token0 for token1
            amount0 = amount_specified as i128;
            // Calculate amount1 based on current price and liquidity
            amount1 = -(amount0 * self.sqrt_price_x96 as i128 / 1000000);
        } else {
            // Swapping token1 for token0
            amount1 = amount_specified as i128;
            // Calculate amount0 based on current price and liquidity
            amount0 = -(amount1 * 1000000 / self.sqrt_price_x96 as i128);
        }

        // Update fee growth
        let fee_amount = (amount_specified as u128 * self.fee as u128) / 10000;
        if zero_for_one {
            self.fee_growth_global_0 += fee_amount * 1000000 / self.liquidity;
        } else {
            self.fee_growth_global_1 += fee_amount * 1000000 / self.liquidity;
        }

        Ok((amount0 as Amount, amount1 as Amount))
    }

    /// Collect fees from a position
    pub fn collect_fees(&mut self, position_id: &str) -> Result<(Amount, Amount), String> {
        let position = self.positions.get_mut(position_id)
            .ok_or("Position not found")?;

        let tokens_owed_0 = position.tokens_owed_0 as Amount;
        let tokens_owed_1 = position.tokens_owed_1 as Amount;

        position.tokens_owed_0 = 0;
        position.tokens_owed_1 = 0;

        Ok((tokens_owed_0, tokens_owed_1))
    }
}

/// Manager for concentrated liquidity pools
#[derive(Debug)]
pub struct ConcentratedLiquidityManager {
    pub pools: HashMap<String, ConcentratedPool>,
}

impl ConcentratedLiquidityManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
        }
    }

    /// Create a new pool
    pub fn create_pool(
        &mut self,
        id: String,
        token0: TokenId,
        token1: TokenId,
        fee: u32,
        tick_spacing: i32,
        initial_price: Price,
    ) -> String {
        let pool = ConcentratedPool::new(id.clone(), token0, token1, fee, tick_spacing, initial_price);
        self.pools.insert(id.clone(), pool);
        id
    }

    /// Get a pool by ID
    pub fn get_pool(&self, id: &str) -> Option<&ConcentratedPool> {
        self.pools.get(id)
    }

    /// Get a mutable reference to a pool by ID
    pub fn get_pool_mut(&mut self, id: &str) -> Option<&mut ConcentratedPool> {
        self.pools.get_mut(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let mut manager = ConcentratedLiquidityManager::new();
        let pool_id = manager.create_pool(
            "pool1".to_string(),
            "TOKEN0".to_string(),
            "TOKEN1".to_string(),
            30, // 0.3% fee
            10,
            1000000, // 1:1 price (scaled)
        );

        let pool = manager.get_pool(&pool_id).unwrap();
        assert_eq!(pool.id, "pool1");
        assert_eq!(pool.token0, "TOKEN0");
        assert_eq!(pool.token1, "TOKEN1");
        assert_eq!(pool.fee, 30);
    }

    #[test]
    fn test_position_creation() {
        let mut manager = ConcentratedLiquidityManager::new();
        let pool_id = manager.create_pool(
            "pool1".to_string(),
            "TOKEN0".to_string(),
            "TOKEN1".to_string(),
            30,
            10,
            1000000,
        );

        let pool = manager.get_pool_mut(&pool_id).unwrap();
        let result = pool.create_position(
            "user1".to_string(),
            -100,  // tick_lower
            100,   // tick_upper
            1000,  // amount0
            1000,  // amount1
        );

        assert!(result.is_ok());
        assert_eq!(pool.positions.len(), 1);
    }

    #[test]
    fn test_swap() {
        let mut manager = ConcentratedLiquidityManager::new();
        let pool_id = manager.create_pool(
            "pool1".to_string(),
            "TOKEN0".to_string(),
            "TOKEN1".to_string(),
            30,
            10,
            1000000,
        );

        // Add liquidity
        {
            let pool = manager.get_pool_mut(&pool_id).unwrap();
            pool.create_position(
                "user1".to_string(),
                -100,
                100,
                10000,
                10000,
            ).unwrap();
        }

        // Perform swap
        let pool = manager.get_pool_mut(&pool_id).unwrap();
        let result = pool.swap(true, 1000); // Swap 1000 token0 for token1

        assert!(result.is_ok());
        let (amount0, amount1) = result.unwrap();
        assert_eq!(amount0, 1000); // Positive amount0 means we're giving token0
        assert!(amount1 < 0); // Negative amount1 means we're receiving token1
    }
}