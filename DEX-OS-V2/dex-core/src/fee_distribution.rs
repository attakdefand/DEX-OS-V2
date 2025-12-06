//! Fee distribution implementation using Balanced BST for the DEX-OS core engine
//!
//! This module implements the Priority 2 feature from DEX-OS-V1.csv:
//! "Core Trading,AMM,AMM,Balanced BST,Fee Distribution,Medium"
//!
//! It provides functionality for distributing fees to liquidity providers
//! using a balanced binary search tree to ensure fair and efficient distribution.

use crate::types::{Quantity, TokenId, TraderId};
use std::collections::BTreeMap;
use thiserror::Error;

/// Represents a fee distribution entry for a liquidity provider
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeDistribution {
    /// Trader who provided liquidity
    pub trader_id: TraderId,
    /// Token being distributed
    pub token_id: TokenId,
    /// Amount of fees to distribute
    pub amount: Quantity,
    /// Timestamp when the distribution was calculated
    pub timestamp: u64,
}

/// Manages fee distribution using a balanced binary search tree (BTreeMap)
#[derive(Debug, Clone)]
pub struct FeeDistributionManager {
    /// Balanced BST of fee distributions indexed by trader ID
    distributions: BTreeMap<TraderId, FeeDistribution>,
    /// Total fees to be distributed
    total_fees: Quantity,
}

impl FeeDistributionManager {
    /// Create a new fee distribution manager
    pub fn new() -> Self {
        Self {
            distributions: BTreeMap::new(),
            total_fees: 0,
        }
    }

    /// Add a fee distribution for a trader
    pub fn add_distribution(&mut self, distribution: FeeDistribution) {
        self.total_fees += distribution.amount;
        self.distributions
            .insert(distribution.trader_id.clone(), distribution);
    }

    /// Remove a fee distribution for a trader
    pub fn remove_distribution(&mut self, trader_id: &TraderId) -> Option<FeeDistribution> {
        if let Some(distribution) = self.distributions.remove(trader_id) {
            self.total_fees = self.total_fees.saturating_sub(distribution.amount);
            Some(distribution)
        } else {
            None
        }
    }

    /// Get a fee distribution for a specific trader
    pub fn get_distribution(&self, trader_id: &TraderId) -> Option<&FeeDistribution> {
        self.distributions.get(trader_id)
    }

    /// Update the fee amount for a trader
    pub fn update_distribution_amount(
        &mut self,
        trader_id: &TraderId,
        new_amount: Quantity,
    ) -> Result<(), FeeDistributionError> {
        if let Some(distribution) = self.distributions.get_mut(trader_id) {
            // Update total fees
            self.total_fees = self.total_fees.saturating_sub(distribution.amount);
            self.total_fees += new_amount;

            // Update the distribution
            distribution.amount = new_amount;
            Ok(())
        } else {
            Err(FeeDistributionError::TraderNotFound)
        }
    }

    /// Get all fee distributions in sorted order by trader ID
    pub fn get_all_distributions(&self) -> Vec<&FeeDistribution> {
        self.distributions.values().collect()
    }

    /// Get the total amount of fees to be distributed
    pub fn total_fees(&self) -> Quantity {
        self.total_fees
    }

    /// Get the number of traders receiving fee distributions
    pub fn trader_count(&self) -> usize {
        self.distributions.len()
    }

    /// Check if there are any fee distributions
    pub fn has_distributions(&self) -> bool {
        !self.distributions.is_empty()
    }

    /// Clear all fee distributions
    pub fn clear_distributions(&mut self) {
        self.distributions.clear();
        self.total_fees = 0;
    }

    /// Get traders in a range (useful for pagination or batch processing)
    pub fn get_traders_in_range(&self, start: &TraderId, end: &TraderId) -> Vec<&FeeDistribution> {
        self.distributions
            .range::<std::string::String, _>((
                std::ops::Bound::Included(start),
                std::ops::Bound::Included(end),
            ))
            .map(|(_, dist)| dist)
            .collect()
    }

