//! Duplicate trade prevention implementation using Hash Set for the DEX-OS core engine
//!
//! This module implements the Priority 2 feature from DEX-OS-V1.csv:
//! "Core Trading,DEX Aggregator,DEX Aggregator,Hash Set,Duplicate Trade Prevention,Medium"
//!
//! It provides functionality for preventing duplicate trades using a hash set
//! to track already processed trades.

use crate::types::{TokenId, TradeId, TraderId};
use std::collections::HashSet;
use thiserror::Error;

/// Represents a trade that has been processed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessedTrade {
    /// Unique identifier for the trade
    pub trade_id: TradeId,
    /// Trader who initiated the trade
    pub trader_id: TraderId,
    /// Base token in the trade
    pub base_token: TokenId,
    /// Quote token in the trade
    pub quote_token: TokenId,
    /// Price of the trade
    pub price: u64,
    /// Quantity of the trade
    pub quantity: u64,
}

/// Manages duplicate trade prevention using a hash set
#[derive(Debug, Clone)]
pub struct DuplicateTradePrevention {
    /// Set of processed trades to prevent duplicates
    processed_trades: HashSet<ProcessedTrade>,
    /// Set of trade IDs to quickly check if a trade has been processed
    processed_trade_ids: HashSet<TradeId>,
}

impl DuplicateTradePrevention {
    /// Create a new duplicate trade prevention manager
    pub fn new() -> Self {
        Self {
            processed_trades: HashSet::new(),
            processed_trade_ids: HashSet::new(),
        }
    }

    /// Check if a trade has already been processed
    pub fn is_trade_processed(&self, trade_id: TradeId) -> bool {
        self.processed_trade_ids.contains(&trade_id)
    }

    /// Check if a trade with specific details has already been processed
    pub fn is_trade_details_processed(&self, trade: &ProcessedTrade) -> bool {
        self.processed_trades.contains(trade)
    }

    /// Add a trade to the processed set
    pub fn add_processed_trade(
        &mut self,
        trade: ProcessedTrade,
    ) -> Result<(), DuplicateTradeError> {
        // Check if trade ID already exists
        if self.processed_trade_ids.contains(&trade.trade_id) {
            return Err(DuplicateTradeError::TradeAlreadyProcessed);
        }

        // Add to both sets
        self.processed_trades.insert(trade.clone());
        self.processed_trade_ids.insert(trade.trade_id);

        Ok(())
    }

    /// Remove a trade from the processed set (in case of rollback or correction)
    pub fn remove_processed_trade(
        &mut self,
        trade_id: TradeId,
    ) -> Result<ProcessedTrade, DuplicateTradeError> {
        // Find the trade with this ID
        let trade = self
            .processed_trades
            .iter()
            .find(|t| t.trade_id == trade_id)
            .cloned();

        if let Some(trade) = trade {
            // Remove from both sets
            self.processed_trades.remove(&trade);
            self.processed_trade_ids.remove(&trade_id);
            Ok(trade)
        } else {
            Err(DuplicateTradeError::TradeNotFound)
        }
    }

    /// Get the number of processed trades
    pub fn processed_trade_count(&self) -> usize {
        self.processed_trades.len()
    }

    /// Check if any trades have been processed
    pub fn has_processed_trades(&self) -> bool {
        !self.processed_trades.is_empty()
    }

    /// Clear all processed trades
    pub fn clear_processed_trades(&mut self) {
        self.processed_trades.clear();
        self.processed_trade_ids.clear();
    }

    /// Get all processed trades
    pub fn get_all_processed_trades(&self) -> Vec<&ProcessedTrade> {
        self.processed_trades.iter().collect()
    }

    /// Get processed trades for a specific trader
    pub fn get_trades_for_trader(&self, trader_id: &TraderId) -> Vec<&ProcessedTrade> {
        self.processed_trades
            .iter()
            .filter(|trade| &trade.trader_id == trader_id)
            .collect()
    }

