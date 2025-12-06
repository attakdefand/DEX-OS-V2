//! Cross-chain asset mapping implementation for the DEX-OS core engine
//!
//! This module implements the Priority 3 feature from DEX-OS-V1.csv:
//! "Core Trading,Bridge,Bridge,Hash Map,Cross-chain Asset Mapping,Medium"
//!
//! It provides functionality for mapping assets across different blockchain networks,
//! enabling seamless cross-chain trading and asset transfers.

use crate::types::{TokenId, TraderId};
use std::collections::HashMap;
use thiserror::Error;

/// Represents a cross-chain asset mapping
#[derive(Debug, Clone, PartialEq)]
pub struct CrossChainAssetMapping {
    /// The asset identifier on the source chain
    pub source_asset_id: TokenId,
    /// The source blockchain network
    pub source_chain: String,
    /// The asset identifier on the destination chain
    pub destination_asset_id: TokenId,
    /// The destination blockchain network
    pub destination_chain: String,
    /// Conversion rate from source to destination (if applicable)
    pub conversion_rate: Option<f64>,
}

/// Manages cross-chain asset mappings
#[derive(Debug, Clone)]
pub struct CrossChainAssetMapper {
    /// Maps source chain asset identifiers to their cross-chain mappings
    mappings: HashMap<(TokenId, String), CrossChainAssetMapping>,
    /// Reverse mapping for quick lookup from destination to source
    reverse_mappings: HashMap<(TokenId, String), CrossChainAssetMapping>,
    /// Tracks which traders have access to which cross-chain mappings
    trader_access: HashMap<TraderId, Vec<(TokenId, String)>>,
}