    /// Get the first N traders in sorted order
    pub fn get_first_n_traders(&self, n: usize) -> Vec<&FeeDistribution> {
        self.distributions
            .iter()
            .take(n)
            .map(|(_, dist)| dist)
            .collect()
    }
}

/// Errors that can occur during fee distribution operations
#[derive(Debug, Error)]
pub enum FeeDistributionError {
    #[error("Trader not found in distribution list")]
    TraderNotFound,
    #[error("Invalid fee amount")]
    InvalidAmount,
    #[error("Insufficient fees available for distribution")]
    InsufficientFees,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_distribution_manager_creation() {
        let manager = FeeDistributionManager::new();
        assert_eq!(manager.total_fees(), 0);
        assert_eq!(manager.trader_count(), 0);
        assert!(!manager.has_distributions());
        assert!(manager.get_all_distributions().is_empty());
    }

    #[test]
    fn test_add_and_get_distributions() {
        let mut manager = FeeDistributionManager::new();

        // Add some fee distributions
        let dist1 = FeeDistribution {
            trader_id: "trader1".to_string(),
            token_id: "BTC".to_string(),
            amount: 1000,
            timestamp: 1000000,
        };

        let dist2 = FeeDistribution {
            trader_id: "trader2".to_string(),
            token_id: "ETH".to_string(),
            amount: 2000,
            timestamp: 1000001,
        };

        let dist3 = FeeDistribution {
            trader_id: "trader3".to_string(),
            token_id: "USDC".to_string(),
            amount: 1500,
            timestamp: 1000002,
        };

        manager.add_distribution(dist1.clone());
        manager.add_distribution(dist2.clone());
        manager.add_distribution(dist3.clone());

        assert_eq!(manager.total_fees(), 4500);
        assert_eq!(manager.trader_count(), 3);
        assert!(manager.has_distributions());

        // Get specific distributions
        let retrieved_dist = manager.get_distribution(&"trader1".to_string()).unwrap();
        assert_eq!(retrieved_dist.amount, 1000);

        let retrieved_dist = manager.get_distribution(&"trader2".to_string()).unwrap();
        assert_eq!(retrieved_dist.amount, 2000);

        let retrieved_dist = manager.get_distribution(&"trader3".to_string()).unwrap();
        assert_eq!(retrieved_dist.amount, 1500);

        // Non-existent trader
        assert!(manager.get_distribution(&"trader4".to_string()).is_none());
    }

    #[test]
    fn test_remove_distributions() {
        let mut manager = FeeDistributionManager::new();

        let dist = FeeDistribution {
            trader_id: "trader1".to_string(),
            token_id: "BTC".to_string(),
            amount: 1000,
            timestamp: 1000000,
        };

        manager.add_distribution(dist.clone());
        assert_eq!(manager.total_fees(), 1000);
        assert_eq!(manager.trader_count(), 1);

        // Remove the distribution
        let removed = manager.remove_distribution(&"trader1".to_string()).unwrap();
        assert_eq!(removed.amount, 1000);
        assert_eq!(manager.total_fees(), 0);
        assert_eq!(manager.trader_count(), 0);

        // Try to remove non-existent distribution
        assert!(manager
            .remove_distribution(&"trader2".to_string())
            .is_none());
    }

    #[test]
    fn test_update_distribution_amount() {
        let mut manager = FeeDistributionManager::new();

        let dist = FeeDistribution {
            trader_id: "trader1".to_string(),
            token_id: "BTC".to_string(),
            amount: 1000,
            timestamp: 1000000,
        };

        manager.add_distribution(dist.clone());
        assert_eq!(manager.total_fees(), 1000);

        // Update the amount
        assert!(manager
            .update_distribution_amount(&"trader1".to_string(), 1500)
            .is_ok());
        assert_eq!(manager.total_fees(), 1500);

        let updated_dist = manager.get_distribution(&"trader1".to_string()).unwrap();
        assert_eq!(updated_dist.amount, 1500);

        // Try to update non-existent trader
        assert!(manager
            .update_distribution_amount(&"trader2".to_string(), 2000)
            .is_err());
    }