    /// Get processed trades for a specific token pair
    pub fn get_trades_for_token_pair(
        &self,
        base_token: &TokenId,
        quote_token: &TokenId,
    ) -> Vec<&ProcessedTrade> {
        self.processed_trades
            .iter()
            .filter(|trade| &trade.base_token == base_token && &trade.quote_token == quote_token)
            .collect()
    }

    /// Get the memory usage estimate (in bytes)
    pub fn memory_usage_estimate(&self) -> usize {
        // Rough estimate: each ProcessedTrade is about 100 bytes + HashSet overhead
        self.processed_trades.len() * 100 + self.processed_trade_ids.len() * 8
    }
}

/// Errors that can occur during duplicate trade prevention operations
#[derive(Debug, Error)]
pub enum DuplicateTradeError {
    #[error("Trade has already been processed")]
    TradeAlreadyProcessed,
    #[error("Trade not found in processed trades")]
    TradeNotFound,
    #[error("Invalid trade data")]
    InvalidTradeData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_trade_prevention_creation() {
        let prevention = DuplicateTradePrevention::new();
        assert_eq!(prevention.processed_trade_count(), 0);
        assert!(!prevention.has_processed_trades());
        assert_eq!(prevention.get_all_processed_trades().len(), 0);
    }

    #[test]
    fn test_add_and_check_processed_trade() {
        let mut prevention = DuplicateTradePrevention::new();

        let trade = ProcessedTrade {
            trade_id: 1,
            trader_id: "trader1".to_string(),
            base_token: "BTC".to_string(),
            quote_token: "USDC".to_string(),
            price: 50000,
            quantity: 1000,
        };

        // Initially, trade should not be processed
        assert!(!prevention.is_trade_processed(1));
        assert!(!prevention.is_trade_details_processed(&trade));

        // Add the trade
        assert!(prevention.add_processed_trade(trade.clone()).is_ok());

        // Now the trade should be marked as processed
        assert!(prevention.is_trade_processed(1));
        assert!(prevention.is_trade_details_processed(&trade));
        assert_eq!(prevention.processed_trade_count(), 1);
        assert!(prevention.has_processed_trades());
    }

    #[test]
    fn test_prevent_duplicate_trades() {
        let mut prevention = DuplicateTradePrevention::new();

        let trade = ProcessedTrade {
            trade_id: 1,
            trader_id: "trader1".to_string(),
            base_token: "BTC".to_string(),
            quote_token: "USDC".to_string(),
            price: 50000,
            quantity: 1000,
        };

        // Add the trade for the first time
        assert!(prevention.add_processed_trade(trade.clone()).is_ok());

        // Try to add the same trade again (should fail)
        assert!(prevention.add_processed_trade(trade.clone()).is_err());

        // Trade count should still be 1
        assert_eq!(prevention.processed_trade_count(), 1);
    }

    #[test]
    fn test_remove_processed_trade() {
        let mut prevention = DuplicateTradePrevention::new();

        let trade = ProcessedTrade {
            trade_id: 1,
            trader_id: "trader1".to_string(),
            base_token: "BTC".to_string(),
            quote_token: "USDC".to_string(),
            price: 50000,
            quantity: 1000,
        };

        // Add the trade
        prevention.add_processed_trade(trade.clone()).unwrap();
        assert_eq!(prevention.processed_trade_count(), 1);

        // Remove the trade
        let removed_trade = prevention.remove_processed_trade(1).unwrap();
        assert_eq!(removed_trade, trade);
        assert_eq!(prevention.processed_trade_count(), 0);
        assert!(!prevention.is_trade_processed(1));

        // Try to remove the same trade again (should fail)
        assert!(prevention.remove_processed_trade(1).is_err());
    }

