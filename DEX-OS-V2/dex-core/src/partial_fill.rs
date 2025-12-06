//! Partial fill exploration implementation using Depth-First Search for the DEX-OS core engine
//!
//! This module implements the Priority 2 feature from DEX-OS-V1.csv:
//! "Core Trading,DEX Aggregator,DEX Aggregator,Depth-First Search,Partial Fill Exploration,Medium"
//!
//! It provides functionality for exploring partial fill opportunities for large trades
//! that cannot be filled entirely on a single DEX or trading pair.

use crate::types::{Quantity, TokenId};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Represents a partial fill opportunity
#[derive(Debug, Clone, PartialEq)]
pub struct PartialFillOpportunity {
    /// Source token
    pub from_token: TokenId,
    /// Destination token
    pub to_token: TokenId,
    /// DEX where this opportunity exists
    pub dex_name: String,
    /// Available liquidity for this trading pair
    pub available_liquidity: Quantity,
    /// Exchange rate (how much to_token you get for 1 from_token)
    pub exchange_rate: f64,
    /// Fee percentage (0.0 to 1.0)
    pub fee: f64,
}

/// Represents a partial fill plan that breaks a large trade into smaller fills
#[derive(Debug, Clone)]
pub struct PartialFillPlan {
    /// Sequence of partial fills that make up the plan
    pub fills: Vec<PartialFillOpportunity>,
    /// Total amount that can be filled
    pub total_fill_amount: Quantity,
    /// Average exchange rate across all fills
    pub average_exchange_rate: f64,
    /// Total fee across all fills
    pub total_fee: f64,
}

/// Manages partial fill exploration using Depth-First Search
#[derive(Debug, Clone)]
pub struct PartialFillExplorer {
    /// Graph representation: token -> list of partial fill opportunities
    graph: HashMap<TokenId, Vec<PartialFillOpportunity>>,
}

