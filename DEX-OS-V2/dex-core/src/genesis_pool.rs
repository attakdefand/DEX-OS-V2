//! Genesis Pool implementation for secure token launches
//!
//! This module implements a liquidity bootstrapping mechanism similar to 
//! Blackhole DEX's Genesis Pools, allowing projects to securely seed liquidity
//! before token generation events.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::types::{TokenId, TraderId};

/// Represents a Genesis Pool for liquidity bootstrapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisPool {
    pub id: String,
    pub project_token_id: TokenId,
    pub contribution_token_id: TokenId,
    pub project_token_amount: u128,
    pub contribution_target: u128,
    pub min_contribution: u128,
    pub max_contribution: u128,
    pub start_time: u64,
    pub end_time: u64,
    pub finalized: bool,
    pub qualified: bool,
    pub contributions: HashMap<TraderId, u128>,
    pub total_contributions: u128,
}

/// Status of a Genesis Pool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GenesisPoolStatus {
    Pending,
    Active,
    Successful,
    Failed,
    Finalized,
}

/// Contribution to a Genesis Pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub trader_id: TraderId,
    pub amount: u128,
    pub timestamp: u64,
}

impl GenesisPool {
    /// Create a new Genesis Pool
    pub fn new(
        id: String,
        project_token_id: TokenId,
        contribution_token_id: TokenId,
        project_token_amount: u128,
        contribution_target: u128,
        min_contribution: u128,
        max_contribution: u128,
        start_time: u64,
        end_time: u64,
    ) -> Self {
        Self {
            id,
            project_token_id,
            contribution_token_id,
            project_token_amount,
            contribution_target,
            min_contribution,
            max_contribution,
            start_time,
            end_time,
            finalized: false,
            qualified: false,
            contributions: HashMap::new(),
            total_contributions: 0,
        }
    }

    /// Get the current status of the Genesis Pool
    pub fn status(&self, current_time: u64) -> GenesisPoolStatus {
        if self.finalized {
            GenesisPoolStatus::Finalized
        } else if current_time < self.start_time {
            GenesisPoolStatus::Pending
        } else if current_time > self.end_time {
            if self.total_contributions >= self.contribution_target {
                GenesisPoolStatus::Successful
            } else {
                GenesisPoolStatus::Failed
            }
        } else {
            GenesisPoolStatus::Active
        }
    }

    /// Contribute to the Genesis Pool
    pub fn contribute(&mut self, trader_id: TraderId, amount: u128, current_time: u64) -> Result<(), String> {
        // Check if the pool is active
        if current_time < self.start_time || current_time > self.end_time {
            return Err("Pool is not active".to_string());
        }

        // Check contribution limits
        let existing_contribution = self.contributions.get(&trader_id).unwrap_or(&0);
        let new_total = *existing_contribution + amount;
        
        if new_total < self.min_contribution {
            return Err("Contribution below minimum".to_string());
        }
        
        if new_total > self.max_contribution {
            return Err("Contribution exceeds maximum".to_string());
        }

        // Update contribution
        self.contributions.insert(trader_id, new_total);
        self.total_contributions += amount;
        
        Ok(())
    }

    /// Check if the pool has reached its target
    pub fn is_target_reached(&self) -> bool {
        self.total_contributions >= self.contribution_target
    }

    /// Calculate LP tokens to distribute to a contributor
    pub fn calculate_lp_tokens(&self, trader_id: &TraderId) -> u128 {
        if !self.qualified {
            return 0;
        }
        
        let contribution = self.contributions.get(trader_id).unwrap_or(&0);
        if *contribution == 0 {
            return 0;
        }
        
        // Simple proportional distribution
        (*contribution * self.project_token_amount) / self.total_contributions
    }

    /// Finalize the pool after the contribution period
    pub fn finalize(&mut self, current_time: u64) -> Result<(), String> {
        if current_time <= self.end_time {
            return Err("Cannot finalize before end time".to_string());
        }
        
        if self.total_contributions >= self.contribution_target {
            self.qualified = true;
        }
        
        self.finalized = true;
        Ok(())
    }

    /// Get all contributors
    pub fn contributors(&self) -> Vec<&TraderId> {
        self.contributions.keys().collect()
    }
}