impl CrossChainAssetMapper {
    /// Create a new cross-chain asset mapper
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            reverse_mappings: HashMap::new(),
            trader_access: HashMap::new(),
        }
    }

    /// Add a new cross-chain asset mapping
    pub fn add_mapping(
        &mut self,
        mapping: CrossChainAssetMapping,
    ) -> Result<(), CrossChainAssetError> {
        // Check if mapping already exists
        let key = (
            mapping.source_asset_id.clone(),
            mapping.source_chain.clone(),
        );
        if self.mappings.contains_key(&key) {
            return Err(CrossChainAssetError::MappingAlreadyExists);
        }

        // Add the mapping
        self.mappings.insert(key, mapping.clone());

        // Add reverse mapping
        let reverse_key = (
            mapping.destination_asset_id.clone(),
            mapping.destination_chain.clone(),
        );
        self.reverse_mappings.insert(reverse_key, mapping);

        Ok(())
    }

    /// Remove a cross-chain asset mapping
    pub fn remove_mapping(
        &mut self,
        source_asset_id: &TokenId,
        source_chain: &str,
    ) -> Result<(), CrossChainAssetError> {
        // Get the mapping to remove
        let key = (source_asset_id.clone(), source_chain.to_string());
        let mapping = self
            .mappings
            .remove(&key)
            .ok_or(CrossChainAssetError::MappingNotFound)?;

        // Remove reverse mapping
        let reverse_key = (mapping.destination_asset_id, mapping.destination_chain);
        self.reverse_mappings.remove(&reverse_key);

        // Remove from trader access
        for (_, access_list) in self.trader_access.iter_mut() {
            access_list.retain(|k| *k != key);
        }

        Ok(())
    }

    /// Get a cross-chain asset mapping by source asset and chain
    pub fn get_mapping(
        &self,
        source_asset_id: &TokenId,
        source_chain: &str,
    ) -> Option<&CrossChainAssetMapping> {
        let key = (source_asset_id.clone(), source_chain.to_string());
        self.mappings.get(&key)
    }

    /// Get a cross-chain asset mapping by destination asset and chain
    pub fn get_mapping_by_destination(
        &self,
        destination_asset_id: &TokenId,
        destination_chain: &str,
    ) -> Option<&CrossChainAssetMapping> {
        let key = (destination_asset_id.clone(), destination_chain.to_string());
        self.reverse_mappings.get(&key)
    }

    /// Get all mappings for a specific source chain
    pub fn get_mappings_for_source_chain(
        &self,
        source_chain: &str,
    ) -> Vec<&CrossChainAssetMapping> {
        self.mappings
            .iter()
            .filter(|((_, chain), _)| chain == source_chain)
            .map(|(_, mapping)| mapping)
            .collect()
    }

    /// Get all mappings for a specific destination chain
    pub fn get_mappings_for_destination_chain(
        &self,
        destination_chain: &str,
    ) -> Vec<&CrossChainAssetMapping> {
        self.reverse_mappings
            .iter()
            .filter(|((_, chain), _)| chain == destination_chain)
            .map(|(_, mapping)| mapping)
            .collect()
    }

    /// Grant a trader access to a cross-chain mapping
    pub fn grant_trader_access(
        &mut self,
        trader_id: TraderId,
        source_asset_id: TokenId,
        source_chain: String,
    ) {
        let key = (source_asset_id, source_chain);
        self.trader_access
            .entry(trader_id)
            .or_insert_with(Vec::new)
            .push(key);
    }

    /// Revoke a trader's access to a cross-chain mapping
    pub fn revoke_trader_access(
        &mut self,
        trader_id: &TraderId,
        source_asset_id: &TokenId,
        source_chain: &str,
    ) {
        if let Some(access_list) = self.trader_access.get_mut(trader_id) {
            let key = (source_asset_id.clone(), source_chain.to_string());
            access_list.retain(|k| *k != key);
        }
    }

    /// Check if a trader has access to a specific cross-chain mapping
    pub fn has_trader_access(
        &self,
        trader_id: &TraderId,
        source_asset_id: &TokenId,
        source_chain: &str,
    ) -> bool {
        if let Some(access_list) = self.trader_access.get(trader_id) {
            let key = (source_asset_id.clone(), source_chain.to_string());
            access_list.contains(&key)
        } else {
            false
        }
    }

    /// Get all mappings accessible to a trader
    pub fn get_trader_mappings(&self, trader_id: &TraderId) -> Vec<&CrossChainAssetMapping> {
        if let Some(access_list) = self.trader_access.get(trader_id) {
            access_list
                .iter()
                .filter_map(|key| self.mappings.get(key))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get the number of mappings
    pub fn mapping_count(&self) -> usize {
        self.mappings.len()
    }

    /// Check if there are no mappings
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Convert an amount from source to destination chain using the mapping's conversion rate
    pub fn convert_amount(
        &self,
        source_asset_id: &TokenId,
        source_chain: &str,
        amount: f64,
    ) -> Result<f64, CrossChainAssetError> {
        let mapping = self
            .get_mapping(source_asset_id, source_chain)
            .ok_or(CrossChainAssetError::MappingNotFound)?;

        if let Some(rate) = mapping.conversion_rate {
            Ok(amount * rate)
        } else {
            Err(CrossChainAssetError::NoConversionRate)
        }
    }
}

impl Default for CrossChainAssetMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during cross-chain asset mapping operations
#[derive(Debug, Error)]
pub enum CrossChainAssetError {
    #[error("Cross-chain asset mapping already exists")]
    MappingAlreadyExists,
    #[error("Cross-chain asset mapping not found")]
    MappingNotFound,
    #[error("No conversion rate available for this mapping")]
    NoConversionRate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_chain_asset_mapper_creation() {
        let mapper = CrossChainAssetMapper::new();
        assert!(mapper.is_empty());
        assert_eq!(mapper.mapping_count(), 0);
    }

    #[test]
    fn test_add_mapping() {
        let mut mapper = CrossChainAssetMapper::new();

        let mapping = CrossChainAssetMapping {
            source_asset_id: "BTC".to_string(),
            source_chain: "Bitcoin".to_string(),
            destination_asset_id: "WBTC".to_string(),
            destination_chain: "Ethereum".to_string(),
            conversion_rate: Some(1.0),
        };

        assert!(mapper.add_mapping(mapping).is_ok());
        assert_eq!(mapper.mapping_count(), 1);
        assert!(!mapper.is_empty());
    }

    #[test]
    fn test_duplicate_mapping() {
        let mut mapper = CrossChainAssetMapper::new();

        let mapping1 = CrossChainAssetMapping {
            source_asset_id: "BTC".to_string(),
            source_chain: "Bitcoin".to_string(),
            destination_asset_id: "WBTC".to_string(),
            destination_chain: "Ethereum".to_string(),
            conversion_rate: Some(1.0),
        };

        let mapping2 = CrossChainAssetMapping {
            source_asset_id: "BTC".to_string(),
            source_chain: "Bitcoin".to_string(),
            destination_asset_id: "BTCB".to_string(),
            destination_chain: "BinanceSmartChain".to_string(),
            conversion_rate: Some(1.0),
        };

        // First mapping should succeed
        assert!(mapper.add_mapping(mapping1).is_ok());

        // Second mapping with same source should fail
        let result = mapper.add_mapping(mapping2);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CrossChainAssetError::MappingAlreadyExists
        ));
    }

    #[test]
    fn test_get_mapping() {
        let mut mapper = CrossChainAssetMapper::new();

        let mapping = CrossChainAssetMapping {
            source_asset_id: "BTC".to_string(),
            source_chain: "Bitcoin".to_string(),
            destination_asset_id: "WBTC".to_string(),
            destination_chain: "Ethereum".to_string(),
            conversion_rate: Some(1.0),
        };

        mapper.add_mapping(mapping.clone()).unwrap();

        // Test forward lookup
        let retrieved = mapper.get_mapping(&"BTC".to_string(), "Bitcoin");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &mapping);

        // Test reverse lookup
        let retrieved_reverse = mapper.get_mapping_by_destination(&"WBTC".to_string(), "Ethereum");
        assert!(retrieved_reverse.is_some());
        assert_eq!(retrieved_reverse.unwrap(), &mapping);
    }

    #[test]
    fn test_remove_mapping() {
        let mut mapper = CrossChainAssetMapper::new();

        let mapping = CrossChainAssetMapping {
            source_asset_id: "BTC".to_string(),
            source_chain: "Bitcoin".to_string(),
            destination_asset_id: "WBTC".to_string(),
            destination_chain: "Ethereum".to_string(),
            conversion_rate: Some(1.0),
        };

        mapper.add_mapping(mapping).unwrap();
        assert_eq!(mapper.mapping_count(), 1);

        // Remove the mapping
        assert!(mapper.remove_mapping(&"BTC".to_string(), "Bitcoin").is_ok());
        assert_eq!(mapper.mapping_count(), 0);
        assert!(mapper.is_empty());

        // Try to remove non-existent mapping
        let result = mapper.remove_mapping(&"BTC".to_string(), "Bitcoin");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CrossChainAssetError::MappingNotFound
        ));
    }

    #[test]
    fn test_get_mappings_by_chain() {
        let mut mapper = CrossChainAssetMapper::new();

        // Add mappings from different source chains
        let mapping1 = CrossChainAssetMapping {
            source_asset_id: "BTC".to_string(),
            source_chain: "Bitcoin".to_string(),
            destination_asset_id: "WBTC".to_string(),
            destination_chain: "Ethereum".to_string(),
            conversion_rate: Some(1.0),
        };

        let mapping2 = CrossChainAssetMapping {
            source_asset_id: "ETH".to_string(),
            source_chain: "Ethereum".to_string(),
            destination_asset_id: "WETH".to_string(),
            destination_chain: "BinanceSmartChain".to_string(),
            conversion_rate: Some(1.0),
        };

        let mapping3 = CrossChainAssetMapping {
            source_asset_id: "SOL".to_string(),
            source_chain: "Bitcoin".to_string(), // Same source chain as mapping1
            destination_asset_id: "WSOL".to_string(),
            destination_chain: "Polygon".to_string(),
            conversion_rate: Some(1.0),
        };

        mapper.add_mapping(mapping1).unwrap();
        mapper.add_mapping(mapping2).unwrap();
        mapper.add_mapping(mapping3).unwrap();

        // Test getting mappings by source chain
        let bitcoin_mappings = mapper.get_mappings_for_source_chain("Bitcoin");
        assert_eq!(bitcoin_mappings.len(), 2);

        let ethereum_mappings = mapper.get_mappings_for_source_chain("Ethereum");
        assert_eq!(ethereum_mappings.len(), 1);

        // Test getting mappings by destination chain
        let eth_mappings = mapper.get_mappings_for_destination_chain("Ethereum");
        assert_eq!(eth_mappings.len(), 1);
    }

    #[test]
    fn test_trader_access() {
        let mut mapper = CrossChainAssetMapper::new();

        let mapping = CrossChainAssetMapping {
            source_asset_id: "BTC".to_string(),
            source_chain: "Bitcoin".to_string(),
            destination_asset_id: "WBTC".to_string(),
            destination_chain: "Ethereum".to_string(),
            conversion_rate: Some(1.0),
        };

        mapper.add_mapping(mapping).unwrap();

        let trader_id = "trader1".to_string();

        // Grant access
        mapper.grant_trader_access(trader_id.clone(), "BTC".to_string(), "Bitcoin".to_string());

        // Check access
        assert!(mapper.has_trader_access(&trader_id, &"BTC".to_string(), "Bitcoin"));
        assert!(!mapper.has_trader_access(&trader_id, &"ETH".to_string(), "Ethereum"));

        // Get trader mappings
        let trader_mappings = mapper.get_trader_mappings(&trader_id);
        assert_eq!(trader_mappings.len(), 1);

        // Revoke access
        mapper.revoke_trader_access(&trader_id, &"BTC".to_string(), "Bitcoin");
        assert!(!mapper.has_trader_access(&trader_id, &"BTC".to_string(), "Bitcoin"));

        // Get trader mappings after revocation
        let trader_mappings = mapper.get_trader_mappings(&trader_id);
        assert_eq!(trader_mappings.len(), 0);
    }

    #[test]
    fn test_amount_conversion() {
        let mut mapper = CrossChainAssetMapper::new();

        // Add mapping with conversion rate
        let mapping_with_rate = CrossChainAssetMapping {
            source_asset_id: "BTC".to_string(),
            source_chain: "Bitcoin".to_string(),
            destination_asset_id: "USD".to_string(),
            destination_chain: "Fiat".to_string(),
            conversion_rate: Some(50000.0), // 1 BTC = 50000 USD
        };

        // Add mapping without conversion rate
        let mapping_without_rate = CrossChainAssetMapping {
            source_asset_id: "ETH".to_string(),
            source_chain: "Ethereum".to_string(),
            destination_asset_id: "WETH".to_string(),
            destination_chain: "Ethereum".to_string(),
            conversion_rate: None, // Wrapped token, 1:1 but no explicit rate
        };

        mapper.add_mapping(mapping_with_rate).unwrap();
        mapper.add_mapping(mapping_without_rate).unwrap();

        // Test conversion with rate
        let converted_amount = mapper.convert_amount(&"BTC".to_string(), "Bitcoin", 2.5);
        assert!(converted_amount.is_ok());
        assert_eq!(converted_amount.unwrap(), 125000.0); // 2.5 * 50000

        // Test conversion without rate
        let no_rate_result = mapper.convert_amount(&"ETH".to_string(), "Ethereum", 1.0);
        assert!(no_rate_result.is_err());
        assert!(matches!(
            no_rate_result.unwrap_err(),
            CrossChainAssetError::NoConversionRate
        ));

        // Test conversion with non-existent mapping
        let not_found_result = mapper.convert_amount(&"DOGE".to_string(), "Dogecoin", 1000.0);
        assert!(not_found_result.is_err());
        assert!(matches!(
            not_found_result.unwrap_err(),
            CrossChainAssetError::MappingNotFound
        ));
    }
}