    #[test]
    fn test_get_trades_for_trader() {
        let mut prevention = DuplicateTradePrevention::new();

        // Add trades for different traders
        let trade1 = ProcessedTrade {
            trade_id: 1,
            trader_id: "trader1".to_string(),
            base_token: "BTC".to_string(),
            quote_token: "USDC".to_string(),
            price: 50000,
            quantity: 1000,
        };

        let trade2 = ProcessedTrade {
            trade_id: 2,
            trader_id: "trader2".to_string(),
            base_token: "ETH".to_string(),
            quote_token: "USDC".to_string(),
            price: 3000,
            quantity: 5000,
        };

        let trade3 = ProcessedTrade {
            trade_id: 3,
            trader_id: "trader1".to_string(),
            base_token: "BTC".to_string(),
            quote_token: "USDC".to_string(),
            price: 51000,
            quantity: 2000,
        };

        prevention.add_processed_trade(trade1.clone()).unwrap();
        prevention.add_processed_trade(trade2.clone()).unwrap();
        prevention.add_processed_trade(trade3.clone()).unwrap();

        // Get trades for trader1
        let trader1_trades = prevention.get_trades_for_trader(&"trader1".to_string());
        assert_eq!(trader1_trades.len(), 2);

        // Get trades for trader2
        let trader2_trades = prevention.get_trades_for_trader(&"trader2".to_string());
        assert_eq!(trader2_trades.len(), 1);
        assert_eq!(trader2_trades[0].trade_id, 2);
    }

    #[test]
    fn test_get_trades_for_token_pair() {
        let mut prevention = DuplicateTradePrevention::new();

        // Add trades for different token pairs
        let trade1 = ProcessedTrade {
            trade_id: 1,
            trader_id: "trader1".to_string(),
            base_token: "BTC".to_string(),
            quote_token: "USDC".to_string(),
            price: 50000,
            quantity: 1000,
        };

        let trade2 = ProcessedTrade {
            trade_id: 2,
            trader_id: "trader2".to_string(),
            base_token: "ETH".to_string(),
            quote_token: "USDC".to_string(),
            price: 3000,
            quantity: 5000,
        };

        let trade3 = ProcessedTrade {
            trade_id: 3,
            trader_id: "trader3".to_string(),
            base_token: "BTC".to_string(),
            quote_token: "USDC".to_string(),
            price: 51000,
            quantity: 2000,
        };

        prevention.add_processed_trade(trade1.clone()).unwrap();
        prevention.add_processed_trade(trade2.clone()).unwrap();
        prevention.add_processed_trade(trade3.clone()).unwrap();

        // Get trades for BTC/USDC pair
        let btc_usdc_trades =
            prevention.get_trades_for_token_pair(&"BTC".to_string(), &"USDC".to_string());
        assert_eq!(btc_usdc_trades.len(), 2);

        // Get trades for ETH/USDC pair
        let eth_usdc_trades =
            prevention.get_trades_for_token_pair(&"ETH".to_string(), &"USDC".to_string());
        assert_eq!(eth_usdc_trades.len(), 1);
        assert_eq!(eth_usdc_trades[0].trade_id, 2);
    }

    #[test]
    fn test_clear_processed_trades() {
        let mut prevention = DuplicateTradePrevention::new();

        let trade = ProcessedTrade {
            trade_id: 1,
            trader_id: "trader1".to_string(),
            base_token: "BTC".to_string(),
            quote_token: "USDC".to_string(),
            price: 50000,
            quantity: 1000,
        };

        prevention.add_processed_trade(trade.clone()).unwrap();
        assert_eq!(prevention.processed_trade_count(), 1);

        // Clear all processed trades
        prevention.clear_processed_trades();
        assert_eq!(prevention.processed_trade_count(), 0);
        assert!(!prevention.has_processed_trades());
        assert_eq!(prevention.get_all_processed_trades().len(), 0);
    }

    #[test]
    fn test_memory_usage_estimate() {
        let mut prevention = DuplicateTradePrevention::new();

        let trade = ProcessedTrade {
            trade_id: 1,
            trader_id: "trader1".to_string(),
            base_token: "BTC".to_string(),
            quote_token: "USDC".to_string(),
            price: 50000,
            quantity: 1000,
        };

        assert_eq!(prevention.memory_usage_estimate(), 0);

        prevention.add_processed_trade(trade.clone()).unwrap();
        let estimate = prevention.memory_usage_estimate();
        assert!(estimate > 0);
        // Rough estimate: 100 bytes for ProcessedTrade + 8 bytes for TradeId = 108 bytes
        assert!(estimate >= 108);
    }
}