/// Manager for multiple Genesis Pools
#[derive(Debug)]
pub struct GenesisPoolManager {
    pub pools: HashMap<String, GenesisPool>,
}

impl GenesisPoolManager {
    /// Create a new Genesis Pool Manager
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
        }
    }

    /// Add a new Genesis Pool
    pub fn add_pool(&mut self, pool: GenesisPool) {
        self.pools.insert(pool.id.clone(), pool);
    }

    /// Get a Genesis Pool by ID
    pub fn get_pool(&self, id: &str) -> Option<&GenesisPool> {
        self.pools.get(id)
    }

    /// Get a mutable reference to a Genesis Pool by ID
    pub fn get_pool_mut(&mut self, id: &str) -> Option<&mut GenesisPool> {
        self.pools.get_mut(id)
    }

    /// Remove a Genesis Pool
    pub fn remove_pool(&mut self, id: &str) -> bool {
        self.pools.remove(id).is_some()
    }

    /// Get all pools
    pub fn get_all_pools(&self) -> Vec<&GenesisPool> {
        self.pools.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_genesis_pool_creation() {
        let pool = GenesisPool::new(
            "pool1".to_string(),
            "PROJECT".to_string(),
            "USDC".to_string(),
            1000000,
            5000000,
            1000,
            100000,
            1000000,
            2000000,
        );

        assert_eq!(pool.id, "pool1");
        assert_eq!(pool.project_token_amount, 1000000);
        assert_eq!(pool.contribution_target, 5000000);
        assert_eq!(pool.status(500000), GenesisPoolStatus::Pending);
        assert_eq!(pool.status(1500000), GenesisPoolStatus::Active);
    }

    #[test]
    fn test_contribution() {
        let mut pool = GenesisPool::new(
            "pool1".to_string(),
            "PROJECT".to_string(),
            "USDC".to_string(),
            1000000,
            5000000,
            1000,
            100000,
            1000000,
            2000000,
        );

        let result = pool.contribute("trader1".to_string(), 5000, 1500000);
        assert!(result.is_err()); // Below minimum

        let result = pool.contribute("trader1".to_string(), 50000, 1500000);
        assert!(result.is_ok());

        assert_eq!(pool.total_contributions, 50000);
        assert_eq!(*pool.contributions.get("trader1").unwrap(), 50000);
    }

    #[test]
    fn test_finalization() {
        let mut pool = GenesisPool::new(
            "pool1".to_string(),
            "PROJECT".to_string(),
            "USDC".to_string(),
            1000000,
            5000000,
            1000,
            100000,
            1000000,
            2000000,
        );

        // Add contributions
        pool.contribute("trader1".to_string(), 2000000, 1500000).unwrap();
        pool.contribute("trader2".to_string(), 3000000, 1500000).unwrap();

        // Try to finalize before end time
        let result = pool.finalize(1500000);
        assert!(result.is_err());

        // Finalize after end time
        let result = pool.finalize(2500000);
        assert!(result.is_ok());
        assert!(pool.finalized);
        assert!(pool.qualified); // Target reached
    }

    #[test]
    fn test_lp_token_calculation() {
        let mut pool = GenesisPool::new(
            "pool1".to_string(),
            "PROJECT".to_string(),
            "USDC".to_string(),
            1000000, // 1M project tokens
            5000000, // 5M contribution target
            1000,
            100000,
            1000000,
            2000000,
        );

        // Add contributions
        pool.contribute("trader1".to_string(), 2000000, 1500000).unwrap(); // 40% of target
        pool.contribute("trader2".to_string(), 3000000, 1500000).unwrap(); // 60% of target

        // Finalize pool
        pool.finalize(2500000).unwrap();

        // Calculate LP tokens
        let trader1_lp = pool.calculate_lp_tokens(&"trader1".to_string());
        let trader2_lp = pool.calculate_lp_tokens(&"trader2".to_string());

        // Trader1 should get 40% of project tokens: 1000000 * 0.4 = 400000
        assert_eq!(trader1_lp, 400000);
        
        // Trader2 should get 60% of project tokens: 1000000 * 0.6 = 600000
        assert_eq!(trader2_lp, 600000);
    }
}