    #[test]
    fn test_get_all_distributions_sorted() {
        let mut manager = FeeDistributionManager::new();

        // Add distributions in non-alphabetical order
        let dist_c = FeeDistribution {
            trader_id: "traderC".to_string(),
            token_id: "BTC".to_string(),
            amount: 1000,
            timestamp: 1000000,
        };

        let dist_a = FeeDistribution {
            trader_id: "traderA".to_string(),
            token_id: "ETH".to_string(),
            amount: 2000,
            timestamp: 1000001,
        };

        let dist_b = FeeDistribution {
            trader_id: "traderB".to_string(),
            token_id: "USDC".to_string(),
            amount: 1500,
            timestamp: 1000002,
        };

        manager.add_distribution(dist_c.clone());
        manager.add_distribution(dist_a.clone());
        manager.add_distribution(dist_b.clone());

        // Get all distributions - should be sorted by trader ID
        let all_distributions = manager.get_all_distributions();
        assert_eq!(all_distributions.len(), 3);
        assert_eq!(all_distributions[0].trader_id, "traderA");
        assert_eq!(all_distributions[1].trader_id, "traderB");
        assert_eq!(all_distributions[2].trader_id, "traderC");
    }

    #[test]
    fn test_clear_distributions() {
        let mut manager = FeeDistributionManager::new();

        let dist = FeeDistribution {
            trader_id: "trader1".to_string(),
            token_id: "BTC".to_string(),
            amount: 1000,
            timestamp: 1000000,
        };

        manager.add_distribution(dist.clone());
        assert_eq!(manager.total_fees(), 1000);
        assert_eq!(manager.trader_count(), 1);

        // Clear all distributions
        manager.clear_distributions();
        assert_eq!(manager.total_fees(), 0);
        assert_eq!(manager.trader_count(), 0);
        assert!(!manager.has_distributions());
    }

    #[test]
    fn test_get_traders_in_range() {
        let mut manager = FeeDistributionManager::new();

        // Add distributions with different trader IDs
        for i in 1..=5 {
            let trader_id = format!("trader{}", i);
            let dist = FeeDistribution {
                trader_id: trader_id.clone(),
                token_id: "BTC".to_string(),
                amount: 1000 * i as u64,
                timestamp: 1000000 + i as u64,
            };
            manager.add_distribution(dist);
        }

        // Get traders in a range
        let ranged_distributions =
            manager.get_traders_in_range(&"trader2".to_string(), &"trader4".to_string());
        assert_eq!(ranged_distributions.len(), 3);
        assert_eq!(ranged_distributions[0].trader_id, "trader2");
        assert_eq!(ranged_distributions[1].trader_id, "trader3");
        assert_eq!(ranged_distributions[2].trader_id, "trader4");
    }

    #[test]
    fn test_get_first_n_traders() {
        let mut manager = FeeDistributionManager::new();

        // Add distributions
        for i in 1..=5 {
            let trader_id = format!("trader{}", i);
            let dist = FeeDistribution {
                trader_id: trader_id.clone(),
                token_id: "BTC".to_string(),
                amount: 1000 * i as u64,
                timestamp: 1000000 + i as u64,
            };
            manager.add_distribution(dist);
        }

        // Get first 3 traders
        let first_traders = manager.get_first_n_traders(3);
        assert_eq!(first_traders.len(), 3);
        assert_eq!(first_traders[0].trader_id, "trader1");
        assert_eq!(first_traders[1].trader_id, "trader2");
        assert_eq!(first_traders[2].trader_id, "trader3");

        // Get more than available
        let all_traders = manager.get_first_n_traders(10);
        assert_eq!(all_traders.len(), 5);
    }
}