impl PartialFillExplorer {
    /// Create a new partial fill explorer
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }

    /// Add a partial fill opportunity to the graph
    pub fn add_opportunity(&mut self, opportunity: PartialFillOpportunity) {
        self.graph
            .entry(opportunity.from_token.clone())
            .or_insert_with(Vec::new)
            .push(opportunity);
    }

    /// Remove all opportunities for a specific DEX
    pub fn remove_dex_opportunities(&mut self, dex_name: &str) {
        for opportunities in self.graph.values_mut() {
            opportunities.retain(|opp| opp.dex_name != dex_name);
        }

        // Clean up empty opportunity lists
        self.graph
            .retain(|_, opportunities| !opportunities.is_empty());
    }

    /// Explore partial fill opportunities using Depth-First Search
    ///
    /// This function finds ways to fill a large trade by breaking it into smaller fills
    /// across multiple DEXes or trading pairs.
    pub fn explore_partial_fills(
        &self,
        source: &TokenId,
        destination: &TokenId,
        target_amount: Quantity,
    ) -> Result<Vec<PartialFillPlan>, PartialFillError> {
        if source == destination {
            return Err(PartialFillError::SameSourceDestination);
        }

        if target_amount == 0 {
            return Err(PartialFillError::InvalidAmount);
        }

        let mut plans = Vec::new();
        let mut visited = HashSet::new();
        let mut current_path = Vec::new();

        self.dfs_explore(
            source,
            destination,
            target_amount,
            0,
            &mut visited,
            &mut current_path,
            &mut plans,
        );

        Ok(plans)
    }

    /// Depth-First Search implementation for exploring partial fill opportunities
    fn dfs_explore(
        &self,
        current_token: &TokenId,
        destination: &TokenId,
        target_amount: Quantity,
        filled_amount: Quantity,
        visited: &mut HashSet<TokenId>,
        current_path: &mut Vec<PartialFillOpportunity>,
        plans: &mut Vec<PartialFillPlan>,
    ) {
        // If we've reached the destination token, create a plan
        if current_token == destination {
            if let Some(plan) = self.create_plan_from_path(current_path, target_amount) {
                plans.push(plan);
            }
            return;
        }

        // Mark current token as visited
        visited.insert(current_token.clone());

        // Explore all opportunities from the current token
        if let Some(opportunities) = self.graph.get(current_token) {
            for opportunity in opportunities {
                // Skip if we've already visited the destination token of this opportunity
                if visited.contains(&opportunity.to_token) {
                    continue;
                }

                // Add this opportunity to the current path
                current_path.push(opportunity.clone());

                // Continue DFS exploration
                self.dfs_explore(
                    &opportunity.to_token,
                    destination,
                    target_amount,
                    filled_amount
                        + opportunity
                            .available_liquidity
                            .min(target_amount - filled_amount),
                    visited,
                    current_path,
                    plans,
                );

                // Backtrack: remove this opportunity from the current path
                current_path.pop();
            }
        }

        // Unmark current token as visited (for other paths)
        visited.remove(current_token);
    }

    /// Create a partial fill plan from a path of opportunities
    fn create_plan_from_path(
        &self,
        path: &[PartialFillOpportunity],
        target_amount: Quantity,
    ) -> Option<PartialFillPlan> {
        if path.is_empty() {
            return None;
        }

        let mut fills = Vec::new();
        let mut total_fill_amount = 0;
        let mut weighted_exchange_rate_sum = 0.0;
        let mut total_fee = 0.0;

        // Process all opportunities in the path
        for (i, opportunity) in path.iter().enumerate() {
            // Calculate how much we can fill with this opportunity
            let fill_amount = if i == 0 {
                // For the first opportunity, the fill amount depends on whether it's a single-hop or multi-hop trade
                if path.len() == 1 {
                    // Single-hop trade: use the target amount (limited by available liquidity)
                    target_amount.min(opportunity.available_liquidity)
                } else {
                    // Multi-hop trade: use the available liquidity on the first hop
                    opportunity.available_liquidity
                }
            } else {
                // For subsequent opportunities in a multi-hop trade, use the available liquidity
                opportunity.available_liquidity
            };

            // Add to fills
            fills.push(PartialFillOpportunity {
                from_token: opportunity.from_token.clone(),
                to_token: opportunity.to_token.clone(),
                dex_name: opportunity.dex_name.clone(),
                available_liquidity: fill_amount,
                exchange_rate: opportunity.exchange_rate,
                fee: opportunity.fee,
            });

            // Update totals
            if i == 0 {
                total_fill_amount = fill_amount;
            }
            weighted_exchange_rate_sum += opportunity.exchange_rate * fill_amount as f64;
            total_fee += opportunity.fee;
        }

        // Calculate average exchange rate
        let average_exchange_rate = if total_fill_amount > 0 {
            weighted_exchange_rate_sum / total_fill_amount as f64
        } else {
            0.0
        };

        Some(PartialFillPlan {
            fills,
            total_fill_amount,
            average_exchange_rate,
            total_fee,
        })
    }

    /// Find the best partial fill plan based on exchange rate
    pub fn find_best_plan(
        &self,
        source: &TokenId,
        destination: &TokenId,
        target_amount: Quantity,
    ) -> Result<Option<PartialFillPlan>, PartialFillError> {
        let plans = self.explore_partial_fills(source, destination, target_amount)?;

        // Find the plan with the best (highest) average exchange rate
        let best_plan = plans.into_iter().max_by(|a, b| {
            a.average_exchange_rate
                .partial_cmp(&b.average_exchange_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(best_plan)
    }

    /// Get all opportunities from a specific token
    pub fn get_opportunities_from_token(
        &self,
        token: &TokenId,
    ) -> Option<&Vec<PartialFillOpportunity>> {
        self.graph.get(token)
    }

    /// Get the number of tokens in the graph
    pub fn token_count(&self) -> usize {
        self.graph.len()
    }

    /// Get the number of opportunities in the graph
    pub fn opportunity_count(&self) -> usize {
        self.graph
            .values()
            .map(|opportunities| opportunities.len())
            .sum()
    }

    /// Clear all opportunities
    pub fn clear_opportunities(&mut self) {
        self.graph.clear();
    }
}

/// Errors that can occur during partial fill exploration
#[derive(Debug, Error)]
pub enum PartialFillError {
    #[error("Source and destination tokens are the same")]
    SameSourceDestination,
    #[error("Invalid target amount")]
    InvalidAmount,
    #[error("No partial fill opportunities found")]
    NoOpportunitiesFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_fill_explorer_creation() {
        let explorer = PartialFillExplorer::new();
        assert_eq!(explorer.token_count(), 0);
        assert_eq!(explorer.opportunity_count(), 0);
    }

    #[test]
    fn test_add_opportunity() {
        let mut explorer = PartialFillExplorer::new();

        let opportunity = PartialFillOpportunity {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            available_liquidity: 1000000,
            exchange_rate: 13.5,
            fee: 0.003,
        };

        explorer.add_opportunity(opportunity.clone());

        assert_eq!(explorer.token_count(), 1);
        assert_eq!(explorer.opportunity_count(), 1);

        let opportunities = explorer
            .get_opportunities_from_token(&"BTC".to_string())
            .unwrap();
        assert_eq!(opportunities.len(), 1);
        assert_eq!(opportunities[0], opportunity);
    }

    #[test]
    fn test_explore_partial_fills_simple() {
        let mut explorer = PartialFillExplorer::new();

        // Add a simple opportunity: BTC -> ETH
        let opportunity = PartialFillOpportunity {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            available_liquidity: 1000000,
            exchange_rate: 13.5,
            fee: 0.003,
        };

        explorer.add_opportunity(opportunity.clone());

        // Explore partial fills from BTC to ETH
        let plans = explorer
            .explore_partial_fills(&"BTC".to_string(), &"ETH".to_string(), 500000)
            .unwrap();
        assert_eq!(plans.len(), 1);

        let plan = &plans[0];
        assert_eq!(plan.fills.len(), 1);
        assert_eq!(plan.total_fill_amount, 500000);
        assert_eq!(plan.average_exchange_rate, 13.5);
        assert_eq!(plan.total_fee, 0.003);
    }

    #[test]
    fn test_explore_partial_fills_multi_hop() {
        let mut explorer = PartialFillExplorer::new();

        // Add opportunities: BTC -> ETH -> USDC
        let opportunity1 = PartialFillOpportunity {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            available_liquidity: 1000000,
            exchange_rate: 13.5,
            fee: 0.003,
        };

        let opportunity2 = PartialFillOpportunity {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            available_liquidity: 50000000,
            exchange_rate: 3200.0,
            fee: 0.003,
        };

        explorer.add_opportunity(opportunity1.clone());
        explorer.add_opportunity(opportunity2.clone());

        // Explore partial fills from BTC to USDC
        let plans = explorer
            .explore_partial_fills(&"BTC".to_string(), &"USDC".to_string(), 500000)
            .unwrap();
        assert_eq!(plans.len(), 1);

        let plan = &plans[0];
        assert_eq!(plan.fills.len(), 2);
        assert_eq!(plan.total_fill_amount, 1000000); // Limited by BTC->ETH liquidity
        assert_eq!(plan.total_fee, 0.006); // 0.003 + 0.003
    }

    #[test]
    fn test_find_best_plan() {
        let mut explorer = PartialFillExplorer::new();

        // Add two opportunities for BTC -> ETH with different exchange rates
        let opportunity1 = PartialFillOpportunity {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            available_liquidity: 1000000,
            exchange_rate: 13.5, // Lower rate
            fee: 0.003,
        };

        let opportunity2 = PartialFillOpportunity {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "SushiSwap".to_string(),
            available_liquidity: 500000,
            exchange_rate: 13.8, // Higher rate
            fee: 0.003,
        };

        explorer.add_opportunity(opportunity1.clone());
        explorer.add_opportunity(opportunity2.clone());

        // Find the best plan
        let best_plan = explorer
            .find_best_plan(&"BTC".to_string(), &"ETH".to_string(), 500000)
            .unwrap();
        assert!(best_plan.is_some());

        let plan = best_plan.unwrap();
        assert_eq!(plan.fills.len(), 1);
        assert_eq!(plan.fills[0].dex_name, "SushiSwap"); // Should choose the higher rate
        assert_eq!(plan.average_exchange_rate, 13.8);
    }

    #[test]
    fn test_remove_dex_opportunities() {
        let mut explorer = PartialFillExplorer::new();

        // Add opportunities from different DEXes
        let opportunity1 = PartialFillOpportunity {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            available_liquidity: 1000000,
            exchange_rate: 13.5,
            fee: 0.003,
        };

        let opportunity2 = PartialFillOpportunity {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex_name: "SushiSwap".to_string(),
            available_liquidity: 50000000,
            exchange_rate: 3200.0,
            fee: 0.003,
        };

        explorer.add_opportunity(opportunity1.clone());
        explorer.add_opportunity(opportunity2.clone());

        assert_eq!(explorer.opportunity_count(), 2);

        // Remove Uniswap opportunities
        explorer.remove_dex_opportunities("Uniswap");

        assert_eq!(explorer.opportunity_count(), 1);
        assert_eq!(
            explorer
                .get_opportunities_from_token(&"BTC".to_string())
                .map_or(0, |opportunities| opportunities.len()),
            0
        );
        assert_eq!(
            explorer
                .get_opportunities_from_token(&"ETH".to_string())
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_clear_opportunities() {
        let mut explorer = PartialFillExplorer::new();

        let opportunity = PartialFillOpportunity {
            from_token: "BTC".to_string(),
            to_token: "ETH".to_string(),
            dex_name: "Uniswap".to_string(),
            available_liquidity: 1000000,
            exchange_rate: 13.5,
            fee: 0.003,
        };

        explorer.add_opportunity(opportunity.clone());
        assert_eq!(explorer.token_count(), 1);
        assert_eq!(explorer.opportunity_count(), 1);

        // Clear all opportunities
        explorer.clear_opportunities();
        assert_eq!(explorer.token_count(), 0);
        assert_eq!(explorer.opportunity_count(), 0);
    }
}
